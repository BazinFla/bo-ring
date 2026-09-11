pub use crate::platform::accent_color::detect_os_accent_color;

pub fn parse_hex_color(hex: &str) -> Option<eframe::egui::Color32> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(eframe::egui::Color32::from_rgb(r, g, b))
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        let a = u8::from_str_radix(&s[6..8], 16).ok()?;
        Some(eframe::egui::Color32::from_rgba_unmultiplied(r, g, b, a))
    } else {
        None
    }
}

pub fn color_to_hex(color: eframe::egui::Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

pub fn with_alpha(color: eframe::egui::Color32, alpha: u8) -> eframe::egui::Color32 {
    let a = ((color.a() as u16 * alpha as u16) / 255) as u8;
    eframe::egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}

pub fn contrasting_text_color(bg: eframe::egui::Color32) -> eframe::egui::Color32 {
    // WCAG standard perceived luminance formula: 0.299*R + 0.587*G + 0.114*B
    let luminance = 0.299 * (bg.r() as f32) + 0.587 * (bg.g() as f32) + 0.114 * (bg.b() as f32);
    if luminance > 160.0 {
        eframe::egui::Color32::from_rgb(20, 20, 24) // Dark text for bright backgrounds
    } else {
        eframe::egui::Color32::WHITE // Light text for dark backgrounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(
            parse_hex_color("#FF0000"),
            Some(eframe::egui::Color32::from_rgb(255, 0, 0))
        );
        assert_eq!(
            parse_hex_color("00D7AF"),
            Some(eframe::egui::Color32::from_rgb(0, 215, 175))
        );
        assert_eq!(
            parse_hex_color("#12345680"),
            Some(eframe::egui::Color32::from_rgba_unmultiplied(0x12, 0x34, 0x56, 0x80))
        );
        assert_eq!(parse_hex_color("invalid"), None);
        assert_eq!(parse_hex_color("#12345"), None);
    }

    #[test]
    fn test_color_to_hex() {
        let c = eframe::egui::Color32::from_rgb(0, 215, 175);
        assert_eq!(color_to_hex(c), "#00D7AF");
    }

    #[test]
    fn test_contrasting_text_color() {
        let white = eframe::egui::Color32::WHITE;
        let black = eframe::egui::Color32::BLACK;
        let yellow = eframe::egui::Color32::from_rgb(255, 255, 0);
        let dark_blue = eframe::egui::Color32::from_rgb(20, 30, 50);

        assert_eq!(contrasting_text_color(white), eframe::egui::Color32::from_rgb(20, 20, 24));
        assert_eq!(contrasting_text_color(yellow), eframe::egui::Color32::from_rgb(20, 20, 24));
        assert_eq!(contrasting_text_color(black), eframe::egui::Color32::WHITE);
        assert_eq!(contrasting_text_color(dark_blue), eframe::egui::Color32::WHITE);
    }
}
