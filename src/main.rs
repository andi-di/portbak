#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod app;
mod engine;
mod model;

use anyhow::Context;
use eframe::egui;
use fs2::FileExt;
use model::Config;
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

const CONFIG_VERSION: u32 = 1;

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
            .with_inner_size([1100.0, 960.0])
            .with_min_inner_size([800.0, 600.0])
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
        engine::load(path)?
    } else {
        Config::default()
    };
    anyhow::ensure!(
        config.version == CONFIG_VERSION,
        "Unknown configuration version"
    );
    engine::save(path, &config)?;
    Ok(config)
}
