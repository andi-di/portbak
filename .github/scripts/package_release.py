"""Package only the executable and redistribution notices, never local app data."""

import argparse
import io
from pathlib import Path
import re
import tarfile
import zipfile


def package(root, output, version, platform):
    if not re.fullmatch(r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)", version):
        raise ValueError("Expected a stable major.minor.patch version")
    binary = "portbak.exe" if platform == "windows" else "portbak"
    files = [(binary, (root / "target/release" / binary).read_bytes(), 0o755),
             ("LICENSE", (root / "LICENSE").read_bytes(), 0o644)]
    for notice in sorted((root / "assets/fonts").glob("OFL-*.txt")):
        files.append((f"licenses/{notice.name}", notice.read_bytes(), 0o644))
    note = (f"portbak {version}\n\n"
            "Extract into a writable folder and launch the executable.\n"
            "To upgrade, close portbak and replace its executable; keep settings and backups.\n"
            "Scheduled backups require the app to stay running.\n"
            "https://github.com/andi-di/portbak\n")
    files.append(("README.txt", note.encode(), 0o644))
    output.mkdir(parents=True, exist_ok=True)
    suffix = "zip" if platform == "windows" else "tar.gz"
    archive = output / f"portbak-v{version}-{platform}-x86_64.{suffix}"
    if platform == "windows":
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as zipped:
            for name, data, mode in files:
                entry = zipfile.ZipInfo(name)
                entry.create_system = 3
                entry.external_attr = (0o100000 | mode) << 16
                zipped.writestr(entry, data, compress_type=zipfile.ZIP_DEFLATED)
    else:
        with tarfile.open(archive, "w:gz") as tar:
            for name, data, mode in files:
                entry = tarfile.TarInfo(name)
                entry.size, entry.mode = len(data), mode
                tar.addfile(entry, io.BytesIO(data))
    return archive


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--platform", choices=["linux", "windows"], required=True)
    parser.add_argument("--output", type=Path, default=Path("release-assets"))
    args = parser.parse_args()
    print(package(Path.cwd(), args.output, args.version, args.platform))
