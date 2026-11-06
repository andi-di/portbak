# Portable Backup

Lightweight backup app for Linux and Windows, written in Rust with egui. No browser runtime, server, or installer required.

## Product specification

See [SPECIFICATION.md](SPECIFICATION.md) for standalone, implementation-independent requirements suitable for rebuilding the application in another language.

## Getting started

Copy the executable for your platform into a **writable folder** and launch it. On Linux, run `chmod +x portable-backup` if necessary. The app creates `portable-backup.json` (settings and status) and `portable-backup.lock` (instance lock) alongside the executable. To move your configuration, copy the JSON file too and update source and destination paths. Administrator privileges are not required.

1. Select **New job** and choose a name and existing source and destination folders.
2. Optionally enable **Compress as ZIP**. The default is a full folder copy.
3. Select **Check now** to create the first backup. Subsequent backups are created only when file contents change, files are added or removed, paths are renamed, or empty directories change.
4. Jobs start with automatic backups disabled. Press the green **Enable automatic backups** button to arm the schedule: interval (the default, every 60 minutes, with selectable seconds, minutes, or hours), daily, or selected weekdays. Active jobs show a countdown updated every second in the job list and details. Press the red **Disable automatic backups** button at any time to remove pending work; a running backup finishes safely. **Check now** remains available when automatic backups are disabled. Re-enabling starts a fresh schedule.
5. Optionally enable **rolling retention** with a maximum count and/or total size. Both limits can be combined. Retention is disabled by default.

**The app must be running for scheduled backups.** You can minimize it. Closing the window asks for confirmation and stops scheduling. Missed checks are caught up once on startup or resume. Schedules use the local time zone: a time skipped by daylight saving is moved to the first valid minute, and a repeated time runs only once.

## Storage format and recovery

Backups are stored directly in the selected destination, alongside a per-job `manifest-<UUID>.json` and a destination `.lock` file. Backups are named `2026-11-06_18-30-00_Project` or `2026-11-06_18-30-00_Project.zip`. Name collisions receive a numeric suffix. Existing `backup-<UUID>` folders from older versions are left untouched; new checks start a separate history directly in the destination. Old backups remain available for manual restoration and are not included in the new retention history.

Each backup contains the full source contents. To restore a version, copy its files into a folder of your choice or extract the ZIP archive. Check existing data before overwriting it. Removing a job in the app does not delete its backups.

The size limit counts the sizes of stored backup files, or the archive size for ZIP backups, rather than physically allocated disk blocks. After a successful backup, the oldest backups recorded in the manifest are removed. The latest successful backup is always kept; if it alone exceeds the limit, the app displays a notice. Unrelated files and incomplete backups are not deleted automatically. There must be enough free space for the new backup before cleanup begins.

## Integrity and limitations

- SHA-256 covers file contents, relative paths, and empty directories. Timestamp-only changes do not trigger a backup. Each check reads all source file contents.
- Backups are written to a temporary location. Copied or archived contents are verified, then the source is inventoried again before publication. Read/write failures or source changes during copying prevent the backup from being recorded as successful and prevent retention cleanup.
- This is not a file system snapshot. Close applications with continuously changing databases before backing them up, or use their export feature. Unchanged checks compare against the stored manifest; they do not check existing backups again for later corruption.
- Symbolic links, Windows reparse points/junctions, and special files are skipped with a notice. Hard links are backed up as separate files. Source and destination folders must not overlap.
- ACLs, extended attributes, Windows streams, and timestamps are not guaranteed to be preserved. Folder copies on Unix preserve file permissions. ZIP archives provide portable content backups.
- File names must be valid UTF-8. Backslashes in Unix file names are rejected with an error. Encryption is not provided.
- A power failure or forced process termination may leave `.partial-*` entries or published backups that have not yet been recorded in the manifest. These are intentionally not deleted automatically. Settings and manifests are replaced atomically; complete protection against power loss at the storage level is not guaranteed.
- The app works offline. Linux requires a graphical session with OpenGL and standard system libraries, including libc, X11/Wayland, and xkbcommon. The Linux build is not a fully static universal executable; build against a suitably old base system for older distributions. Linux folder selection uses XDG Desktop Portal and its desktop backend.

## Development and builds

Install stable Rust, then run:

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
cargo build --release --locked
```

The output is `target/release/portable-backup` on Linux or `target/release/portable-backup.exe` on Windows. The release profile uses size optimization, LTO, and stripped debug symbols. UI resources are embedded. Each platform requires its own build.

Debian/Ubuntu build dependencies include `build-essential pkg-config libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxinerama-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev`. Windows builds require Rust with the MSVC toolchain and Visual Studio C++ Build Tools.

The GitHub Actions workflow runs formatting checks, tests, Clippy, and release builds on Linux and Windows, and uploads both executables as artifacts. It runs once the project is pushed to a GitHub repository.

Tests cover content changes and renames, empty directories, ZIP readability, links, overlapping paths, failure cleanup, sources changing during backup, rolling retention, missing backups, atomic saves, and scheduling including daylight saving transitions.

### Linux builds targeting older glibc

Use `cargo-zigbuild` with Zig for broader Linux compatibility:

```sh
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.28 --locked
```

The `zig` executable must be in PATH. The output is `target/x86_64-unknown-linux-gnu/release/portable-backup`. Check the actual glibc symbol versions with `readelf --version-info` and launch the release executable. Graphics drivers and desktop libraries must still be available on the target system.

### Linux UI smoke test

```sh
python3 tools/ui_smoke.py target/release/portable-backup
```

Requires an X11/XWayland session, libX11, and ImageMagick `import`. The test creates only temporary jobs, checks automatic execution and persisted completion in both themes, measures memory usage, and captures screenshots of the app window only. It terminates its test processes afterward. Artifacts remain in `/tmp/portable-backup-ui-*` for inspection. To test the Windows executable under Wine, pass its path and add `--wine`.

### Included executables

Local distribution files are `dist/linux-x86_64/portable-backup` and `dist/windows-x86_64/portable-backup.exe`. Each platform needs only its executable; the configuration is created alongside it on first launch. The Linux distribution build targets glibc 2.28 or newer. The Windows executable targets Windows 10/11 x86-64 and statically links its unwind runtime. Both require working system graphics drivers.

The local Windows cross-build uses LLVM-MinGW and the Rust target `x86_64-pc-windows-gnullvm`. After installing these toolchains, run:

```sh
CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER=x86_64-w64-mingw32-clang \
  cargo rustc --release --target x86_64-pc-windows-gnullvm --locked \
  -- -C target-feature=+crt-static
```

LLVM-MinGW tools must be in PATH. The Windows CI workflow uses MSVC with a statically linked C runtime instead.
