use ratatui::style::Color;
use std::str::FromStr;

pub fn parse_color(s: &str) -> Color {
    if let Ok(color) = Color::from_str(&s.to_lowercase()) {
        return color;
    }
    if let Some(hex_str) = s.strip_prefix('#') {
        if let Ok(rgb) = u32::from_str_radix(hex_str, 16) {
            let r = ((rgb >> 16) & 0xFF) as u8;
            let g = ((rgb >> 8) & 0xFF) as u8;
            let b = (rgb & 0xFF) as u8;
            return Color::Rgb(r, g, b);
        }
    }
    if let Some(rgb_str) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<Result<u8, _>> = rgb_str.split(',').map(|p| p.trim().parse()).collect();
        if parts.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (&parts[0], &parts[1], &parts[2]) {
                return Color::Rgb(*r, *g, *b);
            }
        }
    }
    Color::Reset
}