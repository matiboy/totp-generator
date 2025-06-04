use std::collections::HashMap;

pub fn utf8_font(input: &str) -> Vec<String> {
    let font: HashMap<char, [&str; 5]> = HashMap::from([
        ('0', [
         " ████  ",
         "██  ██ ",
         "██  ██ ",
         "██  ██ ",
         " ████  ",
        ]),
        ('1', [
         "  ██  ",
         " ███  ",
         "  ██  ",
         "  ██  ",
         " ████ ",
        ]),
        ('2', [
         " ████  ",
         "    ██ ",
         "  ███  ",
         " ██    ",
         " █████ ",
        ]),
        ('3', [
         " ████  ",
         "    ██ ",
         "  ███  ",
         "    ██ ",
         " ████  ",
        ]),
        ('4', [
         " ██ ██ ",
         " ██ ██ ",
         " █████ ",
         "    ██ ",
         "    ██ ",
        ]),
        ('5', [
         " ████ ",
         " ██   ",
         " ████ ",
         "   ██ ",
         " ████ ",
        ]),
        ('6', [
         "  ███  ",
         " ██    ",
         " ████  ",
         " ██ ██ ",
         "  ███  ",
        ]),
        ('7', [
         " █████ ",
         "    ██ ",
         "   ██  ",
         "  ██   ",
         " ██    ",
        ]),
        ('8', [
         "  ███  ",
         " ██ ██ ",
         "  ███  ",
         " ██ ██ ",
         "  ███  ",
        ]),
        ('9', [
         "  ███  ",
         " ██ ██ ",
         "  ████ ",
         "    ██ ",
         "  ███  ",
        ]),
        ]);
    let mut output = vec![String::new(); 5];
    for ch in input.chars() {
        if let Some(rows) = font.get(&ch) {
            for (i, row) in rows.iter().enumerate() {
                output[i].push_str(row);
                output[i].push(' ');
            }
        }
    }
    output
}
