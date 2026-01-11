use flexi_logger::{FileSpec, LoggerHandle};

use crate::env::get_home_dir;

pub fn setup_logger_handle() -> LoggerHandle {
    let home_dir = get_home_dir().expect("$HOME is set");
    let log_file_directory = home_dir + "/.cache/file-explorer-tui/";

    let default_file_spec = FileSpec::default().directory(log_file_directory);

    flexi_logger::Logger::try_with_str("info")
        .unwrap()
        .append()
        .log_to_file(FileSpec::suppress_timestamp(default_file_spec))
        .start()
        .unwrap()
}
