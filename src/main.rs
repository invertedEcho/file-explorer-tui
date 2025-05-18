use log::{error, info};
use notify::Watcher;
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};

use color_eyre::Result;
use directory_watcher::watcher::{handle_notify_watcher_event, setup_directory_watcher};
use input_action::input_action::InputAction;
use keys::keys::handle_key_event;
use logger::logger::setup_logger_handle;
use ratatui::{widgets::ListState, DefaultTerminal};

use env::env::get_home_dir;
use file::file::{get_files_for_dir, sort_file_paths_dirs_first_then_files, File};
use widget::widget::{draw_widgets_to_frame, Pane};

mod cmd;
mod directory_watcher;
mod env;
mod file;
mod input_action;
mod keys;
mod logger;
mod mpsc_utils;
mod utils;
mod widget;

// TODO:
// fix: truncate filename in deletion message (and other places too)
// fix: permission errors (navigate into /root)
//
// IDEAS:
// - config support to edit keybinds also interactive ui in tui itself to edit these
// - archive navigation and editing
// - live preview files (images as ascii previews or sixel)
// - git integration?

struct AppState {
    files: Vec<File>,
    selected_files: Vec<File>,
    working_directory: String,
    pane: Pane,
    user_input: String,
    input_action: InputAction,
    file_list_state: ListState,
    list_state_index_of_directory: HashMap<String, usize>,
    selected_files_list_state: ListState,
    show_cheatsheet: bool,
    show_selected_files_pane: bool,
    show_hidden_files: bool,
    sender_for_ui_message: Sender<String>,
}

struct AppStateMessage {
    // previous_messages: Vec<String>,
    current_message: String,
}

fn main() -> Result<()> {
    setup_logger_handle();
    info!("file-explorer-tui is starting...");

    // installs error handling hook
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    info!("file-explorer-tui is stopping...");
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    // TODO: fall back to something sane
    let initial_directory = get_home_dir().expect("$HOME is set");

    let show_hidden_files = false;

    let initial_files = get_files_for_dir(&initial_directory, show_hidden_files);
    let sorted_initial_files = sort_file_paths_dirs_first_then_files(&initial_files);

    let (sender_for_ui_message, receiver_for_ui_message) = channel();

    let mut app_state = AppState {
        files: sorted_initial_files,
        selected_files: vec![],
        working_directory: initial_directory.clone(),
        pane: Pane::Files,
        user_input: String::from(""),
        input_action: InputAction::None,
        file_list_state: ListState::default(),
        selected_files_list_state: ListState::default(),
        list_state_index_of_directory: HashMap::new(),
        show_cheatsheet: false,
        show_selected_files_pane: true,
        show_hidden_files,
        sender_for_ui_message,
    };

    let mut app_state_message = AppStateMessage {
        current_message: String::from("Initial message"),
        // previous_messages: vec![],
    };

    app_state
        .list_state_index_of_directory
        .insert(initial_directory.clone(), 0);

    let list_state_index_of_initial_directory = Some(
        *app_state
            .list_state_index_of_directory
            .get(&initial_directory)
            .unwrap(),
    );
    app_state
        .file_list_state
        .select(list_state_index_of_initial_directory);

    let (mut notify_watcher, directory_watcher_receiver) =
        setup_directory_watcher(initial_directory);

    loop {
        let maybe_directory_watcher_receiver_result = directory_watcher_receiver.try_recv();

        match maybe_directory_watcher_receiver_result {
            Ok(result_event_or_error) => match result_event_or_error {
                Ok(event) => handle_notify_watcher_event(event, &mut app_state),

                // what errors are these?
                Err(error) => {
                    error!("error from result_event_or_error: {:?}", error);
                }
            },
            Err(error) => {
                // Ignore Empty message
                if error.to_string() != "receiving on an empty channel" {
                    info!("error from try_recv: {:?}", error);
                }
            }
        }
        let previous_working_directory = app_state.working_directory.clone();

        terminal.draw(|frame| {
            draw_widgets_to_frame(frame, &mut app_state, &app_state_message.current_message)
        })?;

        let handle_key_event_result = handle_key_event(&mut app_state);
        if handle_key_event_result == "quit" {
            break Ok(());
        }

        // UI Message stuff
        let maybe_result_from_ui_message_receiver = receiver_for_ui_message.try_recv();
        match maybe_result_from_ui_message_receiver {
            Ok(result) => {
                app_state_message.current_message = result;
            }
            Err(_) => {}
        }

        // Directory watcher stuff
        // if our working directory changed, we need to stop previous directory watcher and start new
        // one.
        if previous_working_directory != app_state.working_directory {
            info!("Our working directory changed! Stopping previous notify_watcher and starting a new one");
            let unwatch_result = notify_watcher.unwatch(Path::new(&previous_working_directory));
            match unwatch_result {
                Ok(_) => {
                    let new_watch_result = notify_watcher.watch(
                        Path::new(&app_state.working_directory),
                        notify::RecursiveMode::NonRecursive,
                    );
                    match new_watch_result {
                        Ok(_) => {
                            info!(
                                "Successfully watching new working directory: {:?}",
                                app_state.working_directory
                            )
                        }
                        Err(error) => {
                            error!("Failed to watch new working directory: {:?}", error);
                        }
                    }
                }
                Err(error) => {
                    error!("Failed to unwatch directory: {:?}", error);
                }
            }
        }
    }
}
