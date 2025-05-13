pub mod logger {
    use flexi_logger::{FileSpec, LoggerHandle};

    pub fn setup_logger_handle() -> LoggerHandle {
        let default_file_spec = FileSpec::default();

        flexi_logger::Logger::try_with_str("info")
            .unwrap()
            .append()
            .log_to_file(FileSpec::suppress_timestamp(default_file_spec))
            .start()
            .unwrap()
    }
}
