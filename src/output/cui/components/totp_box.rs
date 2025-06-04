use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::{layout::{Alignment, Constraint, Flex, Layout, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Frame};

use crate::{config::{configuration::NumberStyle, secrets::ConfigEntry}, output::cui::numbers::{pipe::big_number_font, utf8::utf8_font}, totp::{generate_totp, Totp}};

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

    pub fn render(totp_box: Option<&TotpBox>, frame: &mut Frame, area: Rect, index: u8, number_style: NumberStyle ) {
        let size = area.as_size();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));
        // 1️⃣ Outer block
        frame.render_widget(block, area);

        // 2️⃣ Layout: vertical split
        let [top_row, second_row, main_row, bottom_row ]: [Rect; 4]  = Layout::vertical([
            Constraint::Length(1),  // Top row
            Constraint::Length(2),  // Second row
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
            .flex(Flex::Center).areas(second_row);
        let main_content = if let Some(t) = totp_box {
            match number_style {
                NumberStyle::Utf8 => font_to_lines(utf8_font(t.totp.token.as_str())),
                NumberStyle::Standard => vec![Line::from(t.totp.token.clone())],
                NumberStyle::Pipe => font_to_lines(big_number_font(t.totp.token.as_str())),
                NumberStyle::Lite => render_lite_number_lines(t.totp.token.as_str()),
            }
        } else {
            vec![Line::from("N/A".to_owned())]
        };
        let [ main_cell] = Layout::horizontal([Constraint::Fill(1)])
            .flex(Flex::Center).areas(main_row);

        frame.render_widget(
            Paragraph::new(main_content).alignment(Alignment::Center), main_cell);

        if let Some(t) = totp_box {
            // Avoid trouble with margin and padding simply add an extra space
            frame.render_widget(Paragraph::new(format!(" {} ", index_to_char(index).unwrap())), top_left);
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

fn index_to_char(index: u8) -> Option<char> {
    "0123456789abcdefghijklmnop".chars().nth(index.into())
}

fn font_to_lines(font: Vec<String>) -> Vec<Line<'static>> {
    font
        .into_iter()
        .map(|l| Line::from(Span::raw(l)))
        .collect()
}

fn render_lite_number_lines(input: &str) -> Vec<Line<'static>> {
    let digits = [
        [
            "  ___  ",
            " / _ \\ ",
            "| | | |",
            "| | | |",
            "| |_| |",
            " \\___/ ",
        ], // 0
        [
            " __ ", 
            "/_ |",
            " | |",
            " | |",
            " | |",
            " |_|",
        ], // 1
        [
            " ___  ",
            "|__ \\ ",
            "   ) |",
            "  / / ",
            " / /_ ",
            "|____|",
        ], // 2
        [
            " ____  ",
            "|___ \\ ",
            "  __) |",
            " |__ < ",
            " ___) |",
            "|____/ ",
        ], // 3
        [
            " _  _   ",
            "| || |  ",
            "| || |_ ",
            "|__   _|",
            "   | |  ",
            "   |_|  ",
        ], // 4
        [
            " _____ ",
            "| ____|",
            "| |__  ",
            "|___ \\ ",
            " ___) |",
            "|____/ ",
        ], // 5
        [
            "   __  ",
            "  / /  ",
            " / /_  ",
            "| '_ \\ ",
            "| (_) |",
            " \\___/ ",
        ], // 6
        [
            " ______",
            "|____  |",
            "    / / ",
            "   / /  ",
            "  / /   ",
            " /_/    ",
        ], // 7
        [
            "  ___  ",
            " / _ \\ ",
            "| (_) |",
            " > _ < ",
            "| (_) |",
            " \\ _ / ",
        ], // 8
        [
            "  ___  ",
            " / _ \\ ",
            "| (_) |",
            " \\__, |",
            "   / / ",
            "  /_/  ",
        ], // 9
    ];

    let mut output = vec!["".to_string(); input.len()];

    for ch in input.chars() {
        if let Some(d) = ch.to_digit(10) {
            let digit_lines = &digits[d as usize];
            for (i, line) in digit_lines.iter().enumerate() {
                output[i].push_str(line);
            }
        } else {
            for line in &mut output {
                line.push_str("       "); // space for unknown chars
            }
        }
    }

    output
        .into_iter()
        .map(|l| Line::from(Span::raw(l)))
        .collect()
}

