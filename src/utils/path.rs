use std::env;
use std::path::{Path, PathBuf};

/// Expands `~` or `$HOME` prefixes in file/directory path strings to the user's home directory.
pub fn expand_home_path(path_str: &str) -> PathBuf {
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return PathBuf::new();
    }

    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

    if trimmed == "~" || trimmed == "$HOME" {
        return PathBuf::from(home);
    }

    if let Some(rest) = trimmed.strip_prefix("~/") {
        return PathBuf::from(&home).join(rest);
    }

    if let Some(rest) = trimmed.strip_prefix("$HOME/") {
        return PathBuf::from(&home).join(rest);
    }

    PathBuf::from(trimmed)
}

/// Expands a path and converts it back to a `String` (useful for command line arguments).
pub fn expand_path_arg(arg: &str) -> String {
    let p = expand_home_path(arg);
    if p.as_os_str().is_empty() {
        arg.to_string()
    } else {
        p.to_string_lossy().to_string()
    }
}

/// Resolves an asset / icon path checking standard user and system asset directories.
pub fn resolve_asset_path(path_str: &str) -> PathBuf {
    let expanded = expand_home_path(path_str);
    if expanded.is_file() {
        return expanded;
    }

    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return PathBuf::new();
    }

    if let Ok(home) = env::var("HOME") {
        let user_share = PathBuf::from(&home).join(".local/share/bo-ring").join(trimmed);
        if user_share.is_file() {
            return user_share;
        }

        let user_config = PathBuf::from(&home).join(".config/bo-ring").join(trimmed);
        if user_config.is_file() {
            return user_config;
        }

    }

    let sys_local_share = PathBuf::from("/usr/local/share/bo-ring").join(trimmed);
    if sys_local_share.is_file() {
        return sys_local_share;
    }

    let sys_share = PathBuf::from("/usr/share/bo-ring").join(trimmed);
    if sys_share.is_file() {
        return sys_share;
    }

    expanded
}

/// Checks if a file path points to a known supported image format based on its extension.
pub fn is_supported_image(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "png" | "webp" | "jpg" | "jpeg" | "gif" | "bmp")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_home_path() {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

        assert_eq!(expand_home_path("~"), PathBuf::from(&home));
        assert_eq!(expand_home_path("$HOME"), PathBuf::from(&home));
        assert_eq!(expand_home_path("~/Downloads"), PathBuf::from(&home).join("Downloads"));
        assert_eq!(expand_home_path("$HOME/Documents"), PathBuf::from(&home).join("Documents"));
        assert_eq!(expand_home_path("/etc/hosts"), PathBuf::from("/etc/hosts"));
        assert_eq!(expand_home_path("relative/path"), PathBuf::from("relative/path"));
    }

    #[test]
    fn test_expand_path_arg() {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        assert_eq!(expand_path_arg("~"), home);
        assert_eq!(expand_path_arg("~/Downloads"), format!("{}/Downloads", home));
        assert_eq!(expand_path_arg("$HOME/Documents"), format!("{}/Documents", home));
        assert_eq!(expand_path_arg("firefox"), "firefox");
    }

    #[test]
    fn test_is_supported_image() {
        assert!(is_supported_image(Path::new("icon.png")));
        assert!(is_supported_image(Path::new("photo.WEBP")));
        assert!(is_supported_image(Path::new("/tmp/image.jpeg")));
        assert!(!is_supported_image(Path::new("script.sh")));
        assert!(!is_supported_image(Path::new("no_ext")));
    }
}
