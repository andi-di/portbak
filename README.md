# portbak <img src="assets/icon/portbak.svg" width="24" />

Lightweight backup app for Linux and Windows, written in Rust with egui. No browser runtime, server, or installer required.

![Empty portbak window in Retro Dark](assets/screenshots/retro-dark.png)

## Product specification

See [SPECIFICATION.md](SPECIFICATION.md) for standalone, implementation-independent requirements suitable for rebuilding the application in another language.

## Getting started

Copy the executable for your platform into a **writable folder** and launch it. On Linux, run `chmod +x portbak` if necessary. The app creates `portbak.json` (settings and status) and `portbak.lock` (instance lock) alongside the executable. To move your configuration, copy the JSON file too and update source and destination paths. Administrator privileges are not required. Drag the divider beside the job list to adjust its width; the app remembers it across restarts.

1. Select **New job** and choose a name (up to 50 characters) and source folder. By default, the destination is created beside the app under `backups/<job UUID>-<job name>`. Automatic destinations are saved relative to the app directory, use the job name, and follow job renames. Move the executable, `portbak.json`, and `backups` folder together to preserve them. Manual destination paths stay absolute. You can choose a different destination afterward; manual destinations stay unchanged on renames. Each job has an **Auto destination folder** checkbox. Choosing or typing a manual destination unchecks it; checking it again restores the generated destination.
2. New jobs use **Compress as ZIP** by default. Uncheck it for a full folder copy.
3. Select **Check now** to create the first backup. Subsequent backups are created only when file contents change, files are added or removed, paths are renamed, or empty directories change.
4. Jobs start with automatic backups disabled. Press the green **Enable automatic backups** button to arm the schedule: interval (the default, 00:10:00, entered as hours:minutes:seconds), daily, or selected weekdays. Active jobs show a green dot beside their names in the job list and detail heading, and a countdown updated every second in both views. Press the red **Disable automatic backups** button at any time to remove pending work; a running backup finishes safely. **Check now** remains available when automatic backups are disabled. Re-enabling starts a fresh schedule.
5. New jobs enable **rolling retention** with a maximum count of **20**. The total-size limit starts unchecked at **10 GiB**. Both limits can be combined or disabled.

Use **Settings > Job settings** to change these defaults for future jobs. Existing jobs keep their defaults. Automatic destinations always use the job name; older source-named automatic destinations update when opened for editing, preserving backups. Enable **Check immediately when enabled** in a job to queue one check whenever it is enabled, while keeping its next scheduled check in the future. The settings template supplies its initial value for new jobs; it starts unchecked. The **Theme** tab contains look selection.

**The app must be running for scheduled backups.** You can minimize it. Closing the window immediately exits without confirmation, stops scheduling, and terminates running operations. Missed checks are caught up once on startup or resume. Schedules use the local time zone: a time skipped by daylight saving is moved to the first valid minute, and a repeated time runs only once.

Choose **Retro**, **Dark Fantasy**, **Modern**, **Frutiger Aero**, or **Midnight Neko** in Settings. Use the Light/Dark toggle beside Settings in the top toolbar to switch brightness. On first launch, the app uses the system appearance and saves it in `portbak.json`; later launches use the saved choice. The app remembers the selected look too.

Midnight Neko pairs Saya-inspired burgundy and plum-black surfaces with ivory,
silver, and moss-green accents. Its individual pixel icons distinguish actions,
backup types, scheduling options, and navigation in both light and dark modes.
Bold silver silhouettes with subtle bevels, high-contrast light/dark variants and borderless utility
controls improve readability. Buttons choose icon contrast for their own fill.
Alert and resting Saya heads distinguish jobs with automatic backups enabled or disabled. Saya's
pixel head marks the header; her static full-body figure appears on the welcome
screen, while the sidebar cycles through her refined expressions when clicked.
A quiet moonlit rose garden has larger cursor-aware bats that hang from the top
edge on individual timers, scatter into separate roaming areas, and return to
roost after randomized delays. Their near-black light-mode and dark-gray dark-mode colors keep the
bats subdued. The garden also has falling petals and a sparse
petal/crescent pointer trail. Effects are always enabled in this look. Bats use
smooth wing motion and face their direction of travel; varied petals tumble
through the garden. A faceted silver arrowhead and matching pointing gauntlet
style the normal and clickable cursors. Saya's sidebar
expression stays while hovered and returns to neutral three seconds after the
pointer leaves, with varied dialogue for each expression. She also reacts on
randomized timers and to job and settings actions; hovering freezes her reaction
and dialogue until the pointer leaves.
The footer's silver crescent and crimson gem animate during backup activity,
with progress available in its tooltip.

On Windows, the app icon is embedded in the executable for Explorer and shortcuts, as well as set on the app window. No separate icon file is needed.

On Linux, the embedded window icon is used directly under X11. Under Wayland, portbak also offers it to compositors supporting `xdg_toplevel_icon_v1` (including current KDE Plasma). This is optional: if unsupported or unsuccessful, the app keeps the compositor’s default icon and starts normally. No `.desktop` file, separate icon file, or installation is needed.

## Storage format and recovery

Backups are stored directly in the selected destination, alongside a per-job `manifest-<UUID>.json` and a destination `.lock` file. Backups are named `2026-11-06_18-30-00_Project` or `2026-11-06_18-30-00_Project.zip`. Name collisions receive a numeric suffix.

Each backup contains the full source contents. Open the job’s **Backups** tab, click the restore icon beside a saved folder or ZIP version, then confirm in the modal with **Replace contents and restore**. This replaces **all current contents of the source folder**, including additional files and subfolders. Close programs that write to that folder first. The job remains disabled after restoration until you enable it again. Restore is available when no operation is running or queued; scheduling pauses while the Backups tab is open.

The selected version is prepared and fingerprint-verified beside the source before replacement. Enough free space and write access to the source's parent folder are required. If replacement fails, the app attempts to restore the previous contents. If rollback or cleanup fails, the status reports the recovery folder path; keep that folder until you have recovered your data. A crash may leave `.portbak-restore-*` or `.portbak-previous-*` folders beside the source. Folder contents and empty directories are restored; metadata limitations below still apply.

You can also restore manually by copying a backup folder's contents or extracting a ZIP. Removing a job in the app does not delete its backups.

The size limit counts the sizes of stored backup files, or the archive size for ZIP backups, rather than physically allocated disk blocks. After a successful backup, the oldest unprotected backups recorded in the manifest are removed. Use the **Locked** checkbox beside each saved version on the **Backups** tab to protect it from automatic deletion or remove its protection. Changes are saved immediately; removing protection does not immediately delete anything. The per-job **Count protected backups toward retention limits** checkbox controls whether protected versions count toward the count and size budgets. It defaults to unchecked and is also available in the settings template. Protected versions are never deleted by retention. The newest entry of each local day is subtly highlighted in the Backups tab. The latest successful backup and all protected versions are always kept. If the latest backup alone exceeds the size limit, the app displays a notice. Unrelated files and incomplete backups are not deleted automatically. There must be enough free space for the new backup before cleanup begins. With rolling retention and counting protected backups enabled, warnings appear in the overview and job details when protected versions leave at most three count places or at most 20% of the size budget. Limits may remain exceeded when protected versions and the newest backup cannot be removed. These warnings describe the configured retention budget, not free disk space.

## Integrity and limitations

- SHA-256 covers file contents, relative paths, and empty directories. Timestamp-only changes do not trigger a backup. Each check reads all source file contents.
- Backups are written to a temporary location. Copied or archived contents are verified, then the source is inventoried again before publication. Read/write failures or source changes during copying prevent the backup from being recorded as successful and prevent retention cleanup.
- This is not a file system snapshot. Close applications with continuously changing databases before backing them up, or use their export feature. Unchanged checks compare against the stored manifest; they do not check existing backups again for later corruption.
- Symbolic links, Windows reparse points/junctions, and special files are skipped with a notice. Hard links are backed up as separate files. Source and destination folders must not overlap.
- ACLs, extended attributes, Windows streams, and timestamps are not guaranteed to be preserved. Folder copies on Unix preserve file permissions. ZIP archives provide portable content backups.
- File names must be valid UTF-8. Backslashes in Unix file names are rejected with an error. Encryption is not provided.
- A power failure or forced process termination may leave `.partial-*` entries or published backups that have not yet been recorded in the manifest. These are intentionally not deleted automatically. Settings and manifests are replaced atomically; complete protection against power loss at the storage level is not guaranteed.
- The app works offline. Linux requires a graphical session with OpenGL and standard system libraries, including libc, X11/Wayland, and xkbcommon. The Linux build is not a fully static universal executable; build against a suitably old base system for older distributions. Linux folder selection uses XDG Desktop Portal and its desktop backend. Opening backup locations or configured folders in the file manager requires `xdg-open` on Linux.

## Contributing and branch rules

Start each change on a dedicated branch from current `origin/main`, using names such as `feat/<topic>`, `fix/<topic>`, or `chore/<topic>`. Commit related changes separately with conventional commit messages, push the branch, and open a pull request to `main`. Do not commit or push directly to `main`. Agents can open PRs but must not merge them. Contributors with repository write access may merge PRs after both required CI builds pass.

The intended GitHub protections are versioned in [.github/rulesets](.github/rulesets). `main-pr-and-ci` requires a PR and successful GitHub Actions checks `build (ubuntu-22.04)` and `build (windows-latest)` against an up-to-date branch, and blocks force pushes and branch deletion without exceptions. All contributors with repository write access may merge passing PRs; no owner-only merge restriction or bypass is configured. No approving review is required.

These JSON files document the configuration; committing them does not activate GitHub settings. An authenticated repository administrator must apply them through GitHub Settings → Rules → Rulesets (Import a ruleset), or the GitHub REST API. Existing rulesets must be updated by ID rather than duplicated. Verify `main-pr-and-ci` is active and targets `refs/heads/main`. Remove the legacy `main-owner-merges` ruleset if present. Do not add broad bypass permissions or grant apps permission to bypass them. GitHub authorizes accounts, so the agent merge prohibition also applies when an agent uses a contributor’s credentials.

## Development and builds

Install stable Rust (1.92 or newer), then run:

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
cargo build --release --locked
```

The output is `target/release/portbak` on Linux or `target/release/portbak.exe` on Windows. The release profile uses size optimization, LTO, and stripped debug symbols. UI resources are embedded. Each platform requires its own build. The icon master is `assets/icon/portbak.svg`; regenerate the PNG sizes and `portbak.ico` with `python tools/make_icons.py` (requires Pillow and librsvg’s `rsvg-convert`, available on Debian/Ubuntu as `librsvg2-bin`). The retro gaming icon shows a gold folder (`#FFD23F`) and cyan backup arrow (`#2CE8F5`) with dark outlines (`#0E081D`) on a filled purple circle (`#2B1E52`) with a lighter purple rim (`#4C3B85`). The area outside the circle is transparent. The SVG uses a finer 64 × 64 pixel grid; exports use nearest-neighbor scaling to preserve sharp pixel edges.

Debian/Ubuntu build dependencies include `build-essential pkg-config libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxinerama-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev`. Windows builds require Rust with the MSVC toolchain and Visual Studio C++ Build Tools with the Windows SDK resource compiler (`rc.exe`) to embed the executable icon.

The GitHub Actions workflow runs formatting checks, all tests, Clippy, and release builds on Linux and Windows, and uploads both executables as artifacts. It runs for pull requests, pushes to `main`, and manual dispatches. Feature-branch pushes are checked through their PR, avoiding duplicate push/PR runs. New commits cancel obsolete runs for the same PR or branch. Both platform jobs finish independently so one failure does not hide the other result.

Rust dependencies and build artifacts are cached per platform, compiler, dependency set, and build environment. Warm runs avoid rebuilding unchanged dependencies; the first run still needs to populate its cache. All tests and both release builds remain enabled. The test suite exercises restore/rollback, retention, integrity and scheduling; observed CI test execution is a few seconds, with compilation accounting for most of the `cargo test` step.

Tests cover folder/ZIP restore, full source replacement, rollback and recovery, unsafe archives, content changes and renames, empty directories, ZIP readability, links, overlapping paths, failure cleanup, sources changing during backup, rolling retention, missing backups, atomic saves, and scheduling including daylight saving transitions.

### Linux builds targeting older glibc

Use `cargo-zigbuild` with Zig for broader Linux compatibility:

```sh
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.28 --locked
```

The `zig` executable must be in PATH. The output is `target/x86_64-unknown-linux-gnu/release/portbak`. Check the actual glibc symbol versions with `readelf --version-info` and launch the release executable. Graphics drivers and desktop libraries must still be available on the target system.

### App source layout

`src/app.rs` owns application state and background operations. Modules under `src/app/` are grouped by responsibility:

- `views/`: main window, job editor, settings, backup history, and shared view widgets.
- `rendering/`: controls, icons, decorations, glass surfaces, and GPU effects.
- `scenery/`: animated wallpaper, aquarium, bubbles, wildlife, and splashes.
- `theme/`: style definitions and the `skins/` registry and presets.
- `platform/`: platform integration, including the optional Wayland window icon.

### Linux UI smoke test

```sh
python3 tools/ui_smoke.py target/release/portbak
```

Requires an X11/XWayland session, libX11, libXtst, and ImageMagick `import`. The test creates only temporary jobs, checks automatic execution at a five-second interval, protection toggles and GUI restore/cancel for folder and ZIP versions, and persisted completion in both themes, measures memory usage, and captures only the app window at its default and minimum sizes. It terminates its test processes afterward. Artifacts remain in `/tmp/portbak-ui-*` for inspection. To test the Windows executable under Wine, pass its path and add `--wine`.

### Included executables

Local distribution files are `dist/linux-x86_64/portbak` and `dist/windows-x86_64/portbak.exe`. Each platform needs only its executable; the configuration is created alongside it on first launch. The Linux distribution build targets glibc 2.28 or newer. The Windows executable targets Windows 10/11 x86-64 and statically links its unwind runtime. Both require working system graphics drivers.

The local Windows cross-build uses LLVM-MinGW and the Rust target `x86_64-pc-windows-gnullvm`. After installing these toolchains, run:

```sh
CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER=x86_64-w64-mingw32-clang \
  cargo rustc --release --target x86_64-pc-windows-gnullvm --locked \
  -- -C target-feature=+crt-static
```

LLVM-MinGW tools must be in PATH. The Windows CI workflow uses MSVC with a statically linked C runtime instead.
