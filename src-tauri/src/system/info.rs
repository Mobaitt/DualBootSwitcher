#[cfg(target_os = "linux")]
use std::fs;

pub fn current_os_name() -> String {
    #[cfg(target_os = "linux")]
    {
        let content = fs::read_to_string("/etc/os-release").unwrap_or_default();
        if let Some(value) = content
            .lines()
            .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        {
            return value.trim_matches('"').to_string();
        }
        return "Linux".into();
    }
    #[cfg(target_os = "windows")]
    {
        return "Windows".into();
    }
    #[allow(unreachable_code)]
    std::env::consts::OS.to_string()
}

pub fn secure_boot_state() -> Option<bool> {
    #[cfg(target_os = "linux")]
    {
        let directory = fs::read_dir("/sys/firmware/efi/efivars").ok()?;
        for item in directory.flatten() {
            if item
                .file_name()
                .to_string_lossy()
                .starts_with("SecureBoot-")
            {
                let bytes = fs::read(item.path()).ok()?;
                return bytes.last().map(|value| *value == 1);
            }
        }
        None
    }
    #[cfg(target_os = "windows")]
    {
        return None;
    }
    #[allow(unreachable_code)]
    None
}
