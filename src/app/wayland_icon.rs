//! Optional in-memory window icons; unsupported compositors keep their default icon.

use std::{
    fs::File,
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{bail, ensure, Context, Result};
use eframe::egui;
use rustix::{
    event::{poll, PollFd, PollFlags, Timespec},
    fs::{memfd_create, MemfdFlags},
};
use wayland_client::{
    backend::{Backend, ObjectId},
    delegate_noop,
    protocol::{wl_buffer, wl_callback, wl_registry, wl_shm, wl_shm_pool},
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};
use wayland_protocols::xdg::{
    shell::client::xdg_toplevel::XdgToplevel,
    toplevel_icon::v1::client::{xdg_toplevel_icon_manager_v1, xdg_toplevel_icon_v1},
};
use winit::{
    platform::wayland::WindowExtWayland,
    raw_window_handle::{HasDisplayHandle, RawDisplayHandle},
    window::Window,
};

const TIMEOUT: Duration = Duration::from_millis(500);

pub fn install(creation: &eframe::CreationContext<'_>, icon: egui::IconData) {
    let Some(window) = creation.winit_window().cloned() else {
        return;
    };
    if window.xdg_toplevel().is_none() {
        return;
    }
    let ctx = creation.egui_ctx.clone();
    // Icon failures and thread-creation failures must never prevent startup.
    let _ = std::thread::Builder::new()
        .name("wayland-icon".into())
        .spawn(move || {
            if set_window_icon(&window, &icon).is_ok() {
                ctx.request_repaint();
            }
        });
}

fn set_window_icon(window: &Arc<Window>, icon: &egui::IconData) -> Result<()> {
    let RawDisplayHandle::Wayland(display) = window.display_handle()?.as_raw() else {
        return Ok(());
    };
    let Some(toplevel) = window.xdg_toplevel() else {
        return Ok(());
    };
    // The worker owns an Arc<Window>, keeping both borrowed native handles alive.
    let backend = unsafe { Backend::from_foreign_display(display.display.as_ptr().cast()) };
    let connection = Connection::from_backend(backend);
    // The pointer comes from winit's public Wayland API and is never destroyed here.
    let id = unsafe { ObjectId::from_ptr(XdgToplevel::interface(), toplevel.as_ptr().cast()) }?;
    let toplevel = XdgToplevel::from_id(&connection, id)?;
    transfer_icon(&connection, &toplevel, icon, TIMEOUT)
}

#[derive(Default)]
struct IconState {
    globals: Vec<(u32, String, u32)>,
    synced: bool,
}

fn transfer_icon(
    connection: &Connection,
    toplevel: &XdgToplevel,
    image: &egui::IconData,
    timeout: Duration,
) -> Result<()> {
    let deadline = Instant::now() + timeout;
    let mut queue = connection.new_event_queue::<IconState>();
    let qh = queue.handle();
    let mut state = IconState::default();
    let registry = connection.display().get_registry(&qh, ());
    synchronize(connection, &mut queue, &mut state, deadline)?;
    let manager_name = xdg_toplevel_icon_manager_v1::XdgToplevelIconManagerV1::interface().name;
    let Some(manager_id) = state.global(manager_name) else {
        return Ok(());
    };
    let Some(shm_id) = state.global(wl_shm::WlShm::interface().name) else {
        return Ok(());
    };
    let pixels = argb_pixels(image)?;
    let side = i32::try_from(image.width)?;
    let size = i32::try_from(pixels.len())?;
    let mut memory = File::from(memfd_create(c"portbak-icon", MemfdFlags::CLOEXEC)?);
    memory.write_all(&pixels)?;

    let manager: xdg_toplevel_icon_manager_v1::XdgToplevelIconManagerV1 =
        registry.bind(manager_id, 1, &qh, ());
    let shm: wl_shm::WlShm = registry.bind(shm_id, 1, &qh, ());
    let pool = shm.create_pool(std::os::fd::AsFd::as_fd(&memory), size, &qh, ());
    let buffer = pool.create_buffer(0, side, side, side * 4, wl_shm::Format::Argb8888, &qh, ());
    let icon = manager.create_icon(&qh, ());
    icon.add_buffer(&buffer, 1);
    manager.set_icon(toplevel, Some(&icon));
    // The protocol retains the assigned icon; destroy it before its backing buffer.
    icon.destroy();
    buffer.destroy();
    pool.destroy();
    manager.destroy();
    synchronize(connection, &mut queue, &mut state, deadline)
}

impl IconState {
    fn global(&self, interface: &str) -> Option<u32> {
        self.globals
            .iter()
            .find(|(_, name, version)| name == interface && *version >= 1)
            .map(|(id, _, _)| *id)
    }
}

fn synchronize(
    connection: &Connection,
    queue: &mut EventQueue<IconState>,
    state: &mut IconState,
    deadline: Instant,
) -> Result<()> {
    state.synced = false;
    connection.display().sync(&queue.handle(), ());
    loop {
        ensure!(Instant::now() < deadline, "Wayland icon request timed out");
        queue.dispatch_pending(state)?;
        if state.synced {
            return Ok(());
        }
        match connection.flush() {
            Ok(()) => {}
            Err(wayland_client::backend::WaylandError::Io(error))
                if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error.into()),
        }
        let Some(reader) = queue.prepare_read() else {
            continue;
        };
        let remaining = deadline.saturating_duration_since(Instant::now());
        let timeout = Timespec::try_from(remaining)?;
        let fd = reader.connection_fd();
        let mut fds = [PollFd::new(&fd, PollFlags::IN)];
        match poll(&mut fds, Some(&timeout)) {
            Ok(0) => bail!("Wayland icon request timed out"),
            Ok(_) => {}
            Err(rustix::io::Errno::INTR) => continue,
            Err(error) => return Err(error.into()),
        }
        match reader.read() {
            Ok(_) => {}
            Err(wayland_client::backend::WaylandError::Io(error))
                if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error.into()),
        }
    }
}

fn argb_pixels(icon: &egui::IconData) -> Result<Vec<u8>> {
    ensure!(
        icon.width > 0 && icon.width == icon.height && icon.width <= 1024,
        "Invalid icon dimensions"
    );
    let expected = usize::try_from(icon.width)?
        .checked_mul(usize::try_from(icon.height)?)
        .and_then(|count| count.checked_mul(4))
        .context("Icon size overflow")?;
    ensure!(icon.rgba.len() == expected, "Invalid icon pixel count");
    let mut pixels = Vec::with_capacity(expected);
    for rgba in icon.rgba.as_chunks::<4>().0 {
        let alpha = u32::from(rgba[3]);
        let premultiply = |channel: u8| (u32::from(channel) * alpha + 127) / 255;
        let argb = (alpha << 24)
            | (premultiply(rgba[0]) << 16)
            | (premultiply(rgba[1]) << 8)
            | premultiply(rgba[2]);
        pixels.extend_from_slice(&argb.to_ne_bytes());
    }
    Ok(pixels)
}

impl Dispatch<wl_registry::WlRegistry, ()> for IconState {
    fn event(
        state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => state.globals.push((name, interface, version)),
            wl_registry::Event::GlobalRemove { name } => {
                state.globals.retain(|(id, _, _)| *id != name)
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_callback::WlCallback, ()> for IconState {
    fn event(
        state: &mut Self,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if matches!(event, wl_callback::Event::Done { .. }) {
            state.synced = true;
        }
    }
}

delegate_noop!(IconState: ignore wl_shm::WlShm);
delegate_noop!(IconState: ignore wl_shm_pool::WlShmPool);
delegate_noop!(IconState: ignore wl_buffer::WlBuffer);
delegate_noop!(IconState: ignore xdg_toplevel_icon_manager_v1::XdgToplevelIconManagerV1);
delegate_noop!(IconState: ignore xdg_toplevel_icon_v1::XdgToplevelIconV1);

#[cfg(test)]
mod tests;
