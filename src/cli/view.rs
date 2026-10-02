use anyhow::{Context, Result};
use clap::Args;
use lipsum::LOREM_IPSUM;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tui_term::{vt100, widget::PseudoTerminal};

const TREE: &str = "
.
├── assets
├── Cargo.lock
├── Cargo.toml
├── mise.toml
└── src
    ├── assets.rs
    ├── cli
    │   ├── attach.rs
    │   ├── create.rs
    │   ├── delete.rs
    │   ├── herdr.rs
    │   ├── init.rs
    │   ├── list.rs
    │   ├── open.rs
    │   └── view.rs
    ├── cli.rs
    ├── config.rs
    ├── herdr.rs
    ├── main.rs
    ├── sandbox.rs
    └── session.rs
";

#[derive(Args, Clone, Debug, PartialEq)]
pub struct ViewCommand {}

impl ViewCommand {
    pub fn run(self) -> Result<()> {
        ratatui::run(app).context("error running app")?;
        Ok(())
    }
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(30),
            Constraint::Length(80),
            Constraint::Fill(1),
            Constraint::Length(30),
        ])
        .split(frame.area());

    let block_tasks = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Tasks ");
    let block_agent = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Agent ");
    let block_editor = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Editor ");
    let block_files = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Files ");

    frame.render_widget(&block_tasks, layout[0]);
    frame.render_widget(&block_agent, layout[1]);
    frame.render_widget(&block_editor, layout[2]);
    frame.render_widget(&block_files, layout[3]);

    let block_tasks_inner = block_tasks.inner(layout[0]);

    let block_tasks_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(block_tasks_inner);

    frame.render_widget(Paragraph::new("* foo"), block_tasks_rows[0]);
    frame.render_widget(Paragraph::new("+ bar"), block_tasks_rows[1]);
    frame.render_widget(Paragraph::new("= baz"), block_tasks_rows[2]);

    let block_agent_inner = block_agent.inner(layout[1]);

    frame.render_widget(Paragraph::new(LOREM_IPSUM), block_agent_inner);

    let block_editor_inner = block_editor.inner(layout[2]);

    // TODO: Wire up parser to terminal process
    let parser = vt100::Parser::new(24, 80, 0);
    frame.render_widget(PseudoTerminal::new(parser.screen()), block_editor_inner);

    let block_files_inner = block_files.inner(layout[3]);

    frame.render_widget(Paragraph::new(TREE.trim()), block_files_inner);
}
