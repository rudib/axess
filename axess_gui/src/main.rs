#![allow(unused_imports)]

extern crate fractal_protocol;
extern crate broadcaster;

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

extern crate packed_struct;
#[macro_use] extern crate packed_struct_codegen;

use windows::{common::{WindowApi, FractalWindow}, main::MainWindow};
use log4rs::{append::console::{Target, ConsoleAppender}, config::{Appender, Config, Root}, append::file::FileAppender};
use log::{LevelFilter, info, trace};
use axess_core::{FractalCoreError, backend::UiBackend};

mod windows;
mod device_state;

fn main() -> Result<(), FractalCoreError> {
    // init logging
    #[cfg(build_profile="debug")]
    {        
        let stdout = ConsoleAppender::builder().target(Target::Stdout).build();
        let file = FileAppender::builder().build("axess.log")?;
        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .appender(Appender::builder().build("file", Box::new(file)))
            .build(Root::builder()
                .appender("stdout")
                .appender("file")
                .build(LevelFilter::Trace)
            ).unwrap();
        log4rs::init_config(config).unwrap();
    }
    #[cfg(not(build_profile="debug"))]
    {
        let file = FileAppender::builder().build("axess.log")?;
        let config = Config::builder()
            .appender(Appender::builder().build("file", Box::new(file)))
            .build(Root::builder()
                .appender("file")
                .build(LevelFilter::Trace)
            ).unwrap();
        log4rs::init_config(config).unwrap();
    }

    info!("Axess starting. Git SHA {}, build profile {}", env!("VERGEN_SHA_SHORT"), env!("BUILD_PROFILE"));

    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let ui_api = UiBackend::spawn()?;
    let window_api = WindowApi::new(ui_api);

    let main_window_thread = MainWindow::spawn((), window_api).unwrap();
    main_window_thread.join().unwrap();

    trace!("Stop.");

    Ok(())
}