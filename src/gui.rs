use std::sync::{Arc, RwLock};

use gio::prelude::*;
use gtk::prelude::*;
use tokio::runtime::Runtime;

use state::*;

mod state;

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = Arc::clone(&$n); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = Arc::clone(&$n); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

pub fn initialize(application: &gtk::Application) {
    let (app, gui) = setup_state(application);
    app.write().unwrap().setup_camera("/dev/video0", "MJPG");
    app.write().unwrap().setup_udp("0.0.0.0:3000", "0.0.0.0:3000");

    let (local_feed_sender, local_feed_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    // Read from camera and send frame to glib channel
    app.read().unwrap().runtime.spawn({
        let app = Arc::clone(&app);
        async move {
            tokio::task::spawn_blocking(move || {
                loop {
                    match local_feed_sender.send(app.read().unwrap().capture_frame().to_vec()) {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error sending frame through channel: {}", e);
                            break;
                        },
                    };
                }
            }).await
        }
    });

    // Read from channel and update local feed
    local_feed_receiver.attach(None, clone!(gui => move |buf| {
        gui.read().unwrap().update_local_feed(&buf);
        Continue(true)
    }));

    // TODO: Socket communication

    gui.read().unwrap().application.connect_shutdown(clone!(app => move |_| {
        app.write().unwrap().is_alive = false;
    }));
}

fn setup_state(application: &gtk::Application) -> (Arc<RwLock<AppState>>, Arc<RwLock<GuiState>>) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("Niagara");
    window.set_position(gtk::WindowPosition::Center);

    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let local_feed = gtk::Image::new();
    let remote_feed = gtk::Image::new();
    let button = gtk::Button::new_with_label("Click me!");
    box_.add(&local_feed);
    box_.add(&remote_feed);
    box_.add(&button);

    window.add(&box_);
    window.show_all();

    let gui = Arc::new(RwLock::new(GuiState {
        application: application.clone(),
        window,
        box_,
        local_feed,
        remote_feed,
        button,
    }));

    let app = Arc::new(RwLock::new(AppState {
        runtime: Runtime::new().unwrap(),
        socket: None,
        camera: None,
        is_alive: true,
    }));

    (app, gui)
}
