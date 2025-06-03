use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::{layout::{Alignment, Constraint, Flex, Layout, Margin, Rect}, style::{Color, Style}, text::Text, widgets::{Block, Borders, Clear, Padding, Paragraph}, Frame};

use crate::{config::secrets::ConfigEntry, totp::{generate_totp, Totp}};

#[derive(Debug)]
pub struct TotpBox {
    pub totp: Totp,
    pub name: String,
    pub code: String,
    secret : String,
    pub timestep: u8,
}

impl From<&ConfigEntry> for TotpBox {
    fn from(entry: &ConfigEntry) -> Self {
        TotpBox {
            name: entry.name.clone(),
            code: entry.code.clone(),
            secret: entry.secret.clone(),
            timestep: entry.timestep,
            totp: generate_totp(entry.secret.as_str(), entry.timestep.into(), None, None)
        }
    }

}

impl TotpBox {
    pub fn valid_duration(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        (self.totp.valid_until as i64) - now as i64
    }

    pub fn render(totp_box: Option<&TotpBox>, frame: &mut Frame, area: Rect, index: u8 ) {
        let size = area.as_size();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));
        // 1️⃣ Outer block
        frame.render_widget(block, area);

        // 2️⃣ Layout: vertical split
        let [top_row, second_row, main_row, bottom_row ]: [Rect; 4]  = Layout::vertical([
            Constraint::Length(1),  // Top row
            Constraint::Length(3),  // Second row
            Constraint::Fill(1),     // Center area
            Constraint::Length(2),  // Bottom row
        ])
            .areas(area);

        // 3️⃣ Top row: horizontal split
        let [top_left, _tc, top_right]: [Rect; 3] = Layout::horizontal([
            Constraint::Length(10), // Top left (number)
            Constraint::Min(0),
            Constraint::Length(10), // Top right
        ])
            .horizontal_margin(1)
            .areas(top_row);

        let [ bottom_cell ] = Layout::horizontal([Constraint::Min(3)])
            .horizontal_margin(2)
            .areas(bottom_row);
        let [ second_cell ] = Layout::horizontal([Constraint::Length(size.width)])
            .horizontal_margin(2)
            .vertical_margin(1)
            .flex(Flex::Center).areas(second_row);
        let main_content = if let Some(t) = totp_box {
            t.totp.token.clone()
        } else {
            "N/A".to_owned()
        };
        let main_content = Text::raw(main_content);
        let [ main_cell] = Layout::horizontal([Constraint::Length(main_content.width() as u16)])
            .flex(Flex::Center).areas(main_row);

        frame.render_widget(
            Paragraph::new(main_content.clone()), main_cell);

        if let Some(t) = totp_box {
            // Avoid trouble with margin and padding simply add an extra space
            frame.render_widget(Paragraph::new(format!(" {} ", index)), top_left);
            let content = if t.code.is_empty() { "".to_owned() } else { format!(" {} ", t.code) };
            frame.render_widget(Paragraph::new(content)
                .alignment(Alignment::Right), top_right);
            frame.render_widget(
                Paragraph::new(t.name.clone()), second_cell
            );
            frame.render_widget(
                Paragraph::new(format!("{}s", t.valid_duration().to_string())).alignment(Alignment::Right), 
                bottom_cell
            );
        }

    }

    pub fn refresh(&mut self)  {
        self.totp = generate_totp(self.secret.as_str(), self.timestep.into(), None, None);
    }
}

