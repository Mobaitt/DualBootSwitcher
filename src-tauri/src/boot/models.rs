use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootEntry {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub path: Option<String>,
    pub device: Option<String>,
    pub active: bool,
    pub current: bool,
    pub next: bool,
    pub order: Option<u32>,
    pub entry_type: BootEntryType,
    pub suspected_invalid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BootEntryType {
    Windows,
    Linux,
    Removable,
    Network,
    Firmware,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootInfo {
    pub current_os: String,
    pub uefi: bool,
    pub secure_boot: Option<bool>,
    pub boot_current: Option<String>,
    pub boot_next: Option<String>,
    pub boot_order: Vec<String>,
    pub entries: Vec<BootEntry>,
}
