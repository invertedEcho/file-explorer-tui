pub mod logger {
    use flexi_logger::{FileSpec, LoggerHandle};

    pub fn setup_logger_handle() -> LoggerHandle {
        flexi_logger::Logger::try_with_str("info")
            .unwrap()
            .log_to_file(FileSpec::default())
            .start()
            .unwrap()
    }
}
