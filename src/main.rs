use color_eyre::Result;
use input_action::input_action::InputAction;
use keys::keys::handle_key_event;
use ratatui::{widgets::ListState, DefaultTerminal};

use env::env::get_home_dir;
use file::file::{get_files_for_dir, sort_file_paths_dirs_first_then_files, File};
use widget::widget::{draw_widgets_to_frame, Pane};

mod cmd;
mod env;
mod file;
mod input_action;
mod keys;
mod utils;
mod widget;

// TODO:
// fix: remember where we left selected state at when going into dir and going back
// feat: hotkey cheatsheet in-app
// feat: open file via enter with system provided program -> WIP
// fix: hot-reload of files via watcher or just simple key to reload?
// fix: truncate filename in deletion message (and other places too)
// feat: rename files

struct AppState {
    files: Vec<File>,
    selected_files: Vec<File>,
    working_directory: String,
    pane: Pane,
    message: String,
    user_input: String,
    input_action: InputAction,
    file_list_state: ListState,
    selected_files_list_state: ListState,
}

fn main() -> Result<()> {
    // installs error handling hook
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    // TODO: fall back to something sane
    let initial_directory = get_home_dir().expect("$HOME is set");

    let initial_files = get_files_for_dir(&initial_directory);
    let sorted_initial_files = sort_file_paths_dirs_first_then_files(&initial_files);

    // setup app state
    let mut app_state = AppState {
        files: sorted_initial_files,
        selected_files: vec![],
        working_directory: initial_directory,
        pane: Pane::Files,
        message: String::from("Hi!"),
        user_input: String::from(""),
        input_action: InputAction::None,
        file_list_state: ListState::default(),
        selected_files_list_state: ListState::default(),
    };
    app_state.file_list_state.select(Some(0));

    loop {
        terminal.draw(|frame| draw_widgets_to_frame(frame, &mut app_state))?;
        let result = handle_key_event(&mut app_state);
        match result {
            Ok(value) => {
                // TODO: eehhhh i dont know about this
                if value == "quit" {
                    break Ok(());
                }
            }
            Err(_) => continue,
        }
    }
}
