pub mod watcher {
    use core::panic;
    use std::{path::Path, sync::mpsc::Sender};

    use log::{info, warn};
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

    use crate::mpsc_utils::mpsc_utils::send_message_or_panic;

    pub fn setup_directory_watcher(
        directory: &String,
        watch_for_hidden_files: bool,
        mut sender_for_directory_watcher: Sender<String>,
    ) {
        info!("Setting up directory watcher...");
        let (sender, receiver) = std::sync::mpsc::channel();

        // we pass in the sender so notify can send events
        let mut watcher = RecommendedWatcher::new(sender, Config::default())
            .expect("Can setup recommended_watcher from notify");

        info!("Trying to watch given directory in non recursive mode");
        let result = watcher.watch(Path::new(directory), RecursiveMode::NonRecursive);
        match result {
            Ok(()) => {
                info!(
                    "Successfully setup directory watcher for directory {:?} with recursive mode.",
                    directory
                )
            }
            Err(err) => {
                warn!("Failed to setup directory watcher: {:?}", err)
            }
        }

        // we care about
        // when a file is created in the same directory
        // when a file is removed in the same directory

        // here we just infinitely loop over any events that the receiver receives from the sender
        for res in receiver {
            match res {
                Ok(event) => match event.kind {
                    notify::EventKind::Create(_) => {
                        info!("Received create event in directory watcher: {:?}", event);
                        send_message_or_panic(
                            &mut sender_for_directory_watcher,
                            "create_event".to_string(),
                        );
                    }
                    _ => {}
                },
                Err(e) => panic!("watch error: {:?}", e),
            }
        }
    }
}
