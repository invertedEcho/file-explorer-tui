pub mod widget {
    use crate::{file::file::File, AppState};

    use ratatui::{
        layout::{Constraint, Direction, Layout, Position},
        style::{Color, Modifier, Style, Stylize},
        widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
        Frame,
    };

    #[derive(PartialEq, Debug)]
    pub enum InputAction {
        None,
        DeleteFile,
    }

    #[derive(PartialEq, Debug)]
    pub enum Pane {
        Files,
        SelectedFiles,
    }

    const SELECTED_STYLE: Style = Style::new()
        .add_modifier(Modifier::BOLD)
        .fg(Color::LightGreen);

    // Draws all widgets to the passed frame
    pub fn draw_widgets_to_frame(frame: &mut Frame, app_state: &mut AppState) {
        let files_block_border_style = if app_state.pane == Pane::Files {
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
            .border_style(Style::new().light_green());

        let current_directory_paragraph =
            Paragraph::new(app_state.working_directory.clone()).block(current_dir_block);

        let files_list_widget_with_block = List::new(app_state.files.clone())
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

        let current_message_or_user_input_widget_title =
            if app_state.input_action == InputAction::None {
                "Current message".to_string()
            } else {
                app_state.message.clone()
            };

        let text = if app_state.input_action != InputAction::None {
            app_state.user_input.clone()
        } else {
            app_state.message.clone()
        };

        let current_message_or_user_input_widget = Paragraph::new(text).block(
            Block::new()
                .borders(Borders::all())
                .title(current_message_or_user_input_widget_title),
        );
        frame.render_widget(current_message_or_user_input_widget, lower_layout);

        let inner_left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(3), Constraint::Percentage(93)])
            .split(inner_upper_layout[0]);

        frame.render_stateful_widget(
            &files_list_widget_with_block,
            inner_left_layout[1],
            &mut app_state.file_list_state,
        );

        let selected_files_list_item: Vec<ListItem> = app_state
            .selected_files
            .iter()
            .map(|selected_file| ListItem::new(selected_file.full_path.clone()))
            .collect();

        let selected_files_block_style = if app_state.pane == Pane::SelectedFiles {
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
            inner_upper_layout[1],
            &mut app_state.selected_files_list_state,
        );

        frame.render_widget(&current_directory_paragraph, inner_left_layout[0]);
        if app_state.input_action != InputAction::None {
            frame.set_cursor_position(Position::new(
                lower_layout.x + app_state.user_input.len() as u16 + 1,
                lower_layout.y + 1,
            ))
        }
    }

    pub fn reset_current_message_and_input(app_state: &mut AppState) {
        app_state.user_input = "".to_string();
        app_state.input_action = InputAction::None;
        app_state.message = "".to_string();
    }

    pub fn add_char_input(new_char: char, app_state: &mut AppState) {
        let input_length = app_state.user_input.len();
        app_state.user_input.insert(input_length, new_char);
    }

    pub fn handle_backspace(app_state: &mut AppState) {
        if app_state.input_action == InputAction::None {
            return;
        }
        app_state.user_input.pop();
    }

    pub fn get_selected_item_from_list_state<'a>(
        state: &ListState,
        list: &'a Vec<File>,
    ) -> &'a File {
        let selected_index = state.selected().expect("something should be selected");
        let selected_item = list
            .get(selected_index)
            .expect("given list actually contains item from given index");
        return selected_item;
    }
}
