#![allow(dead_code)]

use gio::prelude::*;

mod gui;

// TODO: Rough sketch of architecture:
// [CAMERA] -> async task, owns camera, reads from camera and sends frame to broadcasting channel
//          -> async task, owns local feed widget, reads from channel, updates local feed
//          -> async task, reads from channel, sends frame to remote address if desired
// [SOCKET] -> async task, owns socket, reads from socket and sends frame to broadcasting channel
//          -> async task, owns remote feed widget, reads from channel if desired, updates remote feed
fn main() {
    let app = gtk::Application::new(
        Some("com.github.lucis-fluxum.niagra-rs"),
        Default::default(),
    )
    .expect("Initialization failed.");

    app.connect_activate(move |app| gui::initialize(app));
    app.run(&[]);
}
