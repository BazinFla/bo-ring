//! Centralized constants for physical mouse button event codes.
//!
//! Standard Linux evdev mouse codes:
//! - BTN_LEFT: 272
//! - BTN_RIGHT: 273
//! - BTN_MIDDLE: 274
//! - BTN_SIDE / BTN_BACK: 275
//! - BTN_EXTRA / BTN_FORWARD: 276
//! - BTN_TASK / BTN_GESTURES: 277
//! - BTN_RING (Thumb rest): 278

pub const BTN_LEFT_CODE: u16 = 272;
pub const BTN_RIGHT_CODE: u16 = 273;
pub const BTN_MIDDLE_CODE: u16 = 274;
pub const BTN_BACK_CODE: u16 = 275;
pub const BTN_FORWARD_CODE: u16 = 276;
pub const BTN_GESTURES_CODE: u16 = 277;
pub const BTN_RING_CODE: u16 = 278;
pub const BTN_SMARTSHIFT_CODE: u16 = 280;
pub const BTN_HAPTIC_CODE: u16 = 281;

/// Bluetooth alternative scancodes mapped to BTN_BACK (275)
pub const BTN_BACK_BT_ALIASES: &[u16] = &[158];

/// Bluetooth alternative scancodes mapped to BTN_FORWARD (276)
pub const BTN_FORWARD_BT_ALIASES: &[u16] = &[159];

/// Bluetooth alternative scancodes mapped to BTN_GESTURES (277)
pub const BTN_GESTURES_BT_ALIASES: &[u16] = &[120, 148, 171, 204, 256, 279];

/// Bluetooth alternative scancodes mapped to BTN_RING (278)
pub const BTN_RING_BT_ALIASES: &[u16] = &[149, 172, 257, 999];
