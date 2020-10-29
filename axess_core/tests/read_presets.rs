use std::time::Duration;

use log4rs::{append::{console::ConsoleAppender, file::FileAppender, console::Target}, config::Config, config::Appender, config::Root};
use fractal_protocol::messages::{preset::PresetHelper, FractalAudioMessages, scene::SceneHelper, preset::PresetAndName, effects::BlocksHelper};
use axess_core::{FractalCoreError, transport::midi::Midi, backend::UiBackend, transport::Transport, transport::write_struct, transport::write_struct_dyn};
use packed_struct::PackedStructSlice;
use log::{trace, LevelFilter};

extern crate log4rs;

#[test]
#[ignore = "needs a single physical device"]
fn read_all_presets() -> Result<(), FractalCoreError> {

    async fn test() -> Result<(), FractalCoreError> {
        let midi = Midi::new()?;
        let transports: Vec<Box<dyn Transport>> = vec![Box::new(midi)];

        let endpoints = UiBackend::list_endpoints_from_transports(&transports);
        println!("detected endpoints: {:?}", endpoints);

        let mut connection = None;
        for endpoint in endpoints {
            match UiBackend::connect(&endpoint, &transports).await {
                Ok(c) => { connection = Some(c); }
                Err(_) => {}
            }
        }
        let mut connection = connection.ok_or(FractalCoreError::MissingValue("endpoint".into()))?;
        println!("connected?");

        let total_presets = connection.device.model.number_of_presets().unwrap();
        
        for i in 0..total_presets {
            let preset: PresetAndName = connection.send_and_wait_for(&mut PresetHelper::get_preset_info(connection.device.model, i))
                .await.map_err(|_| FractalCoreError::MissingValue("Preset".into()))?;
            println!("preset: {:?}", preset);
        }

        Ok(())
    }

    tokio_test::block_on(test())?;
    Ok(())
}

#[test]
#[ignore = "needs a single physical device"]
fn block_effects() -> Result<(), FractalCoreError> {

    // init logging
    {
        let stdout = ConsoleAppender::builder().target(Target::Stdout).build();
        let file = FileAppender::builder().build("axess-test.log")?;
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

    trace!("log test");

    async fn test() -> Result<(), FractalCoreError> {
        let midi = Midi::new()?;
        let transports: Vec<Box<dyn Transport>> = vec![Box::new(midi)];

        let endpoints = UiBackend::list_endpoints_from_transports(&transports);
        println!("detected endpoints: {:?}", endpoints);

        let mut connection = None;
        for endpoint in endpoints {
            match UiBackend::connect(&endpoint, &transports).await {
                Ok(c) => { connection = Some(c); }
                Err(_) => {}
            }
        }
        let mut connection = connection.ok_or(FractalCoreError::MissingValue("endpoint".into()))?;
        println!("connected?");

        write_struct_dyn(&mut *connection.transport_endpoint, &mut BlocksHelper::get_current_blocks(connection.device.model))?;

        tokio::time::delay_for(Duration::from_millis(500)).await;


        /*
        let total_presets = connection.device.model.number_of_presets().unwrap();
        
        for i in 0..total_presets {
            let preset: PresetAndName = connection.send_and_wait_for(&PresetHelper::get_preset_info(connection.device.model, i).pack_to_vec()?)
                .await.map_err(|_| FractalCoreError::MissingValue("Preset".into()))?;
            println!("preset: {:?}", preset);
        }
        */

        Ok(())
    }

    tokio_test::block_on(test())?;
    Ok(())
}