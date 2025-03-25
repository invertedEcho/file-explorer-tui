pub mod keys {
    use crossterm::event::{self, Event, KeyCode};

    use crate::{
        file::file::{
            delete_file, get_files_for_dir, get_parent_dir, is_path_directory,
            sort_file_paths_dirs_first_then_files, toggle_selected_file,
        },
        widget::widget::{
            add_char_input, get_selected_item_from_list_state, pop_char_input,
            reset_current_message_and_input, InputAction, Pane,
        },
        AppState,
    };

    pub fn handle_key_event(app_state: &mut AppState) -> Result<&str, ()> {
        let event = event::read().expect("can read event");
        if let Event::Key(key) = event {
            match app_state.input_action != InputAction::None {
                true => match key.code {
                    KeyCode::Backspace => {
                        pop_char_input(app_state);
                    }
                    KeyCode::Char(to_insert) => {
                        add_char_input(to_insert, app_state);
                    }
                    KeyCode::Esc => {
                        reset_current_message_and_input(app_state);
                    }
                    KeyCode::Enter => {
                        let user_input = &app_state.user_input;
                        let is_confirmed = user_input == "y" || user_input == "yes";
                        if is_confirmed {
                            match app_state.pane {
                                Pane::Files => {
                                    let file = get_selected_item_from_list_state(
                                        &app_state.file_list_state,
                                        &app_state.files,
                                    );

                                    let delete_result = delete_file(file);
                                    match delete_result {
                                        Ok(_) => {
                                            app_state.message = String::from(format!(
                                                "Successfully deleted {:?}.",
                                                file.full_path
                                            ));
                                            app_state.files = app_state
                                                .files
                                                .iter()
                                                .filter(|f| file.full_path != f.full_path)
                                                .map(|file| file.clone())
                                                .collect();
                                        }
                                        Err(err) => {
                                            app_state.message = String::from(format!(
                                                "Failed to delete file {:?}\nError: {:?}",
                                                file.full_path, err
                                            ));
                                        }
                                    }
                                    app_state.input_action = InputAction::None;
                                    app_state.user_input = "".to_string();
                                }
                                Pane::SelectedFiles => {
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
                                    if failed_count == 0 {
                                        app_state.message =
                                            "Successfully deleted all files.".to_string();
                                    } else {
                                        let thing =
                                            if failed_count == 1 { "file" } else { "files" };
                                        app_state.message =
                                            format!("Failed to delete {} {}.", failed_count, thing);
                                    }
                                    app_state.input_action = InputAction::None;
                                }
                            }
                        } else {
                            reset_current_message_and_input(app_state);
                        }
                    }
                    _ => {}
                },
                false => match key.code {
                    KeyCode::Char('a') => {
                        app_state.input_action = InputAction::CreateFile;
                        app_state.message = "Type in the filename:".to_string();
                    }
                    KeyCode::Char('q') => return Ok("quit"),
                    KeyCode::Char('j') => match app_state.pane {
                        Pane::Files => app_state.file_list_state.select_next(),
                        Pane::SelectedFiles => app_state.selected_files_list_state.select_next(),
                    },
                    KeyCode::Char('k') => match app_state.pane {
                        Pane::Files => app_state.file_list_state.select_previous(),
                        Pane::SelectedFiles => {
                            app_state.selected_files_list_state.select_previous()
                        }
                    },
                    KeyCode::Char(' ') => {
                        let selected_file_index = app_state.file_list_state.selected();
                        let selected_file = app_state
                            .files
                            .get(selected_file_index.expect("there should be a selected file"))
                            .expect("the selected file should exist");

                        let new_selected_files =
                            toggle_selected_file(&app_state.selected_files, selected_file);
                        app_state.selected_files = new_selected_files;
                    }
                    KeyCode::Char('h') | KeyCode::Char('-') => {
                        app_state.working_directory = get_parent_dir(&app_state.working_directory);
                        app_state.files = sort_file_paths_dirs_first_then_files(
                            &get_files_for_dir(&app_state.working_directory),
                        );
                        match app_state.file_list_state.selected() {
                            None => {
                                app_state.file_list_state.select(Some(0));
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Enter => {
                        // TODO: duplicated twice. if thrice, create function
                        let selected_file_index = app_state.file_list_state.selected();
                        let selected_file = app_state
                            .files
                            .get(selected_file_index.expect("there should be a selected file"))
                            .expect("the selected file should exist");

                        if is_path_directory(&selected_file.full_path) {
                            app_state.working_directory = selected_file.full_path.to_string();
                            app_state.files = sort_file_paths_dirs_first_then_files(
                                &get_files_for_dir(&app_state.working_directory),
                            );
                        }
                    }
                    KeyCode::Char('1') => {
                        if app_state.pane != Pane::Files {
                            app_state.pane = Pane::Files;
                        }
                    }
                    KeyCode::Char('2') => {
                        if app_state.pane != Pane::SelectedFiles {
                            app_state.pane = Pane::SelectedFiles;
                            if app_state.selected_files_list_state.selected() == None
                                && !app_state.selected_files.is_empty()
                            {
                                app_state.selected_files_list_state.select(Some(0));
                            }
                        }
                    }
                    KeyCode::Char('D') => match app_state.pane {
                        Pane::Files => {
                            let file = get_selected_item_from_list_state(
                                &app_state.file_list_state,
                                &app_state.files,
                            );
                            app_state.input_action = InputAction::DeleteFile;
                            app_state.message = String::from(format!(
                                "Please confirm deletion of file {} with y/yes. Esc to abort",
                                file.full_path
                            ));
                        }
                        Pane::SelectedFiles => {
                            app_state.input_action = InputAction::DeleteFile;
                            app_state.message =
                                "Please confirm deletion of all selected files".to_string();
                        }
                    },
                    _ => {}
                },
            }
        }
        return Ok("ok");
    }
}
