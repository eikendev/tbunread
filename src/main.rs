mod count;
mod profiles;
mod settings;

use anyhow::{Context, Result};
use count::count_all;
use env_logger::Env;
use lazy_static::*;
use log::{error, info};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use settings::Settings;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::mpsc::channel;
use std::time::Duration;
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
    info!("Watching {}", path.display());
    thread::spawn(move || {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
        watcher.watch(path.to_owned(), RecursiveMode::Recursive).unwrap();
        loop {
            match rx.recv() {
                Ok(DebouncedEvent::Write(_)) => {
                    update_and_log(s, path);
                }
                Ok(_) => {}
                Err(e) => error!("Watch error: {}", e.to_string()),
            }
        }
    });

    Ok(())
}

fn watch_process(s: &Settings, path: &Path) -> Result<()> {
    let delay = time::Duration::from_secs(s.interval);
    let mut sys = System::new_with_specifics(RefreshKind::new().with_processes());
    let mut was_running = true;
    let mut first = true;

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
        } else if first || (!was_running && running) {
            update_count(s, path)?;
        }

        was_running = running;
        first = false;
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
