use serde::{Deserialize, Serialize};

pub const QUALIFIER: &str = "dev";
pub const APP_NAME: &str = "Calar";
pub const ORG_NAME: &str = "calar";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub app_name: String,
    pub addr: String,
    pub port: u16,
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
            addr: String::from("0.0.0.0"),
            port: 1414,
            tracto_prefix: String::from("https://scribaproject.space/api/v1.0"),
            translator_substr: String::from("(перевод.)"),
            semester: Semester {
                start_md: (2, 6),
                end_md: (5, 31),
            },
        }
    }
}
