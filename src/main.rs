#![allow(dead_code)]

use gio::prelude::*;

mod gui;

// TODO: Rough sketch of architecture:
// [CAMERA] -> async task, reads from camera and sends frame to glib channel
//          -> read from channel, update local feed
//          -> reads from channel, start async task to send frame to remote address if desired
// [SOCKET] -> async task, reads from socket and sends frame to glib channel
//          -> reads from channel if desired, updates remote feed
fn main() {
    let app = gtk::Application::new(
        Some("com.github.lucis-fluxum.niagara-rs"),
        Default::default(),
    )
    .expect("Initialization failed.");

    app.connect_activate(move |app| gui::initialize(app));
    app.run(&[]);
}
