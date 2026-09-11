use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub struct TextureCache {
    textures: HashMap<String, egui::TextureHandle>,
}

pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let font_paths = [
        "/usr/share/fonts/gdouros-symbola/Symbola.ttf",
        "/usr/share/fonts/google-noto-emoji-fonts/NotoEmoji-Regular.ttf",
        "/usr/share/fonts/google-noto/NotoSansSymbols2-Regular.ttf",
        "/usr/share/fonts/google-noto-vf/NotoSansSymbols[wght].ttf",
        "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
    ];

    let mut loaded_count = 0;
    for (idx, path) in font_paths.iter().enumerate() {
        if std::path::Path::new(path).exists() {
            if let Ok(font_data) = std::fs::read(path) {
                let font_name = format!("system_emoji_font_{}", idx);
                fonts.font_data.insert(
                    font_name.clone(),
                    egui::FontData::from_owned(font_data),
                );

                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push(font_name.clone());

                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push(font_name);

                loaded_count += 1;
            }
        }
    }

    if loaded_count > 0 {
        ctx.set_fonts(fonts);
        println!("✅ Emoji and symbol fonts loaded ({}) in egui.", loaded_count);
    }
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn get_or_load(&mut self, ctx: &egui::Context, icon_str: &str) -> Option<&egui::TextureHandle> {
        let expanded_path = super::path::resolve_asset_path(icon_str);
        if !expanded_path.is_file() {
            return None;
        }

        let key = expanded_path.to_string_lossy().to_string();

        if self.textures.contains_key(&key) {
            return self.textures.get(&key);
        }

        if let Ok(img) = image::open(&expanded_path) {
            let size = [img.width() as usize, img.height() as usize];
            let rgba = img.to_rgba8();
            let color_img = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            let handle = ctx.load_texture(&key, color_img, Default::default());
            self.textures.insert(key.clone(), handle);
            return self.textures.get(&key);
        }

        None
    }
}

pub fn expand_path(path_str: &str) -> PathBuf {
    super::path::resolve_asset_path(path_str)
}

pub fn is_image_path(icon_str: &str) -> bool {
    let p = expand_path(icon_str);
    p.is_file() && super::path::is_supported_image(&p)
}

pub fn render_icon_or_emoji(
    painter: &egui::Painter,
    ctx: &egui::Context,
    texture_cache: &mut TextureCache,
    center: egui::Pos2,
    radius: f32,
    icon_str: &str,
    font_size: f32,
    text_color: egui::Color32,
) {
    if is_image_path(icon_str) {
        if let Some(texture) = texture_cache.get_or_load(ctx, icon_str) {
            let img_size = radius * 1.35;
            let img_rect = egui::Rect::from_center_size(center, egui::vec2(img_size, img_size));
            painter.image(
                texture.id(),
                img_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            return;
        }
    }

    // Default UTF-8 emoji rendering
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        icon_str,
        egui::FontId::proportional(font_size),
        text_color,
    );
}
