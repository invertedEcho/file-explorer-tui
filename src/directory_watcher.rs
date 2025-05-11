pub mod watcher {
    use core::panic;
    use std::{path::Path, sync::mpsc::Sender, thread, time::Duration};

    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

    use crate::mpsc_utils::mpsc_utils::send_message_or_panic;

    pub fn setup_directory_watcher(
        directory: &String,
        watch_for_hidden_files: bool,
        mut sender_for_directory_watcher: Sender<String>,
    ) {
        let (sender, receiver) = std::sync::mpsc::channel();

        // we pass in the sender so notify can send events
        let mut watcher = RecommendedWatcher::new(sender, Config::default())
            .expect("Can setup recommended_watcher from notify");

        watcher
            .watch(Path::new(directory), RecursiveMode::Recursive)
            .expect("Can watch given directory");

        thread::sleep(Duration::from_secs(3));
        send_message_or_panic(
            &mut sender_for_directory_watcher,
            "hello from setup_watcher".to_string(),
        );

        // we care about
        // when a file is created in the same directory
        // when a file is removed in the same directory

        // here we just infinitely loop over any events that the receiver receives from the sender
        for res in receiver {
            match res {
                Ok(event) => match event.kind {
                    notify::EventKind::Create(_) => {
                        send_message_or_panic(
                            &mut sender_for_directory_watcher,
                            "create_event!".to_string(),
                        );
                    }
                    _ => {}
                },
                Err(e) => panic!("watch error: {:?}", e),
            }
        }
    }
}
