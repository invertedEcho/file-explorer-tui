use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal,
};

mod env;
use env::env::get_home_dir;

mod file;
use file::file::{
    delete_file, get_files_for_dir, get_parent_dir, is_path_directory,
    sort_file_paths_dirs_first_then_files, File,
};

// FIXME I reallllllyyyyy need to clean this file up the last time i had this much indentantion and
// LOC was when writing flutter code

struct ApplicationState {
    files: Vec<File>,
    selected_files: Vec<File>,
    current_directory: String,
    current_pane: Pane,
    current_message: String,
    // holds data from current user input
    input: String,
    // whether our Current message block is accepting user input or just showing static message.
    // TODO: should move to enum and better name
    is_currently_input: bool,
}

#[derive(PartialEq, Debug)]
enum Pane {
    Files,
    SelectedFiles,
}

const SELECTED_STYLE: Style = Style::new()
    .add_modifier(Modifier::BOLD)
    .fg(Color::LightGreen);

fn main() -> Result<()> {
    // installs error handling hook
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn add_char_input(new_char: char, app_state: &mut ApplicationState) {
    let input_length = app_state.input.len();
    app_state.input.insert(input_length, new_char);
}

fn pop_char_input(app_state: &mut ApplicationState) {
    app_state.input.pop();
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    // TODO: fall back to something sane
    let initial_directory = get_home_dir().expect("$HOME is set");

    let initial_files = get_files_for_dir(&initial_directory);
    let sorted_initial_files = sort_file_paths_dirs_first_then_files(&initial_files);

    // setup application state
    let mut application_state = ApplicationState {
        files: sorted_initial_files,
        selected_files: vec![],
        current_directory: initial_directory,
        current_pane: Pane::Files,
        current_message: String::from("Hi!"),
        input: String::from(""),
        is_currently_input: false,
    };

    let mut file_list_state = ListState::default();
    file_list_state.select(Some(0));

    let mut selected_files_state = ListState::default();

    // TODO: check whether its okay that the widgets are reconstructed each iteration
    loop {
        terminal.draw(|frame| {
            let files_block_border_style = if application_state.current_pane == Pane::Files {
                Style::new().light_green()
            } else {
                Style::new()
            };
            let files_block = Block::new()
                .title("Files [1]")
                .title_bottom(Line::raw("(D)elete single").right_aligned())
                .borders(Borders::all())
                .border_style(files_block_border_style);

            let current_dir_block = Block::new()
                .title("Current directory")
                .borders(Borders::all())
                .border_style(Style::new().light_green())
                .title_top(Line::from("[h or -] -> Parent Dir").right_aligned())
                .title_top(Line::from("[l or Enter] -> Go into Dir").right_aligned());

            let current_directory_paragraph =
                Paragraph::new(application_state.current_directory.clone())
                    .block(current_dir_block);

            let files_list_widget_with_block = List::new(application_state.files.clone())
                .block(files_block.clone())
                .highlight_style(SELECTED_STYLE)
                .highlight_symbol(">");

            let root_layout =
                Layout::vertical([Constraint::Percentage(90), Constraint::Percentage(10)]);
            let [upper_layout, lower_layout] = root_layout.areas(frame.area());

            let inner_upper_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(upper_layout);

            let current_message_or_user_input_widget_title = if application_state.is_currently_input
            {
                application_state.current_message.clone()
            } else {
                "Current message".to_string()
            };
            let text = if application_state.is_currently_input {
                application_state.input.clone()
            } else {
                application_state.current_message.clone()
            };

            let current_message_or_user_input_widget = Paragraph::new(text).block(
                Block::new()
                    .borders(Borders::all())
                    .title(current_message_or_user_input_widget_title),
            );
            frame.render_widget(current_message_or_user_input_widget, lower_layout);

            let inner_left_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(7), Constraint::Percentage(93)])
                .split(inner_upper_layout[0]);

            frame.render_stateful_widget(
                &files_list_widget_with_block,
                inner_left_layout[1],
                &mut file_list_state,
            );

            let selected_files_list_item: Vec<ListItem> = application_state
                .selected_files
                .iter()
                .map(|selected_file| ListItem::new(selected_file.full_path.clone()))
                .collect();

            let selected_files_block_style =
                if application_state.current_pane == Pane::SelectedFiles {
                    Style::new().light_green()
                } else {
                    Style::new()
                };
            let selected_files_block = Block::new()
                .title("Selected files [2]")
                .title_bottom(Line::raw("(D)elete all").right_aligned())
                .borders(Borders::all())
                .border_style(selected_files_block_style);

            let selected_files_list_widget = List::new(selected_files_list_item)
                .block(selected_files_block)
                .highlight_style(SELECTED_STYLE)
                .highlight_symbol(">");

            frame.render_stateful_widget(
                selected_files_list_widget,
                inner_upper_layout[1],
                &mut selected_files_state,
            );

            frame.render_widget(&current_directory_paragraph, inner_left_layout[0]);
            if application_state.is_currently_input {
                frame.set_cursor_position(Position::new(
                    lower_layout.x + application_state.input.len() as u16 + 1,
                    lower_layout.y + 1,
                ))
            }
        })?;

        if let Event::Key(key) = event::read()? {
            match application_state.is_currently_input {
                true => match key.code {
                    KeyCode::Backspace => {
                        pop_char_input(&mut application_state);
                    }
                    KeyCode::Char(to_insert) => {
                        add_char_input(to_insert, &mut application_state);
                    }
                    KeyCode::Esc => {
                        reset_current_message_thing(&mut application_state);
                    }
                    KeyCode::Enter => {
                        // TODO: i think what we should do is have a mode in our application_state or something that tells
                        // us what the user is currently typing in, like confirmation of deletion,
                        // or creating a new file.
                        let current_input = &application_state.input;
                        let is_confirmed = current_input == "y" || current_input == "yes";
                        if is_confirmed {
                            match application_state.current_pane {
                                Pane::Files => {
                                    let file = get_selected_item_from_list_state(
                                        &file_list_state,
                                        &application_state.files,
                                    );

                                    let delete_result = delete_file(file);
                                    match delete_result {
                                        Ok(_) => {
                                            application_state.current_message =
                                                String::from(format!(
                                                    "Successfully deleted {:?}.",
                                                    file.full_path
                                                ));
                                            application_state.files = application_state
                                                .files
                                                .iter()
                                                .filter(|f| file.full_path != f.full_path)
                                                .map(|file| file.clone())
                                                .collect();
                                        }
                                        Err(err) => {
                                            application_state.current_message =
                                                String::from(format!(
                                                    "Failed to delete file {:?}\nError: {:?}",
                                                    file.full_path, err
                                                ));
                                        }
                                    }
                                    application_state.is_currently_input = false;
                                    application_state.input = "".to_string();
                                }
                                Pane::SelectedFiles => {
                                    let files = application_state.selected_files.clone();
                                    let mut results: Vec<Result<&String, std::io::Error>> = vec![];
                                    for file in &files {
                                        let result = delete_file(file);
                                        results.push(result);
                                    }
                                    let mut failed_count = 0;
                                    for result in results {
                                        match result {
                                            Ok(file_path) => {
                                                application_state.selected_files =
                                                    application_state
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
                                        application_state.current_message =
                                            "Successfully deleted all files.".to_string();
                                    } else {
                                        application_state.current_message =
                                            format!("Failed to delete {} files.", failed_count);
                                    }
                                    application_state.is_currently_input = false;
                                }
                            }
                        } else {
                            reset_current_message_thing(&mut application_state);
                        }
                    }
                    _ => {}
                },
                false => match key.code {
                    KeyCode::Char('q') => break Ok(()),
                    KeyCode::Char('j') => match application_state.current_pane {
                        Pane::Files => file_list_state.select_next(),
                        Pane::SelectedFiles => selected_files_state.select_next(),
                    },
                    KeyCode::Char('k') => match application_state.current_pane {
                        Pane::Files => file_list_state.select_previous(),
                        Pane::SelectedFiles => selected_files_state.select_previous(),
                    },
                    KeyCode::Char(' ') => {
                        let selected_file_index = file_list_state.selected();
                        let selected_file = application_state
                            .files
                            .get(selected_file_index.expect("there should be a selected file"))
                            .expect("the selected file should exist");

                        let new_selected_files =
                            toggle_selected_file(&application_state.selected_files, selected_file);
                        application_state.selected_files = new_selected_files;
                    }
                    KeyCode::Char('h') | KeyCode::Char('-') => {
                        application_state.current_directory =
                            get_parent_dir(&application_state.current_directory);
                        application_state.files = sort_file_paths_dirs_first_then_files(
                            &get_files_for_dir(&application_state.current_directory),
                        );
                        match file_list_state.selected() {
                            None => {
                                file_list_state.select(Some(0));
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Enter => {
                        // TODO: duplicated twice. if thrice, create function
                        let selected_file_index = file_list_state.selected();
                        let selected_file = application_state
                            .files
                            .get(selected_file_index.expect("there should be a selected file"))
                            .expect("the selected file should exist");

                        if is_path_directory(&selected_file.full_path) {
                            application_state.current_directory =
                                selected_file.full_path.to_string();
                            application_state.files = sort_file_paths_dirs_first_then_files(
                                &get_files_for_dir(&application_state.current_directory),
                            );
                        }
                    }
                    KeyCode::Char('1') => {
                        if application_state.current_pane != Pane::Files {
                            application_state.current_pane = Pane::Files;
                        }
                    }
                    KeyCode::Char('2') => {
                        if application_state.current_pane != Pane::SelectedFiles {
                            application_state.current_pane = Pane::SelectedFiles;
                            if selected_files_state.selected() == None
                                && !application_state.selected_files.is_empty()
                            {
                                selected_files_state.select(Some(0));
                            }
                        }
                    }
                    KeyCode::Char('D') => match application_state.current_pane {
                        Pane::Files => {
                            let file = get_selected_item_from_list_state(
                                &file_list_state,
                                &application_state.files,
                            );
                            application_state.is_currently_input = true;
                            application_state.current_message = String::from(format!(
                                "Please confirm deletion of file {} with y/yes. Esc to abort",
                                file.full_path
                            ));
                        }
                        Pane::SelectedFiles => {
                            let files = &application_state.selected_files;
                            application_state.is_currently_input = true;
                            application_state.current_message = format!(
                                "Please confirm deletion of all {} selected files",
                                files.len()
                            )
                        }
                    },
                    _ => {}
                },
            }
        }
    }
}

/// TODO: Might just be a generic function.
/// This function checks if the newly selected file already exists in the existing selected files.
/// If yes, it will be removed. Otherwise it will be added.
fn toggle_selected_file(selected_files: &Vec<File>, selected_file: &File) -> Vec<File> {
    let file_exists = selected_files
        .iter()
        .any(|file| file.full_path == selected_file.full_path);
    if file_exists {
        let files_vec_without_selected_file: Vec<File> = selected_files
            .iter()
            .filter(|file| *file.full_path != selected_file.full_path)
            .map(|file| file.clone())
            .collect();
        files_vec_without_selected_file.to_vec()
    } else {
        let new_thing: Vec<File> = vec![selected_file.clone()];
        let new_selected_files = new_thing
            .iter()
            .chain(selected_files)
            .map(|file| file.clone())
            .collect();
        new_selected_files
    }
}

// isnt it really good if a function only uses borrowed data and also returns borrowed data?
// so no new memory is allocated?
fn get_selected_item_from_list_state<'a>(state: &ListState, list: &'a Vec<File>) -> &'a File {
    let selected_index = state.selected().expect("something should be selected");
    let selected_item = list
        .get(selected_index)
        .expect("given list actually contains item from given index");
    return selected_item;
}

fn reset_current_message_thing(application_state: &mut ApplicationState) {
    application_state.input = "".to_string();
    application_state.is_currently_input = false;
    application_state.current_message = "".to_string();
}
