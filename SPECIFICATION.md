# portbak — Product Specification

Version: 1.0  
Language: English  
Purpose: Standalone requirements for rebuilding the application with any implementation language or desktop UI toolkit.

This document defines the product's observable behavior. It does not prescribe a programming language, framework, application architecture, build system, or dependency set. A new implementation must satisfy the requirements below without needing the existing source code. Binary compatibility with the existing application's settings and manifests is not required.

## 1. Product goal

Provide a lightweight, portable desktop application that creates scheduled local backups of user-selected folders. Create a new backup only when the source contents differ from the latest successful backup of that job. Offer optional ZIP compression and optional limits on retained backup count and total size.

The intended user wants to select folders, configure a schedule, and inspect backup status without using a terminal or installing a service.

## 2. Supported platforms and portability

- Support Linux x86-64 and Windows 10/11 x86-64 with equivalent functionality.
- Prefer one executable per platform with application resources included. Each platform has its own executable.
- Launch without an installer or administrator privileges from a writable folder.
- Store settings and operational state beside the executable so the application and its configuration can be moved together.
- Small auxiliary settings and lock files are allowed. Backups are stored separately in the destination selected by the user.
- Do not silently fall back to a different settings directory if the executable's directory is not writable. Display a clear startup error.
- Work offline, without accounts, cloud services, telemetry, or mandatory network access.
- Keep executable size, memory use, and idle CPU activity low. Measure and document release size and memory usage; no fixed numerical limits are prescribed.
- Document required operating system facilities and the supported Linux compatibility baseline.

## 3. UI and interaction

All application-owned interface text, errors, defaults, comments, and documentation must be in English. User-provided names and filesystem paths must remain unchanged. Operating system dialogs may follow the system language.

### Main window

Provide:

- A job list with job name, schedule enabled/disabled state, running/queued indication, and latest result. Its divider remains at the selected width after dragging and is restored after restarting.
- An editor for the selected job. Show a green status dot beside enabled job names in both the list and the detail heading, including while waiting for the next check. Hide it when automatic backups are disabled; an inactive job running a manual check does not show the dot.
- A global theme selector: **System**, **Light**, **Dark**. Default: **System**. Persist the selection.
- A look selector in the settings view: **Retro** (pixel fonts and arcade colors), **Dark Fantasy** (serif capitals, engraved icons, forged plate buttons, crimson on black, drifting embers), **Modern** (neutral Zinc surfaces, blue accents, sans-serif typography, rounded controls and simple line icons; job counts, ZIP icons, last-backup readouts and job icons use neutral text colors) and **Frutiger Aero** (glossy icons, clear glass and gel buttons on frosted glass panes, dialogs, menus and tooltips that blur what lies behind them (with OpenGL 3; tinted see-through glass otherwise), with bevelled edges, reflections and shimmer, over an animated scene: sunburst and lens flare or stars and aurora by night, clouds, a spinning glass globe, gulls, schooling fish in five shapes and two colors each that chase and gulp bubbles, breathe bubbles and flee the pointer (pufferfish blow up), falling leaves, hills, a rolling sea and rising bubbles that pop under the pointer; clicks send a refracting shockwave (plain ripples without OpenGL 3) and the pointer trails bubbles), and **Midnight Neko** (opaque matte charcoal/crimson panels in Dark mode and pale-stone panels in Light mode, silver lace-like corner filigree, thorn-and-diamond borders, crescent motifs, subtly patterned panels, flat controls, serif headings, and a matching pixel-art black-haired catgirl face icon. One adult black-haired, green-eyed gothic catgirl uses her full-figure avatar only in the spacious welcome view; use close-up portraits in the sidebar when space permits. Normalize every jobs-view portrait to the same 384 � 448 resolution, eye-line position and face scale while preserving the approved head tilts and hand pose. Clicking her selects one of thirteen illustrated reactions/poses without repeating the previous selection: angry, `:3` pout, excited, happy, surprised, superior, heart eyes, cold stare, sleepy with drool, bored, cheek resting on a hand with a sideways glance, wide-eyed yandere with tiny pupils, or shy. Each lasts four seconds before returning to the neutral default. Preserve the detailed illustrated eyes; no cursor-tracking overlays. Glossy red hearts pop under the pointer and a sparse fading heart-and-stardust trail has small sparkle bursts on clicks. Decorations do not intercept input; only the companion portrait accepts clicks). Default: **Retro**. Each look has a light and a dark variant that follows the theme selector. Persist the selection; an unknown saved look falls back to the default.
- A status area showing current activity, progress, and settings-save errors.
- An empty state explaining how to create the first job.

The interface must remain responsive during hashing, copying, compression, verification, and cleanup. Long content must be scrollable. All controls must remain accessible at smaller window sizes. Vertically center all elements within each horizontal row. At narrow widths, place the job activation control below the job heading so it remains accessible. The header shows the application name and theme selector without a slogan. Default to an 1100 × 960 logical-pixel window, with room below the job controls.

The application remains a single distributable executable with an embedded icon. Under Linux, offer that icon directly to X11 and to Wayland compositors supporting window-icon pixel data. Missing support, icon setup errors, or an unresponsive icon negotiation must not block startup, show an error dialog, or prevent normal use; retain the compositor default icon. Do not install desktop entries or extract icon files.

### Job actions

- **New job**: create and select a new independent job.
- **Check now**: queue a check regardless of whether automatic scheduling is enabled. This is not an unconditional backup command.
- **Restore backup…**: when no operation is active or queued, asynchronously list this job's recorded versions newest first with date, name, size, and folder/ZIP type. Mark missing versions unavailable. Pause automatic starts while the dialog is open. Show the full resolved source path, selected version, and an explicit warning that all current source contents will be replaced. Require a red **Replace contents and restore** confirmation; cancellation changes nothing.
- **Manage backups…**: beside Restore, open the same history overview when no operation is running or queued, without requiring an available source folder. Pause automatic starts while open. Show each saved version’s protection checkbox and save changes in the background. Disable further edits and closing while saving; report errors without pretending the protection changed. Closing preserves job activation. Missing versions are marked unavailable, but existing protection can be removed.
- At the default main-window size, the restore dialog must show its instructions, full source path, selected-version details, warning, and action buttons without outer scrolling in every look, both before and after selecting a backup. The backup list may scroll independently. Keep outer scrolling available for smaller windows or unusually long content.
- In both history dialogs, subtly highlight the first list entry of each local calendar day, newest first. Keep selection and protection independently recognizable. Wrap job action buttons on narrow windows.
- Show retention capacity warnings in the management overview and job details; refresh after protection changes, limit changes, and backup completion.
- On confirmed restore, persist this job as disabled with no next scheduled time before starting work. It stays disabled after success or failure. Run restore through the shared background worker with progress and error status. Prevent edits and manual starts while restoring; other jobs resume scheduling afterward.
- Enable or disable automatic scheduling for an individual job using a prominent green enable button or red disable button.
- **Remove job**: remove the job configuration without deleting any existing backups.
- Save changes automatically and preserve them across restarts.
- Prevent editing or removing a running or queued job until its pending work finishes, except that automatic scheduling can be toggled at any time. Disabling removes queued work for that job but lets a running backup finish. Manual checks remain available when scheduling is disabled.

### Window lifecycle

Minimizing keeps the application and scheduler running. Closing the window immediately exits without confirmation, including while a backup or restore is running. Attempt to save settings before exit; a save failure is logged to stderr and does not block closing. Exit stops scheduling and terminates background operations without waiting for completion. Interrupted operations have the same recovery limitations as forced process termination. No system tray or always-running service is required.

## 4. Job configuration and defaults

Each job is independent and has a persistent unique identifier. Jobs may share source folders or a destination parent directory; their managed backups must remain separate.

| Setting | Behavior and default |
| --- | --- |
| Name | Editable display name; default **New job** |
| Source folder | Existing folder selected through a folder picker or editable path; initially empty |
| Destination folder | Existing folder selected through a folder picker or editable path; initially empty |
| Automatic scheduling | Disabled by default |
| Schedule mode | Interval by default; display choices in order: Interval, Daily, Weekdays |
| Local time | 18:00 by default; hours 0–23, minutes 0–59 |
| Selected weekdays | All seven selected by default; used only in weekday mode |
| Interval | 60 Minutes by default; positive whole values with selectable Seconds (1–31,536,000), Minutes (1–525,600), or Hours (1–8,760) |
| Compress as ZIP | Checkbox; disabled by default |
| Rolling retention | Disabled by default |
| Maximum count | Independently enabled; initial value when enabled: 10; positive whole number, supporting at least 1–100,000 |
| Maximum total size | Independently enabled; initial value when enabled: 10 GiB; positive value, supporting at least 0.001–100,000 GiB |

A GiB is 1,073,741,824 bytes. Count and size limits are remembered while rolling retention is disabled, but are not enforced. Enabling rolling retention without an active limit does not delete anything.

Also persist the next scheduled check, last successful backup time, and latest job result. The initial result is **Not checked yet**. An unchanged check must not update the last successful backup time.

## 5. Scheduling

### Modes

1. **Interval:** check every configured number of seconds, minutes, or hours. Changing the unit preserves the numeric value, clamped to the new unit’s supported range. Store the value and selected unit independently.
2. **Daily:** check once per day at the configured local time.
3. **Weekdays:** check at the configured local time only on selected weekdays. Display weekdays Monday through Sunday. Require at least one selected weekday before enabling automatic execution.

### Execution rules

- Scheduling works only while the application is running, including when minimized. Do not register system tasks, install services, or wake an off computer.
- Disabling clears the next scheduled time. Each activation, including reactivation, starts a fresh future schedule.
- Enabled jobs show a countdown to the next automatic check in both the job list and details, updated every second. Format as HH:MM:SS with a day prefix for durations of at least 24 hours; clamp overdue values to zero. Keep the absolute next check time in the details. Running/queued indicators remain visible.
- On first enabling a schedule, plan the next future occurrence; do not immediately run a backup solely because scheduling was enabled.
- Recalculate the next future occurrence when the user changes schedule settings.
- Persist scheduled due times across normal restarts.
- If the saved due time is overdue after startup or resume, queue exactly one catch-up check. Do not queue one check for every missed occurrence.
- After handling a due occurrence, calculate the next future occurrence. For intervals, schedule the next check one interval after the time the overdue/due occurrence is handled, rather than replaying missed intervals.
- Manual checks do not reset the automatic schedule.
- Run only one backup job at a time across the application. Queue other jobs in arrival order.
- Coalesce repeated triggers for the same job. A running or already queued job must not acquire another queued copy.
- Automatic triggers must not create overlapping work or interrupt an active backup.
- A failed check reports its error and leaves the next normal schedule in place. No immediate retry loop is required.

### Local time and daylight saving

Use the computer's local time zone for calendar schedules. If a configured local time does not exist because the clock moves forward, execute at the first valid local minute afterward. If a configured time occurs twice because the clock moves backward, execute only on its first occurrence that day.

## 6. Source validation and file handling

Before a check:

- Require nonempty source and destination selections and confirm both folders are available.
- Resolve actual filesystem locations. Reject identical folders and any case where one is inside the other, including overlap through a path alias.
- Never modify source contents.
- Store backup folders or ZIP files directly in the selected destination, without an extra job subfolder. Use a separate `manifest-<job-id>.json` for each job and lock the destination while checking, publishing, and pruning.
- Reject links or directory redirections for manifest and lock files.

Include all ordinary files and directories recursively, including hidden files and empty directories. There are no include/exclude filters in this version.

Skip symbolic links, Windows junctions/reparse points, and special filesystem entries such as device nodes, sockets, and named pipes. Never follow them into other directories. Show the number of skipped entries in the result. Treat hard links as separate ordinary files.

Support valid UTF-8 names. Reject unsupported names with an actionable error instead of silently dropping or renaming them. For this version, backslashes in Unix filename components are unsupported. Preserve original relative names and directory structure in every backup.

Folder copies on Unix preserve regular file permissions. Full preservation of ACLs, extended attributes, alternate data streams, timestamps, ownership, directory metadata, or ZIP permissions is outside the guaranteed behavior.

## 7. Change detection

For every manual or scheduled check:

1. Read the contents of every included source file and calculate its SHA-256 content hash.
2. Build a deterministic inventory of relative paths, entry types, file sizes, file content hashes, and directories, including empty directories.
3. Order inventory entries consistently and derive an overall fingerprint.
4. Compare it with the fingerprint of the latest successful managed backup for this job in the current destination.

The fingerprint must not depend on filesystem enumeration order, absolute source location, file timestamps, or permissions.

Create a new backup when:

- No successful backup exists yet.
- File contents change, including same-length edits.
- A file or directory is added, removed, or renamed.
- An empty directory is added or removed.
- The latest recorded backup has been removed from the destination.

If the fingerprint matches and the latest recorded backup still exists, report **Unchanged** and create no new backup. Do not run retention cleanup on an unchanged check. Existing backup files do not need to be rehashed on every unchanged check.

A change to ZIP or retention settings alone does not force a backup. ZIP selection applies to the next backup triggered by source changes. All source data must be read for hashing; timestamp/size-only shortcuts must not hide changes.

## 8. Backup format and naming

Each successful backup is a complete, independent snapshot of the included source contents. There are no incremental chains or dependencies on older backups.

- Default format: a directly accessible folder copy.
- Optional format: a standard compressed ZIP archive, readable by ordinary extraction tools.
- Name format: `YYYY-MM-DD_HH-MM-SS_<source-folder-name>` using the local creation time.
- ZIP example: `2026-11-06_18-30-00_Project.zip`.
- Folder example: `2026-11-06_18-30-00_Project`.
- Preserve all existing backups. If a name already exists, add `_1`, `_2`, and so on before the optional extension.
- The backup folder or archive contains the source's contents with their relative directory structure, including empty directories.

Restoration must also remain possible without this application by copying a backup folder's contents or extracting an archive.

### Restore execution

- Restore only a version recorded in the selected job's manifest, from a folder or ZIP. Preserve the source folder itself but replace all its contents, including hidden entries, extra files, and extra directories.
- Reject missing or damaged backups, foreign manifests, overlapping paths, linked source roots, filesystem roots, links or special files in backups, unsafe ZIP paths, and duplicate normalized ZIP entries.
- Prepare the chosen backup in a temporary sibling folder and verify its content fingerprint before modifying the source. This requires writable space beside the source and enough space for the prepared version in addition to the current contents.
- Move current contents into a sibling recovery folder, install and verify the prepared version, then remove the previous contents without following links. If replacement fails, roll back; if rollback cannot complete, preserve original data and report the recovery folder path.
- Preparation failures leave the source untouched. If removal of old recovery data fails after successful replacement, report that path and the cleanup error. A crash can leave `.portbak-restore-*` or `.portbak-previous-*` folders; complete crash atomicity is not guaranteed.
- Restore must not change backup history, retention, or the last successful backup time. Existing metadata limitations apply; close applications that write to the source before restoring.

## 9. Integrity and failure handling

A backup operation must proceed in this order:

1. Validate paths and job ownership; prevent concurrent execution against the same managed job destination.
2. Inventory and hash the source; return without creating a backup if unchanged.
3. Write a new backup to a uniquely named temporary location within the job destination.
4. Verify the copied contents or archived file contents against the initial inventory.
5. Inventory the source again. If it changed during the operation, reject this attempt.
6. Publish the verified backup under its final name and persist its successful record.
7. Only then apply rolling retention.

Display the current phase and useful progress, such as files processed and current relative path. Report unavailable folders, unreadable files, write failures, insufficient space, verification mismatches, and source changes clearly.

On failure before successful publication/recording, do not advance the comparison baseline or delete previous successful backups. Clean up temporary output where possible. If retention fails after a successful backup, retain the new backup and report successful creation with a separate cleanup warning.

This application does not provide a transactional filesystem snapshot. Explain that continuously changing databases should be closed or exported before backup.

After a crash, forced termination, or power loss, temporary output or a published but unrecorded backup may remain. Do not automatically treat it as successful or delete it as part of retention. Do not promise full storage-level protection against power loss.

## 10. Rolling retention

Rolling retention is configured per job and disabled by default. Maximum count and maximum total size can be enabled separately or together.

After a newly successful backup:

- Count only successful backups owned by this job and recorded in its manifest.
- Evaluate both active limits, counting protected versions as well. If either is exceeded, delete the oldest unprotected managed successful backup other than the newest version, repeating until the limits are satisfied or no eligible version remains.
- Use successful creation order to determine the oldest backup.
- For folder backups, total size means the sum of regular file lengths. For ZIP backups, use archive file length. Do not use source size, allocated disk blocks, settings, or unrelated files.
- Always keep the newest successful backup. If it alone exceeds the size limit, retain it and show a notice.
- Never delete protected versions, unrelated files, another job's backups, temporary output, unrecorded output, or the latest successful backup.
- Never follow links when deleting managed backup entries. Refuse unsafe deletion and report a cleanup warning.

Each saved version has a persistent protection flag, false by default (including records that predate the flag). Changing this flag must preserve all other metadata and must not trigger immediate cleanup. Use destination locking and atomic manifest replacement for protection changes.

When rolling retention is enabled and at least one version is protected, warn if the count limit minus the protected count is at most three places, or if at most 20% of the size limit remains after subtracting protected backup sizes. Show concrete remaining values, clamp negative values to zero, and separately report exceeded limits. These are retention budgets, not free disk space; warnings do not block new backups. Protected versions and the newest backup can cause limits to remain exceeded.

Do not delete old backups in advance to make room for a new one. If there is not enough space to create the new backup first, report failure and retain previous backups.

## 11. Persistent state and instance safety

Persist application preferences and all job settings automatically. Persist successful backup records separately in each managed job destination. Each record needs enough information to identify its owned backup, content fingerprint, creation time/order, and stored size.

- Version settings and backup metadata so unsupported formats can be rejected clearly.
- Save settings and manifests using whole-file replacement semantics that do not expose partially written documents during normal operation.
- Report corrupt or unsupported metadata; do not silently overwrite it or delete files based on it.
- Validate stored backup names before using them for filesystem operations. Reject absolute paths and path traversal outside the managed job directory.
- Prevent simultaneous application instances from using the same portable settings state.
- Prevent two processes or portable copies from running the same job against the same managed destination at once.
- Preserve existing backups when removing jobs, editing paths, or moving the application. Changing destination uses the backup history in the newly selected destination; it does not move or delete the previous history.

No exact on-disk serialization or existing-file migration format is prescribed. The new implementation must document its own storage layout and recovery procedure.

## 12. Acceptance scenarios

Use temporary fixtures for automated tests and keep real user data untouched.

| Scenario | Expected result |
| --- | --- |
| First launch | No jobs; System theme; clear empty state |
| New job | Defaults match section 4; ZIP, scheduling, and retention are off |
| First manual check | One full backup with timestamp prefix and correct contents |
| Unchanged second check | No additional backup; Unchanged result; last backup time unchanged |
| Same-length file edit | New backup despite equal file length |
| Timestamp-only or permission-only change | No new backup |
| File add, delete, or rename | A new backup for each changed inventory |
| Empty directory add or delete | A new backup reflecting the directory change |
| Enumeration order changes | No false content change |
| Hidden files and nested folders | Contents and relative paths preserved |
| ZIP enabled | Standard readable archive, correct file contents and empty directories |
| ZIP toggled without source changes | No new backup solely because format changed |
| Two backups within the same second | Unique names; neither existing backup overwritten |
| Latest backup removed externally | Next check recreates a full backup |
| Linked directory loops or external links | Links skipped; no recursion or external content copied |
| Identical or overlapping source/destination | Clear error; no backup or source modification |
| Missing source/destination or failed read/write | Clear error; old backups and comparison baseline preserved |
| Source changes while copying/compressing | Attempt rejected; no successful record or retention deletion |
| Retention disabled with configured limits | All successful backups retained |
| Count limit | Oldest managed backups removed only after new success |
| Size limit | Stored folder/archive sizes used; oldest removed first |
| Combined limits | Both enforced; newest successful backup preserved |
| Newest backup alone too large | Kept with an explicit size-limit notice |
| Unrelated files in destination | Never removed by retention |
| Cleanup failure | New successful backup kept; cleanup warning shown |
| Due schedule and manual trigger collide | One active/queued instance of the job |
| Multiple jobs due together | Sequential execution; no overlapping jobs |
| App restart or resume after missed checks | Exactly one catch-up check, then a future due time |
| New schedule enabled | First execution planned for the future |
| DST missing/repeated local time | First valid minute / first occurrence only |
| App minimized | Schedule and progress continue |
| Quit request while idle | Save settings and exit immediately without confirmation |
| Quit request while backup or restore runs | Exit immediately without waiting for completion |
| Second instance or destination lock conflict | Clear refusal; no concurrent writes |
| GUI restore selected folder or ZIP | Entire source contents replaced; extra and hidden entries removed; saved empty directories restored |
| Restore cancelled | Source and job activation unchanged |
| Damaged, missing, unsafe or foreign backup | Restore refused before source replacement |
| Restore replacement fails | Original contents restored, or remaining recovery path reported without deleting original data |
| Restore completes or fails | Job remains disabled; history and last backup time unchanged |
| Protected versions with exceeded limits | Keep protected and newest versions; delete only eligible unprotected versions; report remaining excess |
| Protection consumes count/size budget | Warn at three remaining places or 20% remaining size budget, inclusive |
| History crosses local midnight | Highlight the first listed version of each local day in both dialogs |
| Narrow window with a wide job list | Selected job remains visible and usable without a crash |
| Resize job list, then release and restart | Divider stays at the chosen width and is restored after restart |
| Restart after normal edits | Settings, theme, next due time, and results restored |
| Corrupt metadata or unsafe stored path | Explicit error; no destructive fallback |
| Light and Dark themes | All job controls and results readable in each theme |

Verify the user interface, folder selection, window lifecycle, and scheduling on both target operating systems. Document any checks performed only under compatibility software rather than on a native system.

## 13. Deliverables and scope boundaries

Deliver the complete source project, runnable portable executables for both platforms, automated tests, English usage/recovery instructions, measured release size and memory usage, and a verification report describing platform coverage and remaining limitations.

The initial version does not require cloud storage, encryption, compression formats other than ZIP, deduplication, incremental backups, file exclusion rules, live filesystem watching, system task integration, automatic startup, a tray icon, or guaranteed preservation of all filesystem metadata.

When rebuilding from this document, implement the complete behavior and acceptance scenarios before adding optional features. Document any unavoidable platform limitation rather than silently changing backup or deletion rules.
