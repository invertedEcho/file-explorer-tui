use std::{
    env::{self, VarError},
    fs,
    path::Path,
};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal,
};

// TODO: use me
#[derive(Clone)]
struct File {
    display_name: String,
    full_path: String,
}

impl ToString for File {
    fn to_string(&self) -> String {
        self.display_name.to_string()
    }
}

impl From<File> for String {
    fn from(value: File) -> String {
        value.display_name
    }
}

impl From<File> for Text<'_> {
    fn from(value: File) -> Self {
        Text::raw(value.display_name)
    }
}

struct ApplicationState {
    files: Vec<File>,
    selected_files: Vec<File>,
    current_directory: String,
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

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let initial_directory = get_home_dir()?;

    // setup application state
    let mut application_state = ApplicationState {
        files: get_files_for_dir(&initial_directory),
        selected_files: vec![],
        current_directory: initial_directory,
    };

    let files_block = Block::new()
        .title("Files")
        .borders(Borders::all())
        .border_style(Style::new().light_green());

    let mut file_list_state = ListState::default();
    file_list_state.select(Some(0));

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            // These two things are not ideal as they will be computed every frame draw ):
            let current_dir_block = Block::new()
                .title("Current directory")
                .borders(Borders::all())
                .border_style(Style::new().light_green())
                .title_top(Line::from("h Parent Dir").right_aligned())
                .title_top(Line::from("l Go into Dir").right_aligned());
            let current_directory_paragraph =
                Paragraph::new(application_state.current_directory.clone())
                    .block(current_dir_block);

            let files_list_widget_with_block = List::new(application_state.files.clone())
                .block(files_block.clone())
                .highlight_style(SELECTED_STYLE)
                .highlight_symbol(">");

            let root_outer_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(area);

            let inner_left_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(5), Constraint::Percentage(95)])
                .split(root_outer_layout[0]);

            frame.render_stateful_widget(
                &files_list_widget_with_block,
                inner_left_layout[1],
                &mut file_list_state,
            );

            let selected_files_list_item: Vec<ListItem> = application_state
                .selected_files
                .iter()
                .map(|selected_file| ListItem::new(selected_file.to_string()))
                .collect();

            let selected_files_block = Block::new().title("Selected Files").borders(Borders::all());
            let selected_files_list_widget =
                List::new(selected_files_list_item).block(selected_files_block);
            frame.render_widget(selected_files_list_widget, root_outer_layout[1]);

            frame.render_widget(&current_directory_paragraph, inner_left_layout[0]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break Ok(()),
                KeyCode::Char('j') => {
                    file_list_state.select_next();
                }
                KeyCode::Char('k') => {
                    file_list_state.select_previous();
                }
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
                KeyCode::Char('h') => {
                    application_state.current_directory =
                        get_parent_dir(&application_state.current_directory);
                    application_state.files =
                        get_files_for_dir(&application_state.current_directory);
                }
                KeyCode::Char('l') | KeyCode::Enter => {
                    // TODO: duplicated twice. if thrice, create function
                    let selected_file_index = file_list_state.selected();
                    let selected_file = application_state
                        .files
                        .get(selected_file_index.expect("there should be a selected file"))
                        .expect("the selected file should exist");

                    if is_path_directory(&selected_file.full_path) {
                        application_state.current_directory = selected_file.full_path.to_string();
                        application_state.files =
                            get_files_for_dir(&application_state.current_directory);
                    }
                }
                _ => {}
            }
        }
    }
}

/// TODO: Might just be a generic function.
/// This function checks if the newly selected file already exists in the existing selected files.
/// If yes, it will be removed. Otherwise it will be added.
/// Creates a new vec with new Strings
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
        vec![]
        // let new_selected_files = vec![selected_file]
        //     .iter()
        //     .chain(selected_files)
        //     .map(|x| x.clone())
        //     .collect();
        // new_selected_files
    }
}

fn get_home_dir() -> Result<String, VarError> {
    let home_env_var_result = env::var("HOME");
    home_env_var_result
}

fn get_files_for_dir(dir: &String) -> Vec<File> {
    let files = fs::read_dir(dir).expect("Can read from dir");

    let file_items: Vec<File> = files
        .into_iter()
        .map(|file| {
            // i have a feeling this is not the way to go
            let dir_entry = file.expect("can unwrap file");
            let full_path = dir_entry.path().to_string_lossy().to_string();
            let splitted: Vec<&str> = full_path.split("/").collect();
            let (last, _) = splitted
                .split_last()
                .expect("Should be able to split to get relative path");

            return File {
                display_name: last.to_string(),
                full_path,
            };
        })
        .collect();
    return file_items;
}

// TODO: Write unit tests for this function
fn get_parent_dir(current_path: &String) -> String {
    let splitted_path: Vec<&str> = current_path.split("/").collect();
    let split_last_result = splitted_path.split_last();
    return match split_last_result {
        None => current_path.to_string(),
        Some(result) => {
            let (_, elements) = result;
            if elements.len() == 1 && elements[0] == "" {
                return String::from("/");
            }
            return elements.join("/");
        }
    };
}

fn is_path_directory(path: &String) -> bool {
    Path::new(path).is_dir()
}
