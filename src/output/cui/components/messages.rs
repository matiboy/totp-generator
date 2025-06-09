pub struct Messages {
    messages: Vec<String>
}

impl Messages {
    pub fn new() -> Self {
        let messages: Vec<String> = vec![];
        Messages { messages }
    }

    pub fn push(&mut self, v: String) {
        push_limited(&mut self.messages, v, 100);
    }

    pub fn last(&self) -> String {
        self.messages.last().cloned().unwrap_or_else(String::new)
    }
}

fn push_limited(messages: &mut Vec<String>, new: String, max: usize) {
    messages.push(new);
    if messages.len() > max {
        messages.drain(0..(messages.len() - max));
    }
}
