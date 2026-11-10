use std::{io::{Read, Write}, sync::{Arc, RwLock}, time::Duration};

use anyhow::{Context, Result};
use clap::Args;
use crossterm::{ExecutableCommand, event::{DisableMouseCapture, EnableMouseCapture, MouseButton, MouseEvent, MouseEventKind}};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::layout::Rect;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tui_term::{vt100, widget::PseudoTerminal};

use crate::herdr::Herdr;

#[derive(Args, Clone, Debug, PartialEq)]
pub struct ViewCommand {}

impl ViewCommand {
    pub fn run(self) -> Result<()> {
        ratatui::run(app).context("error running app")?;
        Ok(())
    }
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    run_app(terminal)
}

fn run_app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let workspaces = Arc::new(RwLock::new(Herdr::new()
        .list_workspaces()
        .map_err(std::io::Error::other)?));
    let workspace_refresh = workspaces.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        if let Ok(updated) = Herdr::new().list_workspaces()
            && let Ok(mut current) = workspace_refresh.write()
        {
            *current = updated;
        }
    });
    
    let mut stdout = std::io::stdout();
    let size = terminal.size()?;
    let pty = NativePtySystem::default().openpty(PtySize {
        rows: size.height.saturating_sub(2), cols: size.width.saturating_sub(32),
        pixel_width: 0, pixel_height: 0,
    }).map_err(std::io::Error::other)?;
    let mut command = CommandBuilder::new_default_prog();
    command.cwd(std::env::current_dir()?);
    let mut child = pty.slave.spawn_command(command).map_err(std::io::Error::other)?;
    drop(pty.slave);
    let mut reader = pty.master.try_clone_reader().map_err(std::io::Error::other)?;
    let mut writer = pty.master.take_writer().map_err(std::io::Error::other)?;
    let parser = Arc::new(RwLock::new(vt100::Parser::new(size.height.saturating_sub(2), size.width.saturating_sub(32), 0)));
    let output = parser.clone();
    let (mode_tx, mode_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buffer = [0; 8192];
        let mut last_mode = vt100::MouseProtocolMode::None;
        while let Ok(count) = reader.read(&mut buffer) {
            if count == 0 { break; }
            if let Ok(mut parser) = output.write() {
                parser.process(&buffer[..count]);
                let mode = parser.screen().mouse_protocol_mode();
                if mode != last_mode { let _ = mode_tx.send(mode); last_mode = mode; }
            }
        }
    });
    let mut mouse_mode = vt100::MouseProtocolMode::None;
    loop {
        while let Ok(mode) = mode_rx.try_recv() {
            mouse_mode = mode;
            if mode == vt100::MouseProtocolMode::None {
                stdout.execute(DisableMouseCapture)?;
            } else {
                stdout.execute(EnableMouseCapture)?;
            }
        }
        terminal.draw(|frame| render(frame, &parser, &workspaces))?;
        if crossterm::event::poll(Duration::from_millis(30))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) if key.is_press() => {
                    use crossterm::event::KeyCode;
                    if key.code == KeyCode::Esc { break; }
                    let bytes: &[u8] = match key.code {
                        KeyCode::Enter => b"\r", KeyCode::Backspace => b"\x7f", KeyCode::Tab => b"\t",
                        KeyCode::Up => b"\x1b[A", KeyCode::Down => b"\x1b[B", KeyCode::Right => b"\x1b[C", KeyCode::Left => b"\x1b[D",
                        KeyCode::Char(c) => { let mut s = [0; 4]; writer.write_all(c.encode_utf8(&mut s).as_bytes())?; continue; },
                        _ => continue,
                    };
                    writer.write_all(bytes)?;
                }
                crossterm::event::Event::Resize(cols, rows) => {
                    let area = Rect::new(0, 0, cols, rows);
                    let inner = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Length(30), Constraint::Fill(1)]).split(area)[1];
                    let inner = Rect::new(inner.x+1, inner.y+1, inner.width.saturating_sub(2), inner.height.saturating_sub(2));
                    pty.master.resize(PtySize { rows: inner.height, cols: inner.width, pixel_width: 0, pixel_height: 0 }).map_err(std::io::Error::other)?;
                    if let Ok(mut p) = parser.write() { p.screen_mut().set_size(inner.height, inner.width); }
                }
                crossterm::event::Event::Mouse(mouse) => {
                    if mouse_mode == vt100::MouseProtocolMode::None { continue; }
                    let size = terminal.size()?;
                    let area = Rect::new(0, 0, size.width, size.height);
                    let pane = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Length(30), Constraint::Fill(1)]).split(area)[1];
                    let x = mouse.column.saturating_sub(pane.x.saturating_add(1));
                    let y = mouse.row.saturating_sub(pane.y.saturating_add(1));
                    if mouse.column > pane.x && mouse.column < pane.x + pane.width.saturating_sub(1)
                        && mouse.row > pane.y && mouse.row < pane.y + pane.height.saturating_sub(1) {
                        write_mouse(&mut writer, mouse, x, y, mouse_mode)?;
                    }
                }
                _ => {}
            }
        }
    }
    stdout.execute(DisableMouseCapture)?;
    let _ = child.kill();
    Ok(())
}

fn write_mouse(writer: &mut impl Write, mouse: MouseEvent, x: u16, y: u16, mode: vt100::MouseProtocolMode) -> std::io::Result<()> {
    let allowed = match mouse.kind {
        MouseEventKind::Down(_) | MouseEventKind::ScrollUp | MouseEventKind::ScrollDown | MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => true,
        MouseEventKind::Up(_) => matches!(mode, vt100::MouseProtocolMode::PressRelease | vt100::MouseProtocolMode::ButtonMotion | vt100::MouseProtocolMode::AnyMotion),
        MouseEventKind::Drag(_) => matches!(mode, vt100::MouseProtocolMode::ButtonMotion | vt100::MouseProtocolMode::AnyMotion),
        MouseEventKind::Moved => mode == vt100::MouseProtocolMode::AnyMotion,
    };
    if !allowed { return Ok(()); }
    let (button, release) = match mouse.kind {
        MouseEventKind::Down(button) => (mouse_button(button), false),
        MouseEventKind::Up(button) => (mouse_button(button), true),
        MouseEventKind::Drag(button) => (mouse_button(button) | 32, false),
        MouseEventKind::Moved => (35, false),
        MouseEventKind::ScrollUp => (64, false),
        MouseEventKind::ScrollDown => (65, false),
        MouseEventKind::ScrollLeft => (66, false),
        MouseEventKind::ScrollRight => (67, false),
    };
    let modifier = (u8::from(mouse.modifiers.contains(crossterm::event::KeyModifiers::SHIFT)) * 4)
        | (u8::from(mouse.modifiers.contains(crossterm::event::KeyModifiers::ALT)) * 8)
        | (u8::from(mouse.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)) * 16);
    let code = button + modifier;
    write!(writer, "\x1b[<{code};{};{}{}", x + 1, y + 1, if release { 'm' } else { 'M' })
}

fn mouse_button(button: MouseButton) -> u8 {
    match button { MouseButton::Left => 0, MouseButton::Middle => 1, MouseButton::Right => 2 }
}

fn render(
    frame: &mut Frame,
    parser: &Arc<RwLock<vt100::Parser>>,
    workspaces: &Arc<RwLock<Vec<String>>>,
) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Fill(1)])
        .split(frame.area());

    let block_tasks = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Tasks ");
    let block_terminal = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Terminal ");

    frame.render_widget(&block_tasks, layout[0]);
    frame.render_widget(&block_terminal, layout[1]);

    let block_tasks_inner = block_tasks.inner(layout[0]);

    if let Ok(workspaces) = workspaces.read() {
        let workspace_lines = workspaces
            .iter()
            .map(|workspace| format!("• {workspace}"))
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(workspace_lines.join("\n")), block_tasks_inner);
    }

    let block_terminal_inner = block_terminal.inner(layout[1]);

    if let Ok(parser) = parser.read() {
        frame.render_widget(PseudoTerminal::new(parser.screen()), block_terminal_inner);
    }
}
