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

- A job list showing only each job’s icon, name, and right-aligned next-check countdown in one vertically centered row. Box the countdown like the scheduling readout, without a text prefix; hide it when scheduling is disabled. Its divider remains at the selected width after dragging and is restored after restarting.
  - While a job runs, restores, or waits in the queue, show RUNNING, RESTORING, or QUEUED in the countdown's place. Hold the label as long as the footer activity so quick checks do not flash.
- Clicking the selected job deselects it and returns to the start screen.
- At the top of the job editor, place the power and **Check now** buttons on the left, **Remove job** on the right, and center the job icon and name. In narrow views, center the identity above the action row. Omit the redundant activation-status text.
- Show the last backup time and latest result in a compact status row below the job header; show errors prominently. Do not use a separate Status pane.
- An editor for the selected job.
- Settings, Light/Dark, Choose folder, and automatic-backup power buttons use icons with descriptive hover tooltips and accessible names. Preserve selected-state indicators.
- Place an icon-only open-in-file-manager button beside each job source and destination folder chooser. Enable it only for an existing folder; provide a tooltip and report launch failures.
- Opening Settings clears the selected job. Closing Settings returns to the main screen. Selecting a job closes Settings and opens that job; clicking the selected job again deselects it and returns to the main screen. There is no Back to jobs button.
- A **Light/Dark** toggle in the top toolbar next to **Settings**. On first launch without `portbak.json`, resolve the system appearance and persist **Light** or **Dark**; later launches use the saved choice. Resolve legacy **System** settings once, then persist the result.
- A look selector in the settings view: **Retro** (pixel fonts and arcade colors), **Modern** (neutral Zinc surfaces, blue accents, sans-serif typography, rounded controls and simple line icons; job counts, ZIP icons, last-backup readouts and job icons use neutral text colors) and **Frutiger Aero** (glossy icons, clear glass and gel buttons on frosted glass panes, dialogs, menus and tooltips that blur what lies behind them (with OpenGL 3; tinted see-through glass otherwise), with bevelled edges, reflections and shimmer, over an animated scene: sunburst and lens flare or stars and aurora by night, clouds, a spinning glass globe, gulls, schooling fish in five shapes and two colors each that chase and gulp bubbles, breathe bubbles and flee the pointer (pufferfish blow up), falling leaves, hills, a rolling sea and rising bubbles that pop under the pointer; clicks send a refracting shockwave (plain ripples without OpenGL 3) and the pointer trails bubbles), and **Midnight Neko** (opaque matte plum-black/burgundy panels in Dark mode and warm ivory panels in Light mode, with silver, muted rose and moss-green accents, silver lace-like corner filigree, thorn-and-diamond borders, crescent motifs, subtly patterned panels, flat controls, serif headings, and individually generated, simple function-specific pixel silhouettes with matching light/dark silver variants selected for their actual control background, with shallow inset bevels and broad metallic shading that preserve contrast. Icons share a 128 × 128 transparent canvas and a centered, proportionally fitted 104-pixel maximum symbol extent. Saya uses a static full-body pixel render on the welcome screen, her family emblem beside the app name, and individual refined pixel busts inside an ornate silver and burgundy frame in the sidebar). Default: **Retro**. Each look has a light and a dark variant that follows the theme selector. Persist the selection; an unknown saved look falls back to the default.
- Midnight Neko uses 32-pixel utility and job action icons and a 40-pixel New job icon, with contrast adapted to the control foreground. Utility frames appear on hover, keyboard focus, or selection; important and destructive actions remain framed.
- Midnight Neko Saya reacts after randomized 18–55 second delays. Timers wait for neutral. Outcome reactions take priority over navigation, with a two-second cooldown.
  - Successful backup completion and no-change checks do not trigger reactions. Failures trigger Sad.
  - Enable automatic backups: Energetic; disable: Pouting; restore confirmation: Question; restored: Happy; protect a backup: Dominant.
  - New job: Happy; Check now: Energetic; Remove job: Pouting; Settings: Curious; select job: Waiting; Dark mode: Happy; Light mode: Sleepy.
- When Saya enters her in-love expression, three ruby hearts briefly rise and fade inside the sidebar frame. Holding the expression does not replay the effect.
- Saya's dialogue is centered in a letterbox attached to her portrait, with matching silver and burgundy rails, filigree corners, and a ruby seal. Size the portrait to leave room for wrapped dialogue.
- Midnight Neko uses Saya's ornate Kurobane family emblem beside the app and job names at 56 and 80 logical pixels respectively. The 72-pixel job status heads retain the same reference colors in both modes and use separately rendered play and pause badges.
- Midnight Neko job cards use distinct Saya heads for jobs with automatic backups enabled (open eyes and small fangs visible beneath closed lips with a play badge) or disabled (sleeping Saya with a pause badge).
- Midnight Neko sidebar clicks select among all 26 refined non-neutral Saya expressions without immediate repeats. Direct clicks switch immediately, including while hovered or during a cooldown. Keep expression and dialogue unchanged while hovered otherwise, deferring automatic and action reactions until the pointer leaves; return to neutral after three seconds away. The welcome figure stays static and does not react to clicks. Fit the square frame to the sidebar width and remaining height, allowing space for dialogue and a scrollable job list.
  - Each expression, including neutral, has at least four matching dialogue lines. Pick a line on expression changes, keep it stable while visible, and avoid repeating that expression's previous line during the session.
- Midnight Neko has an asymmetric moonlit vampire estate with a castle and winding stairs at lower right, a silver crescent below the topbar, detailed layered rocky mountains, thorn roses, two sculpted pixel-art cloud variants drifting at individual speeds and layered curling mist, and subtly flickering burgundy windows, twelve bats that fly, perch, avoid the pointer and scatter on clicks, varied falling petals, and short-lived petal/crescent/silver-spark cursor trails. Decorations never intercept controls. Preserve native text, resize and other specialized cursors.
  - The pointing hand appears over clickable controls and Saya's portrait. Its pressed variant stays active through a held primary click, with specialized cursors taking precedence.
  - Bats independently return to the top edge and hang upside down, scatter from the cursor into separate roaming areas, and return after randomized delays. Use larger silhouettes with black bodies in Dark mode and near-black bodies in Light mode, muted wing accents, red eyes, and no separate gray rim. Some bats occasionally circle a castle tower before resuming their usual roaming and roost timers.
  - Use smooth wing deformation of one consistent bat design and mirror it with travel direction. Show 60 tumbling petals in three shapes and cap pointer particles at 72. Bats react within 180 logical pixels of the pointer and 320 pixels of a click. Each bat keeps a randomized 40–84 pixel size. Direct clicks trigger a 3.5-second escape at up to 400 logical pixels per second, with farther waypoints and continued flight away from window edges before roaming resumes.
  - All effects are always enabled. Use a faceted silver arrowhead with a shallow rear notch, no protruding shaft, a burgundy accent and a tip hotspot. Clickable controls use a matching silver gauntlet with a raised index finger, burgundy cuff, crimson gem and fingertip hotspot. Holding a primary click compresses the hand into a pressed pose and brightens its gem, retaining the fingertip hotspot and pressed hand until release, including when dragged away from the button.
- A footer with steady text and a dedicated, right-aligned activity indicator in a fixed space. Keep it active for 750 ms after checks finish so fast operations remain visible; expose progress in its tooltip. Show settings-save errors clearly; omit the settings-file path.
  - Midnight Neko uses a silver crescent around a crimson gem and a fine diamond chain. During activity, pulse the gem, orbit a silver highlight and move a diamond along the chain; keep a still, dimmer emblem when idle.
- An empty state explaining how to create the first job.

The interface must remain responsive during hashing, copying, compression, verification, and cleanup. Long content must be scrollable. Rows wrap in narrow windows; content that still does not fit scrolls sideways instead of pushing scroll bars out of view. All controls must remain accessible at smaller window sizes. Vertically center all elements within each horizontal row. At narrow widths, place the job activation control below the job heading so it remains accessible. The header shows the application name, at most a short tagline per look, and the theme selector. Default to a 1280 × 960 logical-pixel window, with room below the job controls.

The application remains a single distributable executable with an embedded icon. On Windows, use the app icon for the executable in Explorer and shortcuts as well as the app window. Under Linux, offer that icon directly to X11 and to Wayland compositors supporting window-icon pixel data. Missing support, icon setup errors, or an unresponsive icon negotiation must not block startup, show an error dialog, or prevent normal use; retain the compositor default icon. Do not install desktop entries or extract icon files.

### Job actions

- **New job**: create and select a new independent job in its settings tab, including while browsing backups. Disable during restoration or while saving backup protection; preserve existing jobs when leaving their history.
- **Check now**: queue a check regardless of whether automatic scheduling is enabled. This is not an unconditional backup command.
- Each job has **Job settings** and **Backups** tabs. The Backups tab replaces restoration and management overview dialogs; load recorded versions asynchronously when no operation is active or queued. Pause automatic starts while browsing backups; leaving the tab preserves job activation.
- List saved versions newest first with date, name, size, and folder/ZIP type. Mark missing versions unavailable. Each entry has a **Locked** checkbox protecting it from retention deletion; save changes in the background, disable further edits while saving, and report failures without displaying unsaved protection. Missing versions may be unlocked. Browsing and protection do not require an available source folder.
- Each backup row has an icon-only button to open its location in the system file manager. Open folder backups directly; select ZIP backups in Windows Explorer or open their parent folder on Linux. Disable for unavailable backups and report launch failures.
- Backup rows are not selectable. Each row has an icon-only restore button with a tooltip. Clicking it opens a confirmation modal for that backup, showing the resolved source path, version, replacement warning, **Cancel**, and a red **Replace contents and restore** button. Only confirmation starts restoration; Cancel, Escape, or backdrop dismissal changes neither source nor activation. Require an available backup and source folder to restore.
- Keep the Backups tab usable in narrow windows in every look. Allow the backup list and the surrounding content to scroll; wrap long names and paths.
- With no destination selected, prompt the user to choose one in Job settings instead of loading history or showing a filesystem error.
- Size restore confirmation to avoid scrolling at the default window size; allow scrolling in smaller windows. Place **Cancel** on the left and the restore confirmation button on the right.
- In the Backups tab, subtly highlight the first list entry of each local calendar day, newest first. Keep protection clearly recognizable. Wrap job action buttons on narrow windows.
- Show retention capacity warnings in the Backups tab and job details; refresh after protection changes, limit changes, and backup completion.
- On confirmed restore, persist this job as disabled with no next scheduled time before starting work. It stays disabled after success or failure. Run restore through the shared background worker with progress and error status, shown on the Backups tab. Prevent edits and manual starts while restoring; other jobs resume scheduling afterward.
- Enable or disable automatic scheduling for an individual job using a prominent green enable button or red disable button.
- **Remove job**: remove the job configuration without deleting any existing backups.
- Save changes automatically and preserve them across restarts.
- Prevent editing or removing a running or queued job until its pending work finishes, except that automatic scheduling can be toggled at any time. Disabling removes queued work for that job but lets a running backup finish. Manual checks remain available when scheduling is disabled.
  - Keep a busy job's settings looking unchanged; ignore edits, folder selection, **Check now**, and **Remove job** instead of disabling them, so quick checks do not flicker.

### Window lifecycle

Minimizing keeps the application and scheduler running. Closing the window immediately exits without confirmation, including while a backup or restore is running. Attempt to save settings before exit; a save failure is logged to stderr and does not block closing. Exit stops scheduling and terminates background operations without waiting for completion. Interrupted operations have the same recovery limitations as forced process termination. No system tray or always-running service is required.

## 4. Job configuration and defaults

Each job is independent and has a persistent UUID. Jobs may share source folders or a destination parent directory; their managed backups must remain separate.

- Settings has **Job settings** and **Theme** tabs. Automatic destination and immediate-check options appear in both the settings template and individual jobs; new jobs inherit the template, then retain independent choices.
- Display Windows drive and network paths without extended-length prefixes in path fields, the settings-file location, and restore details; preserve internal paths for file operations.
- Persist editable new-job defaults for automatic destination, compression, scheduling, and retention in `portbak.json`; changes affect future jobs only.
- On source selection, create `<app-dir>/backups/<job UUID>-<job name>`. An empty job name uses only the UUID, without a suffix. Replace characters unsafe in filenames with underscores; reject destinations within the source.
- Save automatic destination paths relative to the app directory; resolve them against the executable directory for all backup, history, protection, retention, and restore operations. Moving the app, configuration, and backups together preserves functionality. Convert legacy absolute automatic paths to relative paths; keep manual destinations absolute.
- Automatic destinations always use the job name and follow job renames; move the existing destination with its backups, refuse collisions, and report failures.
- Each job has an **Auto destination folder** checkbox. Checking it restores automatic destination updates without moving manual-folder contents.
- Choosing or typing a manual destination unchecks **Auto destination folder**, including choosing the same folder. Older source-named automatic destinations migrate to job-name paths when opened for editing while idle, preserving backups. Ignore obsolete naming-choice fields.
- Existing jobs retain their saved values when loading older configurations.
- Enable automatic backups only after both source and destination folders are chosen.

| Setting | Behavior and default |
| --- | --- |
| Name | Editable display name, limited to 50 characters; initially empty; shown as **Unnamed job** when blank |
| Source folder | Existing folder selected through a folder picker or editable path; initially empty |
| Destination folder | Automatic destination enabled by default; generated on source selection; manually editable |
| Automatic scheduling | Disabled by default |
| Schedule mode | Interval by default; display choices in order: Interval, Daily, Weekdays |
| Local time | 18:00 by default; hours 0–23, minutes 0–59 |
| Selected weekdays | All seven selected by default; used only in weekday mode |
| Interval | 00:10:00 by default; editable hours:minutes:seconds, from 00:00:01 to 8760:00:00 |
| Compress as ZIP | Checkbox; checked by default |
| Rolling retention | Enabled by default |
| Maximum count | Checked by default at 20; positive whole number, supporting at least 1–100,000 |
| Maximum total size | Unchecked by default at 10 GiB; positive value, supporting at least 0.001–100,000 GiB |

A GiB is 1,073,741,824 bytes. Count and size limits are remembered while rolling retention is disabled, but are not enforced. Enabling rolling retention without an active limit does not delete anything.

Also persist the next scheduled check, last successful backup time, and latest job result. The initial result is **Not checked yet**. An unchanged check must not update the last successful backup time.

## 5. Scheduling

### Modes

- In Daily and Weekdays job settings, show the next check in a boxed readout without a label, with spacing after the local-time controls, based on the current schedule even while disabled. Vertically center controls and readout in every theme; always display minutes with two digits. In Weekdays mode, place day selection above local time.

1. **Interval:** enter a duration as hours:minutes:seconds in jobs and new-job defaults. Minutes and seconds range from 00 to 59; allow hours above 23. Clamp the total to one second through 365 days. Preserve the duration of older saved value/unit settings.
2. **Daily:** check once per day at the configured local time.
3. **Weekdays:** check at the configured local time only on selected weekdays. Display weekdays Monday through Sunday. Require at least one selected weekday before enabling automatic execution.

### Execution rules

- Scheduling works only while the application is running, including when minimized. Do not register system tasks, install services, or wake an off computer.
- Disabling clears the next scheduled time. Each activation, including reactivation, starts a fresh future schedule.
- Enabled jobs show a countdown to the next automatic check in both the job list and details, updated every second. Format as HH:MM:SS with a day prefix for durations of at least 24 hours; clamp overdue values to zero. Keep the absolute next check time in the details. Running/queued indicators remain visible.
  - The details show the activity label, countdown, and next check time in the status row below the job header.
- On enabling a schedule, plan its next future occurrence.
- The per-job **Check immediately when enabled** option inherits the settings template when created; the template starts unchecked. Later template changes do not affect existing jobs. When checked, each disabled-to-enabled transition queues one immediate change check, without changing the future scheduled time. Deduplicate active/queued checks. Startup alone does not trigger this option.
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

Expose the current phase and useful progress, such as files processed and current relative path, in the footer activity indicator’s tooltip. Report unavailable folders, unreadable files, write failures, insufficient space, verification mismatches, and source changes clearly.

On failure before successful publication/recording, do not advance the comparison baseline or delete previous successful backups. Clean up temporary output where possible. If retention fails after a successful backup, retain the new backup and report successful creation with a separate cleanup warning.

This application does not provide a transactional filesystem snapshot. Explain that continuously changing databases should be closed or exported before backup.

After a crash, forced termination, or power loss, temporary output or a published but unrecorded backup may remain. Do not automatically treat it as successful or delete it as part of retention. Do not promise full storage-level protection against power loss.

## 10. Rolling retention

Rolling retention is configured per job and enabled by default for new jobs, with a maximum count of 20. Maximum count and maximum total size can be enabled separately or together.

After a newly successful backup:

- Count only successful backups owned by this job and recorded in its manifest.
- The per-job and template checkbox **Count protected backups toward retention limits** defaults to unchecked. Unchecked, exclude protected versions from both budgets; checked, include them. Protected backups are never eligible for deletion.
- Evaluate both active limits using the counted versions. If either is exceeded, delete the oldest unprotected managed successful backup other than the newest version, repeating until the limits are satisfied or no eligible version remains.
- Use successful creation order to determine the oldest backup.
- For folder backups, total size means the sum of regular file lengths. For ZIP backups, use archive file length. Do not use source size, allocated disk blocks, settings, or unrelated files.
- Always keep the newest successful backup. If it alone exceeds the size limit, retain it and show a notice.
- Never delete protected versions, unrelated files, another job's backups, temporary output, unrecorded output, or the latest successful backup.
- Never follow links when deleting managed backup entries. Refuse unsafe deletion and report a cleanup warning.

Each saved version has a persistent protection flag, false by default (including records that predate the flag). Changing this flag must preserve all other metadata and must not trigger immediate cleanup. Use destination locking and atomic manifest replacement for protection changes.

When rolling retention and counting protected backups are enabled and at least one version is protected, warn if the count limit minus the protected count is at most three places, or if at most 20% of the size limit remains after subtracting protected backup sizes. Show concrete remaining values, clamp negative values to zero, and separately report exceeded limits. These are retention budgets, not free disk space; warnings do not block new backups. Counted protected versions and the newest backup can cause limits to remain exceeded.

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
| First launch | No jobs; system appearance saved as Light or Dark; clear empty state |
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
| History crosses local midnight | Highlight the first listed version of each local day in the Backups tab |
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

- Open the app with no backup job selected; select a job only on explicit user action.
