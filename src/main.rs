mod count;
mod profiles;
mod settings;

use anyhow::{Context, Result, anyhow};
use count::count_all;
use env_logger::Env;
use lazy_static::lazy_static;
use log::{error, info};
use notify::{DebouncedEvent, RecursiveMode, Watcher, watcher};
use settings::Settings;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::time::Duration;
use std::{ffi, fs, thread, time};
use sysinfo::{ProcessExt, RefreshKind, System, SystemExt};

lazy_static! {
    static ref SETTINGS: Settings = argh::from_env();
    static ref PROFILE_DIR: PathBuf = profiles::get_watch_dir();
}

fn write_count(s: &Settings, data: &str) -> Result<()> {
    if !s.quiet {
        println!("{data}");
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

#[allow(clippy::collapsible_if)]
fn update_and_log(s: &Settings, watch_dir: &Path, err_tx: &Sender<anyhow::Error>) {
    if let Err(err) = update_count(s, watch_dir) {
        if err_tx
            .send(err.context("failed to update unread count after change event"))
            .is_err()
        {
            error!("Failed to forward update error to main thread");
        }
    }
}

fn watch_filesystem(s: &'static Settings, path: &'static Path, err_tx: Sender<anyhow::Error>) -> Result<()> {
    info!("Watching {}", path.display());
    thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let mut watcher = match watcher(tx, Duration::from_secs(2)) {
            Ok(w) => w,
            Err(e) => {
                let _ = err_tx.send(anyhow!("unable to start filesystem watcher: {}", e));
                return;
            }
        };

        if let Err(e) = watcher.watch(path, RecursiveMode::Recursive) {
            let _ = err_tx.send(anyhow!("unable to watch {} recursively: {}", path.display(), e));
            return;
        }

        loop {
            match rx.recv() {
                Ok(DebouncedEvent::Write(_)) => {
                    update_and_log(s, path, &err_tx);
                }
                Ok(_) => {}
                Err(e) => {
                    let _ = err_tx.send(anyhow!("watch event channel failed: {}", e));
                    break;
                }
            }
        }
    });

    Ok(())
}

fn watch_thunderbird_process(s: &Settings, path: &Path, err_rx: &Receiver<anyhow::Error>) -> Result<()> {
    let delay = time::Duration::from_secs(s.interval);
    let mut sys = System::new_with_specifics(RefreshKind::new().with_processes());
    let mut was_running = true;
    let mut first = true;

    loop {
        match err_rx.try_recv() {
            Ok(err) => return Err(err),
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => return Err(anyhow!("file watcher thread disconnected")),
        }

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

    if SETTINGS.quiet && SETTINGS.output.is_none() {
        error!("Cannot be quiet and have no output");
        exit(1);
    }

    let (err_tx, err_rx) = mpsc::channel();

    if let Err(e) = watch_filesystem(&SETTINGS, &PROFILE_DIR, err_tx) {
        error!("{}", e);
        std::process::exit(1);
    }
    if let Err(e) = watch_thunderbird_process(&SETTINGS, &PROFILE_DIR, &err_rx) {
        error!("{}", e);
        std::process::exit(1);
    }
}
