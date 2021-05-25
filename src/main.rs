mod count;
mod profiles;
mod settings;

use count::count_all;
use hotwatch::blocking::{Flow, Hotwatch};
use hotwatch::Event;
use settings::Settings;
use std::fs;
use std::path::{Path, PathBuf};

fn update_count(s: &Settings, watch_dir: &Path) {
    let output = count_all(watch_dir).display();
    println!("[*] Writing: {}", output);

    if let Some(path) = &s.output {
        let file = PathBuf::from(path);
        fs::write(file, output).expect("failed to write file");
    }
}

fn main() {
    let settings: Settings = argh::from_env();
    let watch_dir = profiles::get_watch_dir();

    let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize");
    hotwatch
        .watch(watch_dir.to_owned(), move |event: Event| {
            if let Event::Write(_) = event {
                update_count(&settings, &watch_dir);
            } else if let Event::Create(_) = event {
                update_count(&settings, &watch_dir);
            }
            Flow::Continue
        })
        .expect("failed to watch directory");
    hotwatch.run();
}
