use std::io;
use std::net::{ToSocketAddrs, UdpSocket};

use gdk_pixbuf::Pixbuf;
use gio::MemoryInputStream;
use gio::prelude::*;
use glib::Bytes;
use gtk::prelude::*;
use rscam::{Camera, FormatInfo, Frame};

pub(crate) struct AppState {
    pub(crate) socket: Option<UdpSocket>,
    pub(crate) camera: Option<Camera>,
    pub(crate) is_alive: bool,
}

impl AppState {
    pub(crate) fn setup_udp(&mut self, local_addr: impl ToSocketAddrs, remote_addr: impl ToSocketAddrs) {
        let socket = UdpSocket::bind(local_addr).unwrap();
        socket.connect(remote_addr).unwrap();
        self.socket = Some(socket);
    }

    pub(crate) fn setup_camera(&mut self, camera_device: &str, camera_format: &str) {
        let mut camera = rscam::new(camera_device).unwrap();
        camera
            .start(&rscam::Config {
                interval: (1, 30),
                resolution: (640, 360),
                format: camera_format.as_bytes(),
                ..Default::default()
            })
            .unwrap();
        self.camera = Some(camera);
    }

    pub(crate) fn print_camera_info(&self) {
        let camera = self.camera.as_ref().unwrap();
        let formats: Vec<FormatInfo> = camera
            .formats()
            .collect::<io::Result<Vec<FormatInfo>>>()
            .unwrap();
        let resolutions = camera.resolutions(&formats[0].format).unwrap();

        println!("{:#?}", formats);
        println!("{:#?}", resolutions);
    }

    pub(crate) fn capture_frame(&self) -> Frame {
        let camera = self.camera.as_ref().unwrap();
        camera.capture().unwrap()
    }
}

pub(crate) struct GuiState {
    pub(crate) application: gtk::Application,
    pub(crate) window: gtk::ApplicationWindow,
    pub(crate) box_: gtk::Box,
    pub(crate) local_feed: gtk::Image,
    pub(crate) remote_feed: gtk::Image,
    pub(crate) button: gtk::Button,
}

impl GuiState {
    pub(crate) fn update_local_feed(&self, buf: &[u8]) {
        let stream = MemoryInputStream::new_from_bytes(&Bytes::from(&buf));
        let cancellable: Option<&gio::Cancellable> = None;
        let pixbuf = Pixbuf::new_from_stream(&stream, cancellable).unwrap();
        self.local_feed.set_from_pixbuf(Some(&pixbuf));
    }

    pub(crate) fn update_remote_feed(&self, buf: &[u8]) {
        let stream = MemoryInputStream::new_from_bytes(&Bytes::from(&buf));
        let cancellable: Option<&gio::Cancellable> = None;
        let pixbuf = Pixbuf::new_from_stream(&stream, cancellable).unwrap();
        self.remote_feed.set_from_pixbuf(Some(&pixbuf));
    }
}
