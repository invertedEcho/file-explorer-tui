pub mod input_action {
    #[derive(PartialEq, Debug)]
    pub enum InputAction {
        None,
        DeleteFile,
        CreateFile,
        RenameFile,
    }

    use std::{fs::rename, io::Error};

    use crate::{
        file::file::create_file,
        utils::utils::{
            delete_currently_selected_file, delete_selected_files,
            refresh_files_for_working_directory,
        },
        widget::widget::{
            get_selected_item_from_list_state, reset_current_message_and_input, reset_input, Pane,
        },
        AppState,
    };

    pub fn handle_create_file(app_state: &mut AppState) {
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
    }

    pub fn handle_delete_file(app_state: &mut AppState) {
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
    }

    pub fn handle_rename_file(app_state: &mut AppState) -> Result<(), Error> {
        let file = get_selected_item_from_list_state(&app_state.file_list_state, &app_state.files);
        let new_file_name = &app_state.user_input;
        let result = rename(&file.full_path, new_file_name);
        refresh_files_for_working_directory(app_state);
        reset_current_message_and_input(app_state);
        result
    }
}
