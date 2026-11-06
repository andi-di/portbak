#!/usr/bin/env python3
"""Isolated Linux/X11 smoke test; uses ctypes Xlib and ImageMagick import.

Runs only temporary fixtures. Captures just the app window, never the desktop.
Usage: python3 tools/ui_smoke.py [release-binary]
"""
import ctypes as C
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import uuid

x = C.CDLL("libX11.so.6")
x.XOpenDisplay.argtypes = [C.c_char_p]
x.XOpenDisplay.restype = C.c_void_p
x.XDefaultRootWindow.argtypes = [C.c_void_p]
x.XDefaultRootWindow.restype = C.c_ulong
x.XQueryTree.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_ulong), C.POINTER(C.POINTER(C.c_ulong)), C.POINTER(C.c_uint)]
x.XFetchName.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_char_p)]
x.XFree.argtypes = [C.c_void_p]
x.XCloseDisplay.argtypes = [C.c_void_p]
display = x.XOpenDisplay(None)
assert display, "X11 display unavailable"
root = x.XDefaultRootWindow(display)

def find_window(parent):
    r, p, children, count = C.c_ulong(), C.c_ulong(), C.POINTER(C.c_ulong)(), C.c_uint()
    if not x.XQueryTree(display, parent, C.byref(r), C.byref(p), C.byref(children), C.byref(count)):
        return None
    ids = list(children[:count.value])
    if children:
        x.XFree(children)
    for wid in ids:
        name = C.c_char_p()
        if x.XFetchName(display, wid, C.byref(name)) and name.value:
            title = name.value.decode(errors="replace")
            x.XFree(name)
            if title == "Portable Backup":
                return wid
        found = find_window(wid)
        if found:
            return found
    return None

wine = "--wine" in sys.argv
arguments = [a for a in sys.argv[1:] if a != "--wine"]
binary = Path(arguments[0] if arguments else "target/release/portable-backup").resolve()
def fixture_path(path):
    return "Z:" + str(path).replace("/", "\\") if wine else str(path)
output = Path(tempfile.mkdtemp(prefix="portable-backup-ui-"))
env = dict(os.environ, WINIT_UNIX_BACKEND="x11")
env.pop("WAYLAND_DISPLAY", None)
try:
    for theme in ["Light", "Dark"]:
        case = output / theme
        case.mkdir()
        app = case / ("portable-backup.exe" if wine else "portable-backup")
        shutil.copy2(binary, app)
        source, target = case / "Documents", case / "Backups"
        source.mkdir()
        target.mkdir()
        (source / "Example.txt").write_text("UI smoke test\n")
        job_id = str(uuid.uuid4())
        config = {"version": 1, "theme": theme, "jobs": [{
            "id": job_id, "name": "My documents", "source": fixture_path(source), "target": fixture_path(target),
            "enabled": True, "zip": False, "schedule": "Daily", "next_run": "2020-01-01T00:00:00Z",
            "rolling": True, "max_count": 10, "max_bytes": 10737418240
        }]}
        (case / "portable-backup.json").write_text(json.dumps(config))
        with (case / "app.log").open("w") as log:
            process = subprocess.Popen((["wine", str(app)] if wine else [str(app)]), env=env, stdout=log, stderr=log)
            try:
                manifest = target / ("backup-" + job_id) / "manifest.json"
                deadline = time.monotonic() + 20
                while not manifest.exists() and time.monotonic() < deadline:
                    assert process.poll() is None, (case / "app.log").read_text()
                    time.sleep(0.1)
                assert manifest.exists(), "Scheduled worker did not create backup"
                data = json.loads(manifest.read_text())
                assert len(data["records"]) == 1
                copied = manifest.parent / data["records"][0]["name"] / "Example.txt"
                assert copied.read_text() == "UI smoke test\n"
                time.sleep(1.5)
                wid = find_window(root)
                assert wid, "App window not found"
                subprocess.run(["import", "-window", str(wid), str(case / "screenshot.png")], check=True, timeout=10)
                with open(f"/proc/{process.pid}/status") as status:
                    memory = next(line.strip() for line in status if line.startswith("VmRSS:"))
                saved = json.loads((case / "portable-backup.json").read_text())
                assert saved["jobs"][0]["last_success"], "Backup completion not persisted"
                print(f"{theme}: scheduled backup + persisted completion + screenshot OK; {memory}")
            finally:
                process.terminate()
                process.wait(timeout=5)
finally:
    x.XCloseDisplay(display)
print(f"Artifacts: {output}")
