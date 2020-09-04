extern crate fractal_protocol;
extern crate fractal_core;
extern crate broadcaster;

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

use std::thread;
use std::{cell::RefCell};

use fractal_backend::{UiApi, UiBackend, UiPayload};
use futures::executor::block_on;
use windows::main::MainWindow;
//use bus::BusReader;

mod windows;

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let ui_api = UiBackend::spawn();

    let mut main_window = MainWindow::default();
    main_window.ui_api = Some(RefCell::new(ui_api));

    let app = MainWindow::build_ui(main_window).expect("Failed to build UI");

    /*
    let connect_window = ConnectWindow {
        ui_api: RefCell::new(ui_api.clone()),
        window: Default::default(),
        grid: Default::default(),
        label_midi_input: Default::default(),
        midi_input: Default::default(),
        backend_response_notifier: Default::default()
    };

    let app: connect_window_ui::ConnectWindowUi = ConnectWindow::build_ui(connect_window).expect("Failed to build UI");
    
    let notice_sender = app.backend_response_notifier.sender();
    
    // backend msg notifier pump    
    thread::spawn(move || {        
        loop {
            if let Some(_) = block_on(ui_api.channel.recv()) {
                notice_sender.notice();
            } else {
                break;
            }
        }

        println!("pump out!");
    });
    */

    nwg::dispatch_thread_events();
}