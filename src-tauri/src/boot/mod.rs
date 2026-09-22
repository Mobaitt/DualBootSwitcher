pub mod linux;
pub mod models;
pub mod parser;
pub mod windows;

use crate::error::{BootError, BootResult};
use models::{BootEntry, BootInfo};

pub fn visible_boot_info(mut info: BootInfo) -> BootInfo {
    info.entries.retain(|entry| !entry.suspected_invalid);
    let valid_ids = info
        .entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let is_linux_order = info
        .boot_order
        .iter()
        .all(|id| id.len() == 4 && id.chars().all(|character| character.is_ascii_hexdigit()));
    if is_linux_order {
        info.boot_order.retain(|id| valid_ids.contains(id.as_str()));
    } else {
        let has_windows_entry = info
            .entries
            .iter()
            .any(|entry| matches!(entry.entry_type, models::BootEntryType::Windows));
        info.boot_order.retain(|id| {
            valid_ids.contains(id.as_str()) || (id == "{bootmgr}" && has_windows_entry)
        });
    }
    if info
        .boot_current
        .as_ref()
        .is_some_and(|id| is_linux_order && !valid_ids.contains(id.as_str()))
    {
        info.boot_current = None;
    }
    if info
        .boot_next
        .as_ref()
        .is_some_and(|id| is_linux_order && !valid_ids.contains(id.as_str()))
    {
        info.boot_next = None;
    }
    info
}

pub trait BootManager: Send + Sync {
    fn get_boot_info(&self) -> BootResult<BootInfo>;
    fn list_entries(&self) -> BootResult<Vec<BootEntry>> {
        Ok(visible_boot_info(self.get_boot_info()?).entries)
    }
    fn set_boot_next(&self, id: &str) -> BootResult<()>;
    fn set_default_boot(&self, id: &str) -> BootResult<()>;
    fn reboot(&self) -> BootResult<()>;
}

pub fn current_manager() -> BootResult<Box<dyn BootManager>> {
    #[cfg(target_os = "windows")]
    {
        return Ok(Box::new(windows::WindowsBootManager::new()));
    }
    #[cfg(target_os = "linux")]
    {
        return Ok(Box::new(linux::LinuxBootManager::new()));
    }
    #[allow(unreachable_code)]
    Err(BootError::UnsupportedPlatform)
}
