pub mod codes;
pub mod hidpp;
pub mod model;
pub mod registry;

pub use codes::*;
pub use hidpp::*;
pub use model::*;
pub use registry::*;

use eframe::egui;
use evdev::enumerate;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use crate::i18n::tr;

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionType {
    UsbReceiver,
    Bluetooth,
    UsbCable,
}

#[derive(Clone, Debug)]
pub struct MouseDeviceInfo {
    pub name: String,
    pub connection_type: ConnectionType,
    pub battery_level: u8,
    pub is_charging: bool,
    pub channel_count: usize,
    pub detected_profile_id: Option<String>,
    pub vid_pid: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MouseInputEvent {
    pub code: u16,
    pub raw_code: u16,
    pub dev_name: Option<String>,
    pub vid_pid: Option<String>,
}

static LAST_KNOWN_BATTERY_LEVEL: AtomicU8 = AtomicU8::new(60);

pub fn check_usb_direct_mouse_cable_connected() -> bool {
    if let Ok(entries) = std::fs::read_dir("/sys/bus/usb/devices") {
        for entry in entries.flatten() {
            let path = entry.path();
            let vendor_path = path.join("idVendor");
            let product_path = path.join("product");
            if vendor_path.exists() {
                if let Ok(vendor) = std::fs::read_to_string(&vendor_path) {
                    if vendor.trim().to_lowercase() == "046d" {
                        let product = std::fs::read_to_string(&product_path)
                            .unwrap_or_default()
                            .to_lowercase();
                        if (product.contains("mouse") || product.contains("mx master"))
                            && !product.contains("receiver")
                            && !product.contains("keyboard")
                        {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn query_hidpp_battery_status() -> Option<(u8, bool)> {
    // Queries HID++ battery on /dev/hidraw*
    let mut hidpp_level = None;
    let mut hidpp_charging = false;

    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("hidraw") {
                let dev_path = entry.path();

                // 1. Verify via sysfs that this hidraw node belongs to a Logitech device (046d) and is not a keyboard
                let uevent_path = std::path::PathBuf::from("/sys/class/hidraw").join(&name).join("device/uevent");
                if let Ok(uevent) = std::fs::read_to_string(&uevent_path) {
                    let uevent_low = uevent.to_lowercase();
                    if !uevent_low.contains("v0000046d") || uevent_low.contains("keyboard") {
                        continue;
                    }
                }

                // 2. Open with O_NONBLOCK (2048) so read never blocks the thread
                use std::os::unix::fs::OpenOptionsExt;
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .custom_flags(2048) // libc::O_NONBLOCK
                    .open(&dev_path)
                {
                    use std::io::{Read, Write};

                    // Short HID++ 1.0/2.0 battery query report
                    let mut req = [0u8; 7];
                    req[0] = 0x10; // Report ID (Short report)
                    req[1] = 0xff; // Device Index (0xFF = Receiver / default)
                    req[2] = 0x0d; // Battery Feature ID / SubID
                    req[3] = 0x00;

                    if file.write_all(&req).is_ok() {
                        std::thread::sleep(std::time::Duration::from_millis(15));
                        let mut resp = [0u8; 20];
                        if let Ok(read_res) = file.read(&mut resp) {
                            if read_res >= 5 && resp[0] == 0x10 {
                                let level = resp[4];
                                if level > 0 && level <= 100 {
                                    hidpp_level = Some(level);
                                    let status_byte = resp[5];
                                    if status_byte == 1 || status_byte == 3 || status_byte == 4 {
                                        hidpp_charging = true;
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }


    if let Some(level) = hidpp_level {
        Some((level, hidpp_charging))
    } else {
        None
    }
}

pub fn scan_battery_level() -> (u8, bool) {
    let mut detected_capacity = None;
    let mut detected_charging = false;

    // 1. Try sysfs detection /sys/class/power_supply (e.g. Bluetooth)
    if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
        for entry in entries.flatten() {
            let path = entry.path();
            let name_lower = entry.file_name().to_string_lossy().to_lowercase();
            if name_lower.contains("keyboard") {
                continue;
            }

            let cap_path = path.join("capacity");
            if cap_path.exists() {
                if let Ok(cap_str) = std::fs::read_to_string(&cap_path) {
                    if let Ok(val) = cap_str.trim().parse::<u8>() {
                        if detected_capacity.is_none() {
                            detected_capacity = Some(val);
                            LAST_KNOWN_BATTERY_LEVEL.store(val, Ordering::Relaxed);
                        }
                    }
                }
            }

            let status_path = path.join("status");
            if status_path.exists() {
                if let Ok(s) = std::fs::read_to_string(&status_path) {
                    let s_low = s.trim().to_lowercase();
                    if s_low == "charging" || s_low == "full" {
                        detected_charging = true;
                    }
                }
            }
        }
    }

    // 2. If sysfs did not detect charge (USB Receiver), query HID++ on /dev/hidraw*
    if detected_capacity.is_none() || !detected_charging {
        if let Some((hidpp_level, hidpp_charging)) = query_hidpp_battery_status() {
            if detected_capacity.is_none() {
                detected_capacity = Some(hidpp_level);
                LAST_KNOWN_BATTERY_LEVEL.store(hidpp_level, Ordering::Relaxed);
            }
            if hidpp_charging {
                detected_charging = true;
            }
        }
    }

    let is_usb_cable = check_usb_direct_mouse_cable_connected();
    let is_charging = detected_charging || is_usb_cable;
    let final_battery = detected_capacity.unwrap_or_else(|| LAST_KNOWN_BATTERY_LEVEL.load(Ordering::Relaxed));

    (final_battery, is_charging)
}

pub fn enumerate_input_device_names() -> Vec<(PathBuf, String, Option<String>, bool)> {
    let mut devices = Vec::new();

    // 1. Try sysfs /sys/class/input/event*/device/name (immune to EBUSY / EVIOCGRAB locks from background daemon)
    if let Ok(entries) = std::fs::read_dir("/sys/class/input") {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with("event") {
                let name_path = entry.path().join("device/name");
                if let Ok(name) = std::fs::read_to_string(&name_path) {
                    let dev_path = PathBuf::from("/dev/input").join(&file_name);
                    let vendor_path = entry.path().join("device/id/vendor");
                    let product_path = entry.path().join("device/id/product");
                    let vid_pid = if let (Ok(v), Ok(p)) = (std::fs::read_to_string(&vendor_path), std::fs::read_to_string(&product_path)) {
                        let v_str = v.trim().to_lowercase();
                        let p_str = p.trim().to_lowercase();
                        if !v_str.is_empty() && !p_str.is_empty() {
                            Some(format!("{}:{}", v_str, p_str))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Detect mouse via evdev capabilities when possible
                    let is_mouse = if let Ok(device) = evdev::Device::open(&dev_path) {
                        let has_btn_left = device.supported_keys()
                            .map_or(false, |keys| keys.contains(evdev::Key::BTN_LEFT));
                        let has_letter_keys = device.supported_keys()
                            .map_or(false, |keys| keys.contains(evdev::Key::KEY_A));
                        has_btn_left && !has_letter_keys
                    } else {
                        // Device busy (grabbed by daemon) — fallback to name heuristic
                        let name_lower = name.trim().to_lowercase();
                        !name_lower.contains("keyboard")
                            && (name_lower.contains("mouse")
                                || name_lower.contains("pointer")
                                || name_lower.contains("touchpad"))
                    };

                    devices.push((dev_path, name.trim().to_string(), vid_pid, is_mouse));
                }
            }
        }
    }

    // 2. Fallback to evdev::enumerate() if sysfs was empty
    if devices.is_empty() {
        for (path, device) in enumerate() {
            let name = device.name().unwrap_or("Unknown device").to_string();
            let input_id = device.input_id();
            let vid_pid = Some(format!("{:04x}:{:04x}", input_id.vendor(), input_id.product()));
            let has_btn_left = device.supported_keys()
                .map_or(false, |keys| keys.contains(evdev::Key::BTN_LEFT));
            let has_letter_keys = device.supported_keys()
                .map_or(false, |keys| keys.contains(evdev::Key::KEY_A));
            let is_mouse = has_btn_left && !has_letter_keys;
            devices.push((path, name, vid_pid, is_mouse));
        }
    }

    devices
}

pub fn scan_mouse_device() -> Option<MouseDeviceInfo> {
    let mut found_name = String::new();
    let mut found_vid_pid: Option<String> = None;
    let mut has_bluetooth_mouse = false;
    let mut has_receiver_mouse = false;
    let mut count = 0;

    let devices = enumerate_input_device_names();

    for (_path, name, vid_pid, is_mouse) in devices {
        let name_lower = name.to_lowercase();

        // Skip our own virtual device and keyboards
        if name_lower.contains("keyboard") || name_lower.contains("bo-ring virtual") {
            continue;
        }

        // Use capability-based detection instead of brand-specific name matching
        if !is_mouse {
            continue;
        }

        count += 1;

        // Determine connection type from device name hints
        if name_lower.contains("receiver") || name_lower.contains("bolt") || name_lower.contains("unifying") || name_lower.contains("dongle") {
            has_receiver_mouse = true;
        } else if name_lower.contains("bluetooth") || name_lower.contains(" bt") {
            has_bluetooth_mouse = true;
        } else {
            // Default assumption for non-receiver devices
            has_bluetooth_mouse = true;
        }

        // Keep the most specific name found (prefer non-generic names)
        if found_name.is_empty() || (!name_lower.contains("receiver") && !name_lower.contains("dongle")) {
            found_name = name.clone();
        }
        if found_vid_pid.is_none() && vid_pid.is_some() {
            found_vid_pid = vid_pid.clone();
        }
    }

    if count > 0 {
        let (battery_level, is_charging) = scan_battery_level();
        let connection_type = if has_bluetooth_mouse {
            ConnectionType::Bluetooth
        } else if check_usb_direct_mouse_cable_connected() {
            ConnectionType::UsbCable
        } else if has_receiver_mouse {
            ConnectionType::UsbReceiver
        } else {
            ConnectionType::Bluetooth
        };

        // Let DeviceRegistry handle profile matching — no hardcoded fallback name
        let detected_profile_id = DeviceRegistry::new()
            .match_device(&found_name, found_vid_pid.as_deref())
            .map(|p| p.config.id.clone());

        Some(MouseDeviceInfo {
            name: found_name,
            connection_type,
            battery_level,
            is_charging,
            channel_count: count,
            detected_profile_id,
            vid_pid: found_vid_pid,
        })
    } else {
        None
    }
}

pub fn attach_new_mouse_listeners(
    ctx: egui::Context,
    tx: mpsc::Sender<MouseInputEvent>,
    listened_paths: Arc<Mutex<HashSet<PathBuf>>>,
) {
    let sock_path = crate::platform::ipc::daemon_socket_path();
    let tx_ipc = tx.clone();
    let ctx_ipc = ctx.clone();

    // 1. Attempt Unix Socket connection to background daemon
    thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        use std::os::unix::net::UnixStream;
        use crate::platform::ipc::DaemonIpcEvent;

        if let Ok(stream) = UnixStream::connect(&sock_path) {
            println!("📡 Config UI connected to daemon IPC socket at {:?}", sock_path);
            let reader = BufReader::new(stream);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(event) = DaemonIpcEvent::parse_line(&line) {
                    let (code_opt, raw_opt, vid_pid, dev_name) = match event {
                        DaemonIpcEvent::ButtonPressed { raw_code, normalized_code, vid_pid, dev_name } => {
                            (Some(normalized_code), Some(raw_code), vid_pid, dev_name)
                        }
                        DaemonIpcEvent::ButtonCode(code) => (Some(code), Some(code), None, None),
                        _ => (None, None, None, None),
                    };
                    if let (Some(code), Some(raw_code)) = (code_opt, raw_opt) {
                        if is_supported_button_code(raw_code) || is_supported_button_code(code) {
                            println!("⚡ Event captured via IPC Socket: Button {} (from {:?}, {:?})", code, dev_name, vid_pid);
                            let _ = tx_ipc.send(MouseInputEvent {
                                code,
                                raw_code,
                                dev_name,
                                vid_pid,
                            });
                            ctx_ipc.request_repaint();
                        }
                    }
                }
            }
        }
    });

    // 2. Direct evdev listening (fallback when daemon is not running)
    let tx_evdev = tx.clone();
    let ctx_evdev = ctx.clone();
    thread::spawn(move || {
        for (path, device) in enumerate() {
            let name = device.name().unwrap_or("Unknown device");
            let name_lower = name.to_lowercase();

            // Ignore keyboards
            if name_lower.contains("keyboard") {
                continue;
            }

            if name_lower.contains("logitech") || name_lower.contains("mouse") || name_lower.contains("receiver") || name_lower.contains("mx master") {
                let mut paths_lock = listened_paths.lock().unwrap();
                if !paths_lock.contains(&path) {
                    paths_lock.insert(path.clone());
                    drop(paths_lock);

                    let tx = tx_evdev.clone();
                    let ctx = ctx_evdev.clone();
                    let dev_name = name.to_string();
                    let dev_path = path.clone();
                    let listened_paths_worker = Arc::clone(&listened_paths);
                    let dev_path_worker = path.clone();

                    println!("🔍 Hot-Plug Listener: Attaching receiver to [{:?}] {}", dev_path, dev_name);

                    thread::spawn(move || {
                        if let Ok(mut device) = evdev::Device::open(&dev_path) {
                            let input_id = device.input_id();
                            let dev_vid_pid = format!("{:04x}:{:04x}", input_id.vendor(), input_id.product());
                            println!("✅ Hot-Plug Listener: Successfully opened {:?} [{}] ({})", dev_path, dev_vid_pid, dev_name);
                            loop {
                                if let Ok(events) = device.fetch_events() {
                                    for ev in events {
                                        if ev.value() == 1 && ev.event_type().0 != 0 {
                                            let raw_code = ev.code();
                                            if is_supported_button_code(raw_code) {
                                                let code = normalize_button_code(raw_code);
                                                println!("⚡ Event captured ({}) : Raw code {} -> Normalized {}", dev_name, raw_code, code);
                                                let _ = tx.send(MouseInputEvent {
                                                    code,
                                                    raw_code,
                                                    dev_name: Some(dev_name.clone()),
                                                    vid_pid: Some(dev_vid_pid.clone()),
                                                });
                                                ctx.request_repaint();
                                            }
                                        }
                                    }
                                } else {
                                    println!("🔌 Disconnected/Closed evdev channel {:?}", dev_path);
                                    break;
                                }
                            }
                        }

                        // Cleanup: free path for future reconnections
                        let mut lock = listened_paths_worker.lock().unwrap();
                        lock.remove(&dev_path_worker);
                    });
                }
            }
        }
    });

    // 3. Direct HID++ listening on /dev/hidraw* (detects diverted buttons when daemon is not running)
    let diverted_hidpp_paths = Arc::new(Mutex::new(HashSet::new()));
    hidpp::attach_hidpp_gui_listener(ctx, tx, diverted_hidpp_paths);
}

pub fn normalize_button_code(code: u16) -> u16 {
    if code == BTN_MIDDLE_CODE {
        BTN_MIDDLE_CODE
    } else if code == BTN_FORWARD_CODE || BTN_FORWARD_BT_ALIASES.contains(&code) {
        BTN_FORWARD_CODE
    } else if code == BTN_BACK_CODE || BTN_BACK_BT_ALIASES.contains(&code) {
        BTN_BACK_CODE
    } else if code == BTN_GESTURES_CODE || BTN_GESTURES_BT_ALIASES.contains(&code) {
        BTN_GESTURES_CODE
    } else if code == BTN_RING_CODE || BTN_RING_BT_ALIASES.contains(&code) {
        BTN_RING_CODE
    } else if code == BTN_SMARTSHIFT_CODE {
        BTN_SMARTSHIFT_CODE
    } else if code == BTN_HAPTIC_CODE {
        BTN_HAPTIC_CODE
    } else {
        code
    }
}

pub fn is_supported_button_code(code: u16) -> bool {
    // Primary left and right clicks must NEVER be remapped or captured
    if code == BTN_LEFT_CODE || code == BTN_RIGHT_CODE {
        return false;
    }
    let norm = normalize_button_code(code);
    matches!(norm, BTN_RING_CODE | BTN_GESTURES_CODE | BTN_FORWARD_CODE | BTN_BACK_CODE | BTN_MIDDLE_CODE | BTN_SMARTSHIFT_CODE | BTN_HAPTIC_CODE)
        || (256..=350).contains(&code)
        || (704..=745).contains(&code)
        || BTN_BACK_BT_ALIASES.contains(&code)
        || BTN_FORWARD_BT_ALIASES.contains(&code)
        || BTN_GESTURES_BT_ALIASES.contains(&code)
        || BTN_RING_BT_ALIASES.contains(&code)
}

pub fn get_button_name_lang(code: u16, lang: &str) -> String {
    let norm = normalize_button_code(code);
    match norm {
        BTN_RING_CODE => tr(lang, "btn_thumb_ring").to_string(),
        BTN_GESTURES_CODE => tr(lang, "btn_gestures").to_string(),
        BTN_FORWARD_CODE => tr(lang, "btn_next").to_string(),
        BTN_BACK_CODE => tr(lang, "btn_back").to_string(),
        BTN_MIDDLE_CODE => tr(lang, "btn_middle_wheel").to_string(),
        BTN_SMARTSHIFT_CODE => tr(lang, "btn_smartshift").to_string(),
        BTN_HAPTIC_CODE => tr(lang, "btn_haptic").to_string(),
        _ => format!("{} {}", tr(lang, "button_code_prefix"), code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_button_code() {
        assert_eq!(normalize_button_code(BTN_RING_CODE), BTN_RING_CODE);
        assert_eq!(normalize_button_code(172), BTN_RING_CODE);
        assert_eq!(normalize_button_code(149), BTN_RING_CODE);
        assert_eq!(normalize_button_code(257), BTN_RING_CODE);
        assert_eq!(normalize_button_code(999), BTN_RING_CODE);

        assert_eq!(normalize_button_code(BTN_GESTURES_CODE), BTN_GESTURES_CODE);
        assert_eq!(normalize_button_code(120), BTN_GESTURES_CODE);
        assert_eq!(normalize_button_code(171), BTN_GESTURES_CODE);
        assert_eq!(normalize_button_code(204), BTN_GESTURES_CODE);

        assert_eq!(normalize_button_code(BTN_FORWARD_CODE), BTN_FORWARD_CODE);
        assert_eq!(normalize_button_code(159), BTN_FORWARD_CODE);

        assert_eq!(normalize_button_code(BTN_BACK_CODE), BTN_BACK_CODE);
        assert_eq!(normalize_button_code(158), BTN_BACK_CODE);

        assert_eq!(normalize_button_code(BTN_MIDDLE_CODE), BTN_MIDDLE_CODE);
        assert_eq!(normalize_button_code(9999), 9999);
    }

    #[test]
    fn test_is_supported_button_code() {
        assert!(!is_supported_button_code(BTN_LEFT_CODE));
        assert!(!is_supported_button_code(BTN_RIGHT_CODE));
        assert!(is_supported_button_code(BTN_RING_CODE));
        assert!(is_supported_button_code(BTN_GESTURES_CODE));
        assert!(is_supported_button_code(BTN_FORWARD_CODE));
        assert!(is_supported_button_code(BTN_BACK_CODE));
        assert!(is_supported_button_code(BTN_MIDDLE_CODE));
        assert!(is_supported_button_code(172)); // BT alias for ring
        assert!(is_supported_button_code(704)); // BTN_TRIGGER_HAPPY
        assert!(!is_supported_button_code(100)); // KEY_RIGHTALT should not be matched
    }
}
