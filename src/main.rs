#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod app;
mod destinations;
mod engine;
mod model;
mod persistence;

use anyhow::Context;
use eframe::egui;
use fs2::FileExt;
use model::{Config, CONFIG_VERSION};
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

const MIN_WINDOW_SIZE: egui::Vec2 = egui::vec2(1024.0, 940.0);

fn main() {
    if let Err(error) = startup() {
        let message = format!("portbak could not start:\n{error:#}");
        eprintln!("{message}");
        rfd::MessageDialog::new()
            .set_title("portbak")
            .set_description(&message)
            .set_level(rfd::MessageLevel::Error)
            .show();
    }
}

fn app_icon() -> egui::IconData {
    let image = image::load_from_memory(include_bytes!("../assets/icon/portbak-64.png"))
        .expect("the embedded icon is a valid PNG");
    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    egui::IconData {
        width,
        height,
        rgba: rgba.into_raw(),
    }
}

fn startup() -> anyhow::Result<()> {
    let path = config_path()?;
    let lock = lock_config(&path)?;
    let config = load_config(&path)?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 960.0])
            .with_min_inner_size(MIN_WINDOW_SIZE)
            .with_icon(app_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "portbak",
        options,
        Box::new(move |creation| {
            #[cfg(target_os = "linux")]
            app::platform::wayland_icon::install(creation, app_icon());
            #[cfg(not(target_os = "linux"))]
            let _ = creation;
            Ok(Box::new(app::App::new(config, path, lock)))
        }),
    )
    .map_err(|error| anyhow::anyhow!("{error}"))
}

/// The configuration lives beside the executable so the app stays portable.
fn config_path() -> anyhow::Result<PathBuf> {
    Ok(std::env::current_exe()?
        .parent()
        .context("Application directory is missing")?
        .join("portbak.json"))
}

fn lock_config(path: &Path) -> anyhow::Result<File> {
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path.with_extension("lock"))?;
    lock.try_lock_exclusive()
        .map_err(|_| anyhow::anyhow!("portbak is already running with this configuration."))?;
    Ok(lock)
}

/// Loads and immediately re-saves, so an unwritable location fails at startup, not later.
fn load_config(path: &Path) -> anyhow::Result<Config> {
    let config: Config = if path.exists() {
        persistence::load(path)?
    } else {
        Config::default()
    };
    anyhow::ensure!(
        config.version == CONFIG_VERSION,
        "Unknown configuration version"
    );
    persistence::save(path, &config.for_storage())?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_settings_are_rejected_without_overwriting_them() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("portbak.json");
        let original = br#"{"version":1,"jobs":[]}"#;
        std::fs::write(&path, original).unwrap();
        assert!(load_config(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }

    #[test]
    fn new_settings_save_grouped_schedules_with_independent_defaults() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("portbak.json");
        let mut config = load_config(&path).unwrap();
        config.jobs.push(config.job_defaults.new_job());
        persistence::save(&path, &config.for_storage()).unwrap();
        let saved = load_config(&path).unwrap();
        assert_eq!(saved.version, CONFIG_VERSION);
        assert_eq!(saved.jobs[0].timing.interval_seconds(), 600);
        assert_eq!(saved.job_defaults.timing.interval_seconds(), 600);
        assert_eq!(model::Job::default().timing.interval_seconds(), 3600);
        let json: serde_json::Value = persistence::load(&path).unwrap();
        assert_eq!(json["jobs"][0]["timing"]["mode"], "Interval");
        assert!(json["jobs"][0].get("schedule").is_none());
    }
}
