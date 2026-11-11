use std::{
    fs,
    io::{Read, Write},
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Args;
use crossterm::{
    ExecutableCommand,
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    },
};
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

fn terminal_pane(area: Rect) -> Rect {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Fill(1)])
        .split(area)[1]
}

fn terminal_pane_inner(area: Rect) -> Rect {
    let pane = terminal_pane(area);
    Rect::new(
        pane.x + 1,
        pane.y + 1,
        pane.width.saturating_sub(2),
        pane.height.saturating_sub(2),
    )
}

fn encode_key(code: KeyCode, modifiers: KeyModifiers) -> Option<Vec<u8>> {
    let bytes = match code {
        KeyCode::Enter => b"\r".to_vec(),
        KeyCode::Backspace => b"\x7f".to_vec(),
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::F(n @ 1..=4) => format!("\x1b[1;{}P", n).into_bytes(),
        KeyCode::F(n @ 5..=12) => format!("\x1b[{}~", 15 + (n - 5) * 2).into_bytes(),
        KeyCode::F(_) => return None,
        KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) => vec![control_byte(c)?],
        KeyCode::Char(c) => {
            let mut buf = [0; 4];
            c.encode_utf8(&mut buf).as_bytes().to_vec()
        }
        KeyCode::Null => vec![0],
        _ => return None,
    };
    if modifiers.contains(KeyModifiers::ALT) {
        let mut out = vec![0x1b];
        out.extend(bytes);
        Some(out)
    } else {
        Some(bytes)
    }
}

fn control_byte(c: char) -> Option<u8> {
    match c {
        'a'..='z' => Some(c as u8 - b'a' + 1),
        'A'..='Z' => Some(c as u8 - b'A' + 1),
        '@' | ' ' | '2' => Some(0),
        '[' | '3' => Some(0x1b),
        '\\' | '4' => Some(0x1c),
        ']' | '5' => Some(0x1d),
        '^' | '6' => Some(0x1e),
        '_' | '7' | '/' => Some(0x1f),
        '?' | '8' => Some(0x7f),
        _ => None,
    }
}

fn write_mouse(
    writer: &mut impl Write,
    mouse: MouseEvent,
    x: u16,
    y: u16,
    mode: vt100::MouseProtocolMode,
) -> std::io::Result<()> {
    let allowed = match mouse.kind {
        MouseEventKind::Down(_)
        | MouseEventKind::ScrollUp
        | MouseEventKind::ScrollDown
        | MouseEventKind::ScrollLeft
        | MouseEventKind::ScrollRight => true,
        MouseEventKind::Up(_) => matches!(
            mode,
            vt100::MouseProtocolMode::PressRelease
                | vt100::MouseProtocolMode::ButtonMotion
                | vt100::MouseProtocolMode::AnyMotion
        ),
        MouseEventKind::Drag(_) => matches!(
            mode,
            vt100::MouseProtocolMode::ButtonMotion | vt100::MouseProtocolMode::AnyMotion
        ),
        MouseEventKind::Moved => mode == vt100::MouseProtocolMode::AnyMotion,
    };
    if !allowed {
        return Ok(());
    }
    let (button, release) = match mouse.kind {
        MouseEventKind::Down(b) => (mouse_button(b), false),
        MouseEventKind::Up(b) => (mouse_button(b), true),
        MouseEventKind::Drag(b) => (mouse_button(b) | 32, false),
        MouseEventKind::Moved => (35, false),
        MouseEventKind::ScrollUp => (64, false),
        MouseEventKind::ScrollDown => (65, false),
        MouseEventKind::ScrollLeft => (66, false),
        MouseEventKind::ScrollRight => (67, false),
    };
    let modifier = (u8::from(mouse.modifiers.contains(KeyModifiers::SHIFT)) * 4)
        | (u8::from(mouse.modifiers.contains(KeyModifiers::ALT)) * 8)
        | (u8::from(mouse.modifiers.contains(KeyModifiers::CONTROL)) * 16);
    write!(
        writer,
        "\x1b[<{};{};{}{}",
        button + modifier,
        x + 1,
        y + 1,
        if release { 'm' } else { 'M' }
    )
}

fn mouse_button(button: MouseButton) -> u8 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
    }
}

fn run_app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let config_path = std::env::temp_dir().join(format!("herdr-view-{}.toml", std::process::id()));
    let config = toml::toml! {
        onboarding = false

        [ui]
        sidebar_collapsed_mode = "hidden"
        sidebar_start_collapsed = true
    };
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&config_path)?
        .write_all(config.to_string().as_bytes())?;

    let workspaces = Arc::new(RwLock::new(
        Herdr::new()
            .list_workspaces()
            .map_err(std::io::Error::other)?,
    ));
    let workspace_refresh = workspaces.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            if let Ok(updated) = Herdr::new().list_workspaces()
                && let Ok(mut current) = workspace_refresh.write()
            {
                *current = updated;
            }
        }
    });

    let mut stdout = std::io::stdout();
    let size = terminal.size()?;
    let pty = NativePtySystem::default()
        .openpty(PtySize {
            rows: size.height.saturating_sub(2),
            cols: size.width.saturating_sub(32),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(std::io::Error::other)?;
    let mut command = CommandBuilder::new("herdr");
    command.cwd(std::env::current_dir()?);
    command.env("HERDR_CONFIG_PATH", &config_path);
    let mut child = pty
        .slave
        .spawn_command(command)
        .map_err(std::io::Error::other)?;
    drop(pty.slave);
    let mut reader = pty
        .master
        .try_clone_reader()
        .map_err(std::io::Error::other)?;
    let mut writer = pty.master.take_writer().map_err(std::io::Error::other)?;
    let parser = Arc::new(RwLock::new(vt100::Parser::new(
        size.height.saturating_sub(2),
        size.width.saturating_sub(32),
        0,
    )));
    let output = parser.clone();
    let (mode_tx, mode_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buffer = [0; 8192];
        let mut last_mode = vt100::MouseProtocolMode::None;
        while let Ok(count) = reader.read(&mut buffer) {
            if count == 0 {
                break;
            }
            if let Ok(mut parser) = output.write() {
                parser.process(&buffer[..count]);
                let mode = parser.screen().mouse_protocol_mode();
                if mode != last_mode {
                    let _ = mode_tx.send(mode);
                    last_mode = mode;
                }
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
                    if key.code == KeyCode::Esc {
                        break;
                    }
                    if let Some(bytes) = encode_key(key.code, key.modifiers) {
                        writer.write_all(&bytes)?;
                    }
                }
                crossterm::event::Event::Resize(cols, rows) => {
                    let area = Rect::new(0, 0, cols, rows);
                    let inner = terminal_pane(area);
                    pty.master
                        .resize(PtySize {
                            rows: inner.height,
                            cols: inner.width,
                            pixel_width: 0,
                            pixel_height: 0,
                        })
                        .map_err(std::io::Error::other)?;
                    if let Ok(mut p) = parser.write() {
                        p.screen_mut().set_size(inner.height, inner.width);
                    }
                }
                crossterm::event::Event::Mouse(mouse) => {
                    let size = terminal.size()?;
                    let pane = terminal_pane_inner(Rect::new(0, 0, size.width, size.height));
                    if mouse.column >= pane.x
                        && mouse.column < pane.x + pane.width
                        && mouse.row >= pane.y
                        && mouse.row < pane.y + pane.height
                    {
                        write_mouse(
                            &mut writer,
                            mouse,
                            mouse.column - pane.x,
                            mouse.row - pane.y,
                            mouse_mode,
                        )?;
                    }
                }
                _ => {}
            }
        }
        if child.try_wait().map_err(std::io::Error::other)?.is_some() {
            break;
        }
    }
    stdout.execute(DisableMouseCapture)?;
    let _ = child.kill();
    fs::remove_file(config_path)?;
    Ok(())
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
        frame.render_widget(
            Paragraph::new(workspace_lines.join("\n")),
            block_tasks_inner,
        );
    }

    let block_terminal_inner = block_terminal.inner(layout[1]);

    if let Ok(parser) = parser.read() {
        frame.render_widget(PseudoTerminal::new(parser.screen()), block_terminal_inner);
    }
}
