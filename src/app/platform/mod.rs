#[cfg(target_os = "linux")]
pub(crate) mod wayland_icon;

pub(crate) fn open_in_file_manager(path: &std::path::Path) -> anyhow::Result<()> {
    use anyhow::Context;
    let path = std::fs::canonicalize(path).context("Backup is no longer available")?;
    #[cfg(windows)]
    let mut command = {
        use std::os::windows::process::CommandExt;
        let mut command = std::process::Command::new("explorer.exe");
        command.creation_flags(0x08000000);
        let display = crate::app::views::widgets::display_path(&path);
        if path.is_dir() {
            command.arg(display);
        } else {
            // Explorer rejects a quoted `"/select,path"`; only the path may be quoted.
            // Windows paths cannot contain `"`, so this raw argument stays well-formed.
            command.raw_arg(format!("/select,\"{display}\""));
        }
        command
    };
    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = std::process::Command::new("xdg-open");
        let folder = if path.is_dir() {
            path.as_path()
        } else {
            path.parent().context("Backup has no containing folder")?
        };
        command.arg(folder);
        command
    };
    let mut child = command.spawn().context("Could not open the file manager")?;
    std::thread::spawn(move || {
        if let Err(error) = child.wait() {
            eprintln!("Could not wait for the file manager: {error}");
        }
    });
    Ok(())
}
