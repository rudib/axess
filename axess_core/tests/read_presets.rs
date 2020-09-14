use fractal_protocol::messages::{preset::PresetHelper, FractalAudioMessages, scene::SceneHelper};
use axess_core::{FractalCoreError, transport::midi::Midi, backend::UiBackend, transport::Transport};
use packed_struct::PackedStructSlice;

#[test]
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
        let scenes = connection.device.model.number_of_scenes().unwrap();

        //for i in 0..total_presets {
        for i in 0..512 {
            let preset = connection.send_and_wait_for(&PresetHelper::get_preset_info(connection.device.model, i).pack_to_vec()?,
|msg| {
                match msg {
                    FractalAudioMessages::Preset(preset) => {
                        Some(preset.clone())
                    },
                    _ => None
                }
            }).await.map_err(|_| FractalCoreError::MissingValue("Preset".into()))?;

            println!("preset: {:?}", preset);
        }

        Ok(())
    }

    tokio_test::block_on(test())?;
    Ok(())
}