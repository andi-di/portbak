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
import zipfile

x = C.CDLL("libX11.so.6")
x.XOpenDisplay.argtypes = [C.c_char_p]
x.XOpenDisplay.restype = C.c_void_p
x.XDefaultRootWindow.argtypes = [C.c_void_p]
x.XDefaultRootWindow.restype = C.c_ulong
x.XQueryTree.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_ulong), C.POINTER(C.POINTER(C.c_ulong)), C.POINTER(C.c_uint)]
x.XFetchName.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_char_p)]
x.XFree.argtypes = [C.c_void_p]
x.XCloseDisplay.argtypes = [C.c_void_p]
x.XResizeWindow.argtypes = [C.c_void_p, C.c_ulong, C.c_uint, C.c_uint]
x.XFlush.argtypes = [C.c_void_p]
x.XTranslateCoordinates.argtypes = [C.c_void_p, C.c_ulong, C.c_ulong, C.c_int, C.c_int, C.POINTER(C.c_int), C.POINTER(C.c_int), C.POINTER(C.c_ulong)]
x.XRaiseWindow.argtypes = [C.c_void_p, C.c_ulong]
x.XSetInputFocus.argtypes = [C.c_void_p, C.c_ulong, C.c_int, C.c_ulong]
xtest = C.CDLL("libXtst.so.6")
xtest.XTestFakeMotionEvent.argtypes = [C.c_void_p, C.c_int, C.c_int, C.c_int, C.c_ulong]
xtest.XTestFakeButtonEvent.argtypes = [C.c_void_p, C.c_uint, C.c_int, C.c_ulong]
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
            if title == "portbak":
                return wid
        found = find_window(wid)
        if found:
            return found
    return None

def click(wid, horizontal, vertical):
    px, py, child = C.c_int(), C.c_int(), C.c_ulong()
    assert x.XTranslateCoordinates(display, wid, root, horizontal, vertical, C.byref(px), C.byref(py), C.byref(child))
    x.XRaiseWindow(display, wid)
    x.XSetInputFocus(display, wid, 1, 0)
    xtest.XTestFakeMotionEvent(display, -1, px.value, py.value, 0)
    x.XFlush(display)
    time.sleep(0.15)
    xtest.XTestFakeButtonEvent(display, 1, 1, 0)
    x.XFlush(display)
    time.sleep(0.15)
    xtest.XTestFakeButtonEvent(display, 1, 0, 0)
    x.XFlush(display)
    time.sleep(0.3)

wine = "--wine" in sys.argv
arguments = [a for a in sys.argv[1:] if a != "--wine"]
binary = Path(arguments[0] if arguments else "target/release/portbak").resolve()
def fixture_path(path):
    return "Z:" + str(path).replace("/", "\\") if wine else str(path)
output = Path(tempfile.mkdtemp(prefix="portbak-ui-"))
env = dict(os.environ, WINIT_UNIX_BACKEND="x11")
env.pop("WAYLAND_DISPLAY", None)
try:
    for theme in ["Light", "Dark"]:
        case = output / theme
        case.mkdir()
        app = case / ("portbak.exe" if wine else "portbak")
        shutil.copy2(binary, app)
        source, target = case / "Documents", case / "Backups"
        source.mkdir()
        target.mkdir()
        (source / "Example.txt").write_text("UI smoke test\n")
        job_id = str(uuid.uuid4())
        config = {"version": 1, "theme": theme, "jobs": [{
            "id": job_id, "name": "My documents", "source": fixture_path(source), "target": fixture_path(target),
            "enabled": True, "zip": theme == "Dark", "schedule": "Interval", "interval_value": 5, "interval_unit": "Seconds", "next_run": "2020-01-01T00:00:00Z",
            "rolling": True, "max_count": 10, "max_bytes": 10737418240
        }]}
        (case / "portbak.json").write_text(json.dumps(config))
        with (case / "app.log").open("w") as log:
            process = subprocess.Popen((["wine", str(app)] if wine else [str(app)]), env=env, stdout=log, stderr=log)
            try:
                manifest = target / ("manifest-" + job_id + ".json")
                deadline = time.monotonic() + 20
                while not manifest.exists() and time.monotonic() < deadline:
                    assert process.poll() is None, (case / "app.log").read_text()
                    time.sleep(0.1)
                assert manifest.exists(), "Scheduled worker did not create backup"
                data = json.loads(manifest.read_text())
                assert len(data["records"]) == 1
                backup = manifest.parent / data["records"][0]["name"]
                if theme == "Dark":
                    with zipfile.ZipFile(backup) as archive:
                        assert archive.read("Example.txt") == b"UI smoke test\n"
                else:
                    assert (backup / "Example.txt").read_text() == "UI smoke test\n"
                time.sleep(1.5)
                wid = find_window(root)
                assert wid, "App window not found"
                subprocess.run(["import", "-window", str(wid), str(case / "screenshot.png")], check=True, timeout=10)
                x.XResizeWindow(display, wid, 800, 600)
                x.XFlush(display)
                time.sleep(0.5)
                subprocess.run(["import", "-window", str(wid), str(case / "small-window.png")], check=True, timeout=10)
                with open(f"/proc/{process.pid}/status") as status:
                    memory = next(line.strip() for line in status if line.startswith("VmRSS:"))
                saved = json.loads((case / "portbak.json").read_text())
                assert saved["jobs"][0]["last_success"], "Backup completion not persisted"
                assert saved["jobs"][0]["interval_unit"] == "Seconds"
                deadline = time.monotonic() + 10
                while time.monotonic() < deadline:
                    saved = json.loads((case / "portbak.json").read_text())
                    if saved["jobs"][0]["status"].startswith("Unchanged"):
                        break
                    time.sleep(0.1)
                assert saved["jobs"][0]["status"].startswith("Unchanged"), "Second-based interval did not run again"
                x.XResizeWindow(display, wid, 1100, 960)
                x.XFlush(display)
                time.sleep(0.5)
                click(wid, 410, 787)
                time.sleep(0.8)
                subprocess.run(["import", "-window", str(wid), str(case / "restore-dialog.png")], check=True, timeout=10)
                click(wid, 250, 495)
                assert json.loads((case / "portbak.json").read_text())["jobs"][0]["enabled"], "Cancel disabled the job"
                click(wid, 410, 787)
                time.sleep(0.8)
                click(wid, 400, 360)
                (source / "Example.txt").write_text("changed after backup\n")
                (source / "extra" / "nested").mkdir(parents=True)
                (source / "extra" / "nested" / "remove.txt").write_text("remove\n")
                subprocess.run(["import", "-window", str(wid), str(case / "restore-selected.png")], check=True, timeout=10)
                last_backup = saved["jobs"][0]["last_success"]
                click(wid, 400, 516)
                deadline = time.monotonic() + 15
                while time.monotonic() < deadline:
                    saved = json.loads((case / "portbak.json").read_text())
                    if saved["jobs"][0]["status"].startswith("Restore complete"):
                        break
                    assert process.poll() is None, (case / "app.log").read_text()
                    time.sleep(0.1)
                assert saved["jobs"][0]["status"].startswith("Restore complete"), saved["jobs"][0]["status"]
                assert (source / "Example.txt").read_text() == "UI smoke test\n"
                assert not (source / "extra").exists(), "Restore left additional source files"
                assert not saved["jobs"][0]["enabled"]
                assert saved["jobs"][0]["next_run"] is None
                assert saved["jobs"][0]["last_success"] == last_backup
                assert len(json.loads(manifest.read_text())["records"]) == 1
                subprocess.run(["import", "-window", str(wid), str(case / "restore-complete.png")], check=True, timeout=10)
                print(f"{theme}: scheduled backup + restore + persisted completion + screenshots OK; {memory}")
            finally:
                process.terminate()
                process.wait(timeout=5)
finally:
    x.XCloseDisplay(display)
print(f"Artifacts: {output}")
