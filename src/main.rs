use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use gdk_pixbuf::Pixbuf;
use gio::MemoryInputStream;
use gio::prelude::*;
use glib::Bytes;
use gtk::prelude::*;
use rscam::FormatInfo;

struct NiagaraApp {
    application: gtk::Application,
    window: gtk::ApplicationWindow,
    box_: gtk::Box,
    image: gtk::Image,
    button: gtk::Button,
    camera: Option<rscam::Camera>,
    is_streaming: bool,
}

impl NiagaraApp {
    pub fn create(application: &gtk::Application) {
        let state = NiagaraApp::init_ui(application);
        (*state).borrow_mut().setup_camera();
        (*state).borrow_mut().is_streaming = true;

        // TODO: Super hacky, grab a frame in the background and notify the GUI thread instead?
        idle_add(move || {
            let frame = (*state).borrow().get_video_frame();
            (*state).borrow().display_frame(&frame);
            Continue((*state).borrow().is_streaming)
        });
    }

    fn init_ui(application: &gtk::Application) -> Rc<RefCell<Self>> {
        let window = gtk::ApplicationWindow::new(application);
        window.set_title("Niagra");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(350, 70);

        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 10);
        let image = gtk::Image::new();
        let button = gtk::Button::new_with_label("Click me!");
        box_.add(&image);
        box_.add(&button);

        window.add(&box_);
        window.show_all();

        let state = Rc::new(RefCell::new(NiagaraApp {
            application: application.clone(),
            window,
            box_,
            image,
            button,
            camera: None,
            is_streaming: false,
        }));

        let cloned_state = Rc::clone(&state);
        application.connect_shutdown(move |_| {
            (*cloned_state).borrow_mut().is_streaming = false;
        });

        state
    }

    fn setup_camera(&mut self) {
        // TODO: Create the camera in a more general manner
        let mut camera = rscam::new("/dev/video0").unwrap();
        camera
            .start(&rscam::Config {
                interval: (1, 30),
                resolution: (1280, 720),
                format: b"MJPG",
                ..Default::default()
            })
            .unwrap();
        self.camera = Some(camera);
    }

    fn print_camera_info(&self) {
        let camera = self.camera.as_ref().unwrap();
        let formats: Vec<FormatInfo> = camera
            .formats()
            .collect::<io::Result<Vec<FormatInfo>>>()
            .unwrap();
        let resolutions = camera.resolutions(&formats[0].format).unwrap();

        println!("{:#?}", formats);
        println!("{:#?}", resolutions);
    }

    fn get_video_frame(&self) -> rscam::Frame {
        let camera = self.camera.as_ref().unwrap();
        camera.capture().unwrap()
    }

    fn display_frame(&self, buf: &[u8]) {
        let stream = MemoryInputStream::new_from_bytes(&Bytes::from(buf));
        let cancellable: Option<&gio::Cancellable> = None;
        let pixbuf = Pixbuf::new_from_stream(&stream, cancellable).unwrap();
        self.image.set_from_pixbuf(Some(&pixbuf));
    }
}


fn main() {
    let app = gtk::Application::new(
        Some("com.github.lucis-fluxum.niagra-rs"),
        Default::default(),
    ).expect("Initialization failed.");

    app.connect_activate(move |app| NiagaraApp::create(app));
    app.run(&[]);
}
