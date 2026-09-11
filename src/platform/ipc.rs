use std::env;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Returns the XDG runtime socket path (`$XDG_RUNTIME_DIR/bo-ring.sock` or `/tmp/bo-ring.sock`).
pub fn daemon_socket_path() -> PathBuf {
    if let Ok(dir) = env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(&dir).join("bo-ring.sock")
    } else {
        PathBuf::from("/tmp/bo-ring.sock")
    }
}

pub type IpcClients = Arc<Mutex<Vec<UnixStream>>>;

/// Events received from the daemon via IPC.
#[derive(Debug, Clone, PartialEq)]
pub enum DaemonIpcEvent {
    StateHeld,
    StateReleased,
    ButtonPressed {
        raw_code: u16,
        normalized_code: u16,
        vid_pid: Option<String>,
        dev_name: Option<String>,
    },
    ButtonReleased {
        raw_code: u16,
        normalized_code: u16,
    },
    ButtonCode(u16),
}

impl DaemonIpcEvent {
    pub fn parse_line(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if trimmed == "STATE:HELD" {
            Some(DaemonIpcEvent::StateHeld)
        } else if trimmed == "STATE:RELEASED" {
            Some(DaemonIpcEvent::StateReleased)
        } else if let Some(rest) = trimmed.strip_prefix("P:") {
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() >= 2 {
                let raw = parts[0].parse().ok()?;
                let norm = parts[1].parse().ok()?;
                let (vid_pid, dev_name) = if parts.len() >= 4 {
                    let vp = format!("{}:{}", parts[2], parts[3]);
                    let name = if parts.len() > 4 {
                        parts[4..].join(":")
                    } else {
                        String::new()
                    };
                    (Some(vp), if name.is_empty() { None } else { Some(name) })
                } else {
                    (None, None)
                };
                Some(DaemonIpcEvent::ButtonPressed {
                    raw_code: raw,
                    normalized_code: norm,
                    vid_pid,
                    dev_name,
                })
            } else if let Ok(code) = rest.parse() {
                Some(DaemonIpcEvent::ButtonCode(code))
            } else {
                None
            }
        } else if let Some(rest) = trimmed.strip_prefix("R:") {
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() >= 2 {
                let raw = parts[0].parse().ok()?;
                let norm = parts[1].parse().ok()?;
                Some(DaemonIpcEvent::ButtonReleased { raw_code: raw, normalized_code: norm })
            } else {
                None
            }
        } else if let Ok(code) = trimmed.parse::<u16>() {
            Some(DaemonIpcEvent::ButtonCode(code))
        } else {
            None
        }
    }
}

/// Commands sent to the background daemon via IPC.
#[derive(Debug, Clone, PartialEq)]
pub enum DaemonIpcCommand {
    ShowRing,
    HideRing,
    ToggleRing,
    ReloadConfig,
    ExecKey(Vec<String>),
    ExecCmd(String),
}

impl DaemonIpcCommand {
    pub fn to_wire_format(&self) -> String {
        match self {
            DaemonIpcCommand::ShowRing => "RING:SHOW\n".to_string(),
            DaemonIpcCommand::HideRing => "RING:HIDE\n".to_string(),
            DaemonIpcCommand::ToggleRing => "RING:TOGGLE\n".to_string(),
            DaemonIpcCommand::ReloadConfig => "CONFIG:RELOAD\n".to_string(),
            DaemonIpcCommand::ExecKey(keys) => format!("EXEC:KEY:{}\n", keys.join(",")),
            DaemonIpcCommand::ExecCmd(cmd) => format!("EXEC:CMD:{}\n", cmd),
        }
    }

    pub fn parse_wire_line(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if trimmed == "RING:SHOW" {
            Some(DaemonIpcCommand::ShowRing)
        } else if trimmed == "RING:HIDE" {
            Some(DaemonIpcCommand::HideRing)
        } else if trimmed == "RING:TOGGLE" {
            Some(DaemonIpcCommand::ToggleRing)
        } else if trimmed == "CONFIG:RELOAD" || trimmed == "RELOAD" {
            Some(DaemonIpcCommand::ReloadConfig)
        } else if let Some(keys_str) = trimmed.strip_prefix("EXEC:KEY:") {
            let keys: Vec<String> = keys_str.split(',').map(|s| s.trim().to_string()).collect();
            Some(DaemonIpcCommand::ExecKey(keys))
        } else if let Some(cmd) = trimmed.strip_prefix("EXEC:CMD:") {
            Some(DaemonIpcCommand::ExecCmd(cmd.to_string()))
        } else {
            None
        }
    }
}

/// Sends a command to the daemon via UNIX socket. Returns true if sent successfully.
pub fn send_command_to_daemon(cmd: &DaemonIpcCommand) -> bool {
    let sock_path = daemon_socket_path();
    if let Ok(mut stream) = UnixStream::connect(&sock_path) {
        let msg = cmd.to_wire_format();
        return stream.write_all(msg.as_bytes()).is_ok();
    }
    false
}

/// Broadcast a raw/normalized button press/release event to all connected IPC clients.
pub fn broadcast_button_event(
    clients: &IpcClients,
    raw_code: u16,
    norm_code: u16,
    val: i32,
    vid_pid: Option<&str>,
    dev_name: Option<&str>,
) {
    let mut lock = clients.lock().unwrap();
    if lock.is_empty() {
        return;
    }
    let msg = match val {
        1 => {
            if let (Some(vp), Some(name)) = (vid_pid, dev_name) {
                format!("P:{}:{}:{}:{}\n{}\n", raw_code, norm_code, vp, name, norm_code)
            } else {
                format!("P:{}:{}\n{}\n", raw_code, norm_code, norm_code)
            }
        }
        0 => format!("R:{}:{}\n", raw_code, norm_code),
        _ => return,
    };
    lock.retain_mut(|stream| stream.write_all(msg.as_bytes()).is_ok());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_ipc_event_parsing() {
        assert_eq!(DaemonIpcEvent::parse_line("STATE:HELD"), Some(DaemonIpcEvent::StateHeld));
        assert_eq!(DaemonIpcEvent::parse_line("STATE:RELEASED"), Some(DaemonIpcEvent::StateReleased));
        assert_eq!(
            DaemonIpcEvent::parse_line("P:278:278"),
            Some(DaemonIpcEvent::ButtonPressed {
                raw_code: 278,
                normalized_code: 278,
                vid_pid: None,
                dev_name: None,
            })
        );
        assert_eq!(
            DaemonIpcEvent::parse_line("P:278:278:046d:c548:Logitech USB Receiver"),
            Some(DaemonIpcEvent::ButtonPressed {
                raw_code: 278,
                normalized_code: 278,
                vid_pid: Some("046d:c548".to_string()),
                dev_name: Some("Logitech USB Receiver".to_string()),
            })
        );
        assert_eq!(
            DaemonIpcEvent::parse_line("R:278:278"),
            Some(DaemonIpcEvent::ButtonReleased { raw_code: 278, normalized_code: 278 })
        );
        assert_eq!(DaemonIpcEvent::parse_line("278"), Some(DaemonIpcEvent::ButtonCode(278)));
        assert_eq!(DaemonIpcEvent::parse_line("INVALID"), None);
    }

    #[test]
    fn test_daemon_ipc_command_wire() {
        let cmd = DaemonIpcCommand::ExecCmd("xdg-open https://google.com".to_string());
        assert_eq!(cmd.to_wire_format(), "EXEC:CMD:xdg-open https://google.com\n");
        assert_eq!(DaemonIpcCommand::parse_wire_line("EXEC:CMD:xdg-open https://google.com"), Some(cmd));

        let key_cmd = DaemonIpcCommand::ExecKey(vec!["CTRL".to_string(), "t".to_string()]);
        assert_eq!(key_cmd.to_wire_format(), "EXEC:KEY:CTRL,t\n");
        assert_eq!(DaemonIpcCommand::parse_wire_line("EXEC:KEY:CTRL,t"), Some(key_cmd));

        let reload_cmd = DaemonIpcCommand::ReloadConfig;
        assert_eq!(reload_cmd.to_wire_format(), "CONFIG:RELOAD\n");
        assert_eq!(DaemonIpcCommand::parse_wire_line("CONFIG:RELOAD"), Some(DaemonIpcCommand::ReloadConfig));
        assert_eq!(DaemonIpcCommand::parse_wire_line("RELOAD"), Some(DaemonIpcCommand::ReloadConfig));
    }
}
