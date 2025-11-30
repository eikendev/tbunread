use anyhow::{Result, bail};
use ini::Ini;
use log::{debug, error};
use std::path::PathBuf;

pub fn get_thunderbird_home() -> Result<PathBuf> {
    if let Some(home_dir) = dirs::home_dir() {
        return Ok(home_dir.join(".thunderbird"));
    }
    bail!("Unable to get user home directory");
}

fn process_sections(file: Ini) -> Result<PathBuf> {
    for (sec, prop) in file.iter() {
        if let Some(s) = sec {
            if !s.starts_with("Install") {
                continue;
            }

            let name = prop.iter().filter(|x| x.0 == "Default").map(|x| x.1).last();

            if let Some(n) = name {
                return Ok(get_thunderbird_home()?.join(n).join("ImapMail").join("tbunread"));
            }
        }
    }
    bail!("Unable to find default section");
}

pub fn _get_watch_dir() -> Result<PathBuf> {
    let path = get_thunderbird_home()?.join("profiles.ini");
    debug!("Reading {}", path.display());

    if let Ok(file) = Ini::load_from_file(path) {
        let watch_dir = process_sections(file)?;
        return Ok(watch_dir);
    }
    bail!("Unable to read profiles.ini");
}

pub fn get_watch_dir() -> PathBuf {
    let path = _get_watch_dir();
    if let Err(e) = path {
        error!("{}", e);
        std::process::exit(1);
    }

    path.unwrap()
}
