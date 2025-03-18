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
    delete_file, get_files_for_dir, get_parent_dir, is_path_directory,
    sort_file_paths_dirs_first_then_files, File,
};

struct ApplicationState {
    files: Vec<File>,
    selected_files: Vec<File>,
    current_directory: String,
    current_pane: Pane,
    current_message: String,
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
        current_message: String::from("Hi!"),
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

            let root_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
                .split(area);

            let root_upper_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(root_layout[0]);

            let log_text_box = Paragraph::new(application_state.current_message.clone()).block(
                Block::new()
                    .borders(Borders::all())
                    .title("Current message"),
            );
            frame.render_widget(log_text_box, root_layout[1]);

            let inner_left_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(7), Constraint::Percentage(93)])
                .split(root_upper_layout[0]);

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
                root_upper_layout[1],
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
                KeyCode::Char('D') => match application_state.current_pane {
                    Pane::Files => {
                        let file = get_selected_item_from_list_state(
                            &file_list_state,
                            &application_state.files,
                        );
                        let delete_result = delete_file(file);
                        match delete_result {
                            Ok(_) => {
                                application_state.current_message = String::from(format!(
                                    "Successfully deleted {:?}",
                                    file.full_path
                                ))
                            }
                            Err(err) => {
                                application_state.current_message =
                                    String::from(format!("Failed to delete file: {:?}", err))
                            }
                        }
                    }
                    Pane::SelectedFiles => {}
                },
                _ => {}
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
