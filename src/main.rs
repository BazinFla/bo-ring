mod config;
mod config_ui;
pub mod devices;
pub mod platform;
pub mod utils;

pub use platform::input_emitter as keyboard;
pub use platform::ipc::daemon_socket_path;
pub use platform::{autostart, uninstall, window as active_window};
pub use utils::{color, i18n, icon_loader, path};

mod ring_menu;

use config::{ButtonActionConfig, Config};
use config_ui::run_config_gui;
use devices::{is_supported_button_code, normalize_button_code, BTN_LEFT_CODE, BTN_RIGHT_CODE};
use evdev::enumerate;
use platform::ipc::{broadcast_button_event, DaemonIpcCommand, IpcClients};
use ring_menu::{execute_action_direct, run_ring_menu_window};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

static RING_BUTTON_HELD: AtomicBool = AtomicBool::new(false);

fn main() {
    let args: Vec<String> = env::args().collect();

    // Uninstall mode: `bo-ring --uninstall`
    if args.iter().any(|arg| arg == "--uninstall") {
        uninstall::run_uninstall();
        return;
    }

    // Ring Menu mode: `cargo run -- --ring`
    if args.iter().any(|arg| arg == "--ring") {
        println!("🚀 Launching Ring Menu...");
        let initial_app = args.windows(2).find(|w| w[0] == "--app").map(|w| w[1].clone());
        let config = Config::load_or_default();
        let lang = crate::utils::i18n::resolve_language_code(&config.general.language);
        run_ring_menu_window(config.ring_menu, config.app_profiles, initial_app, lang);
        return;
    }

    // Daemon mode: `cargo run -- --daemon`
    let is_daemon_mode = args.iter().any(|arg| arg == "--daemon");

    // Default mode (no args or --config): Open Configuration GUI
    if !is_daemon_mode && !args.iter().any(|arg| arg == "--ring") {
        println!("🎛️ Launching Configuration Menu (Logi Options+ Style)...");
        let config = Config::load_or_default();
        run_config_gui(config);
        return;
    }

    println!("=== Bo-Ring - Daemon & Actions Ring ===");
    keyboard::init_layout_detection();
    keyboard::init_virtual_keyboard();
    autostart::ensure_gnome_show_desktop_keybinding();
    ring_menu::cleanup_stale_ring_pid();
    let config = Config::load_or_default();

    println!("✅ Configuration ready:");
    println!("   - Target device: {}", config.general.device_name);
    println!("   - Configured buttons: {}", config.buttons.len());
    println!("   - Ring Menu slots: {}", config.ring_menu.items.len());

    let (tx, rx) = mpsc::channel();
    let monitored_paths = start_hotplug_and_device_manager(tx.clone());
    let diverted_hidpp = Arc::new(Mutex::new(HashSet::new()));
    devices::hidpp::start_hidpp_manager(tx.clone(), Arc::clone(&diverted_hidpp));
    start_dbus_sleep_listener(Arc::clone(&monitored_paths), Arc::clone(&diverted_hidpp));

    println!("\n========================================================");
    println!(">>> Daemon active with dynamic hotplug & sleep resume recovery.");
    println!(">>> Press the Action Ring button (BTN_RING / 278) to open the menu.");
    println!("========================================================\n");

    let button_mappings = Arc::new(std::sync::RwLock::new(config.buttons));
    let ipc_clients = start_ipc_server(Arc::clone(&button_mappings));

    while let Ok((dev_name, vid_pid, is_grabbed, event)) = rx.recv() {
        let raw_code = event.code();
        let code = normalize_button_code(raw_code);
        let is_key_event = event.event_type() == evdev::EventType::KEY;

        let is_primary_click = raw_code == BTN_LEFT_CODE || raw_code == BTN_RIGHT_CODE;
        let mappings_guard = button_mappings.read().unwrap();
        let is_remapped = !is_primary_click && is_key_event && (mappings_guard.contains_key(&raw_code) || mappings_guard.contains_key(&code));

        // Broadcast physical button press/release events to IPC clients (Config UI listening mode)
        if is_key_event && is_supported_button_code(raw_code) {
            broadcast_button_event(&ipc_clients, raw_code, code, event.value(), vid_pid.as_deref(), Some(&dev_name));
        }

        if is_key_event && is_remapped {
            let action = mappings_guard.get(&raw_code).or_else(|| mappings_guard.get(&code));
            if matches!(action, Some(ButtonActionConfig::ShowRingMenu)) {
                if event.value() == 1 {
                    RING_BUTTON_HELD.store(true, Ordering::SeqCst);
                } else if event.value() == 0 {
                    RING_BUTTON_HELD.store(false, Ordering::SeqCst);
                    keyboard::nudge_mouse();
                }
            }
        }

        if is_remapped {
            // Remapped button: absorb original click and trigger custom action on press (value == 1)
            if event.value() == 1 {
                let action = mappings_guard.get(&raw_code).or_else(|| mappings_guard.get(&code)).cloned();
                drop(mappings_guard);
                if let Some(action) = action {
                    println!("🔘 Button pressed (Raw code {} -> Normalized {}) -> Configured action: {:?}", raw_code, code, action);
                    match action {
                        ButtonActionConfig::ShowRingMenu => {
                            toggle_ring_menu();
                        }
                        _ => {
                            execute_action_direct(&action);
                        }
                    }
                }
            }
        } else {
            drop(mappings_guard);
            // Unmapped event: pass through to system ONLY if this device was exclusively grabbed
            if is_grabbed {
                keyboard::passthrough_event(&event);
            }
        }
    }
}

fn start_hotplug_and_device_manager(
    tx: mpsc::Sender<(String, Option<String>, bool, evdev::InputEvent)>,
) -> Arc<Mutex<HashSet<PathBuf>>> {
    let monitored_paths: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
    let monitored_paths_clone = Arc::clone(&monitored_paths);

    // Continuous hotplug thread (scans /dev/input/event* every 1.5 seconds)
    thread::spawn(move || {
        loop {
            scan_and_attach_devices(&tx, &monitored_paths_clone);
            thread::sleep(Duration::from_millis(1500));
        }
    });

    monitored_paths
}

fn scan_and_attach_devices(
    tx: &mpsc::Sender<(String, Option<String>, bool, evdev::InputEvent)>,
    monitored_paths: &Arc<Mutex<HashSet<PathBuf>>>,
) {
    for (path, device) in enumerate() {
        let name = device.name().unwrap_or("Unknown device");
        let name_lower = name.to_lowercase();

        // Skip keyboards explicitly
        if name_lower.contains("keyboard") {
            continue;
        }

        // Capability-based detection: any device with BTN_LEFT is a mouse candidate
        let supported = device.supported_keys();
        let has_mouse_btn = supported.map_or(false, |keys| keys.contains(evdev::Key::BTN_LEFT));
        let has_letter_keys = supported.map_or(false, |keys| keys.contains(evdev::Key::KEY_A));

        // Skip devices that are clearly not mice (no mouse button, or keyboard-only)
        if !has_mouse_btn || (has_letter_keys && !has_mouse_btn) {
            continue;
        }

        {

            let mut lock = monitored_paths.lock().unwrap();
            if lock.contains(&path) {
                continue;
            }
            lock.insert(path.clone());
            drop(lock);

            println!("🔌 Monitored input device attached: [{:?}] {}", path, name);

            let tx_clone = tx.clone();
            let name_str = name.to_string();
            let path_clone = path.clone();
            let monitored_paths_thread = Arc::clone(monitored_paths);

            thread::spawn(move || {
                let mut dev = match evdev::Device::open(&path_clone) {
                    Ok(dev) => dev,
                    Err(e) => {
                        eprintln!("⚠️ Failed to open {:?}: {} (check udev / input permissions)", path_clone, e);
                        let mut lock = monitored_paths_thread.lock().unwrap();
                        lock.remove(&path_clone);
                        return;
                    }
                };

                let input_id = dev.input_id();
                let vid_pid = format!("{:04x}:{:04x}", input_id.vendor(), input_id.product());

                let is_receiver_only = name_lower.ends_with("receiver") && !name_lower.contains("mouse");
                let should_grab = has_mouse_btn && !has_letter_keys && !is_receiver_only;

                let is_grabbed = if should_grab {
                    match dev.grab() {
                        Ok(()) => {
                            println!("🔒 Exclusive grab enabled on {:?} [{}] — {}", path_clone, vid_pid, name_lower);
                            true
                        }
                        Err(e) => {
                            eprintln!("⚠️ Failed to enable exclusive grab on {:?}: {}", path_clone, e);
                            false
                        }
                    }
                } else {
                    println!("👂 Listening (no grab) on {:?} [{}] — {}", path_clone, vid_pid, name_lower);
                    false
                };

                loop {
                    match dev.fetch_events() {
                        Ok(events) => {
                            for ev in events {
                                if is_grabbed {
                                    if ev.event_type() == evdev::EventType::RELATIVE {
                                        // Pass through pointer motion and scroll wheel immediately
                                        keyboard::passthrough_event(&ev);
                                    } else if ev.event_type() == evdev::EventType::KEY {
                                        let raw_code = ev.code();
                                        if raw_code == BTN_LEFT_CODE || raw_code == BTN_RIGHT_CODE {
                                            keyboard::passthrough_event(&ev);
                                        } else {
                                            let _ = tx_clone.send((name_str.clone(), Some(vid_pid.clone()), is_grabbed, ev));
                                        }
                                    }
                                } else {
                                    if ev.event_type() == evdev::EventType::KEY {
                                        let raw_code = ev.code();
                                        if is_supported_button_code(raw_code) {
                                            let _ = tx_clone.send((name_str.clone(), Some(vid_pid.clone()), is_grabbed, ev));
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️ Device disconnected or read error on {:?}: {}. Unregistering for re-enumeration.", path_clone, e);
                            let mut lock = monitored_paths_thread.lock().unwrap();
                            lock.remove(&path_clone);
                            break;
                        }
                    }
                }
            });
        }
    }
}

fn start_dbus_sleep_listener(
    monitored_paths: Arc<Mutex<HashSet<PathBuf>>>,
    diverted_hidpp: Arc<Mutex<HashSet<PathBuf>>>,
) {
    thread::spawn(move || {
        let child = std::process::Command::new("dbus-monitor")
            .args(["--system", "type='signal',interface='org.freedesktop.login1.Manager',member='PrepareForSleep'"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn();

        if let Ok(mut child) = child {
            if let Some(stdout) = child.stdout.take() {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    // PrepareForSleep boolean signal: false means system woke up from sleep
                    if line.contains("boolean false") {
                        println!("☀️ System wake up (resume from sleep) detected via D-Bus!");
                        ring_menu::cleanup_stale_ring_pid();
                        keyboard::reset_virtual_keyboard();
                        let mut lock = monitored_paths.lock().unwrap();
                        lock.clear();
                        let mut hidpp_lock = diverted_hidpp.lock().unwrap();
                        hidpp_lock.clear();
                    }
                }
            }
        }
    });
}

fn execute_action_daemon_delayed(action: &ButtonActionConfig) {
    let resolved = config::resolve_action(action, &[]);
    match &resolved {
        ButtonActionConfig::ShowRingMenu | ButtonActionConfig::ActionRef { .. } => {},
        ButtonActionConfig::Command { cmd: _ } => {
            execute_action_direct(&resolved);
        }
        ButtonActionConfig::KeyCombo { keys } => {
            let keys_to_send = keys.clone();
            let is_media_key = keys.iter().any(|k| {
                let norm = k.trim().to_lowercase();
                norm.contains("volume") || norm.contains("mute") || norm.contains("play") || norm.contains("song")
            });

            if is_media_key {
                keyboard::send_key_combo(&keys_to_send);
            } else {
                thread::spawn(move || {
                    active_window::focus_window_under_cursor();
                    // Give Wayland/Mutter 180ms to restore focus to the target app window after ring menu closes
                    thread::sleep(Duration::from_millis(180));
                    keyboard::send_key_combo(&keys_to_send);
                });
            }
        }
    }
}

fn start_ipc_server(
    button_mappings: Arc<std::sync::RwLock<std::collections::HashMap<u16, ButtonActionConfig>>>,
) -> IpcClients {
    let sock_path = daemon_socket_path();
    let _ = std::fs::remove_file(&sock_path);

    let clients: IpcClients = Arc::new(Mutex::new(Vec::new()));
    let clients_clone = Arc::clone(&clients);

    if let Ok(listener) = std::os::unix::net::UnixListener::bind(&sock_path) {
        println!("📡 Unix IPC Socket server active at {:?}", sock_path);
        let button_mappings_ipc = Arc::clone(&button_mappings);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                use std::io::{BufRead, BufReader, Write};
                let is_held = RING_BUTTON_HELD.load(Ordering::SeqCst);
                    let init_msg = if is_held { "STATE:HELD\n" } else { "STATE:RELEASED\n" };

                    if let Ok(mut writer_stream) = stream.try_clone() {
                        let _ = writer_stream.write_all(init_msg.as_bytes());
                        let mut lock = clients_clone.lock().unwrap();
                        lock.push(writer_stream);
                    }

                    // Spawn reader thread for incoming EXEC and CONFIG commands
                    let button_mappings_client = Arc::clone(&button_mappings_ipc);
                    thread::spawn(move || {
                        let reader = BufReader::new(stream);
                        for line in reader.lines().map_while(Result::ok) {
                            if let Some(cmd) = DaemonIpcCommand::parse_wire_line(&line) {
                                match cmd {
                                    DaemonIpcCommand::ShowRing | DaemonIpcCommand::ToggleRing => {
                                        toggle_ring_menu();
                                    }
                                    DaemonIpcCommand::HideRing => {
                                        let pid_path = ring_menu::ring_pid_file_path();
                                        if let Ok(content) = std::fs::read_to_string(&pid_path) {
                                            if let Ok(pid) = content.trim().parse::<i32>() {
                                                ring_menu::terminate_ring_process(pid);
                                            }
                                        }
                                        ring_menu::cleanup_stale_ring_pid();
                                    }
                                    DaemonIpcCommand::ExecCmd(c) => {
                                        let action = ButtonActionConfig::Command { cmd: c };
                                        execute_action_direct(&action);
                                    }
                                    DaemonIpcCommand::ExecKey(keys) => {
                                        let action = ButtonActionConfig::KeyCombo { keys };
                                        execute_action_daemon_delayed(&action);
                                    }
                                    DaemonIpcCommand::ReloadConfig => {
                                        println!("🔄 Received CONFIG:RELOAD. Reloading configuration from disk...");
                                        let new_cfg = Config::load_or_default();
                                        if let Ok(mut lock) = button_mappings_client.write() {
                                            *lock = new_cfg.buttons;
                                            println!("✅ Configuration reloaded: {} button mappings active", lock.len());
                                        }
                                    }
                                }
                            }
                        }
                    });
            }
            let _ = std::fs::remove_file(sock_path);
        });
    }

    clients
}

fn toggle_ring_menu() {
    let pid_path = ring_menu::ring_pid_file_path();
    let current_pid = std::process::id() as i32;
    if pid_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pid_path) {
            if let Ok(pid) = content.trim().parse::<i32>() {
                if pid != current_pid {
                    let is_alive = std::path::Path::new(&format!("/proc/{}", pid)).exists();
                    if is_alive && ring_menu::terminate_ring_process(pid) {
                        println!("⏹️ Closed active Ring Menu (PID {})...", pid);
                        let _ = std::fs::remove_file(&pid_path);
                        return;
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&pid_path);
    }

    let active_app = active_window::detect_active_window();

    println!("✨ Opening Ring Menu! (Detected active app: {})", active_app);
    let exe = env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("bo-ring"));
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("--ring");
    if !active_app.is_empty() {
        cmd.arg("--app").arg(&active_app);
    }

    match cmd.spawn() {
        Ok(mut child) => {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => {
            eprintln!("⚠️ Failed to launch Ring Menu process: {}", e);
        }
    }
}


