
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
            for i in 0..3 {
                lines[i].push_str(digits[d as usize][i]);
            }
        } else {
            for i in 0..3 {
                lines[i].push_str("   ");
            }
        }
    }
    lines
}
