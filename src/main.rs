use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    // installs error handling hook
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        let files_block = Block::new().title("Files").borders(Borders::all());
        let selected_files_block = Block::new().title("Selected Files").borders(Borders::all());

        terminal.draw(|frame| {
            let area = frame.area();

            let root_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(area);

            frame.render_widget(files_block, root_layout[0]);
            frame.render_widget(selected_files_block, root_layout[1]);
        })?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break Ok(()),
                _ => {}
            }
        }
    }
}
