//! Privilege boundaries live in their own module so UAC/polkit helpers can be
//! added without changing the Tauri command or boot parsing layers.

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod windows;
