use std::{path::PathBuf, fs};

use directories::BaseDirs;
use log::{error, info, trace};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AxessConfiguration {
    pub keyboard_shortcuts_axe_edit: bool,
    pub keyboard_shortcuts_presets_and_scenes_function_keys: bool
}

impl AxessConfiguration {
    pub fn read() -> Self {
        if let Some(config_file) = get_config_file_path(false) {
            let contents = match fs::read_to_string(config_file) {
                Ok(s) => s,
                Err(e) => {
                    error!("Error reading the config file contents. {:?}", e);
                    return AxessConfiguration::default();
                }
            };

            let cfg = match serde_json::from_str(&contents) {
                Ok(cfg) => cfg,
                Err(e) => {
                    error!("Failed to deserialize the JSON config file's contents. {:?}", e);
                    return AxessConfiguration::default();
                }
            };

            return cfg;
        } else {
            info!("No config file found.");
        }

        AxessConfiguration::default()
    }

    pub fn save(&self) {
        let json = match serde_json::to_string(&self) {
            Ok(j) => j,
            Err(e) => {
                error!("Failed to serialize the JSON config! {:?}", e);
                return;
            }
        };

        if let Some(config_file) = get_config_file_path(true) {
            match fs::write(&config_file, json) {
                Ok(_) => trace!("Saved the config file to {:?}", &config_file),
                Err(e) => error!("Failed to save the config file to {:?}, error {:?}", &config_file, e)
            }
        } else {
            error!("Failed to get the path to the config file");
        }
    }
}

fn get_config_file_path(create_dir: bool) -> Option<PathBuf> {
    if let Some(dir) = BaseDirs::new() {
        let config_dir = dir.config_dir();
        let config_dir = config_dir.join("Axess");

        if !config_dir.exists() {
            if create_dir == false { return None; }
            match fs::create_dir_all(&config_dir) {
                Ok(_) => (),
                Err(e) => {
                    error!("Failed to create the config directory. Path {:?}, error {:?}", &config_dir, e);
                    return None;
                }
            }
        }

        let config_file = config_dir.join("config.json");
        return Some(config_file);
    }

    None
}

impl Default for AxessConfiguration {
    fn default() -> Self {
        AxessConfiguration {
            keyboard_shortcuts_axe_edit: true,
            keyboard_shortcuts_presets_and_scenes_function_keys: false
        }
    }
}
