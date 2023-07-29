use crate::{config, Request};

use icalendar::Calendar;
use std::{io::Write, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create a file")]
    Create(#[source] std::io::Error),

    #[error("failed to write to a file")]
    Write(#[source] std::io::Error),

    #[error("failed to delete a file")]
    Delete(#[source] std::io::Error),
}

pub fn gen_filename<T>(req: &Request) -> String {
    let request_type = std::any::type_name::<T>()
        .split("::")
        .last()
        .unwrap()
        .to_lowercase();
    format!(
        "{}-{}-{}-{}-{}{}.ics",
        request_type,
        req.department,
        req.form,
        req.group,
        req.subgroups.join("_"),
        if req.translator { "-t" } else { "" }
    )
}

fn get_cache_dir() -> PathBuf {
    let proj_dirs =
        directories::ProjectDirs::from(config::QUALIFIER, config::ORG_NAME, config::APP_NAME)
            .expect("No valid config directory could be retrieved from the operating system");

    proj_dirs.cache_dir().join("calendars")
}

pub fn save_to_cache<T>(req: &Request, calendar: Calendar) -> Result<PathBuf, Error> {
    let cache_dir = get_cache_dir();
    std::fs::create_dir_all(cache_dir.clone()).map_err(Error::Create)?;
    let file_path = cache_dir.join(gen_filename::<T>(req));

    let mut file = std::fs::File::create(file_path.clone()).map_err(Error::Create)?;
    file.write_all(calendar.to_string().as_bytes())
        .map_err(Error::Write)?;

    Ok(file_path)
}

pub fn look_up_in_cache<T>(req: &Request) -> Option<PathBuf> {
    let file_path = get_cache_dir().join(gen_filename::<T>(req));

    if file_path.exists() {
        Some(file_path)
    } else {
        None
    }
}

pub fn prune_cache() -> Result<(), Error> {
    std::fs::remove_dir_all(get_cache_dir()).map_err(Error::Delete)
}
