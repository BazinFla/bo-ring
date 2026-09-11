//! Native Logitech HID++ 2.0 implementation for button diversion and capture.
//!
//! Enables Bo-Ring to autonomously divert Logitech special buttons (such as the
//! thumb rest gesture button on MX Master 2S, 3, 4) directly via `/dev/hidraw*`
//! without requiring third-party tools like Solaar or logiops.

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crate::devices::codes::*;

pub const REPORT_ID_SHORT: u8 = 0x10;
pub const REPORT_ID_LONG: u8 = 0x11;

pub const DEV_INDEX_WIRELESS_OR_BT: u8 = 0xFF;

pub const FEATURE_ROOT: u16 = 0x0000;
pub const FEATURE_REPROG_CONTROLS_V4: u16 = 0x1B04;
pub const FEATURE_REPROG_CONTROLS: u16 = 0x1B00;

/// Physical Control IDs (CIDs) on Logitech devices
pub const CID_GESTURE_BUTTON: u16 = 0x00C3; // Thumb rest gesture button (MX Master series)
pub const CID_SMARTSHIFT_BUTTON: u16 = 0x00C4; // Mode-shift / top button behind wheel
pub const CID_APP_SWITCH_BUTTON: u16 = 0x00D0; // Hidden thumb gesture button (M720 Triathlon, M590)
pub const CID_DPI_SWITCH_BUTTON: u16 = 0x00FD; // Top DPI switch button (MX Vertical, Lift)
pub const CID_PRECISION_BUTTON: u16 = 0x00ED; // Trackball precision mode button (MX Ergo)

/// Diversion flags for setCidReporting
/// Bit 0: DIVERTED, Bit 1: DIVERTED_VALID mask -> 0x03
pub const FLAG_DIVERT_ON: u8 = 0x03;
/// Bit 0: 0, Bit 1: DIVERTED_VALID mask -> 0x02
pub const FLAG_DIVERT_OFF: u8 = 0x02;

#[derive(Clone, Debug)]
pub struct LogitechHidDevice {
    pub path: PathBuf,
    pub name: String,
    pub vid_pid: Option<String>,
    pub dev_index: u8,
    pub reprog_feature_index: u8,
}

/// Drains unread incoming bytes in the non-blocking hidraw buffer.
pub fn flush_hidraw(file: &mut File) {
    let mut buf = [0u8; 64];
    while file.read(&mut buf).is_ok() {}
}

/// Discovers connected Logitech mice that support reprogrammable controls (0x1B04 or 0x1B00),
/// skipping any device paths that are already actively monitored.
pub fn discover_logitech_mice(ignored_paths: &HashSet<PathBuf>) -> Vec<LogitechHidDevice> {
    let mut devices = Vec::new();

    let entries = match std::fs::read_dir("/dev") {
        Ok(e) => e,
        Err(_) => return devices,
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.starts_with("hidraw") {
            continue;
        }

        let dev_path = entry.path();
        if ignored_paths.contains(&dev_path) {
            continue;
        }

        let uevent_path = PathBuf::from("/sys/class/hidraw")
            .join(&file_name)
            .join("device/uevent");

        let uevent = match std::fs::read_to_string(&uevent_path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let uevent_low = uevent.to_lowercase();
        // Check Logitech vendor ID 046d and exclude keyboards
        if !uevent_low.contains("v0000046d") || uevent_low.contains("keyboard") {
            continue;
        }

        // Extract device name from uevent HID_NAME
        let name = uevent
            .lines()
            .find(|line| line.starts_with("HID_NAME="))
            .map(|line| line.trim_start_matches("HID_NAME=").to_string())
            .unwrap_or_else(|| "Logitech Device".to_string());

        // Extract VID:PID
        let vid_pid = uevent.lines().find(|line| line.starts_with("HID_ID=")).and_then(|line| {
            let parts: Vec<&str> = line.trim_start_matches("HID_ID=").split(':').collect();
            if parts.len() >= 3 {
                let v = parts[1].trim_start_matches("0000").to_lowercase();
                let p = parts[2].trim_start_matches("0000").to_lowercase();
                Some(format!("{:0>4}:{:0>4}", v, p))
            } else {
                None
            }
        });

        // Determine candidate device indices to probe:
        // Direct Bluetooth or wired devices use 0xFF.
        // USB Receivers can have paired devices at slots 1..=6 as well as 0xFF.
        let is_receiver = name.to_lowercase().contains("receiver");
        let candidate_indices: Vec<u8> = if is_receiver {
            vec![1, 2, 3, 4, 0xFF]
        } else {
            vec![0xFF, 1]
        };

        for dev_idx in candidate_indices {
            if let Ok(mut file) = open_hidraw(&dev_path) {
                flush_hidraw(&mut file);
                // Try to find REPROG_CONTROLS_V4 (0x1B04), then fallback to 0x1B00
                let feature_idx = query_feature_index(&mut file, dev_idx, FEATURE_REPROG_CONTROLS_V4)
                    .or_else(|| query_feature_index(&mut file, dev_idx, FEATURE_REPROG_CONTROLS));

                if let Some(feat_idx) = feature_idx {
                    if feat_idx > 0 {
                        devices.push(LogitechHidDevice {
                            path: dev_path.clone(),
                            name: name.clone(),
                            vid_pid: vid_pid.clone(),
                            dev_index: dev_idx,
                            reprog_feature_index: feat_idx,
                        });
                        break;
                    }
                }
            }
        }
    }

    devices
}

/// Opens a `/dev/hidraw*` device with non-blocking flags.
pub fn open_hidraw(path: &Path) -> std::io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
}

/// Queries feature index from the device's IRoot feature (0x0000) using GetFeature (func 0x00).
pub fn query_feature_index(file: &mut File, dev_idx: u8, feature_id: u16) -> Option<u8> {
    flush_hidraw(file);

    let mut req = [0u8; 20];
    req[0] = REPORT_ID_LONG;
    req[1] = dev_idx;
    req[2] = 0x00; // Root feature index
    req[3] = 0x00; // Function 0: GetFeature
    req[4] = (feature_id >> 8) as u8;
    req[5] = (feature_id & 0xFF) as u8;

    if file.write_all(&req).is_err() {
        return None;
    }

    let start = Instant::now();
    let mut resp = [0u8; 32];
    while start.elapsed() < Duration::from_millis(250) {
        match file.read(&mut resp) {
            Ok(n) if n >= 5 => {
                if resp[0] == REPORT_ID_LONG && resp[1] == dev_idx && resp[2] == 0x00 && resp[3] == 0x00 {
                    let feat_idx = resp[4];
                    return if feat_idx > 0 { Some(feat_idx) } else { None };
                }
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => break,
        }
    }

    None
}

/// Verifies whether a specific Control ID (CID) currently has the DIVERTED flag set.
pub fn is_cid_diverted(
    file: &mut File,
    dev_idx: u8,
    reprog_feature_index: u8,
    cid: u16,
) -> bool {
    flush_hidraw(file);

    let mut req = [0u8; 20];
    req[0] = REPORT_ID_LONG;
    req[1] = dev_idx;
    req[2] = reprog_feature_index;
    req[3] = 0x20; // Function 2: getCidReporting
    req[4] = (cid >> 8) as u8;
    req[5] = (cid & 0xFF) as u8;

    if file.write_all(&req).is_err() {
        return false;
    }

    let start = Instant::now();
    let mut resp = [0u8; 32];
    while start.elapsed() < Duration::from_millis(250) {
        match file.read(&mut resp) {
            Ok(n) if n >= 7 => {
                if resp[0] == REPORT_ID_LONG && resp[1] == dev_idx && resp[2] == reprog_feature_index && resp[3] == 0x20 {
                    let flags = resp[6];
                    return (flags & 0x01) != 0;
                }
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => break,
        }
    }

    false
}

/// Diverts or un-diverts a specific Control ID (CID) using setCidReporting (func 0x30).
pub fn set_cid_diverted(
    file: &mut File,
    dev_idx: u8,
    reprog_feature_index: u8,
    cid: u16,
    divert: bool,
) -> bool {
    flush_hidraw(file);

    let mut req = [0u8; 20];
    req[0] = REPORT_ID_LONG;
    req[1] = dev_idx;
    req[2] = reprog_feature_index;
    req[3] = 0x30; // Function 3: setCidReporting
    req[4] = (cid >> 8) as u8;
    req[5] = (cid & 0xFF) as u8;
    req[6] = if divert { FLAG_DIVERT_ON } else { FLAG_DIVERT_OFF };
    req[7] = 0x00; // remap high byte
    req[8] = 0x00; // remap low byte

    if file.write_all(&req).is_err() {
        return false;
    }

    let start = Instant::now();
    let mut resp = [0u8; 32];
    while start.elapsed() < Duration::from_millis(300) {
        match file.read(&mut resp) {
            Ok(n) if n >= 4 => {
                if resp[0] == REPORT_ID_LONG && resp[1] == dev_idx && resp[2] == reprog_feature_index && resp[3] == 0x30 {
                    return true;
                }
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => break,
        }
    }

    // Double check state if immediate ACK packet was delayed
    if divert {
        is_cid_diverted(file, dev_idx, reprog_feature_index, cid)
    } else {
        false
    }
}

/// Maps a Logitech Control ID (CID) to the standard Bo-Ring button code.
pub fn cid_to_button_code(cid: u16) -> u16 {
    match cid {
        CID_GESTURE_BUTTON => BTN_RING_CODE, // 278: Thumb rest button
        CID_SMARTSHIFT_BUTTON => BTN_SMARTSHIFT_CODE, // 280: Mode-shift button
        CID_APP_SWITCH_BUTTON => BTN_RING_CODE, // 278: M720 thumb gesture button
        CID_DPI_SWITCH_BUTTON => BTN_SMARTSHIFT_CODE, // 280: MX Vertical / Lift top button
        CID_PRECISION_BUTTON => BTN_GESTURES_CODE, // 277: MX Ergo precision button
        0x0053 => BTN_BACK_CODE,     // 275: Back button
        0x0056 => BTN_FORWARD_CODE,  // 276: Forward button
        0x0052 => BTN_MIDDLE_CODE,   // 274: Middle click
        other => other,
    }
}

/// Resolves the list of (Control ID, button_code) to divert for a specific mouse
/// based on device profiles registered in DeviceRegistry.
pub fn resolve_cids_for_mouse(dev_name: &str, vid_pid: Option<&str>) -> Vec<(u16, u16)> {
    let registry = crate::devices::DeviceRegistry::new();
    let matched_profile = registry.match_device(dev_name, vid_pid);
    let mut cids = matched_profile
        .map(|p| p.config.diverted_cids())
        .unwrap_or_default();

    // Fallback if no CIDs were explicitly defined in the profile:
    // Gesture button (0x00C3 -> BTN_RING_CODE 278)
    if cids.is_empty() {
        cids.push((CID_GESTURE_BUTTON, BTN_RING_CODE));
    }

    cids
}

/// Starts the continuous Logitech HID++ manager thread.
///
/// Automatically discovers Logitech mice, enables diversion on all physical controls
/// declared in their device profile, and streams physical press/release events to the channel.
pub fn start_hidpp_manager(
    tx: mpsc::Sender<(String, Option<String>, bool, evdev::InputEvent)>,
    diverted_devices: Arc<Mutex<HashSet<PathBuf>>>,
) {
    thread::spawn(move || {
        loop {
            let active_diverted = {
                let lock = diverted_devices.lock().unwrap();
                lock.clone()
            };
            let mice = discover_logitech_mice(&active_diverted);
            for mouse in mice {
                let mut lock = diverted_devices.lock().unwrap();
                if lock.contains(&mouse.path) {
                    continue;
                }
                lock.insert(mouse.path.clone());
                drop(lock);

                println!(
                    "✨ Logitech HID++ 2.0 Device detected: [{:?}] {} (VID:PID {:?}, DevIdx 0x{:02X}, ReprogFeat 0x{:02X})",
                    mouse.path, mouse.name, mouse.vid_pid, mouse.dev_index, mouse.reprog_feature_index
                );

                let mouse_clone = mouse.clone();
                let tx_clone = tx.clone();
                let diverted_tracker = Arc::clone(&diverted_devices);

                thread::spawn(move || {
                    let mut file = match open_hidraw(&mouse_clone.path) {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("⚠️ Failed to open HID++ device {:?}: {}", mouse_clone.path, e);
                            let mut lock = diverted_tracker.lock().unwrap();
                            lock.remove(&mouse_clone.path);
                            return;
                        }
                    };

                    // Resolve declared CIDs dynamically from DeviceRegistry profile
                    let cids_to_divert = resolve_cids_for_mouse(&mouse_clone.name, mouse_clone.vid_pid.as_deref());

                    for &(cid, code) in &cids_to_divert {
                        let diverted_ok = set_cid_diverted(
                            &mut file,
                            mouse_clone.dev_index,
                            mouse_clone.reprog_feature_index,
                            cid,
                            true,
                        );

                        if diverted_ok {
                            println!(
                                "🎯 Diverted physical control CID 0x{:04X} -> Button Code {} on [{:?}] {}!",
                                cid, code, mouse_clone.path, mouse_clone.name
                            );
                        } else {
                            println!(
                                "⚠️ Could not confirm diversion ACK for CID 0x{:04X} on [{:?}], listening for reports anyway.",
                                cid, mouse_clone.path
                            );
                        }
                    }

                    let cid_map: std::collections::HashMap<u16, u16> = cids_to_divert.into_iter().collect();
                    flush_hidraw(&mut file);

                    let mut previous_cids: HashSet<u16> = HashSet::new();
                    let mut buf = [0u8; 64];

                    loop {
                        match file.read(&mut buf) {
                            Ok(n) if n >= 12 => {
                                // Long report (0x11) for ReprogControls (feat_idx), function 0x00 (DivertedButtonEvent)
                                if buf[0] == REPORT_ID_LONG
                                    && buf[1] == mouse_clone.dev_index
                                    && buf[2] == mouse_clone.reprog_feature_index
                                    && (buf[3] & 0xF0) == 0x00
                                {
                                    // Parse up to 4 active 16-bit CIDs
                                    let mut current_cids = HashSet::new();
                                    for i in 0..4 {
                                        let offset = 4 + (i * 2);
                                        let cid = ((buf[offset] as u16) << 8) | (buf[offset + 1] as u16);
                                        if cid != 0 {
                                            current_cids.insert(cid);
                                        }
                                    }

                                    // Detect newly pressed CIDs (value = 1)
                                    for &cid in &current_cids {
                                        if !previous_cids.contains(&cid) {
                                            let code = cid_map.get(&cid).copied().unwrap_or_else(|| cid_to_button_code(cid));
                                            println!(
                                                "🔘 [HID++ Press] Control 0x{:04X} -> Evdev Code {} on {}",
                                                cid, code, mouse_clone.name
                                            );
                                            let ev = evdev::InputEvent::new(evdev::EventType::KEY, code, 1);
                                            let _ = tx_clone.send((
                                                mouse_clone.name.clone(),
                                                mouse_clone.vid_pid.clone(),
                                                false, // not an exclusive evdev grab
                                                ev,
                                            ));
                                        }
                                    }

                                    // Detect released CIDs (value = 0)
                                    for &cid in &previous_cids {
                                        if !current_cids.contains(&cid) {
                                            let code = cid_map.get(&cid).copied().unwrap_or_else(|| cid_to_button_code(cid));
                                            println!(
                                                "🔘 [HID++ Release] Control 0x{:04X} -> Evdev Code {} on {}",
                                                cid, code, mouse_clone.name
                                            );
                                            let ev = evdev::InputEvent::new(evdev::EventType::KEY, code, 0);
                                            let _ = tx_clone.send((
                                                mouse_clone.name.clone(),
                                                mouse_clone.vid_pid.clone(),
                                                false,
                                                ev,
                                            ));
                                        }
                                    }

                                    previous_cids = current_cids;
                                }
                            }
                            Ok(_) => {}
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                thread::sleep(Duration::from_millis(8));
                            }
                            Err(e) => {
                                eprintln!(
                                    "🔌 HID++ device disconnected or read error {:?}: {}",
                                    mouse_clone.path, e
                                );
                                let mut lock = diverted_tracker.lock().unwrap();
                                lock.remove(&mouse_clone.path);
                                break;
                            }
                        }
                    }
                });
            }

            thread::sleep(Duration::from_millis(1500));
        }
    });
}

/// Attaches direct HID++ listening for the standalone Config UI (fallback when daemon is not running).
pub fn attach_hidpp_gui_listener(
    ctx: eframe::egui::Context,
    tx: mpsc::Sender<crate::devices::MouseInputEvent>,
    diverted_devices: Arc<Mutex<HashSet<PathBuf>>>,
) {
    thread::spawn(move || {
        loop {
            let active_diverted = {
                let lock = diverted_devices.lock().unwrap();
                lock.clone()
            };
            let mice = discover_logitech_mice(&active_diverted);
            for mouse in mice {
                let mut lock = diverted_devices.lock().unwrap();
                if lock.contains(&mouse.path) {
                    continue;
                }
                lock.insert(mouse.path.clone());
                drop(lock);

                let mouse_clone = mouse.clone();
                let tx_clone = tx.clone();
                let ctx_clone = ctx.clone();
                let diverted_tracker = Arc::clone(&diverted_devices);

                thread::spawn(move || {
                    let mut file = match open_hidraw(&mouse_clone.path) {
                        Ok(f) => f,
                        Err(_) => {
                            let mut lock = diverted_tracker.lock().unwrap();
                            lock.remove(&mouse_clone.path);
                            return;
                        }
                    };

                    let cids_to_divert = resolve_cids_for_mouse(&mouse_clone.name, mouse_clone.vid_pid.as_deref());

                    for &(cid, _) in &cids_to_divert {
                        let _ = set_cid_diverted(
                            &mut file,
                            mouse_clone.dev_index,
                            mouse_clone.reprog_feature_index,
                            cid,
                            true,
                        );
                    }

                    let cid_map: std::collections::HashMap<u16, u16> = cids_to_divert.into_iter().collect();
                    flush_hidraw(&mut file);

                    let mut previous_cids: HashSet<u16> = HashSet::new();
                    let mut buf = [0u8; 64];

                    loop {
                        match file.read(&mut buf) {
                            Ok(n) if n >= 12 => {
                                if buf[0] == REPORT_ID_LONG
                                    && buf[1] == mouse_clone.dev_index
                                    && buf[2] == mouse_clone.reprog_feature_index
                                    && (buf[3] & 0xF0) == 0x00
                                {
                                    let mut current_cids = HashSet::new();
                                    for i in 0..4 {
                                        let offset = 4 + (i * 2);
                                        let cid = ((buf[offset] as u16) << 8) | (buf[offset + 1] as u16);
                                        if cid != 0 {
                                            current_cids.insert(cid);
                                        }
                                    }

                                    for &cid in &current_cids {
                                        if !previous_cids.contains(&cid) {
                                            let code = cid_map.get(&cid).copied().unwrap_or_else(|| cid_to_button_code(cid));
                                            let _ = tx_clone.send(crate::devices::MouseInputEvent {
                                                code,
                                                raw_code: code,
                                                dev_name: Some(mouse_clone.name.clone()),
                                                vid_pid: mouse_clone.vid_pid.clone(),
                                            });
                                            ctx_clone.request_repaint();
                                        }
                                    }

                                    previous_cids = current_cids;
                                }
                            }
                            Ok(_) => {}
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                thread::sleep(Duration::from_millis(10));
                            }
                            Err(_) => {
                                let mut lock = diverted_tracker.lock().unwrap();
                                lock.remove(&mouse_clone.path);
                                break;
                            }
                        }
                    }
                });
            }

            thread::sleep(Duration::from_millis(1500));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_to_button_code_mappings() {
        assert_eq!(cid_to_button_code(CID_GESTURE_BUTTON), BTN_RING_CODE);
        assert_eq!(cid_to_button_code(CID_SMARTSHIFT_BUTTON), BTN_SMARTSHIFT_CODE);
        assert_eq!(cid_to_button_code(CID_APP_SWITCH_BUTTON), BTN_RING_CODE);
        assert_eq!(cid_to_button_code(CID_DPI_SWITCH_BUTTON), BTN_SMARTSHIFT_CODE);
        assert_eq!(cid_to_button_code(CID_PRECISION_BUTTON), BTN_GESTURES_CODE);
        assert_eq!(cid_to_button_code(0x0053), BTN_BACK_CODE);
        assert_eq!(cid_to_button_code(0x0056), BTN_FORWARD_CODE);
        assert_eq!(cid_to_button_code(0x0052), BTN_MIDDLE_CODE);
        assert_eq!(cid_to_button_code(0x1234), 0x1234);
    }

    #[test]
    fn test_parse_diverted_button_report() {
        let mut report = [0u8; 20];
        report[0] = REPORT_ID_LONG;
        report[1] = 0xFF;
        report[2] = 0x0A; // Feature index
        report[3] = 0x00; // DivertedButtonEvent
        // Slot 0: CID 0x00C3 (Thumb rest button)
        report[4] = 0x00;
        report[5] = 0xC3;
        // Slot 1: CID 0x0056 (Forward button)
        report[6] = 0x00;
        report[7] = 0x56;

        let mut cids = Vec::new();
        for i in 0..4 {
            let offset = 4 + (i * 2);
            let cid = ((report[offset] as u16) << 8) | (report[offset + 1] as u16);
            if cid != 0 {
                cids.push(cid);
            }
        }

        assert_eq!(cids, vec![0x00C3, 0x0056]);
        assert_eq!(cid_to_button_code(cids[0]), BTN_RING_CODE);
        assert_eq!(cid_to_button_code(cids[1]), BTN_FORWARD_CODE);
    }

    #[test]
    fn test_resolve_cids_for_mouse() {
        let cids = resolve_cids_for_mouse("Wireless Mouse MX Master 2s", Some("046d:b019"));
        assert_eq!(cids, vec![(0x00C3, 278), (0x00C4, 280)]);

        let cids_mx3 = resolve_cids_for_mouse("Logitech MX Master 3S", Some("046d:b034"));
        assert_eq!(cids_mx3, vec![(0x00C3, 278), (0x00C4, 280)]);

        let cids_m720 = resolve_cids_for_mouse("Logitech M720 Triathlon", Some("046d:b015"));
        assert_eq!(cids_m720, vec![(0x00D0, 278)]);

        let cids_vert = resolve_cids_for_mouse("Logitech MX Vertical", Some("046d:b020"));
        assert_eq!(cids_vert, vec![(0x00FD, 280)]);

        let cids_ergo = resolve_cids_for_mouse("Logitech MX Ergo", Some("046d:b01d"));
        assert_eq!(cids_ergo, vec![(0x00ED, 277)]);

        let cids_mx4 = resolve_cids_for_mouse("Logitech MX Master 4", Some("046d:c548"));
        assert_eq!(cids_mx4, vec![(0x00C3, 281), (0x00C4, 280)]);

        let fallback = resolve_cids_for_mouse("Unknown Logitech Mouse", None);
        assert_eq!(fallback, vec![(0x00C3, 278)]);
    }
}
