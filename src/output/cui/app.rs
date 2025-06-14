use ::time::{format_description, OffsetDateTime};
use crossterm::event::{Event, EventStream};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{io, time::SystemTime};
use tokio::time::{self, Duration};
use tokio_stream::StreamExt;

use crate::{
    config::secrets::ConfigEntry, output::cui::input::keyboard::KeyboardAction, state::State,
};

use super::components::{messages::Messages, totp_box::TotpBox};

pub struct App {
    pub totps: Vec<TotpBox>,
    pub state: State,
    secrets: Vec<ConfigEntry>,
    messages: Messages,
}

#[cfg(feature = "cli")]
impl App {
    pub fn new(state: State) -> App {
        App {
            totps: vec![],
            state,
            secrets: vec![],
            messages: Messages::new(),
        }
    }

    pub fn add_message(&mut self, message: String) {
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        let fmt = format_description::parse("[hour]:[minute]:[second]").unwrap();
        let out = now.format(&fmt).unwrap();
        self.messages.push(format!("[{}] {message}", out));
    }

    pub async fn totp_changed(&mut self) {
        let mut interval = time::interval(Duration::from_millis(50));
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Burst);
        loop {
            interval.tick().await;
            if self.update_totps().await {
                break;
            }
        }
    }

    fn render_locked_screen(&self, frame: &mut Frame) {
        let area = frame.area();

        // Create a centered layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(45), // top padding
                Constraint::Min(3),         // centered content
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
            self.render_normal_screen(frame);
        } else {
            self.render_locked_screen(frame)
        }
    }

    fn get_row_and_column_constraints(&self) -> (Vec<Constraint>, Vec<Constraint>) {
        let (r, c) = self.get_rows_and_columns();
        let r = vec![Constraint::Ratio(1, r.into()); r.into()];
        let c = vec![Constraint::Ratio(1, c.into()); c.into()];
        (r, c)
    }
    fn render_normal_screen(&mut self, frame: &mut Frame) {
        let [messages_row, totps_row]: [Rect; 2] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());
        self.render_totps(totps_row, frame);
        frame.render_widget(Paragraph::new(self.messages.last()), messages_row);
    }

    fn render_totps(&mut self, rect: Rect, frame: &mut Frame) {
        let (row_constraint, col_constraint) = self.get_row_and_column_constraints();
        let rows = Layout::vertical(&row_constraint).split(rect);

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

    pub fn is_locked(&self) -> bool {
        self.state.unlocked_since.is_none()
    }

    pub fn unlock(&mut self) {
        self.state.unlocked_since = Some(SystemTime::now());
    }

    pub fn lock(&mut self) {
        self.state.unlocked_since = None;
    }

    fn get_rows_and_columns(&self) -> (u8, u8) {
        match self.secrets.len() {
            n if n <= 4 => (2, 2),
            n if n <= 6 => (2, 3),
            n if n <= 9 => (3, 3),
            n if n <= 12 => (3, 4),
            n if n <= 16 => (4, 4),
            n if n <= 20 => (4, 5),
            _ => (4, 6),
        }
    }

    async fn update_totps(&mut self) -> bool {
        let mut has_changed = false;
        match self.state.secrets_cf.load().await {
            Err(err) => {
                tracing::error!("Error loading secrets file {err}");
                self.secrets = vec![];
                self.add_message(format!("Error loading secrets file {err}"));
            }
            Ok((changed, entries)) => {
                has_changed = changed;
                if changed {
                    self.secrets = entries;
                    self.add_message("Secrets file has changed, reloading".to_owned());
                }
            }
        };
        let secrets = &self.secrets;
        if secrets.len() != self.totps.len() {
            self.totps.truncate(secrets.len());
            has_changed = true;
        }
        for (i, entry) in secrets.iter().enumerate() {
            if let Some(existing) = self.totps.get_mut(i) {
                if entry.code != existing.code {
                    self.totps[i] = TotpBox::from(entry);
                    has_changed = true;
                } else if existing.needs_refresh() {
                    existing.refresh();
                    if !has_changed {
                        // this should log only once
                        tracing::trace!(?existing.valid_duration_seconds, "some otp needs refresh");
                    }
                    has_changed = true;
                }
            } else {
                self.totps.insert(i, TotpBox::from(entry));
                has_changed = true;
            }
        }
        has_changed
    }
}

#[cfg(feature = "cli")]
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let mut reader = EventStream::new();
    loop {
        // Draw the UI
        terminal.draw(|f| app.render(f))?;

        // Wait for either a tick or a key event
        tokio::select! {
            _ = app.totp_changed() => {
                tracing::trace!("Some totp has changed, re-rendering");
            },
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key_code))) => {
                        match app.handle_key(key_code) {
                            KeyboardAction::Exit(reason)=>{
                                println!("{}",reason);return Ok(())
                            }
                            KeyboardAction::Message(m) => {
                                tracing::info!("{}", m);
                                app.add_message(m);
                            },
                            KeyboardAction::ErrorMessage(m) => {
                                tracing::warn!("{}", m);
                                app.add_message(format!("[E] {}", m));
                            },
                            KeyboardAction::NoOp => {
                                tracing::debug!("No op");
                            },
                        }

                    }
                    _ => {
                        tracing::trace!(?maybe_event, "Non key event");
                    }
                }
            }
        }
    }
}
