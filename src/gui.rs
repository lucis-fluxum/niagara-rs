use std::cell::RefCell;
use std::rc::Rc;

use gio::prelude::*;
use gtk::prelude::*;

use state::*;

mod state;

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = Rc::clone(&$n); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = Rc::clone(&$n); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

pub fn initialize(application: &gtk::Application) {
    let (app, gui): (Rc<RefCell<AppState>>, Rc<GuiState>) = setup_state(application);
    app.borrow_mut().setup_camera("/dev/video0", "MJPG");
    app.borrow_mut().setup_udp("127.0.0.1:3000", "127.0.0.1:3000");

    idle_add(clone!(app, gui => move || {
        let app = app.borrow();
        let frame = app.capture_frame();
        // TODO: Local video feed
        // gui.display_frame(&frame);
        let socket = app.socket.as_ref().unwrap();
        let bytes_written = socket.send(&frame).unwrap();
        dbg!(bytes_written);
        Continue(app.is_alive)
    }));

    idle_add(clone!(app, gui => move || {
        let app = app.borrow();
        let socket = app.socket.as_ref().unwrap();
        let mut buf = [0; 65_536];
        let bytes_read = socket.recv(&mut buf).unwrap();
        let buf = buf.split_at(bytes_read).0;
        dbg!(buf.len());
        gui.display_frame(buf);
        Continue(app.is_alive)
    }));

    gui.application.connect_shutdown(clone!(app => move |_| {
        app.borrow_mut().is_alive = false;
    }));
}

fn setup_state(application: &gtk::Application) -> (Rc<RefCell<AppState>>, Rc<GuiState>) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("Niagra");
    window.set_position(gtk::WindowPosition::Center);

    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let image = gtk::Image::new();
    let button = gtk::Button::new_with_label("Click me!");
    box_.add(&image);
    box_.add(&button);

    window.add(&box_);
    window.show_all();

    let gui = Rc::new(GuiState {
        application: application.clone(),
        window,
        box_,
        image,
        button,
    });

    let app = Rc::new(RefCell::new(AppState {
        socket: None,
        camera: None,
        is_alive: true,
    }));

    (app, gui)
}
