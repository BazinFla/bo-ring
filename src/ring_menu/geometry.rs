use eframe::egui;
use std::f32::consts::PI;

/// Determine targeted slot index based on angular sector from center point.
pub fn get_sector_slot_index(pointer_pos: egui::Pos2, center: egui::Pos2, total_slots: usize) -> Option<usize> {
    if total_slots == 0 {
        return None;
    }
    let dx = pointer_pos.x - center.x;
    let dy = pointer_pos.y - center.y;
    let dist = (dx * dx + dy * dy).sqrt();

    // Center deadzone: 35px
    if dist <= 35.0 {
        return None;
    }

    let angle = dy.atan2(dx);
    let step = 2.0 * PI / total_slots as f32;
    let mut norm_angle = angle + (PI / 2.0) + (step / 2.0);
    while norm_angle < 0.0 {
        norm_angle += 2.0 * PI;
    }
    norm_angle %= 2.0 * PI;

    let slot_idx = (norm_angle / step).floor() as usize % total_slots;
    Some(slot_idx)
}
