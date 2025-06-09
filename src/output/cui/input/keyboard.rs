use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::output::cui::app::App;

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) -> KeyboardAction {
        let (code, modifiers) = (key.code, key.modifiers);
        tracing::debug!("Key event received {:?} ", code);
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
            return KeyboardAction::NoOp;
        }

        if code == KeyCode::Char('q') {
            return KeyboardAction::Exit("Pressed <q>, Quitting".to_owned());
        }
        if code == KeyCode::Char('l') {
            self.lock();
            return KeyboardAction::Message("Manually locked".to_owned());
        }
        if let Some(totp) = keyevent_to_char(key)
            .and_then(char_to_index)
            .and_then(|i| self.totps.get(i))
        {
            if let Err(err) = ClipboardContext::new()
                .and_then(|mut ctx| ctx.set_contents(totp.get_token()))
            {
                KeyboardAction::ErrorMessage(format!(
                    "Failed to copy to clipboard {}",
                    err
                ))
            } else {
                KeyboardAction::Message("Copied to clipboard".to_owned())
            }
        } else {
            KeyboardAction::ErrorMessage(
                format!("Character {:?} could not be mapped to existing TOTP", code),
            )
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

pub enum KeyboardAction {
    Message(String),
    ErrorMessage(String),
    NoOp,
    Exit(String),
}
