#![allow(dead_code)]

use gio::prelude::*;

mod gui;

fn main() {
    let app = gtk::Application::new(
        Some("com.github.lucis-fluxum.niagra-rs"),
        Default::default(),
    ).expect("Initialization failed.");

    app.connect_activate(move |app| gui::initialize(app));
    app.run(&[]);
}
