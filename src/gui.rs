use std::sync::{Arc, RwLock};

use gio::prelude::*;
use gtk::prelude::*;
use tokio::runtime::Runtime;
use tokio::sync::watch;

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

    let first_frame = app.read().unwrap().capture_frame();
    let (camera_tx, mut camera_rx) = watch::channel(first_frame.to_vec());

//    app.read().unwrap().runtime.spawn({
//        let app = Arc::clone(&app);
//        async move {
//            tokio::task::spawn_blocking(move || {
//                loop {
//                    let app = app.read().unwrap();
//                    if app.is_alive {
//                        camera_tx.broadcast(app.capture_frame().to_vec()).unwrap();
//                    } else {
//                        break;
//                    }
//                }
//            }).await.unwrap();
//        }
//    });

    // TODO: This works, but blocks main thread. Offloading captures to separate thread makes CPU usage go to 100%
    idle_add(clone!(app, gui => move || {
        gui.read().unwrap().update_local_feed(&app.read().unwrap().capture_frame().to_vec());
        Continue(app.read().unwrap().is_alive)
    }));

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
