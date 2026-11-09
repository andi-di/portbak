#!/usr/bin/env python3
"""Exercise automatic backup change detection using an isolated, real app process."""
import argparse
import ctypes as C
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import uuid
import zipfile

INTERVAL = 2.0
DURATION = 30.0
POLL = 0.025


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2), encoding="utf-8")


def tree(root):
    return {
        path.relative_to(root).as_posix(): None if path.is_dir() else path.read_bytes()
        for path in sorted(root.rglob("*"))
    }


def backup_tree(path, zipped):
    if not zipped:
        return tree(path)
    with zipfile.ZipFile(path) as archive:
        result = {}
        for entry in archive.infolist():
            name = entry.filename.rstrip("/")
            if name in result:
                raise RuntimeError(f"Duplicate ZIP entry: {name}")
            result[name] = None if entry.is_dir() else archive.read(entry)
        return result


def compare(actual, expected, context):
    missing = sorted(expected.keys() - actual.keys())
    extra = sorted(actual.keys() - expected.keys())
    changed = sorted(key for key in actual.keys() & expected.keys()
                     if actual[key] != expected[key])
    if missing or extra or changed:
        raise RuntimeError(f"{context}: missing={missing}, extra={extra}, changed={changed}")


def seed(source):
    source.mkdir()
    for name, content in {
        "same.txt": b"before", "append.txt": b"start", "truncate.txt": b"long contents",
        "binary.bin": bytes(range(256)), "delete.txt": b"delete", "rename.txt": b"rename",
        "move.txt": b"move", "mixed.txt": b"original",
    }.items():
        (source / name).write_bytes(content)
    (source / "subtree" / "nested").mkdir(parents=True)
    (source / "subtree" / "nested" / "file.txt").write_bytes(b"remove tree")


def mutate(source, round_number):
    def put(name, contents):
        (source / name).write_bytes(contents)

    if round_number == 1:
        path = source / "same.txt"
        before = path.stat()
        put("same.txt", b"AFTER!")
        os.utime(path, ns=(before.st_atime_ns, before.st_mtime_ns))
        return "same-length edit, preserved mtime"
    if round_number == 2:
        with (source / "append.txt").open("ab") as file:
            file.write(b" appended")
        return "append"
    if round_number == 3:
        put("truncate.txt", b"x")
        return "truncate"
    if round_number == 4:
        put("binary.bin", bytes(reversed(range(256))))
        return "binary edit"
    if round_number == 5:
        put("added.txt", b"new")
        return "add file"
    if round_number == 6:
        (source / "new" / "nested").mkdir(parents=True)
        put("new/nested/file.txt", b"nested")
        return "add nested tree"
    if round_number == 7:
        put(".hidden", b"hidden")
        put("Grüße mit Leerzeichen.txt", "Grüße".encode())
        return "hidden and Unicode filenames"
    if round_number == 8:
        (source / "delete.txt").unlink()
        return "delete file"
    if round_number == 9:
        (source / "rename.txt").rename(source / "renamed.txt")
        return "rename file"
    if round_number == 10:
        (source / "move.txt").rename(source / "new" / "nested" / "moved.txt")
        return "move file"
    if round_number == 11:
        (source / "empty").mkdir()
        return "add empty directory"
    if round_number == 12:
        (source / "empty").rename(source / "empty-renamed")
        return "rename empty directory"
    if round_number == 13:
        (source / "empty-renamed").rmdir()
        return "delete empty directory"
    if round_number == 14:
        shutil.rmtree(source / "subtree")
        return "delete populated tree"
    if round_number == 15:
        put("mixed-added.txt", b"added together")
        put("mixed.txt", b"changed together")
        (source / "added.txt").unlink()
        return "combined add/edit/delete"
    raise ValueError(f"Unknown round: {round_number}")


class AppWindow:
    """Find only a visible window owned by the isolated application process."""

    def __init__(self):
        self.display = None
        if sys.platform == "win32":
            from ctypes import wintypes as W
            self.user = C.WinDLL("user32", use_last_error=True)
            self.callback = C.WINFUNCTYPE(W.BOOL, W.HWND, W.LPARAM)
            self.user.EnumWindows.argtypes = [self.callback, W.LPARAM]
            self.user.IsWindowVisible.argtypes = [W.HWND]
            self.user.GetWindowThreadProcessId.argtypes = [W.HWND, C.POINTER(W.DWORD)]
            self.user.GetWindowTextW.argtypes = [W.HWND, W.LPWSTR, C.c_int]
            return
        if not sys.platform.startswith("linux"):
            raise RuntimeError("Supported platforms: Linux/X11 and Windows")
        self.x = C.CDLL("libX11.so.6")
        self.x.XOpenDisplay.argtypes = [C.c_char_p]
        self.x.XOpenDisplay.restype = C.c_void_p
        self.x.XDefaultRootWindow.argtypes = [C.c_void_p]
        self.x.XDefaultRootWindow.restype = C.c_ulong
        self.x.XQueryTree.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_ulong),
                                     C.POINTER(C.c_ulong), C.POINTER(C.POINTER(C.c_ulong)),
                                     C.POINTER(C.c_uint)]
        self.x.XInternAtom.argtypes = [C.c_void_p, C.c_char_p, C.c_int]
        self.x.XInternAtom.restype = C.c_ulong
        self.x.XGetWindowProperty.argtypes = [C.c_void_p, C.c_ulong, C.c_ulong,
            C.c_long, C.c_long, C.c_int, C.c_ulong, C.POINTER(C.c_ulong),
            C.POINTER(C.c_int), C.POINTER(C.c_ulong), C.POINTER(C.c_ulong),
            C.POINTER(C.POINTER(C.c_ubyte))]
        self.x.XFetchName.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_char_p)]
        self.x.XFree.argtypes = [C.c_void_p]
        self.x.XCloseDisplay.argtypes = [C.c_void_p]
        self.display = self.x.XOpenDisplay(None)
        if not self.display:
            raise RuntimeError("X11 display unavailable; use an X11/XWayland desktop session")
        self.root = self.x.XDefaultRootWindow(self.display)
        self.pid_atom = self.x.XInternAtom(self.display, b"_NET_WM_PID", 0)
        self.state_atom = self.x.XInternAtom(self.display, b"WM_STATE", 0)

    def property(self, window, atom):
        kind, count, remaining = C.c_ulong(), C.c_ulong(), C.c_ulong()
        fmt, data = C.c_int(), C.POINTER(C.c_ubyte)()
        status = self.x.XGetWindowProperty(self.display, window, atom, 0, 1, 0, 0,
            C.byref(kind), C.byref(fmt), C.byref(count), C.byref(remaining), C.byref(data))
        try:
            if status == 0 and fmt.value == 32 and count.value:
                return C.cast(data, C.POINTER(C.c_ulong))[0]
            return None
        finally:
            if data:
                self.x.XFree(data)

    def visible(self, pid):
        if sys.platform == "win32":
            found = []

            @self.callback
            def visit(window, _):
                owner = C.c_ulong()
                self.user.GetWindowThreadProcessId(window, C.byref(owner))
                title = C.create_unicode_buffer(256)
                self.user.GetWindowTextW(window, title, len(title))
                if owner.value == pid and title.value == "portbak" and self.user.IsWindowVisible(window):
                    found.append(window)
                return True

            self.user.EnumWindows(visit, 0)
            return bool(found)

        def visit(window):
            if self.property(window, self.pid_atom) == pid and self.property(window, self.state_atom) == 1:
                title = C.c_char_p()
                self.x.XFetchName(self.display, window, C.byref(title))
                try:
                    if title.value == b"portbak":
                        return True
                finally:
                    if title:
                        self.x.XFree(title)
            root, parent, count = C.c_ulong(), C.c_ulong(), C.c_uint()
            children = C.POINTER(C.c_ulong)()
            if not self.x.XQueryTree(self.display, window, C.byref(root), C.byref(parent),
                                     C.byref(children), C.byref(count)):
                return False
            ids = list(children[:count.value])
            if children:
                self.x.XFree(children)
            return any(visit(child) for child in ids)

        return visit(self.root)

    def close(self):
        if self.display:
            self.x.XCloseDisplay(self.display)


def wait_for(process, condition, deadline, context):
    while True:
        if process.poll() is not None:
            raise RuntimeError(f"Application exited ({process.returncode}): {context}")
        result = condition()
        if result:
            return result
        if time.monotonic() >= deadline:
            raise RuntimeError(f"Timeout: {context}")
        time.sleep(POLL)


def records(job):
    if not job["manifest"].exists():
        return []
    return read_json(job["manifest"])["records"]


def verify_round(jobs, expected, index):
    for job in jobs:
        entries = records(job)
        if len(entries) > index + 1:
            raise RuntimeError(f"{job['name']}: unexpected extra backup in round {index}")
        if len(entries) <= index:
            return False
        compare(backup_tree(job["target"] / entries[index]["name"], job["zip"]),
                expected, f"{job['name']}, round {index}")
    return True


def run(binary, output, report):
    source = output / "Source"
    seed(source)
    jobs, config_jobs = [], []
    for name, zipped in [("Folder", False), ("ZIP", True)]:
        target = output / name
        target.mkdir()
        job_id = str(uuid.uuid4())
        jobs.append({"name": name, "zip": zipped, "target": target,
                     "manifest": target / f"manifest-{job_id}.json"})
        config_jobs.append({"id": job_id, "name": name, "source": str(source),
            "target": str(target), "enabled": True, "zip": zipped,
            "schedule": "Interval", "interval_value": 2, "interval_unit": "Seconds",
            "next_run": "2020-01-01T00:00:00Z", "rolling": False})
    config_path = output / "portbak.json"
    write_json(config_path, {"version": 1, "theme": "Light", "jobs": config_jobs})
    app = output / ("portbak.exe" if sys.platform == "win32" else "portbak")
    shutil.copy2(binary, app)
    env = os.environ.copy()
    if sys.platform.startswith("linux"):
        env.pop("WAYLAND_DISPLAY", None)
        env["WINIT_UNIX_BACKEND"] = "x11"
    window = AppWindow()
    expected_versions = [tree(source)]
    snapshots = output / "expected"
    snapshots.mkdir()

    def save_expected(index):
        shutil.copytree(source, snapshots / f"{index:02}")

    save_expected(0)
    try:
        with (output / "app.log").open("w", encoding="utf-8") as log:
            process = subprocess.Popen([str(app)], cwd=output, env=env, stdout=log, stderr=log)
            try:
                deadline = time.monotonic() + 20
                wait_for(process, lambda: window.visible(process.pid), deadline, "visible application window")
                wait_for(process, lambda: verify_round(jobs, expected_versions[0], 0), deadline, "initial backups")
                # Start between scheduled checks, leaving room for scheduler jitter.
                start = time.monotonic() + 1.5

                def check_round(expected, index):
                    for job in read_json(config_path)["jobs"]:
                        if job["status"].startswith("Error:"):
                            raise RuntimeError(f"{job['name']}: {job['status']}")
                    return verify_round(jobs, expected, index)

                for index in range(1, 16):
                    due = start + (index - 1) * INTERVAL
                    wait_for(process, lambda: time.monotonic() >= due, due + 1, "round start")
                    description = mutate(source, index)
                    expected = tree(source)
                    expected_versions.append(expected)
                    save_expected(index)
                    result = {"round": index, "change": description,
                              "started_seconds": round(time.monotonic() - start, 3), "passed": False}
                    report["rounds"].append(result)
                    wait_for(process, lambda: check_round(expected, index),
                        start + index * INTERVAL, f"round {index}: {description}; both backups before next change")
                    result.update(completed_seconds=round(time.monotonic() - start, 3), passed=True)
                    print(f"{index:02}/15: {description}: Folder + ZIP OK", flush=True)
                wait_for(process, lambda: time.monotonic() >= start + DURATION,
                         start + DURATION + 1, "end of 30-second change phase")

                def final_completion_saved():
                    saved_jobs = read_json(config_path)["jobs"]
                    for job, fixture in zip(saved_jobs, jobs):
                        if job["status"].startswith("Error:"):
                            raise RuntimeError(job["status"])
                        timestamp = job.get("last_success")
                        created = records(fixture)[-1]["created"]
                        if not timestamp:
                            return False
                        completed_at = datetime.fromisoformat(timestamp.replace("Z", "+00:00"))
                        created_at = datetime.fromisoformat(created.replace("Z", "+00:00"))
                        if completed_at < created_at:
                            return False
                    return saved_jobs

                saved_jobs = wait_for(process, final_completion_saved, time.monotonic() + 5,
                                      "persisted final backup completion")
                success = [job["last_success"] for job in saved_jobs]

                def unchanged():
                    saved_jobs = read_json(config_path)["jobs"]
                    for job, timestamp in zip(saved_jobs, success):
                        if job["status"].startswith("Error:"):
                            raise RuntimeError(job["status"])
                        if job["last_success"] != timestamp:
                            raise RuntimeError("Unchanged check changed last_success")
                    return all(job["status"].startswith("Unchanged") for job in saved_jobs)

                wait_for(process, unchanged, time.monotonic() + 10, "unchanged checks after final backups")
                for index, expected in enumerate(expected_versions):
                    if not verify_version_contents(jobs, expected, index):
                        raise RuntimeError(f"Missing historical backup: round {index}")
                for job in jobs:
                    if len(records(job)) != 16:
                        raise RuntimeError(f"{job['name']}: expected exactly 16 backups")
                print("PASS: 15 changes, 32 verified backups, unchanged checks OK", flush=True)
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
    finally:
        window.close()


def verify_version_contents(jobs, expected, index):
    for job in jobs:
        entries = records(job)
        if len(entries) <= index:
            return False
        compare(backup_tree(job["target"] / entries[index]["name"], job["zip"]),
                expected, f"historical {job['name']}, round {index}")
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", nargs="?", type=Path,
                        default=Path("target/release/portbak.exe" if sys.platform == "win32" else "target/release/portbak"))
    args = parser.parse_args()
    output = Path(tempfile.mkdtemp(prefix="portbak-change-detection-"))
    report = {"platform": sys.platform, "binary": str(args.binary.resolve()),
              "started": datetime.now(timezone.utc).isoformat(), "rounds": [], "passed": False}
    try:
        run(args.binary.resolve(), output, report)
        report["passed"] = True
    except (Exception, KeyboardInterrupt) as error:
        report["error"] = str(error) or type(error).__name__
        print(f"FAIL: {report['error']}", file=sys.stderr, flush=True)
    finally:
        report["finished"] = datetime.now(timezone.utc).isoformat()
        config_path = output / "portbak.json"
        if config_path.exists():
            try:
                report["jobs"] = [{key: job.get(key) for key in
                    ("name", "status", "next_run", "last_success")}
                    for job in read_json(config_path)["jobs"]]
            except (OSError, ValueError, KeyError) as error:
                report["diagnostic_error"] = str(error)
        write_json(output / "report.json", report)
        print(f"Artifacts: {output}", flush=True)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
