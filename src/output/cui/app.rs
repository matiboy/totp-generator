use std::{io, time::{SystemTime, UNIX_EPOCH}};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::Backend, layout::{Alignment, Constraint, Direction, Layout}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Frame, Terminal
};
use tokio::time::{self, Duration};

use crate::{config::secrets::{load_secrets, ConfigEntry}, state::State};

use super::components::totp_box::TotpBox;

pub struct App {
    totps: Vec<TotpBox>,
    refresh_rate: u8,
    state: State,
}

impl App {
    pub fn new(state: State, refresh_rate: u8)-> App {
        App {
            totps: vec![],
            state,
            refresh_rate 
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
        let block = Block::default()
            .title("🔒 Locked")
            .borders(Borders::ALL);

        let lines: Line = if self.state.lock_password.is_none() {
            "Press any key to unlock".into()
        } else {
            vec![
                Span::styled("Enter password: ", Style::default()),
                Span::styled("*".repeat(self.state.buffer.len()), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ].into()
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
                    self.state.unlocked_since = None;
                    return self.render(frame);
                }
            }
            self.render_totps(frame);
        } else {
            return self.render_locked_screen(frame);
        }
    }

    fn get_row_and_column_constraints(&self) -> (Vec<Constraint>, Vec<Constraint>)  {
        let (r, c) = self.get_rows_and_columns();
        let r = vec![Constraint::Ratio(1, r.into()); r.into()];
        let c = vec![Constraint::Ratio(1, c.into()); c.into()]; 
        (r, c)
    }

    fn render_totps(&mut self, frame: &mut Frame) {
        let size = frame.size();
        let (row_constraint, col_constraint) = self.get_row_and_column_constraints();
        let rows = Layout::vertical(&row_constraint)
            .split(size);

        let mut totps = self.totps.iter();
        let mut i: u8 = 0;
        for row in rows.iter() {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(&col_constraint)
                .split(*row);

            for col in cols.iter() {
                i += 1;
                TotpBox::render(totps.next(), frame, *col, i);
            }
        }
    }

    fn is_locked(&self) -> bool {
        self.state.unlocked_since.is_none()
    }

    fn unlock(&mut self) {
        self.state.unlocked_since = Some(SystemTime::now());
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<&str> {
        if self.is_locked() {
            let (code,  modifiers,) = (key.code, key.modifiers);
            let mut should_unlock = false;
            if let Some(password) = self.state.lock_password.clone() {
                if code == KeyCode::Enter {
                    should_unlock = self.state.buffer == password;
                    self.state.buffer = "".to_owned();
                } else if modifiers.is_empty() || modifiers == KeyModifiers::SHIFT {
                    if let Some(ch) = keyevent_to_char(key) {
                        self.state.buffer.push(ch);
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


        if key.code == KeyCode::Char('q') {
            println!("Pressed <q>, Quitting");
            return Some("Pressed <q>, Quitting");
        }
        None
    }

    pub async fn totps_updated(&mut self) {
        // This checks every second whether the OTPs have been updated. Note that this is actually
        // pointless if we are going to display how long a totp is valid 🥲
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            if self.update_totps() {
                break;
            }
        }}

    fn get_secrets_list(&self) -> Vec<ConfigEntry> {
        load_secrets(self.state.secrets_path.as_str()).entries
    }

    fn get_rows_and_columns(&self) -> (u8, u8) {
        match self.get_secrets_list().len() {
            n if n <=4 => (2,2),
            n if n <=6 => (2,3),
            n if n <=9 => (3,3),
            n if n <=12 => (3,4),
            n if n <=16 => (4,4),
            n if n <=20 => (4,5),
            _ => (4,6),
        }
    }

    fn update_totps(&mut self) -> bool {
        let mut has_updated = false;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        // Check whether we are locked in the first place
        let secrets = self.get_secrets_list();
        for (i, entry) in secrets.iter().enumerate() {
            if let Some(existing) = self.totps.get_mut(i) {
                if entry.code != existing.code {
                    has_updated = true;
                    self.totps[i] = TotpBox::from(entry);
                } else if existing.totp.valid_until <= now {
                    has_updated = true;
                    existing.refresh();
                }
            } else {
                has_updated = true;
                self.totps.insert(i, TotpBox::from(entry));
            }
        }
        has_updated
    }
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|f| {
            app.render(f)
        })?;

        // Wait for either a tick or a key event
        tokio::select! {
            _ = app.totps_updated() => {},
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
