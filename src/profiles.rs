use ini::Ini;
use log::{debug, info};
use std::path::PathBuf;

pub fn get_thunderbird_home() -> PathBuf {
    if let Some(home_dir) = dirs::home_dir() {
        return home_dir.join(".thunderbird");
    }
    panic!("cannot get user home directory");
}

fn process_sections(file: Ini) -> PathBuf {
    for (sec, prop) in file.iter() {
        if let Some(s) = sec {
            if !s.starts_with("Install") {
                continue;
            }

            let name = prop.iter().filter(|x| x.0 == "Default").map(|x| x.1).last();

            if let Some(n) = name {
                return get_thunderbird_home().join(n).join("ImapMail").join("tbunread");
            }
        }
    }
    panic!("no default section was found");
}

pub fn get_watch_dir() -> PathBuf {
    let path = get_thunderbird_home().join("profiles.ini");
    debug!("Reading {}", path.display());

    if let Ok(file) = Ini::load_from_file(path) {
        let watch_dir = process_sections(file);
        info!("Watching {}", watch_dir.display());
        return watch_dir;
    }
    panic!("cannot read profiles.ini");
}
