extern crate fractal_protocol;
extern crate broadcaster;

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use windows::{common::{WindowApi, FractalWindow}, main::MainWindow};
use log4rs::{append::console::{Target, ConsoleAppender}, config::{Appender, Config, Root}};
use log::LevelFilter;
use fractal_backend::backend::UiBackend;

mod windows;

fn main() {
    // init logging
    {
        let stdout = ConsoleAppender::builder().target(Target::Stdout).build();
        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .build(Root::builder()
                .appender("stdout")
                .build(LevelFilter::Trace)
            ).unwrap();
        log4rs::init_config(config).unwrap();
    }    

    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let ui_api = UiBackend::spawn();
    let window_api = WindowApi::new(ui_api);

    let main_window_thread = MainWindow::spawn((), window_api).unwrap();
    main_window_thread.join().unwrap();
}