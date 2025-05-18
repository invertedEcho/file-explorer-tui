pub mod widget {
    use crate::{
        file::file::File, input_action::input_action::InputAction, keys::keys::KEYS,
        mpsc_utils::mpsc_utils::send_message_or_panic, AppState,
    };

    use ratatui::{
        layout::{Constraint, Direction, Flex, Layout, Position, Rect},
        style::{Color, Modifier, Style, Stylize},
        text::Line,
        widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
        Frame,
    };

    #[derive(PartialEq, Debug)]
    pub enum Window {
        Files,
        SelectedFiles,
    }

    const SELECTED_STYLE: Style = Style::new()
        .add_modifier(Modifier::BOLD)
        .fg(Color::LightGreen);

    // Draws all needed widgets to the passed frame
    pub fn draw_widgets_to_frame(
        frame: &mut Frame,
        app_state: &mut AppState,
        current_message: &String,
    ) {
        let files_block_border_style = if app_state.current_window == Window::Files {
            Style::new().light_green()
        } else {
            Style::new()
        };
        let files_block = Block::new()
            .title("Files")
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

        let constraints_for_inner_upper_layout = if app_state.show_selected_files_window {
            vec![Constraint::Percentage(70), Constraint::Percentage(30)]
        } else {
            vec![Constraint::Percentage(100)]
        };

        let inner_upper_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints_for_inner_upper_layout)
            .split(upper_layout);

        let current_message_or_user_input_widget_title =
            if app_state.input_action == InputAction::None {
                "Current message".to_string()
            } else {
                current_message.clone()
            };

        let text = if app_state.input_action != InputAction::None {
            app_state.user_input.clone()
        } else {
            current_message.clone()
        };

        let current_message_or_user_input_widget = Paragraph::new(text).block(
            Block::new()
                .borders(Borders::all())
                .title(current_message_or_user_input_widget_title)
                .title_bottom(Line::from("Press c to show the cheatsheet").right_aligned()),
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

        if app_state.show_selected_files_window {
            let selected_files_list_item: Vec<ListItem> = app_state
                .selected_files
                .iter()
                .map(|selected_file| ListItem::new(selected_file.full_path.clone()))
                .collect();

            let selected_files_block_style = if app_state.current_window == Window::SelectedFiles {
                Style::new().light_green()
            } else {
                Style::new()
            };
            let selected_files_block = Block::new()
                .title("Selected files")
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
        }

        frame.render_widget(&current_directory_paragraph, inner_left_layout[0]);
        if app_state.input_action != InputAction::None {
            frame.set_cursor_position(Position::new(
                lower_layout.x + app_state.user_input.len() as u16 + 1,
                lower_layout.y + 1,
            ))
        }

        if app_state.show_cheatsheet {
            let items: Vec<ListItem> = KEYS.iter().map(|key| ListItem::new(*key)).collect();

            let block = Block::bordered().title("Cheatsheet");
            let area = frame.area();
            let area = popup_area(area, 50, 60);

            let list = List::new(items).block(block);
            frame.render_widget(Clear, area);
            frame.render_widget(list, area);
        }
    }

    pub fn reset_input(app_state: &mut AppState) {
        app_state.user_input = "".to_string();
        app_state.input_action = InputAction::None;
    }

    pub fn reset_current_message_and_input(app_state: &mut AppState) {
        app_state.user_input = "".to_string();
        app_state.input_action = InputAction::None;
        send_message_or_panic(&mut app_state.sender_for_ui_message, "".to_string());
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

    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}
