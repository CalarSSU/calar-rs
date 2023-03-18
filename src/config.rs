use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
};

const QUALIFIER: &str = "dev";
const APP_NAME: &str = "Calar";
const ORG_NAME: &str = "calar";
const CONFIG_FILE: &str = "config.toml";

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub app_name: String,
    pub tracto_prefix: String,
    pub translator_substr: String,
    pub semester: Semester,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Semester {
    pub end_md: (u32, u32),
    pub start_md: (u32, u32),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_name: String::from(APP_NAME),
            tracto_prefix: String::from("https://scribabot.tk/api/v1.0"),
            translator_substr: String::from("(перевод.)"),
            semester: Semester {
                start_md: (2, 6),
                end_md: (5, 31),
            },
        }
    }
}

impl Config {
    pub fn from_config_dir() -> Result<Config> {
        let proj_dirs = directories::ProjectDirs::from(QUALIFIER, ORG_NAME, APP_NAME)
            .expect("No valid config directory could be retrieved from the operating system");
        let config_file = proj_dirs.config_dir().join(CONFIG_FILE);

        let mut config_file = match File::open(config_file) {
            Ok(file) => file,
            Err(_) => {
                let cfg = Config::default();
                cfg.dump_to_config_dir()?;
                return Ok(cfg);
            }
        };

        let mut config_content = String::new();
        config_file.read_to_string(&mut config_content)?;

        Ok(toml::from_str(&config_content)?)
    }

    pub fn dump_to_config_dir(&self) -> Result<()> {
        let serialized = toml::to_string(self)?;

        let proj_dirs = directories::ProjectDirs::from(QUALIFIER, ORG_NAME, APP_NAME)
            .expect("No valid config directory could be retrieved from the operating system");
        let config_dir = proj_dirs.config_dir();
        std::fs::create_dir_all(config_dir)?;
        let config_file = config_dir.join(CONFIG_FILE);

        let mut config_file = File::create(config_file)?;
        config_file.write_all(serialized.as_bytes())?;

        Ok(())
    }
}
