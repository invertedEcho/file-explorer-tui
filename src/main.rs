use std::{
    env::{self, VarError},
    fs,
};

use color_eyre::{eyre::Ok, Result};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, ListState},
    DefaultTerminal,
};

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
    let home_dir = get_home_dir()?;
    let mut current_dir = home_dir;

    let file_items = get_files_as_list_item_vec_from_dir(&current_dir);

    let files_block = Block::new()
        .title("Files")
        .borders(Borders::all())
        .border_style(Style::new().light_green());
    let files_list_widget_with_block = List::new(file_items.clone())
        .block(files_block)
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol(">");

    let mut file_list_state = ListState::default();
    file_list_state.select(Some(0));

    // TODO: fix variable name
    let mut active_filters_strings: Vec<String> = vec![];

    let mut selected_files_strings: Vec<String> = vec![];

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let root_outer_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(area);

            let inner_left_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
                .split(root_outer_layout[0]);

            frame.render_stateful_widget(
                &files_list_widget_with_block,
                inner_left_layout[1],
                &mut file_list_state,
            );

            let selected_files: Vec<ListItem> = selected_files_strings
                .iter()
                .map(|selected_file| ListItem::new(selected_file.to_string()))
                .collect();

            let selected_files_block = Block::new().title("Selected Files").borders(Borders::all());
            let selected_files_list_widget = List::new(selected_files).block(selected_files_block);
            frame.render_widget(selected_files_list_widget, root_outer_layout[1]);

            // frame.render_widget(&selected_files_block, root_outer_layout[1]);

            let active_filters: Vec<ListItem> = active_filters_strings
                .iter()
                .map(|active_filter| ListItem::new(active_filter.to_string()))
                .collect();

            // why is there no way to make a horizontal list
            let active_filters_widget =
                List::new(active_filters).block(Block::bordered().title("Active Filters"));
            frame.render_widget(&active_filters_widget, inner_left_layout[0]);
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
                KeyCode::Char('D') => {
                    active_filters_strings
                        .insert(active_filters_strings.len(), String::from("Directories"));
                }
                KeyCode::Char(' ') => {
                    let selected_file_index = file_list_state.selected();
                    let selected_file = file_items
                        .get(selected_file_index.expect("there should be a selected file"))
                        .expect("the selected file should exist");

                    let new_selected_files =
                        toggle_selected_file(&selected_files_strings, selected_file);
                    selected_files_strings = new_selected_files;
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
fn toggle_selected_file(selected_files: &Vec<String>, selected_file: &String) -> Vec<String> {
    let file_exists = selected_files.iter().any(|file| file == selected_file);
    if file_exists {
        let files_vec_without_selected_file: Vec<String> = selected_files
            .iter()
            .filter(|file| *file != selected_file)
            .map(|file| file.to_string())
            .collect();
        files_vec_without_selected_file.to_vec()
    } else {
        let new_selected_files = vec![selected_file.to_string()]
            .iter()
            .chain(selected_files)
            .map(|x| x.clone())
            .collect();
        new_selected_files
    }
}

fn get_home_dir() -> Result<String, VarError> {
    let home_env_var_result = env::var("HOME");
    home_env_var_result
}

// TODO: split up into two functions
// or even better use From trait to say how our DirEntry is converted to a ListItem
fn get_files_as_list_item_vec_from_dir(dir: &String) -> Vec<String> {
    let files = fs::read_dir(dir).expect("Can read from dir");

    let file_items: Vec<String> = files
        .into_iter()
        .map(|file| {
            // i have a feeling this is not the way to go
            let filename = file
                .expect("can unwrap file")
                .path()
                .to_str()
                .expect("can get path as str")
                .to_string();
            return filename;
        })
        .collect();
    return file_items;
}
