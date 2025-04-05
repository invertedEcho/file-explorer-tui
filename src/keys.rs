pub mod keys {
    use crossterm::event::{self, Event, KeyCode};

    use crate::{
        file::file::{create_file, get_files_for_dir, toggle_selected_file},
        utils::utils::{
            delete_currently_selected_file, delete_selected_files, enter_directory,
            get_is_in_input_mode, navigate_to_parent_directory,
            refresh_files_for_working_directory,
        },
        widget::widget::{
            add_char_input, get_selected_item_from_list_state, handle_backspace,
            reset_current_message_and_input, reset_input, InputAction, Pane,
        },
        AppState,
    };

    pub fn handle_key_event(app_state: &mut AppState) -> Result<&str, ()> {
        let event = event::read().expect("can read event");
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Backspace => handle_backspace(app_state),
                KeyCode::Char(char) => return handle_char(char, app_state),
                KeyCode::Esc => handle_escape(app_state),
                KeyCode::Enter => handle_enter(app_state),
                _ => {}
            }
        }
        return Ok("ok");
    }

    fn handle_escape(app_state: &mut AppState) {
        let is_in_input_mode = get_is_in_input_mode(app_state);
        if is_in_input_mode {
            reset_current_message_and_input(app_state);
        }
    }

    // TODO: refactor
    fn handle_enter(app_state: &mut AppState) {
        if app_state.input_action == InputAction::CreateFile {
            let full_path = app_state.working_directory.clone() + "/" + &app_state.user_input;
            let result = create_file(&full_path);
            match result {
                Ok(msg) => {
                    app_state.message = msg;
                }
                Err(error) => {
                    app_state.message = format!("Failed to create file/dir: {}", error);
                }
            }
            refresh_files_for_working_directory(app_state);
            reset_input(app_state);
        } else {
            let is_in_input_mode = get_is_in_input_mode(app_state);
            if is_in_input_mode {
                let user_input = &app_state.user_input;
                let is_confirmed = user_input == "y" || user_input == "yes";
                if is_confirmed {
                    match app_state.pane {
                        Pane::Files => delete_currently_selected_file(app_state),
                        Pane::SelectedFiles => delete_selected_files(app_state),
                    }
                } else {
                    reset_current_message_and_input(app_state);
                }
            } else {
                enter_directory(app_state);
            }
        }
    }

    fn handle_char(char: char, app_state: &mut AppState) -> Result<&str, ()> {
        if app_state.input_action != InputAction::None {
            add_char_input(char, app_state);
        }

        match char {
            'j' => match app_state.pane {
                Pane::Files => app_state.file_list_state.select_next(),
                Pane::SelectedFiles => app_state.selected_files_list_state.select_next(),
            },
            'k' => match app_state.pane {
                Pane::Files => app_state.file_list_state.select_previous(),
                Pane::SelectedFiles => app_state.selected_files_list_state.select_previous(),
            },
            'q' => {
                if app_state.input_action == InputAction::None {
                    return Ok("quit");
                }
            }

            ' ' => {
                let selected_file_index = app_state.file_list_state.selected();
                let selected_file = app_state
                    .files
                    .get(selected_file_index.expect("there should be a selected file"))
                    .expect("the selected file should exist");

                let new_selected_files =
                    toggle_selected_file(&app_state.selected_files, selected_file);
                app_state.selected_files = new_selected_files;
            }
            'h' | '-' => navigate_to_parent_directory(app_state),
            'l' => enter_directory(app_state),
            '1' => {
                if app_state.pane != Pane::Files && app_state.input_action == InputAction::None {
                    app_state.pane = Pane::Files;
                }
            }
            '2' => {
                if app_state.pane != Pane::SelectedFiles
                    && app_state.input_action == InputAction::None
                {
                    app_state.pane = Pane::SelectedFiles;
                    if app_state.selected_files_list_state.selected() == None
                        && !app_state.selected_files.is_empty()
                    {
                        app_state.selected_files_list_state.select(Some(0));
                    }
                }
            }
            'D' => handle_uppercase_d_char(app_state),
            'a' => handle_a_char(app_state),
            _ => {}
        }
        Ok("ok")
    }

    fn handle_uppercase_d_char(app_state: &mut AppState) {
        match app_state.pane {
            Pane::Files => {
                let file =
                    get_selected_item_from_list_state(&app_state.file_list_state, &app_state.files);
                app_state.input_action = InputAction::DeleteFile;
                app_state.message = String::from(format!(
                    "Please confirm deletion of file {} with y/yes. Esc to abort",
                    file.full_path
                ));
            }
            Pane::SelectedFiles => {
                app_state.input_action = InputAction::DeleteFile;
                app_state.message =
                    "Please confirm deletion of all selected files with y/yes. Esc to abort"
                        .to_string();
            }
        }
    }

    fn handle_a_char(app_state: &mut AppState) {
        app_state.input_action = InputAction::CreateFile;
        app_state.message =
            "Enter the name for new filename: (Tip: use a trailing slash to create a directory)"
                .into();
    }
}
