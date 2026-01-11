use std::{
    io::Error,
    process::{Command, Output},
};

const LINUX_OPEN: &str = "xdg-open";
const MACOS_OPEN: &str = "open";
const WINDOWS_OPEN: &str = "start";

fn get_open_command_for_system_arch() -> &'static str {
    if cfg!(windows) {
        WINDOWS_OPEN
    } else if cfg!(target_os = "linux") {
        LINUX_OPEN
    } else if cfg!(target_os = "macos") {
        MACOS_OPEN
    } else {
        panic!("Unsupported system.");
    }
}

pub fn open_file_with_system_app(file_path: &str) -> Result<Output, Error> {
    let open_command = get_open_command_for_system_arch();

    let mut new_command = Command::new(open_command);
    let arg_added = new_command.arg(file_path);

    arg_added.output()
}
