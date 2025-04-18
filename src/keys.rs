pub mod keys {
    pub const KEYS: [&str; 14] = [
        "j to navigate down",
        "k to navigate up",
        "l to enter directory",
        "h or -",
        "a to create file",
        "o to open",
        "D to delete (focused file or when in selected files pane all files)",
        "r to rename",
        "q to quit the tui",
        "c to toggle cheatsheet",
        "1 to focus file pane",
        "2 to focus selected files pane",
        "Space to add/remove file to selected files pane",
        "Esc in input mode to abort input action",
    ];
    use crossterm::event::{self, Event, KeyCode};

    use crate::{
        cmd::cmd::open_file_with_system_app,
        file::file::toggle_selected_file,
        input_action::input_action::{
            handle_create_file, handle_delete_file, handle_rename_file, InputAction,
        },
        utils::utils::{enter_directory, get_is_in_input_mode, navigate_to_parent_directory},
        widget::widget::{
            add_char_input, get_selected_item_from_list_state, handle_backspace,
            reset_current_message_and_input, Pane,
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

    fn handle_enter(app_state: &mut AppState) {
        match app_state.input_action {
            InputAction::None => {
                enter_directory(app_state);
            }
            InputAction::CreateFile => {
                handle_create_file(app_state);
            }
            InputAction::DeleteFile => {
                handle_delete_file(app_state);
            }
            InputAction::RenameFile => {
                let result = handle_rename_file(app_state);
                match result {
                    Ok(()) => app_state.message = "Successfully renamed file!".to_string(),
                    Err(val) => app_state.message = format!("Failed to rename file: {}", val),
                }
            }
        };
    }

    fn handle_char(char: char, app_state: &mut AppState) -> Result<&str, ()> {
        if app_state.input_action != InputAction::None {
            add_char_input(char, app_state);
            return Ok("ok");
        }

        match char {
            'j' => handle_j_char(app_state),
            'k' => handle_k_char(app_state),
            'q' => return handle_q_char(app_state),
            ' ' => handle_space(app_state),
            'h' | '-' => navigate_to_parent_directory(app_state),
            'l' => enter_directory(app_state),
            '1' => handle_one_char(app_state),
            '2' => handle_two_char(app_state),
            'D' => handle_uppercase_d_char(app_state),
            'a' => handle_a_char(app_state),
            'o' => handle_o_char(app_state),
            'r' => handle_r_char(app_state),
            'c' => handle_c_char(app_state),
            _ => {}
        }
        Ok("ok")
    }

    fn handle_c_char(app_state: &mut AppState) {
        app_state.show_cheatsheet = !app_state.show_cheatsheet;
    }

    fn handle_r_char(app_state: &mut AppState) {
        let file = get_selected_item_from_list_state(&app_state.file_list_state, &app_state.files);
        app_state.message = "Please enter the new filename. Esc to abort".to_string();
        app_state.input_action = InputAction::RenameFile;
        app_state.user_input = file.full_path.clone();
    }

    fn handle_j_char(app_state: &mut AppState) {
        match app_state.pane {
            Pane::Files => app_state.file_list_state.select_next(),
            Pane::SelectedFiles => app_state.selected_files_list_state.select_next(),
        }
    }

    fn handle_k_char(app_state: &mut AppState) {
        match app_state.pane {
            Pane::Files => app_state.file_list_state.select_previous(),
            Pane::SelectedFiles => app_state.selected_files_list_state.select_previous(),
        }
    }

    fn handle_q_char(app_state: &mut AppState) -> Result<&str, ()> {
        if app_state.input_action == InputAction::None {
            return Ok("quit");
        }
        return Ok("ok");
    }

    fn handle_space(app_state: &mut AppState) {
        let selected_file_index = app_state.file_list_state.selected();
        let selected_file = app_state
            .files
            .get(selected_file_index.expect("there should be a selected file"))
            .expect("the selected file should exist");

        let new_selected_files = toggle_selected_file(&app_state.selected_files, selected_file);
        app_state.selected_files = new_selected_files;
    }

    fn handle_one_char(app_state: &mut AppState) {
        if app_state.pane != Pane::Files && app_state.input_action == InputAction::None {
            app_state.pane = Pane::Files;
        }
    }

    fn handle_two_char(app_state: &mut AppState) {
        if app_state.pane != Pane::SelectedFiles && app_state.input_action == InputAction::None {
            app_state.pane = Pane::SelectedFiles;
            if app_state.selected_files_list_state.selected() == None
                && !app_state.selected_files.is_empty()
            {
                app_state.selected_files_list_state.select(Some(0));
            }
        }
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

    fn handle_o_char(app_state: &mut AppState) {
        let selected_file =
            get_selected_item_from_list_state(&app_state.file_list_state, &app_state.files);
        let full_path_of_selected_file = &selected_file.full_path;
        let result = open_file_with_system_app(&full_path_of_selected_file);
        match result {
            Ok(_) => {}
            Err(err) => app_state.message = err.to_string(),
        }
    }
}
