use super::{
    models::{BootEntry, BootInfo},
    parser::{parse_efibootmgr, valid_linux_id},
};
use crate::boot::BootManager;
use crate::error::{BootError, BootResult};
use crate::system::info::{current_os_name, secure_boot_state};
use std::process::Command;

pub struct LinuxBootManager;

impl LinuxBootManager {
    pub fn new() -> Self {
        Self
    }
}

fn run_efibootmgr(args: &[&str], privileged: bool) -> BootResult<std::process::Output> {
    let output = if privileged {
        crate::privilege::linux::run_privileged("efibootmgr", args)
    } else {
        Command::new("efibootmgr").args(args).output()
    };
    output.map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            BootError::CommandNotFound(if privileged {
                "pkexec".into()
            } else {
                "efibootmgr".into()
            })
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
        return Err(if message.is_empty()
            || message.to_lowercase().contains("permission")
            || message.contains("权限")
        {
            BootError::PermissionDenied
        } else {
            BootError::CommandFailed(message)
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

impl BootManager for LinuxBootManager {
    fn get_boot_info(&self) -> BootResult<BootInfo> {
        if !std::path::Path::new("/sys/firmware/efi").exists() {
            return Err(BootError::NotUefi);
        }
        let text = output_text(run_efibootmgr(&["-v"], false)?)?;
        let (boot_current, boot_next, boot_order, entries) = parse_efibootmgr(&text)?;
        Ok(BootInfo {
            current_os: current_os_name(),
            uefi: true,
            secure_boot: secure_boot_state(),
            boot_current,
            boot_next,
            boot_order,
            entries,
        })
    }

    fn set_boot_next(&self, id: &str) -> BootResult<()> {
        if !valid_linux_id(id) {
            return Err(BootError::InvalidBootId(id.into()));
        }
        let output = run_efibootmgr(&["-n", id], true)?;
        output_text(output).map(|_| ())
    }

    fn set_default_boot(&self, id: &str) -> BootResult<()> {
        if !valid_linux_id(id) {
            return Err(BootError::InvalidBootId(id.into()));
        }
        let info = self.get_boot_info()?;
        if !info.boot_order.iter().any(|entry| entry == id) {
            return Err(BootError::BootEntryNotFound(id.into()));
        }
        let mut order = Vec::with_capacity(info.boot_order.len());
        order.push(id.to_uppercase());
        order.extend(info.boot_order.into_iter().filter(|entry| entry != id));
        let order = order.join(",");
        let output = run_efibootmgr(&["-o", &order], true)?;
        output_text(output).map(|_| ())
    }

    fn reboot(&self) -> BootResult<()> {
        let output = crate::privilege::linux::run_privileged("systemctl", &["reboot"])
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

#[allow(dead_code)]
fn _entries_are_send(_: &[BootEntry]) {}
