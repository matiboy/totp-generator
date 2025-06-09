
pub fn big_number_font(input: &str) -> Vec<String> {
    let digits = [
        ["╔═╗", "║ ║", "╚═╝"], // 0
        [" ║ ", " ║ ", " ║ "], // 1
        ["╔═╗", "╔═╝", "╚═╝"], // 2
        ["╔═╗", " ═╣", "╚═╝"], // 3
        ["║ ║", "╚═╣", "  ║"], // 4
        ["╔══", "╚═╗", "╚═╝"], // 5
        ["╔══", "╠═╗", "╚═╝"], // 6
        ["══╗", "  ║", "  ║"], // 7
        ["╔═╗", "╠═╣", "╚═╝"], // 8
        ["╔═╗", "╚═╣", "══╝"], // 9
    ];

    let mut lines = vec![String::new(), String::new(), String::new()];

    for ch in input.chars() {
        if let Some(d) = ch.to_digit(10) {
            for (line, ch) in lines.iter_mut().zip(digits[d as usize]) {
                line.push_str(ch);
            }
        } else {
            for line in lines.iter_mut() {
                line.push_str("   ");
            }
        }
    }
    lines
}
