mod count;
mod profiles;
mod settings;

use anyhow::{Context, Result};
use count::count_all;
use env_logger::Env;
use hotwatch::Event;
use hotwatch::Hotwatch;
use lazy_static::*;
use log::error;
use settings::Settings;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{ffi, fs, thread, time};
use sysinfo::{ProcessExt, RefreshKind, System, SystemExt};

fn write_count(s: &Settings, data: &str) -> Result<()> {
    if !s.quiet {
        println!("{}", data);
    }

    if let Some(path) = &s.output {
        let file = PathBuf::from(path);
        fs::write(file, data).context("failed to write file")?;
    }

    Ok(())
}

fn update_count(s: &Settings, watch_dir: &Path) -> Result<()> {
    let data = count_all(watch_dir)?.display();

    write_count(s, &data)?;

    Ok(())
}

fn update_and_log(s: &Settings, watch_dir: &Path) {
    if let Err(e) = update_count(s, watch_dir) {
        error!("{}", e);
        exit(1);
    }
}

fn run_counter(s: &'static Settings, path: &'static Path) -> Result<()> {
    // Fire once on start.
    update_and_log(s, path);

    let mut hotwatch = Hotwatch::new().context("hotwatch failed to initialize")?;
    hotwatch
        .watch(path.to_owned(), move |event: Event| {
            if let Event::Write(_) = event {
                update_and_log(s, path);
            } else if let Event::Create(_) = event {
                update_and_log(s, path);
            }
        })
        .context("Failed to watch directory")?;

    Ok(())
}

fn watch_process(s: &Settings, path: &Path) -> Result<()> {
    let delay = time::Duration::from_secs(s.interval);
    let mut sys = System::new_with_specifics(RefreshKind::new().with_processes());
    let mut was_running = true;

    loop {
        sys.refresh_processes();
        let mut running = false;

        for process in sys.processes().values() {
            let stem = process.exe().file_stem();
            match stem {
                Some(x) if x == ffi::OsStr::new("thunderbird") => {
                    running = true;
                    break;
                }
                _ => {}
            };
        }

        if was_running && !running {
            write_count(s, "???")?;
        } else if !was_running && running {
            update_count(s, path)?;
        }

        was_running = running;
        thread::sleep(delay);
    }
}

fn main() {
    human_panic::setup_panic!();
    env_logger::Builder::from_env(Env::default().filter_or("LOG_LEVEL", "info")).init();

    lazy_static! {
        static ref SETTINGS: Settings = argh::from_env();
        static ref PROFILE_DIR: PathBuf = profiles::get_watch_dir();
    }

    if SETTINGS.quiet && SETTINGS.output.is_none() {
        error!("Cannot be quiet and have no output");
        exit(1);
    }

    if let Err(e) = run_counter(&SETTINGS, &PROFILE_DIR) {
        error!("{}", e);
        std::process::exit(1);
    }
    if let Err(e) = watch_process(&SETTINGS, &PROFILE_DIR) {
        error!("{}", e);
        std::process::exit(1);
    }
}
