use super::{
    models::BootInfo,
    parser::{parse_bcdedit, parse_bcdedit_boot_state, valid_windows_id},
};
use crate::boot::BootManager;
use crate::error::{BootError, BootResult};
use crate::system::info::current_os_name;
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn hidden_command(program: &str) -> Command {
    let mut command = Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

pub struct WindowsBootManager;

impl WindowsBootManager {
    pub fn new() -> Self {
        Self
    }
}

fn run_bcdedit(args: &[&str]) -> BootResult<std::process::Output> {
    hidden_command("bcdedit")
        .args(args)
        .output()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                BootError::CommandNotFound("bcdedit".into())
            } else if error.kind() == std::io::ErrorKind::PermissionDenied {
                BootError::PermissionDenied
            } else {
                BootError::CommandFailed(error.to_string())
            }
        })
}

fn output_text(output: std::process::Output) -> BootResult<String> {
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(
            if message.to_lowercase().contains("access") || message.contains("拒绝") {
                BootError::PermissionDenied
            } else {
                BootError::CommandFailed(message)
            },
        );
    }
    Ok(decode_windows_text(&output.stdout))
}

fn decode_windows_text(bytes: &[u8]) -> String {
    if bytes.len() >= 2 && bytes.windows(2).any(|pair| pair == [0, 0]) {
        let utf16: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        return String::from_utf16_lossy(&utf16);
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }
    let (text, _, _) = encoding_rs::GBK.decode(bytes);
    text.into_owned()
}

impl BootManager for WindowsBootManager {
    fn get_boot_info(&self) -> BootResult<BootInfo> {
        let output = run_bcdedit(&["/enum", "firmware", "/v"])?;
        let text = output_text(output)?;
        let mut entries = parse_bcdedit(&text)?;
        let (boot_order, boot_next) = parse_bcdedit_boot_state(&text);
        for (order, id) in boot_order.iter().enumerate() {
            if let Some(entry) = entries.iter_mut().find(|entry| entry.id == *id) {
                entry.order = Some(order as u32);
            }
        }
        if let Some(id) = boot_next.as_ref() {
            if let Some(entry) = entries.iter_mut().find(|entry| entry.id == *id) {
                entry.next = true;
            }
        }
        Ok(BootInfo {
            current_os: current_os_name(),
            uefi: true,
            secure_boot: None,
            boot_current: None,
            boot_next,
            boot_order,
            entries,
        })
    }

    fn set_boot_next(&self, id: &str) -> BootResult<()> {
        if !valid_windows_id(id) || matches!(id, "{bootmgr}" | "{fwbootmgr}") {
            return Err(BootError::InvalidBootId(id.into()));
        }
        let output = run_bcdedit(&["/set", "{fwbootmgr}", "bootsequence", id])?;
        output_text(output).map(|_| ())
    }

    fn set_default_boot(&self, id: &str) -> BootResult<()> {
        if !valid_windows_id(id) || matches!(id, "{bootmgr}" | "{fwbootmgr}") {
            return Err(BootError::InvalidBootId(id.into()));
        }
        let output = run_bcdedit(&["/set", "{fwbootmgr}", "displayorder", id, "/addfirst"])?;
        output_text(output).map(|_| ())
    }

    fn reboot(&self) -> BootResult<()> {
        let output = hidden_command("shutdown")
            .args(["/r", "/t", "0"])
            .output()
            .map_err(|error| BootError::RebootFailed(error.to_string()))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(BootError::RebootFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
        }
    }
}
