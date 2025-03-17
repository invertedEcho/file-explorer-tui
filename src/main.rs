use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal,
};

mod env;
use env::env::get_home_dir;

mod file;
use file::file::{
    get_files_for_dir, get_parent_dir, is_path_directory, sort_file_paths_dirs_first_then_files,
    File,
};

struct ApplicationState {
    files: Vec<File>,
    selected_files: Vec<File>,
    current_directory: String,
    current_pane: Pane,
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

// BUG: when navigating inside an empty directory and going back for example, nothing is selected
// and trying to use get_selected() on our list state will panic because we expect
// what about wrapper function that always ensures there is something selected?

// BUG: using the tui with different fonts yield different layout sizes etc...

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
    };

    let mut file_list_state = ListState::default();
    file_list_state.select(Some(0));

    let mut selected_files_state = ListState::default();

    // TODO: check whether its okay that the widgets are reconstructed each iteration
    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let files_block_border_style = if application_state.current_pane == Pane::Files {
                Style::new().light_green()
            } else {
                Style::new()
            };
            let files_block = Block::new()
                .title("Files [1]")
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

            let root_outer_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(area);

            let inner_left_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(7), Constraint::Percentage(93)])
                .split(root_outer_layout[0]);

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
                .borders(Borders::all())
                .border_style(selected_files_block_style);

            let selected_files_list_widget = List::new(selected_files_list_item)
                .block(selected_files_block)
                .highlight_style(SELECTED_STYLE)
                .highlight_symbol(">");

            frame.render_stateful_widget(
                selected_files_list_widget,
                root_outer_layout[1],
                &mut selected_files_state,
            );

            frame.render_widget(&current_directory_paragraph, inner_left_layout[0]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
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
        let new_thing: Vec<File> = vec![selected_file.clone()];
        let new_selected_files = new_thing
            .iter()
            .chain(selected_files)
            .map(|file| file.clone())
            .collect();
        new_selected_files
    }
}
