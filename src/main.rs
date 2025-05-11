use core::panic;
use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use color_eyre::Result;
use directory_watcher::watcher::setup_directory_watcher;
use input_action::input_action::InputAction;
use keys::keys::handle_key_event;
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
mod mpsc_utils;
mod utils;
mod widget;

// TODO:
// fix: hot-reload of files via watcher or just simple key to reload?
// fix: truncate filename in deletion message (and other places too)
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
    receiver_for_directory_watcher: Receiver<String>,
    sender_for_draw_widget_function: Sender<String>,
}

struct AppStateMessage {
    previous_messages: Vec<String>,
    current_message: String,
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

    let show_hidden_files = false;

    let initial_files = get_files_for_dir(&initial_directory, show_hidden_files);
    let sorted_initial_files = sort_file_paths_dirs_first_then_files(&initial_files);

    // FIX: we should have one receiver/sender pair for directory watcher and one for message bus (UI
    // message field)
    let (sender_for_directory_watcher, receiver_for_directory_watcher) = channel();
    let sender_for_draw_widget_to_frame = sender_for_directory_watcher.clone();

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
        receiver_for_directory_watcher,
        sender_for_draw_widget_function: sender_for_draw_widget_to_frame,
    };

    let mut app_state_message = AppStateMessage {
        current_message: String::from("Initial message"),
        previous_messages: vec![],
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

    thread::spawn(move || {
        setup_directory_watcher(
            &initial_directory,
            show_hidden_files,
            sender_for_directory_watcher,
        );
    });

    loop {
        let maybe_result_from_sender = app_state.receiver_for_directory_watcher.try_recv();
        match maybe_result_from_sender {
            Ok(val) => {
                app_state_message.current_message = format!("message: {:?}", val);
            }
            Err(err) => {}
        }
        terminal.draw(|frame| {
            draw_widgets_to_frame(frame, &mut app_state, &app_state_message.current_message)
        })?;

        let handle_key_event_result = handle_key_event(&mut app_state);
        match handle_key_event_result {
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
