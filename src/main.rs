mod count;
mod profiles;
mod settings;

use anyhow::{Context, Result};
use count::count_all;
use env_logger::Env;
use hotwatch::blocking::{Flow, Hotwatch};
use hotwatch::Event;
use log::error;
use settings::Settings;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

fn update_count(s: &Settings, watch_dir: &Path) -> Result<()> {
    let output = count_all(watch_dir)?.display();

    if !s.quiet {
        println!("{}", output);
    }

    if let Some(path) = &s.output {
        let file = PathBuf::from(path);
        fs::write(file, output).context("failed to write file")?;
    }

    Ok(())
}

fn update_and_log(s: &Settings, watch_dir: &Path) {
    if let Err(e) = update_count(&s, &watch_dir) {
        error!("{}", e);
        exit(1);
    }
}

fn run_counter(s: Settings) -> Result<()> {
    let watch_dir = profiles::get_watch_dir()?;

    // Fire once on start.
    update_and_log(&s, &watch_dir);

    let mut hotwatch = Hotwatch::new().context("hotwatch failed to initialize")?;
    hotwatch
        .watch(watch_dir.to_owned(), move |event: Event| {
            if let Event::Write(_) = event {
                update_and_log(&s, &watch_dir);
            } else if let Event::Create(_) = event {
                update_and_log(&s, &watch_dir);
            }
            Flow::Continue
        })
        .context("Failed to watch directory")?;
    hotwatch.run();

    Ok(())
}

fn main() {
    human_panic::setup_panic!();
    env_logger::Builder::from_env(Env::default().filter_or("LOG_LEVEL", "info")).init();

    let settings: Settings = argh::from_env();

    if settings.quiet && settings.output.is_none() {
        error!("Cannot be quiet and have no output");
        exit(1);
    }

    if let Err(e) = run_counter(settings) {
        error!("{}", e);
    }
}
