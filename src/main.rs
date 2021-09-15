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

fn run_counter(s: &'static Settings) -> Result<()> {
    let watch_dir = profiles::get_watch_dir()?;

    // Fire once on start.
    update_and_log(s, &watch_dir);

    let mut hotwatch = Hotwatch::new().context("hotwatch failed to initialize")?;
    hotwatch
        .watch(watch_dir.to_owned(), move |event: Event| {
            if let Event::Write(_) = event {
                update_and_log(s, &watch_dir);
            } else if let Event::Create(_) = event {
                update_and_log(s, &watch_dir);
            }
        })
        .context("Failed to watch directory")?;

    Ok(())
}

fn watch_process(s: &Settings) -> Result<()> {
    let delay = time::Duration::from_secs(s.interval);
    let mut sys = System::new_with_specifics(RefreshKind::new().with_processes());

    loop {
        sys.refresh_processes();

        let mut found = false;

        for process in sys.processes().values() {
            let stem = process.exe().file_stem();
            match stem {
                Some(x) if x == ffi::OsStr::new("thunderbird") => found = true,
                _ => {}
            };
        }

        if !found {
            write_count(s, "???")?
        }
        thread::sleep(delay);
    }
}

fn main() {
    human_panic::setup_panic!();
    env_logger::Builder::from_env(Env::default().filter_or("LOG_LEVEL", "info")).init();

    lazy_static! {
        static ref SETTINGS: Settings = argh::from_env();
    }

    if SETTINGS.quiet && SETTINGS.output.is_none() {
        error!("Cannot be quiet and have no output");
        exit(1);
    }

    if let Err(e) = run_counter(&SETTINGS) {
        error!("{}", e);
        std::process::exit(1);
    }

    if let Err(e) = watch_process(&SETTINGS) {
        error!("{}", e);
        std::process::exit(1);
    }
}
