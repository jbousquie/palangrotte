use notify::{RecommendedWatcher, Watcher};
use palangrotte::canary::{handle_event, read_canary_folders, register_canary_folder};
use palangrotte::logger::log_message;
use palangrotte::settings;
use std::process;
use std::sync::mpsc::channel;

fn main() {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = match Watcher::new(
        move |res| {
            if let Ok(event) = res {
                tx.send(event).unwrap();
            }
        },
        Default::default(),
    ) {
        Ok(watcher) => watcher,
        Err(e) => {
            let msg = format!("Failed to create watcher: {}", e);
            log_message(&msg);
            eprintln!("{}", msg);
            process::exit(1);
        }
    };

    let folders = read_canary_folders(settings::FOLDERS_FILE);
    if let Ok(folders) = folders {
        if folders.is_empty() {
            log_message(&format!("{} is empty.", settings::FOLDERS_FILE));
        } else {
            let mut successful_registrations = 0;
            for folder in &folders {
                match register_canary_folder(folder, &mut watcher) {
                    Ok(_) => successful_registrations += 1,
                    Err(e) => log_message(&e),
                }
            }

            if successful_registrations == 0 {
                let msg = "No canary folders could be registered. Exiting.";
                log_message(msg);
                eprintln!("{}", msg);
                process::exit(1);
            }
        }
    }

    // The receiver will block the main thread until a message is received
    for event in rx {
        handle_event(event);
    }
}
