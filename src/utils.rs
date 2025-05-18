pub mod utils {
    use crate::{
        file::file::{
            delete_file, get_files_for_dir, get_parent_dir, is_path_directory,
            sort_file_paths_dirs_first_then_files,
        },
        input_action::input_action::InputAction,
        mpsc_utils::mpsc_utils::send_message_or_panic,
        widget::widget::{
            get_selected_item_from_list_state, reset_current_message_and_input, Pane,
        },
        AppState,
    };

    // TODO: this module just contains functions that dont fit into the other modules.
    // we should split up into more modules.

    pub fn enter_directory(app_state: &mut AppState) {
        let selected_file_index = app_state.file_list_state.selected();
        match selected_file_index {
            None => {}
            Some(index) => {
                app_state
                    .list_state_index_of_directory
                    .insert(app_state.working_directory.clone(), index);
                let selected_file = app_state
                    .files
                    .get(index)
                    .expect("the selected file should exist");
                let selected_file_full_path = &selected_file.full_path;

                let index_of_dir_being_entered = app_state
                    .list_state_index_of_directory
                    .get(selected_file_full_path)
                    .or(Some(&0));

                if is_path_directory(&selected_file.full_path) {
                    let maybe_files =
                        get_files_for_dir(&selected_file.full_path, app_state.show_hidden_files);
                    match maybe_files {
                        Ok(files) => {
                            app_state
                                .file_list_state
                                .select(index_of_dir_being_entered.copied());
                            app_state.working_directory = selected_file.full_path.to_string();
                            app_state.files = files;
                        }
                        Err(error) => {
                            send_message_or_panic(
                                &mut app_state.sender_for_ui_message,
                                format!("Failed to enter directory: {:?}", error),
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn navigate_to_parent_directory(app_state: &mut AppState) {
        app_state.working_directory = get_parent_dir(&app_state.working_directory);
        refresh_files_for_working_directory(app_state);

        let index = app_state
            .list_state_index_of_directory
            .get(&app_state.working_directory)
            .or(Some(&0));

        app_state.file_list_state.select(index.copied());
    }

    pub fn get_is_in_input_mode(app_state: &AppState) -> bool {
        app_state.input_action != InputAction::None
    }

    pub fn delete_currently_selected_file(app_state: &mut AppState) {
        let file = get_selected_item_from_list_state(&app_state.file_list_state, &app_state.files);

        let delete_result = delete_file(file);
        match delete_result {
            Ok(_) => {
                let new_files =
                    get_files_for_dir(&app_state.working_directory, app_state.show_hidden_files)
                        .expect("can get files in same directory when deleting a file");
                app_state.files = new_files;
            }
            Err(err) => {
                send_message_or_panic(
                    &mut app_state.sender_for_ui_message,
                    String::from(format!(
                        "Failed to delete file {:?}\nError: {:?}",
                        file.full_path, err
                    )),
                );
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
            send_message_or_panic(
                &mut app_state.sender_for_ui_message,
                format!("Failed to delete {} {}.", failed_count, thing),
            );
        } else {
            reset_current_message_and_input(app_state);
        }
        app_state.input_action = InputAction::None;
        refresh_files_for_working_directory(app_state);
    }

    /// Only use this function if you are sure the new working directory can be read by the current
    /// user, otherwise this function may panic
    pub fn refresh_files_for_working_directory(app_state: &mut AppState) {
        let files = get_files_for_dir(&app_state.working_directory, app_state.show_hidden_files)
            .expect("can refresh files in new working directory");
        let sorted_files = sort_file_paths_dirs_first_then_files(&files);
        app_state.files = sorted_files;
    }

    pub fn refresh_list_state_index_of_directory(app_state: &mut AppState, current_pane: Pane) {
        let new_index;
        match current_pane {
            Pane::Files => {
                new_index = app_state.file_list_state.selected().or(Some(0));
            }
            Pane::SelectedFiles => {
                new_index = app_state.selected_files_list_state.selected().or(Some(0));
            }
        }
        let current_dir = &app_state.working_directory;

        app_state
            .list_state_index_of_directory
            .insert(current_dir.to_string(), new_index.unwrap());
    }
}
