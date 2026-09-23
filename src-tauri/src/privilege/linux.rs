//! Run only privileged Linux operations through polkit.
//!
//! The Tauri GUI must stay in the user's desktop session so GTK, WebKit and
//! the tray can access the user's D-Bus session. `pkexec` is used only for
//! commands that modify EFI variables or reboot the machine.

use std::process::{Command, Output};

pub fn run_privileged(program: &str, args: &[&str]) -> std::io::Result<Output> {
    Command::new("pkexec").arg(program).args(args).output()
}
