use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, EventType, InputEvent, Key, RelativeAxisType};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

static VIRTUAL_KEYBOARD: Mutex<Option<VirtualDevice>> = Mutex::new(None);

use std::sync::atomic::{AtomicBool, Ordering};

static IS_AZERTY_LAYOUT: AtomicBool = AtomicBool::new(false);
static LAYOUT_CHECKED: AtomicBool = AtomicBool::new(false);

pub fn init_layout_detection() {
    if LAYOUT_CHECKED.load(Ordering::Relaxed) {
        return;
    }
    thread::spawn(|| {
        let mut azerty = false;
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.input-sources", "sources"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if stdout.contains("'fr'") || stdout.contains("'be'") || stdout.contains("azerty") {
                azerty = true;
            }
        }
        if !azerty {
            if let Ok(lang) = std::env::var("LANG") {
                if lang.to_lowercase().starts_with("fr_") {
                    azerty = true;
                }
            }
        }
        IS_AZERTY_LAYOUT.store(azerty, Ordering::Relaxed);
        LAYOUT_CHECKED.store(true, Ordering::Relaxed);
    });
}

pub fn is_azerty() -> bool {
    if !LAYOUT_CHECKED.load(Ordering::Relaxed) {
        if let Ok(lang) = std::env::var("LANG") {
            if lang.to_lowercase().starts_with("fr_") {
                return true;
            }
        }
    }
    IS_AZERTY_LAYOUT.load(Ordering::Relaxed)
}

pub fn str_to_key(s: &str) -> Option<Key> {
    let normalized = s.trim().to_lowercase();
    match normalized.as_str() {
        // Modifiers
        "ctrl" | "ctrl_l" | "ctrl_r" | "control" => Some(Key::KEY_LEFTCTRL),
        "alt" | "alt_l" | "alt_r" => Some(Key::KEY_LEFTALT),
        "shift" | "shift_l" | "shift_r" => Some(Key::KEY_LEFTSHIFT),
        "super" | "super_l" | "super_r" | "meta" | "win" | "cmd" | "command" => Some(Key::KEY_LEFTMETA),

        // Media keys
        "volumeup" | "volume_up" | "volup" | "xf86audioraisevolume" => Some(Key::KEY_VOLUMEUP),
        "volumedown" | "volume_down" | "voldown" | "xf86audiolowervolume" => Some(Key::KEY_VOLUMEDOWN),
        "mute" | "volumemute" | "audiomute" | "xf86audiomute" => Some(Key::KEY_MUTE),
        "playpause" | "play" | "pause" | "audioplay" | "xf86audioplay" => Some(Key::KEY_PLAYPAUSE),
        "nextsong" | "next" | "audionext" | "xf86audionext" => Some(Key::KEY_NEXTSONG),
        "previoussong" | "prev" | "audioprev" | "xf86audioprev" => Some(Key::KEY_PREVIOUSSONG),

        // Mouse buttons & navigation
        "btn_back" | "btn_side" | "back" => Some(Key::BTN_BACK),
        "btn_forward" | "btn_extra" | "forward" => Some(Key::BTN_FORWARD),
        "btn_left" => Some(Key::BTN_LEFT),
        "btn_right" => Some(Key::BTN_RIGHT),
        "btn_middle" => Some(Key::BTN_MIDDLE),

        // Special & navigation keys
        "space" => Some(Key::KEY_SPACE),
        "enter" | "return" => Some(Key::KEY_ENTER),
        "tab" => Some(Key::KEY_TAB),
        "escape" | "esc" => Some(Key::KEY_ESC),
        "backspace" => Some(Key::KEY_BACKSPACE),
        "delete" | "del" => Some(Key::KEY_DELETE),
        "print" | "printscreen" | "sysrq" | "prtscr" => Some(Key::KEY_SYSRQ),
        "left" | "arrowleft" => Some(Key::KEY_LEFT),
        "right" | "arrowright" => Some(Key::KEY_RIGHT),
        "up" | "arrowup" => Some(Key::KEY_UP),
        "down" | "arrowdown" => Some(Key::KEY_DOWN),
        "home" => Some(Key::KEY_HOME),
        "end" => Some(Key::KEY_END),
        "pageup" => Some(Key::KEY_PAGEUP),
        "pagedown" => Some(Key::KEY_PAGEDOWN),

        // Letters A-Z (handling QWERTY vs AZERTY scancode mapping)
        "a" => Some(if is_azerty() { Key::KEY_Q } else { Key::KEY_A }),
        "b" => Some(Key::KEY_B),
        "c" => Some(Key::KEY_C),
        "d" => Some(Key::KEY_D),
        "e" => Some(Key::KEY_E),
        "f" => Some(Key::KEY_F),
        "g" => Some(Key::KEY_G),
        "h" => Some(Key::KEY_H),
        "i" => Some(Key::KEY_I),
        "j" => Some(Key::KEY_J),
        "k" => Some(Key::KEY_K),
        "l" => Some(Key::KEY_L),
        "m" => Some(if is_azerty() { Key::KEY_SEMICOLON } else { Key::KEY_M }),
        "n" => Some(Key::KEY_N),
        "o" => Some(Key::KEY_O),
        "p" => Some(Key::KEY_P),
        "q" => Some(if is_azerty() { Key::KEY_A } else { Key::KEY_Q }),
        "r" => Some(Key::KEY_R),
        "s" => Some(Key::KEY_S),
        "t" => Some(Key::KEY_T),
        "u" => Some(Key::KEY_U),
        "v" => Some(Key::KEY_V),
        "w" => Some(if is_azerty() { Key::KEY_Z } else { Key::KEY_W }),
        "x" => Some(Key::KEY_X),
        "y" => Some(Key::KEY_Y),
        "z" => Some(if is_azerty() { Key::KEY_W } else { Key::KEY_Z }),

        // Numbers 0-9
        "0" => Some(Key::KEY_0),
        "1" => Some(Key::KEY_1),
        "2" => Some(Key::KEY_2),
        "3" => Some(Key::KEY_3),
        "4" => Some(Key::KEY_4),
        "5" => Some(Key::KEY_5),
        "6" => Some(Key::KEY_6),
        "7" => Some(Key::KEY_7),
        "8" => Some(Key::KEY_8),
        "9" => Some(Key::KEY_9),

        // F1-F12 keys
        "f1" => Some(Key::KEY_F1),
        "f2" => Some(Key::KEY_F2),
        "f3" => Some(Key::KEY_F3),
        "f4" => Some(Key::KEY_F4),
        "f5" => Some(Key::KEY_F5),
        "f6" => Some(Key::KEY_F6),
        "f7" => Some(Key::KEY_F7),
        "f8" => Some(Key::KEY_F8),
        "f9" => Some(Key::KEY_F9),
        "f10" => Some(Key::KEY_F10),
        "f11" => Some(Key::KEY_F11),
        "f12" => Some(Key::KEY_F12),

        _ => None,
    }
}

fn all_supported_keys() -> AttributeSet<Key> {
    let mut keys = AttributeSet::<Key>::new();
    let candidates = [
        Key::KEY_LEFTCTRL, Key::KEY_LEFTALT, Key::KEY_LEFTSHIFT, Key::KEY_LEFTMETA,
        Key::KEY_VOLUMEUP, Key::KEY_VOLUMEDOWN, Key::KEY_MUTE, Key::KEY_PLAYPAUSE, Key::KEY_NEXTSONG, Key::KEY_PREVIOUSSONG,
        Key::KEY_SPACE, Key::KEY_ENTER, Key::KEY_TAB, Key::KEY_ESC, Key::KEY_BACKSPACE, Key::KEY_DELETE,
        Key::KEY_LEFT, Key::KEY_RIGHT, Key::KEY_UP, Key::KEY_DOWN, Key::KEY_HOME, Key::KEY_END, Key::KEY_PAGEUP, Key::KEY_PAGEDOWN, Key::KEY_SYSRQ,
        Key::KEY_A, Key::KEY_B, Key::KEY_C, Key::KEY_D, Key::KEY_E, Key::KEY_F, Key::KEY_G, Key::KEY_H, Key::KEY_I, Key::KEY_J,
        Key::KEY_K, Key::KEY_L, Key::KEY_M, Key::KEY_N, Key::KEY_O, Key::KEY_P, Key::KEY_Q, Key::KEY_R, Key::KEY_S, Key::KEY_T,
        Key::KEY_U, Key::KEY_V, Key::KEY_W, Key::KEY_X, Key::KEY_Y, Key::KEY_Z,
        Key::KEY_0, Key::KEY_1, Key::KEY_2, Key::KEY_3, Key::KEY_4, Key::KEY_5, Key::KEY_6, Key::KEY_7, Key::KEY_8, Key::KEY_9,
        Key::KEY_F1, Key::KEY_F2, Key::KEY_F3, Key::KEY_F4, Key::KEY_F5, Key::KEY_F6, Key::KEY_F7, Key::KEY_F8, Key::KEY_F9, Key::KEY_F10, Key::KEY_F11, Key::KEY_F12,
        Key::KEY_SEMICOLON,
        // Physical mouse buttons
        Key::BTN_LEFT, Key::BTN_RIGHT, Key::BTN_MIDDLE, Key::BTN_SIDE, Key::BTN_EXTRA, Key::BTN_FORWARD, Key::BTN_BACK, Key::BTN_TASK,
        Key::BTN_0, Key::BTN_1, Key::BTN_2, Key::BTN_3, Key::BTN_4, Key::BTN_5, Key::BTN_6, Key::BTN_7, Key::BTN_8, Key::BTN_9,
    ];

    for k in candidates {
        keys.insert(k);
    }
    keys
}

fn create_virtual_keyboard() -> std::io::Result<VirtualDevice> {
    let keys = all_supported_keys();
    let mut rel_axes = AttributeSet::<RelativeAxisType>::new();
    rel_axes.insert(RelativeAxisType::REL_X);
    rel_axes.insert(RelativeAxisType::REL_Y);
    rel_axes.insert(RelativeAxisType::REL_WHEEL);
    rel_axes.insert(RelativeAxisType::REL_HWHEEL);
    rel_axes.insert(RelativeAxisType::REL_WHEEL_HI_RES);
    rel_axes.insert(RelativeAxisType::REL_HWHEEL_HI_RES);

    VirtualDeviceBuilder::new()?
        .name("Bo-Ring Virtual Input Device")
        .with_keys(&keys)?
        .with_relative_axes(&rel_axes)?
        .build()
}

pub fn reset_virtual_keyboard() {
    let mut guard = VIRTUAL_KEYBOARD.lock().unwrap();
    *guard = None;
}

pub fn passthrough_event(ev: &InputEvent) {
    let mut guard = VIRTUAL_KEYBOARD.lock().unwrap();
    if guard.is_none() {
        if let Ok(dev) = create_virtual_keyboard() {
            *guard = Some(dev);
        }
    }
    if let Some(ref mut dev) = *guard {
        let events = if ev.event_type() != EventType::SYNCHRONIZATION {
            vec![*ev, InputEvent::new(EventType::SYNCHRONIZATION, 0, 0)]
        } else {
            vec![*ev]
        };
        if let Err(e) = dev.emit(&events) {
            eprintln!("⚠️ Virtual uinput device error during passthrough: {}", e);
        }
    }
}

pub fn init_virtual_keyboard() {
    let mut guard = VIRTUAL_KEYBOARD.lock().unwrap();
    if guard.is_none() {
        if let Ok(dev) = create_virtual_keyboard() {
            *guard = Some(dev);
        }
    }
}

pub fn send_key_combo(keys: &[String]) {
    if keys.is_empty() {
        return;
    }

    let mut key_codes = Vec::new();
    for k_str in keys {
        if let Some(key) = str_to_key(k_str) {
            key_codes.push(key);
        } else {
            eprintln!("⚠️ Unrecognized key in shortcut: '{}'", k_str);
        }
    }

    if key_codes.is_empty() {
        return;
    }

    println!("⌨️ Simulating keyboard shortcut via uinput: {:?}", keys);
    init_virtual_keyboard();

    let mut events = Vec::new();

    // 1. Press all keys
    for &key in &key_codes {
        events.push(InputEvent::new(EventType::KEY, key.code(), 1));
    }
    events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));

    {
        let mut guard = VIRTUAL_KEYBOARD.lock().unwrap();
        if let Some(ref mut dev) = *guard {
            if let Err(e) = dev.emit(&events) {
                eprintln!("⚠️ Error emitting key press events: {}", e);
            }
        }
    }

    // 2. Hold key combo briefly (30 ms) WITHOUT holding the mutex lock
    thread::sleep(Duration::from_millis(30));

    // 3. Release all keys in reverse order
    let mut release_events = Vec::new();
    for &key in key_codes.iter().rev() {
        release_events.push(InputEvent::new(EventType::KEY, key.code(), 0));
    }
    release_events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));

    {
        let mut guard = VIRTUAL_KEYBOARD.lock().unwrap();
        if let Some(ref mut dev) = *guard {
            if let Err(e) = dev.emit(&release_events) {
                eprintln!("⚠️ Error emitting key release events: {}", e);
            }
        }
    }
}

/// Inject a tiny mouse movement (1px via REL_X) through the uinput virtual device.
///
/// This is a workaround for Wayland compositors (GNOME/Mutter, KDE/KWin, wlroots/Sway,
/// Hyprland) where a newly mapped window receives a `wl_pointer.enter` event with stale
/// coordinates. The injected movement generates a real `wl_pointer.motion` event, which
/// causes the compositor to report the accurate cursor position to the application.
///
/// The 1px offset is imperceptible to the user. On X11, this is harmless — the pointer
/// position is already accurate, and the 1px movement has no visible effect.
///
/// Called by the daemon (main.rs) ~300ms after spawning the ring menu process,
/// giving the window time to be fully mapped by the compositor.
pub fn nudge_mouse() {
    let mut guard = VIRTUAL_KEYBOARD.lock().unwrap();
    if guard.is_none() {
        if let Ok(dev) = create_virtual_keyboard() {
            *guard = Some(dev);
        }
    }
    if let Some(ref mut dev) = *guard {
        let step1 = [
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_X.0, 1),
            InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
        ];
        let step2 = [
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_X.0, -1),
            InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
        ];

        let _ = dev.emit(&step1);
        thread::sleep(Duration::from_millis(10));
        let _ = dev.emit(&step2);
        println!("🐭 Mouse nudge injected (net zero)");
    } else {
        eprintln!("⚠️ No virtual device for mouse nudge");
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_to_key_mapping() {
        assert_eq!(str_to_key("CTRL"), Some(Key::KEY_LEFTCTRL));
        assert_eq!(str_to_key("Alt"), Some(Key::KEY_LEFTALT));
        assert_eq!(str_to_key("Shift"), Some(Key::KEY_LEFTSHIFT));
        assert_eq!(str_to_key("Super"), Some(Key::KEY_LEFTMETA));
        assert_eq!(str_to_key("c"), Some(Key::KEY_C));
        assert_eq!(str_to_key("C"), Some(Key::KEY_C));
        assert_eq!(str_to_key("VolumeUp"), Some(Key::KEY_VOLUMEUP));
        assert_eq!(str_to_key("Left"), Some(Key::KEY_LEFT));
        assert_eq!(str_to_key("UnknownKey123"), None);
    }
}
