use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{
    io,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::time::{self, Duration};

use crate::{
    config::secrets::{ConfigEntry, load_secrets},
    state::State,
};

use super::components::totp_box::TotpBox;

pub struct App {
    totps: Vec<TotpBox>,
    refresh_rate: u8,
    state: State,
}

impl App {
    pub fn new(state: State, refresh_rate: u8) -> App {
        App {
            totps: vec![],
            state,
            refresh_rate,
        }
    }

    fn render_locked_screen(&self, frame: &mut Frame) {
        let area = frame.size();

        // Create a centered layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(45), // top padding
                Constraint::Min(1),         // centered content
                Constraint::Percentage(45), // bottom padding
            ])
            .split(area);
        let block = Block::default().title("🔒 Locked").borders(Borders::ALL);

        let lines: Line = if self.state.lock_password.is_none() {
            "Press any key to unlock".into()
        } else {
            vec![
                Span::styled("Enter password: ", Style::default()),
                Span::styled(
                    "*".repeat(self.state.buffer.len()),
                    Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                ),
            ]
                .into()
        };
        let paragraph = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, layout[1]);
    }

    fn render(&mut self, frame: &mut Frame) {
        let now = SystemTime::now();
        if let Some(since) = self.state.unlocked_since {
            // We aren't locked but it's time to lock
            if let Some(d) = self.state.lock_after {
                if now.duration_since(since).expect("Issue computing duration") >= d {
                    self.lock();
                    return self.render(frame);
                }
            }
            self.render_totps(frame);
        } else {
            return self.render_locked_screen(frame);
        }
    }

    fn get_row_and_column_constraints(&self) -> (Vec<Constraint>, Vec<Constraint>) {
        let (r, c) = self.get_rows_and_columns();
        let r = vec![Constraint::Ratio(1, r.into()); r.into()];
        let c = vec![Constraint::Ratio(1, c.into()); c.into()];
        (r, c)
    }

    fn render_totps(&mut self, frame: &mut Frame) {
        let size = frame.size();
        let (row_constraint, col_constraint) = self.get_row_and_column_constraints();
        let rows = Layout::vertical(&row_constraint).split(size);

        let mut totps = self.totps.iter();
        let mut i: u8 = 0;
        for row in rows.iter() {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(&col_constraint)
                .split(*row);

            for col in cols.iter() {
                TotpBox::render(
                    totps.next(),
                    frame,
                    *col,
                    i,
                    self.state.number_style.clone(),
                );
                i += 1;
            }
        }
    }

    fn is_locked(&self) -> bool {
        self.state.unlocked_since.is_none()
    }

    fn unlock(&mut self) {
        self.state.unlocked_since = Some(SystemTime::now());
    }

    fn lock(&mut self) {
        self.state.unlocked_since = None;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<&str> {
        let (code, modifiers) = (key.code, key.modifiers);
        eprintln!("Key event received {:?} ", code);
        if self.is_locked() {
            let mut should_unlock = false;
            let buffer = &mut self.state.buffer;
            if let Some(password) = self.state.lock_password.clone() {
                if code == KeyCode::Enter {
                    should_unlock = *buffer == password;
                    buffer.clear();
                } else if code == KeyCode::Backspace {
                    buffer.pop();
                } else if modifiers.is_empty() || modifiers == KeyModifiers::SHIFT {
                    if let Some(ch) = keyevent_to_char(key) {
                        buffer.push(ch);
                    }
                }
            } else {
                // Unlock when no password with any key
                should_unlock = true;
            }
            if should_unlock {
                self.unlock();
            }
            return None;
        }

        if code == KeyCode::Char('q') {
            println!("Pressed <q>, Quitting");
            return Some("Pressed <q>, Quitting");
        }
        if code == KeyCode::Char('l') {
            self.lock();
            return None;
        }
        if let Some(totp) = keyevent_to_char(key)
            .and_then(char_to_index)
            .and_then(|i| self.totps.get(i)) 
        {
            if let Err(err) = ClipboardContext::new()
                .and_then(|mut ctx| ctx.set_contents(totp.totp.token.clone())) {
                eprintln!("Failed to copy to clipboard {}", err);
            } else {
                eprintln!("Copied to clipboard");
            }
        } else {
            eprintln!("Character could not be mapped to existing TOTP");
        }

        None
    }

    fn get_secrets_list(&self) -> Vec<ConfigEntry> {
        load_secrets(self.state.secrets_path.as_str()).entries
    }

    fn get_rows_and_columns(&self) -> (u8, u8) {
        match self.get_secrets_list().len() {
            n if n <= 4 => (2, 2),
            n if n <= 6 => (2, 3),
            n if n <= 9 => (3, 3),
            n if n <= 12 => (3, 4),
            n if n <= 16 => (4, 4),
            n if n <= 20 => (4, 5),
            _ => (4, 6),
        }
    }

    fn update_totps(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        // Check whether we are locked in the first place
        let secrets = self.get_secrets_list();
        for (i, entry) in secrets.iter().enumerate() {
            if let Some(existing) = self.totps.get_mut(i) {
                if entry.code != existing.code {
                    self.totps[i] = TotpBox::from(entry);
                } else if existing.totp.valid_until <= now {
                    existing.refresh();
                }
            } else {
                self.totps.insert(i, TotpBox::from(entry));
            }
        }
    }
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let mut interval = time::interval(Duration::from_secs(app.refresh_rate.into()));
    loop {
        // Draw the UI
        terminal.draw(|f| app.render(f))?;

        // Wait for either a tick or a key event
        tokio::select! {
            _ = interval.tick() => {
                app.update_totps();
            },
            maybe_event = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(50))) => {
                if maybe_event?? {
                    if let Event::Key(key) = event::read()? {
                        if let Some(reason) = app.handle_key(key) {
                            println!("Exited due to {}", reason);
                            return Ok(())
                        }
                    }
                }
            }
        }
    }
}

fn keyevent_to_char(key_event: KeyEvent) -> Option<char> {
    match key_event.code {
        KeyCode::Char(c) => Some(c),
        _ => None,
    }
}

fn char_to_index(ch: char) -> Option<usize> {
    "0123456789abcdefghijklmnop".find(ch)
}
