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
use windows::{common::{WindowApi, FractalWindow}, main::MainWindow};
//use bus::BusReader;

mod windows;

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let ui_api = UiBackend::spawn();
    let window_api = WindowApi::new(ui_api);

    let mut main_window = MainWindow::default();
    main_window.set_window_api(window_api);

    let app = MainWindow::build_ui(main_window).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}