pub mod watcher {
    use std::{path::Path, sync::mpsc::Receiver};

    use log::{info, warn};
    use notify::{Config, Error, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher};

    pub fn setup_directory_watcher(
        initial_directory: String,
    ) -> (INotifyWatcher, Receiver<Result<notify::Event, Error>>) {
        info!("Setting up directory watcher...");
        let (sender, receiver) = std::sync::mpsc::channel();

        // we pass in the sender so notify can send events
        let mut watcher = RecommendedWatcher::new(sender, Config::default())
            .expect("Can setup recommended_watcher from notify");

        info!("Trying to watch given directory in non recursive mode");
        let result = watcher.watch(Path::new(&initial_directory), RecursiveMode::NonRecursive);
        match result {
            Ok(()) => {
                info!(
                    "Successfully setup directory watcher for directory {:?} with non-recursive mode.",
                    initial_directory
                )
            }
            Err(err) => {
                warn!("Failed to setup directory watcher: {:?}", err)
            }
        }
        // We need to return the watcher too, otherwise it will be dropped and the receiver is
        // disconnected
        return (watcher, receiver);
    }
}
