use crate::{model::FractalModel, structs::FractalCmdWithU14, functions::FractalFunction, structs::FractalCmdWithU7, structs::FractalU14, structs::FractalU7};

pub struct Commands {
    model: FractalModel
}

impl Commands {
    pub fn new (model: FractalModel) -> Self {
        Commands {
            model: model
        }
    }

    pub fn get_current_preset_info(&self) -> FractalCmdWithU14 {
        FractalCmdWithU14::new(self.model, FractalFunction::PRESET_INFO, FractalU14::new_all())
    }

    pub fn get_current_scene_info(&self) -> FractalCmdWithU7 {
        FractalCmdWithU7::new(self.model, FractalFunction::GET_SCENE_NAME, FractalU7::new_all())
    }
}

#[test]
fn test_commands() {
    use packed_struct::PackedStruct;

    let commands = Commands::new(FractalModel::III);

    let cmd = commands.get_current_preset_info();
    assert_eq!(&[0xF0, 0x0, 0x1, 0x74, 0x10, 0xD, 0x7F, 0x7F, 0x18, 0xF7], &cmd.pack());

    let cmd = commands.get_current_scene_info();
    assert_eq!(&[0xF0, 0x0, 0x1, 0x74, 0x10, 0xE, 0x7F, 0x64, 0xF7], &cmd.pack());
}