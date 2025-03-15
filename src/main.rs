use std::{
    env::{self, VarError},
    fs,
};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
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

    let files_block = Block::new().title("Files").borders(Borders::all());
    let files_list_widget_with_block = List::new(file_items)
        .block(files_block)
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol(">");
    let selected_files_block = Block::new().title("Selected Files").borders(Borders::all());

    let mut state = ListState::default();
    state.select_next();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let root_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(area);

            frame.render_stateful_widget(&files_list_widget_with_block, root_layout[0], &mut state);
            frame.render_widget(&selected_files_block, root_layout[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break Ok(()),
                KeyCode::Char('j') => {
                    state.select_next();
                }
                KeyCode::Char('k') => {
                    state.select_previous();
                }
                _ => {}
            }
        }
    }
}

fn get_home_dir() -> Result<String, VarError> {
    let home_env_var_result = env::var("HOME");
    home_env_var_result
}

// TODO: split up into two functions
// or even better use From trait to say how our DirEntry is converted to a ListItem
fn get_files_as_list_item_vec_from_dir(dir: &String) -> Vec<ListItem<'_>> {
    let files = fs::read_dir(dir).expect("Can read from dir");

    let file_items: Vec<ListItem> = files
        .into_iter()
        .map(|file| {
            // i have a feeling this is not the way to go
            let filename = file
                .expect("can unwrap file")
                .path()
                .to_str()
                .expect("can get path as str")
                .to_string();

            let span_widget = Span::raw(filename);
            return ListItem::new(span_widget);
        })
        .collect();
    return file_items;
}
