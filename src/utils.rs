pub mod utils {
    use crate::{
        file::file::{
            delete_file, get_files_for_dir, get_parent_dir, is_path_directory,
            sort_file_paths_dirs_first_then_files,
        },
        input_action::input_action::InputAction,
        widget::widget::{get_selected_item_from_list_state, reset_current_message_and_input},
        AppState,
    };

    // TODO: this module just contains functions that dont fit into the other modules.
    // we should split up into more modules.

    pub fn enter_directory(app_state: &mut AppState) {
        let selected_file_index = app_state.file_list_state.selected();
        match selected_file_index {
            None => {}
            Some(index) => {
                let selected_file = app_state
                    .files
                    .get(index)
                    .expect("the selected file should exist");

                if is_path_directory(&selected_file.full_path) {
                    app_state.working_directory = selected_file.full_path.to_string();
                    refresh_files_for_working_directory(app_state);
                }
            }
        }
    }

    pub fn navigate_to_parent_directory(app_state: &mut AppState) {
        app_state.working_directory = get_parent_dir(&app_state.working_directory);
        refresh_files_for_working_directory(app_state);

        // make sure something is selected if nothing is selected
        match app_state.file_list_state.selected() {
            None => {
                app_state.file_list_state.select(Some(0));
            }
            _ => {}
        }
    }

    pub fn get_is_in_input_mode(app_state: &AppState) -> bool {
        app_state.input_action != InputAction::None
    }

    pub fn delete_currently_selected_file(app_state: &mut AppState) {
        let file = get_selected_item_from_list_state(&app_state.file_list_state, &app_state.files);

        let delete_result = delete_file(file);
        match delete_result {
            Ok(_) => {
                app_state.files = get_files_for_dir(&app_state.working_directory);
            }
            Err(err) => {
                app_state.message = String::from(format!(
                    "Failed to delete file {:?}\nError: {:?}",
                    file.full_path, err
                ));
            }
        }
        reset_current_message_and_input(app_state);
    }

    pub fn delete_selected_files(app_state: &mut AppState) {
        let files = app_state.selected_files.clone();
        let mut results: Vec<Result<&String, std::io::Error>> = vec![];
        for file in &files {
            let result = delete_file(file);
            results.push(result);
        }
        let mut failed_count = 0;
        for result in results {
            match result {
                Ok(file_path) => {
                    app_state.selected_files = app_state
                        .selected_files
                        .iter()
                        .filter(|file| &file.full_path != file_path)
                        .map(|file| file.clone())
                        .collect();
                }
                Err(_) => {
                    failed_count += 1;
                }
            }
        }
        if failed_count != 0 {
            let thing = if failed_count == 1 { "file" } else { "files" };
            app_state.message = format!("Failed to delete {} {}.", failed_count, thing);
        } else {
            reset_current_message_and_input(app_state);
        }
        app_state.input_action = InputAction::None;
        refresh_files_for_working_directory(app_state);
    }

    pub fn refresh_files_for_working_directory(app_state: &mut AppState) {
        let files = get_files_for_dir(&app_state.working_directory);
        let sorted_files = sort_file_paths_dirs_first_then_files(&files);
        app_state.files = sorted_files;
    }
}
