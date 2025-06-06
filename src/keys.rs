pub mod keys {
    pub const KEYS: [&str; 17] = [
        "j to navigate down",
        "k to navigate up",
        "l to enter directory",
        "h or - to navigate to the parent directory",
        "a to create file",
        "o to open selected file",
        "D to delete selected file (or when in selected files window all selected files)",
        "r to rename currently selected file",
        "q to quit the tui",
        "H to toggle hidden files",
        "c to toggle cheatsheet",
        "s to toggle selected files window",
        "1 to focus 'Files' window",
        "2 to focus 'Selected files' window",
        "In 'Files': Space to add/remove file to 'Selected files' window",
        "In 'Selected files': Space to remove selected from the window",
        "Esc in input mode to abort current action",
    ];

    use std::time::Duration;

    use crossterm::event::{poll, read, Event, KeyCode};

    use crate::{
        cmd::cmd::open_file_with_system_app,
        file::file::{
            get_files_for_dir, sort_file_paths_dirs_first_then_files, toggle_selected_file,
        },
        input_action::input_action::{
            handle_create_file, handle_delete_file, handle_rename_file, InputAction,
        },
        mpsc_utils::mpsc_utils::send_message_or_panic,
        utils::utils::{
            enter_directory, get_is_in_input_mode, navigate_to_parent_directory,
            refresh_list_state_index_of_directory,
        },
        widget::widget::{
            add_char_input, get_selected_item_from_list_state, handle_backspace,
            reset_current_message_and_input, Window,
        },
        AppState,
    };

    pub fn handle_key_event(app_state: &mut AppState) -> &str {
        let maybe_key_event =
            poll(Duration::from_millis(100)).expect("can use poll to check if key event");
        if maybe_key_event {
            let event = read().expect("if poll returned true we should be able to read key event");
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char(char) => return handle_char(char, app_state),
                    KeyCode::Backspace => handle_backspace(app_state),
                    KeyCode::Esc => handle_escape(app_state),
                    KeyCode::Enter => handle_enter(app_state),
                    _ => return "ok",
                }
            }
        }
        return "ok";
    }

    fn handle_escape(app_state: &mut AppState) {
        let is_in_input_mode = get_is_in_input_mode(app_state);
        if is_in_input_mode {
            reset_current_message_and_input(app_state);
        } else if app_state.show_cheatsheet {
            app_state.show_cheatsheet = !app_state.show_cheatsheet;
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
                    Ok(()) => {
                        send_message_or_panic(
                            &mut app_state.sender_for_ui_message,
                            "Successfully renamed file!".to_string(),
                        );
                    }
                    Err(val) => {
                        send_message_or_panic(
                            &mut app_state.sender_for_ui_message,
                            format!("Failed to rename file: {}", val),
                        );
                    }
                }
            }
        };
    }

    fn handle_char(char: char, app_state: &mut AppState) -> &str {
        if app_state.input_action != InputAction::None {
            add_char_input(char, app_state);
            return "ok";
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
            's' => handle_s_char(app_state),
            'H' => handle_uppercase_h_char(app_state),
            _ => {}
        }
        "ok"
    }

    fn handle_uppercase_h_char(app_state: &mut AppState) {
        send_message_or_panic(
            &mut app_state.sender_for_ui_message,
            String::from(format!(
                "Hidden files shown: {:?}",
                !app_state.show_hidden_files
            )),
        );

        app_state.show_hidden_files = !app_state.show_hidden_files;
        let new_files =
            get_files_for_dir(&app_state.working_directory, app_state.show_hidden_files)
                .expect("Can get files in same directory when toggling hidden files");
        app_state.files = sort_file_paths_dirs_first_then_files(&new_files);
    }

    fn handle_s_char(app_state: &mut AppState) {
        app_state.show_selected_files_window = !app_state.show_selected_files_window
    }

    fn handle_c_char(app_state: &mut AppState) {
        app_state.show_cheatsheet = !app_state.show_cheatsheet;
    }

    fn handle_r_char(app_state: &mut AppState) {
        let file = get_selected_item_from_list_state(&app_state.file_list_state, &app_state.files);

        send_message_or_panic(
            &mut app_state.sender_for_ui_message,
            "Please enter the new filename. Esc to abort".to_string(),
        );

        app_state.input_action = InputAction::RenameFile;
        app_state.user_input = file.full_path.clone();
    }

    fn handle_j_char(app_state: &mut AppState) {
        match app_state.current_window {
            Window::Files => {
                app_state.file_list_state.select_next();
                refresh_list_state_index_of_directory(app_state, Window::Files);
            }
            Window::SelectedFiles => {
                app_state.selected_files_list_state.select_next();
                refresh_list_state_index_of_directory(app_state, Window::SelectedFiles);
            }
        }
    }

    fn handle_k_char(app_state: &mut AppState) {
        match app_state.current_window {
            Window::Files => {
                app_state.file_list_state.select_previous();
                refresh_list_state_index_of_directory(app_state, Window::Files);
            }
            Window::SelectedFiles => {
                app_state.selected_files_list_state.select_previous();
                refresh_list_state_index_of_directory(app_state, Window::SelectedFiles);
            }
        }
    }

    fn handle_q_char(app_state: &mut AppState) -> &str {
        if app_state.input_action == InputAction::None {
            return "quit";
        }
        return "ok";
    }

    fn handle_space(app_state: &mut AppState) {
        match app_state.current_window {
            Window::Files => {
                let selected_file_index = app_state.file_list_state.selected();
                let selected_file = app_state
                    .files
                    .get(selected_file_index.expect("there should be a selected file"))
                    .expect("[files_window]: the selected file should exist");

                let new_selected_files =
                    toggle_selected_file(&app_state.selected_files, selected_file);
                app_state.selected_files = new_selected_files;
            }
            Window::SelectedFiles => {
                let maybe_index = app_state.selected_files_list_state.selected();
                if let Some(index) = maybe_index {
                    let selected_file = app_state
                        .selected_files
                        .get(index)
                        .expect("[selected_file_window]: the selected file should exist");
                    let new_selected_files =
                        toggle_selected_file(&app_state.selected_files, selected_file);
                    app_state.selected_files = new_selected_files;
                }
            }
        }
    }

    fn handle_one_char(app_state: &mut AppState) {
        if app_state.current_window != Window::Files && app_state.input_action == InputAction::None
        {
            app_state.current_window = Window::Files;
        }
    }

    fn handle_two_char(app_state: &mut AppState) {
        if app_state.current_window != Window::SelectedFiles
            && app_state.input_action == InputAction::None
        {
            app_state.current_window = Window::SelectedFiles;
            if app_state.selected_files_list_state.selected() == None
                && !app_state.selected_files.is_empty()
            {
                app_state.selected_files_list_state.select(Some(0));
            }
        }
    }

    fn handle_uppercase_d_char(app_state: &mut AppState) {
        match app_state.current_window {
            Window::Files => {
                let file =
                    get_selected_item_from_list_state(&app_state.file_list_state, &app_state.files);
                app_state.input_action = InputAction::DeleteFile;

                send_message_or_panic(
                    &mut app_state.sender_for_ui_message,
                    String::from(format!(
                        "Please confirm deletion of file {} with y/yes. Esc to abort",
                        file.full_path
                    )),
                );
            }
            Window::SelectedFiles => {
                app_state.input_action = InputAction::DeleteFile;
                send_message_or_panic(
                    &mut app_state.sender_for_ui_message,
                    "Please confirm deletion of all selected files with y/yes. Esc to abort"
                        .to_string(),
                )
            }
        }
    }

    fn handle_a_char(app_state: &mut AppState) {
        app_state.input_action = InputAction::CreateFile;
        send_message_or_panic(
            &mut app_state.sender_for_ui_message,
            "Enter the name for new filename: (Tip: use a trailing slash to create a directory)"
                .into(),
        );
    }

    fn handle_o_char(app_state: &mut AppState) {
        let selected_file =
            get_selected_item_from_list_state(&app_state.file_list_state, &app_state.files);
        let full_path_of_selected_file = &selected_file.full_path;
        let open_file_with_system_app_result =
            open_file_with_system_app(&full_path_of_selected_file);
        match open_file_with_system_app_result {
            Ok(_) => {}
            Err(error) => {
                send_message_or_panic(&mut app_state.sender_for_ui_message, error.to_string());
            }
        }
    }
}
