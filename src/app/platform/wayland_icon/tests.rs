use super::*;
use std::{
    io::{Read, Seek, SeekFrom},
    os::unix::net::UnixStream,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread::JoinHandle,
};
use wayland_protocols::xdg::{
    shell::server::xdg_toplevel as server_toplevel,
    toplevel_icon::v1::server::{
        xdg_toplevel_icon_manager_v1 as server_manager, xdg_toplevel_icon_v1 as server_icon,
    },
};
use wayland_server::{
    protocol::{wl_buffer as server_buffer, wl_shm as server_shm, wl_shm_pool as server_pool},
    Client, DataInit, Dispatch as ServerDispatch, Display, DisplayHandle, GlobalDispatch, New,
    Resource, WEnum,
};

delegate_noop!(IconState: ignore XdgToplevel);

#[derive(Default)]
struct Compositor {
    events: Vec<&'static str>,
    pixels: Option<Vec<u8>>,
    icon_alive: bool,
}

struct Server {
    connection: Connection,
    paused: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<Compositor>>,
}

impl Server {
    fn new(manager: bool, shm: bool) -> Self {
        let (client_socket, server_socket) = UnixStream::pair().unwrap();
        let mut display = Display::<Compositor>::new().unwrap();
        let mut handle = display.handle();
        handle.insert_client(server_socket, Arc::new(())).unwrap();
        // A directly bindable stand-in avoids unrelated shell/window setup in these protocol tests.
        handle.create_global::<Compositor, server_toplevel::XdgToplevel, _>(1, ());
        if manager {
            handle.create_global::<Compositor, server_manager::XdgToplevelIconManagerV1, _>(1, ());
        }
        if shm {
            handle.create_global::<Compositor, server_shm::WlShm, _>(1, ());
        }
        let paused = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let pause_flag = paused.clone();
        let stop_flag = stop.clone();
        let thread = std::thread::spawn(move || {
            let mut state = Compositor::default();
            while !stop_flag.load(Ordering::Acquire) {
                if !pause_flag.load(Ordering::Acquire) {
                    display.dispatch_clients(&mut state).unwrap();
                    display.flush_clients().unwrap();
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            state
        });
        Self {
            connection: Connection::from_socket(client_socket).unwrap(),
            paused,
            stop,
            thread: Some(thread),
        }
    }

    fn toplevel(&self) -> XdgToplevel {
        let mut queue = self.connection.new_event_queue::<IconState>();
        let registry = self.connection.display().get_registry(&queue.handle(), ());
        let mut state = IconState::default();
        synchronize(
            &self.connection,
            &mut queue,
            &mut state,
            Instant::now() + TIMEOUT,
        )
        .unwrap();
        registry.bind(
            state.global("xdg_toplevel").unwrap(),
            1,
            &queue.handle(),
            (),
        )
    }

    fn finish(&mut self) -> Compositor {
        self.stop.store(true, Ordering::Release);
        self.thread.take().unwrap().join().unwrap()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

macro_rules! global {
    ($interface:ty, $resource:ident, $body:block) => {
        impl GlobalDispatch<$interface, ()> for Compositor {
            fn bind(
                _: &mut Self,
                _: &DisplayHandle,
                _: &Client,
                resource: New<$interface>,
                _: &(),
                init: &mut DataInit<'_, Self>,
            ) {
                let $resource = init.init(resource, ());
                $body
            }
        }
    };
}

global!(server_toplevel::XdgToplevel, _resource, {});
global!(server_manager::XdgToplevelIconManagerV1, resource, {
    resource.done();
});
global!(server_shm::WlShm, resource, {
    resource.format(server_shm::Format::Argb8888);
    resource.format(server_shm::Format::Xrgb8888);
});

impl ServerDispatch<server_toplevel::XdgToplevel, ()> for Compositor {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &server_toplevel::XdgToplevel,
        _: server_toplevel::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        panic!("Unexpected toplevel request");
    }
}

impl ServerDispatch<server_manager::XdgToplevelIconManagerV1, ()> for Compositor {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &server_manager::XdgToplevelIconManagerV1,
        request: server_manager::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        match request {
            server_manager::Request::CreateIcon { id } => {
                init.init(id, Mutex::new(None::<Vec<u8>>));
                state.icon_alive = true;
                state.events.push("create_icon");
            }
            server_manager::Request::SetIcon {
                icon: Some(icon), ..
            } => {
                state.pixels = icon
                    .data::<Mutex<Option<Vec<u8>>>>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .clone();
                assert!(state.pixels.is_some(), "Icon must contain a pixel buffer");
                state.events.push("set_icon");
            }
            server_manager::Request::Destroy => state.events.push("destroy_manager"),
            _ => panic!("Unexpected manager request"),
        }
    }
}

impl ServerDispatch<server_icon::XdgToplevelIconV1, Mutex<Option<Vec<u8>>>> for Compositor {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &server_icon::XdgToplevelIconV1,
        request: server_icon::Request,
        data: &Mutex<Option<Vec<u8>>>,
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        match request {
            server_icon::Request::AddBuffer { buffer, scale } => {
                assert_eq!(scale, 1);
                assert!(state.pixels.is_none(), "Assigned icons are immutable");
                *data.lock().unwrap() = Some(buffer.data::<Vec<u8>>().unwrap().clone());
                state.events.push("add_buffer");
            }
            server_icon::Request::Destroy => {
                state.icon_alive = false;
                state.events.push("destroy_icon");
            }
            _ => panic!("Unexpected icon request"),
        }
    }
}

impl ServerDispatch<server_shm::WlShm, ()> for Compositor {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &server_shm::WlShm,
        request: server_shm::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        let server_shm::Request::CreatePool { id, fd, size } = request else {
            panic!("Unexpected shm request")
        };
        let mut file = File::from(fd);
        file.seek(SeekFrom::Start(0)).unwrap();
        let mut bytes = vec![0; size as usize];
        file.read_exact(&mut bytes).unwrap();
        init.init(id, bytes);
    }
}

impl ServerDispatch<server_pool::WlShmPool, Vec<u8>> for Compositor {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &server_pool::WlShmPool,
        request: server_pool::Request,
        bytes: &Vec<u8>,
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        match request {
            server_pool::Request::CreateBuffer {
                id,
                offset,
                width,
                height,
                stride,
                format,
            } => {
                assert_eq!(offset, 0);
                assert!(width > 0);
                assert_eq!(width, height);
                assert_eq!(stride, width * 4);
                assert_eq!(format, WEnum::Value(server_shm::Format::Argb8888));
                assert_eq!(bytes.len(), (stride * height) as usize);
                init.init(id, bytes.clone());
            }
            server_pool::Request::Destroy => state.events.push("destroy_pool"),
            _ => panic!("Unexpected pool request"),
        }
    }
}

impl ServerDispatch<server_buffer::WlBuffer, Vec<u8>> for Compositor {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &server_buffer::WlBuffer,
        _: server_buffer::Request,
        _: &Vec<u8>,
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        assert!(!state.icon_alive, "Buffer must outlive its icon");
        state.events.push("destroy_buffer");
    }
}

#[test]
fn supported_compositor_receives_pixels_and_ordered_cleanup() {
    let mut server = Server::new(true, true);
    let top = server.toplevel();
    let image = crate::app_icon();
    transfer_icon(&server.connection, &top, &image, TIMEOUT).unwrap();
    let state = server.finish();
    assert_eq!(state.pixels.unwrap(), argb_pixels(&image).unwrap());
    assert_eq!(
        state.events,
        [
            "create_icon",
            "add_buffer",
            "set_icon",
            "destroy_icon",
            "destroy_buffer",
            "destroy_pool",
            "destroy_manager"
        ]
    );
}

#[test]
fn missing_protocols_skip_icon_without_touching_the_window() {
    for (manager, shm) in [(false, true), (true, false), (false, false)] {
        let mut server = Server::new(manager, shm);
        let top = server.toplevel();
        transfer_icon(&server.connection, &top, &crate::app_icon(), TIMEOUT).unwrap();
        let state = server.finish();
        assert!(state.events.is_empty());
        assert!(state.pixels.is_none());
    }
}

#[test]
fn unresponsive_compositor_times_out_and_connection_remains_usable() {
    let mut server = Server::new(true, true);
    let top = server.toplevel();
    server.paused.store(true, Ordering::Release);
    let start = Instant::now();
    let result = transfer_icon(
        &server.connection,
        &top,
        &crate::app_icon(),
        Duration::from_millis(30),
    );
    assert!(result.is_err());
    assert!(start.elapsed() < Duration::from_secs(1));
    server.paused.store(false, Ordering::Release);
    transfer_icon(&server.connection, &top, &crate::app_icon(), TIMEOUT).unwrap();
    assert!(server.finish().pixels.is_some());
}

#[test]
fn disconnected_compositor_returns_an_error_without_panicking() {
    let mut server = Server::new(true, true);
    let top = server.toplevel();
    server.finish();
    assert!(transfer_icon(&server.connection, &top, &crate::app_icon(), TIMEOUT).is_err());
}

#[test]
fn malformed_images_are_rejected_before_icon_requests() {
    let mut server = Server::new(true, true);
    let top = server.toplevel();
    for image in [
        egui::IconData::default(),
        egui::IconData {
            width: 2,
            height: 1,
            rgba: vec![0; 8],
        },
        egui::IconData {
            width: 2,
            height: 2,
            rgba: vec![0; 15],
        },
        egui::IconData {
            width: u32::MAX,
            height: u32::MAX,
            rgba: vec![],
        },
    ] {
        assert!(transfer_icon(&server.connection, &top, &image, TIMEOUT).is_err());
    }
    assert!(server.finish().events.is_empty());
}

#[test]
fn pixels_use_native_endian_premultiplied_argb() {
    let image = egui::IconData {
        width: 2,
        height: 2,
        rgba: vec![
            255, 128, 64, 128, 255, 255, 255, 0, 44, 232, 245, 255, 0, 0, 0, 255,
        ],
    };
    let bytes = argb_pixels(&image).unwrap();
    let pixels: Vec<_> = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| u32::from_ne_bytes(*bytes))
        .collect();
    assert_eq!(pixels, [0x80804020, 0, 0xff2ce8f5, 0xff000000]);
}
