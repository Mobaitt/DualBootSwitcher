use super::models::{BootEntry, BootEntryType};
use crate::error::{BootError, BootResult};

fn normalize(value: &str) -> String {
    value.trim().trim_matches('\u{feff}').to_lowercase()
}

fn field_value(line: &str, aliases: &[&str]) -> Option<String> {
    let (key, value) = if let Some((key, value)) = line.split_once(':') {
        (key.trim(), value.trim())
    } else {
        let mut parts = line.trim().splitn(2, char::is_whitespace);
        (parts.next()?, parts.next()?.trim())
    };
    let key = normalize(key);
    if aliases
        .iter()
        .any(|alias| key == *alias || key.ends_with(alias))
        && !value.is_empty()
    {
        return Some(value.to_string());
    }
    None
}

fn extract_id(value: &str) -> Option<String> {
    let start = value.find('{')?;
    let end = value[start..].find('}')? + start;
    let id = &value[start..=end];
    if id == "{bootmgr}" || id == "{fwbootmgr}" || valid_windows_id(id) {
        Some(id.to_string())
    } else {
        None
    }
}

fn extract_efi_path(value: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let start = lower.find("\\efi\\")?;
    let path = value[start..]
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|character| character == '"' || character == ',');
    (!path.is_empty()).then(|| path.to_string())
}

pub fn valid_windows_id(id: &str) -> bool {
    if matches!(id, "{bootmgr}" | "{fwbootmgr}") {
        return true;
    }
    let inner = id.strip_prefix('{').and_then(|v| v.strip_suffix('}'));
    inner.is_some_and(|value| {
        value.len() == 36
            && value.chars().enumerate().all(|(index, c)| {
                (index == 8 || index == 13 || index == 18 || index == 23) && c == '-'
                    || (index != 8
                        && index != 13
                        && index != 18
                        && index != 23
                        && c.is_ascii_hexdigit())
            })
    })
}

fn entry_type(name: &str, path: Option<&str>, firmware: bool) -> BootEntryType {
    let haystack = format!("{} {}", name, path.unwrap_or_default()).to_lowercase();
    if firmware {
        return BootEntryType::Firmware;
    }
    if haystack.contains("microsoft")
        || haystack.contains("bootmgfw")
        || haystack.contains("windows")
    {
        BootEntryType::Windows
    } else if haystack.contains("ubuntu")
        || haystack.contains("debian")
        || haystack.contains("fedora")
        || haystack.contains("arch")
        || haystack.contains("linux")
        || haystack.contains("grub")
        || haystack.contains("shim")
    {
        BootEntryType::Linux
    } else if haystack.contains("usb") || haystack.contains("removable") {
        BootEntryType::Removable
    } else if haystack.contains("network") || haystack.contains("pxe") {
        BootEntryType::Network
    } else {
        BootEntryType::Unknown
    }
}

pub fn parse_bcdedit(input: &str) -> BootResult<Vec<BootEntry>> {
    let mut entries = Vec::new();
    let mut title = String::new();
    let mut identifier: Option<String> = None;
    let mut description = None;
    let mut device = None;
    let mut path = None;
    let mut firmware = false;

    let flush = |entries: &mut Vec<BootEntry>,
                 title: &str,
                 identifier: &mut Option<String>,
                 description: &mut Option<String>,
                 device: &mut Option<String>,
                 path: &mut Option<String>,
                 firmware: bool| {
        let Some(id) = identifier.take() else {
            return;
        };
        let display = description
            .take()
            .or_else(|| (!title.trim().is_empty()).then(|| title.trim().to_string()))
            .unwrap_or_else(|| id.clone());
        let invalid = device
            .as_deref()
            .is_some_and(|v| normalize(v).contains("unknown"))
            || path.is_none();
        let entry_path = path.take();
        let entry_device = device.take();
        let kind = entry_type(title, entry_path.as_deref(), firmware);
        entries.push(BootEntry {
            id,
            name: display.clone(),
            description: Some(display),
            path: entry_path,
            device: entry_device,
            active: true,
            current: false,
            next: false,
            order: None,
            entry_type: kind,
            suspected_invalid: invalid,
        });
    };

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.chars().all(|c| c == '-') {
            continue;
        }
        let mut recognized_field = false;
        if let Some(value) = field_value(
            line,
            &["identifier", "标识符", "kennung", "identifiant", "識別子"],
        ) {
            identifier = extract_id(&value);
            recognized_field = true;
        } else if let Some(value) = field_value(
            line,
            &["description", "描述", "说明", "beschreibung", "説明"],
        ) {
            description = Some(value);
            recognized_field = true;
        } else if let Some(value) = field_value(line, &["device", "设备", "gerät", "デバイス"])
        {
            device = Some(value);
            recognized_field = true;
        } else if let Some(value) = field_value(line, &["path", "路径", "pfad", "パス"]) {
            path = Some(value);
            recognized_field = true;
        }
        // bcdedit uses the active Windows code page. On some Chinese, Japanese,
        // and German systems the field label may not survive UTF-8 decoding.
        // The identifier and EFI path are ASCII, so retain them structurally.
        if identifier.is_none() {
            if let Some(value) = extract_id(trimmed) {
                identifier = Some(value);
                recognized_field = true;
            }
        }
        if path.is_none() {
            if let Some(value) = extract_efi_path(trimmed) {
                path = Some(value);
                recognized_field = true;
            }
        }
        if recognized_field {
            continue;
        }
        if !line.starts_with(' ')
            && !line.starts_with('\t')
            && !trimmed.contains(':')
            && !trimmed.starts_with('{')
        {
            flush(
                &mut entries,
                &title,
                &mut identifier,
                &mut description,
                &mut device,
                &mut path,
                firmware,
            );
            title = trimmed.to_string();
            firmware = normalize(trimmed).contains("firmware");
            continue;
        }
    }
    flush(
        &mut entries,
        &title,
        &mut identifier,
        &mut description,
        &mut device,
        &mut path,
        firmware,
    );
    if input.trim().is_empty() {
        return Err(BootError::ParseError("bcdedit 输出为空".into()));
    }
    if entries.is_empty() {
        return Err(BootError::ParseError("未找到有效的固件启动项".into()));
    }
    Ok(entries)
}

pub fn parse_bcdedit_boot_state(input: &str) -> (Vec<String>, Option<String>) {
    let mut boot_order = Vec::new();
    let mut boot_next = None;
    let mut active_field = None;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.chars().all(|character| character == '-') {
            continue;
        }
        if let Some(value) = field_value(
            line,
            &[
                "displayorder",
                "display order",
                "显示顺序",
                "显示次序",
                "启动顺序",
            ],
        ) {
            active_field = Some("order");
            if let Some(id) = extract_id(&value) {
                boot_order.push(id);
            }
            continue;
        }
        if let Some(value) = field_value(
            line,
            &["bootsequence", "boot sequence", "启动序列", "下次启动"],
        ) {
            active_field = Some("next");
            boot_next = extract_id(&value);
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some(id) = extract_id(trimmed) {
                match active_field {
                    Some("order") => boot_order.push(id),
                    Some("next") => boot_next = Some(id),
                    _ => {}
                }
            }
        } else {
            active_field = None;
        }
    }

    (boot_order, boot_next)
}

fn parse_four_hex(value: &str) -> Option<String> {
    let value = value.trim();
    (value.len() == 4 && value.chars().all(|c| c.is_ascii_hexdigit())).then(|| value.to_uppercase())
}

pub fn valid_linux_id(id: &str) -> bool {
    id.len() == 4 && id.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn parse_efibootmgr(
    input: &str,
) -> BootResult<(Option<String>, Option<String>, Vec<String>, Vec<BootEntry>)> {
    let mut current = None;
    let mut next = None;
    let mut order = Vec::new();
    let mut entries = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("BootCurrent:") {
            current = parse_four_hex(value);
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("BootNext:") {
            next = parse_four_hex(value);
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("BootOrder:") {
            order = value.split(',').filter_map(parse_four_hex).collect();
            continue;
        }
        if !trimmed.starts_with("Boot") || trimmed.len() < 8 {
            continue;
        }
        let Some(id) = parse_four_hex(&trimmed[4..8]) else {
            continue;
        };
        let rest = trimmed[8..].trim_start();
        let active = rest.starts_with('*');
        let rest = rest.trim_start_matches('*').trim();
        let (name, path) = if let Some(file_start) = rest.find("File(") {
            let name = rest[..file_start]
                .trim()
                .trim_end_matches('/')
                .trim()
                .to_string();
            let path = rest[file_start + 5..]
                .split(')')
                .next()
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(ToOwned::to_owned);
            (name, path)
        } else {
            (rest.to_string(), None)
        };
        let path_ref = path.as_deref();
        let kind = entry_type(&name, path_ref, false);
        let invalid = path.is_none() || rest.to_lowercase().contains("unknown");
        let current_flag = current.as_deref() == Some(id.as_str());
        let next_flag = next.as_deref() == Some(id.as_str());
        let order_number = order
            .iter()
            .position(|value| value == &id)
            .map(|value| value as u32);
        entries.push(BootEntry {
            id,
            name: if name.is_empty() {
                "未命名启动项".into()
            } else {
                name.clone()
            },
            description: Some(name),
            path,
            device: None,
            active,
            current: current_flag,
            next: next_flag,
            order: order_number,
            entry_type: kind,
            suspected_invalid: invalid,
        });
    }
    if input.trim().is_empty() {
        return Err(BootError::ParseError("efibootmgr 输出为空".into()));
    }
    if entries.is_empty() {
        return Err(BootError::ParseError("未找到有效的 EFI 启动项".into()));
    }
    Ok((current, next, order, entries))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_english_bcdedit_entries() {
        let output = "Firmware Application\n---------------------\nidentifier              {12345678-1234-1234-1234-123456789abc}\ndescription             ubuntu\npath                    \\EFI\\ubuntu\\shimx64.efi\n";
        let entries = parse_bcdedit(output).unwrap();
        assert_eq!(entries[0].id, "{12345678-1234-1234-1234-123456789abc}");
        assert_eq!(
            entries[0].path.as_deref(),
            Some("\\EFI\\ubuntu\\shimx64.efi")
        );
    }

    #[test]
    fn parses_chinese_bcdedit_fields() {
        let output = "固件应用程序\n---------------------\n标识符                  {abcdefab-cdef-abcd-efab-cdefabcdefab}\n说明                    Debian\n路径                    \\EFI\\debian\\shimx64.efi\n";
        let entries = parse_bcdedit(output).unwrap();
        assert_eq!(entries[0].name, "Debian");
        assert!(matches!(entries[0].entry_type, BootEntryType::Linux));
    }

    #[test]
    fn parses_efibootmgr_state_and_paths() {
        let output = "BootCurrent: 0001\nBootNext: 0000\nBootOrder: 0001,0000,0003\nBoot0000* Windows Boot Manager HD(1,GPT)/File(\\EFI\\Microsoft\\Boot\\bootmgfw.efi)\nBoot0001* ubuntu HD(1,GPT)/File(\\EFI\\ubuntu\\shimx64.efi)\nBoot0003  old unknown\n";
        let (current, next, order, entries) = parse_efibootmgr(output).unwrap();
        assert_eq!(current.as_deref(), Some("0001"));
        assert_eq!(next.as_deref(), Some("0000"));
        assert_eq!(order, vec!["0001", "0000", "0003"]);
        assert_eq!(
            entries[0].path.as_deref(),
            Some("\\EFI\\Microsoft\\Boot\\bootmgfw.efi")
        );
        assert!(entries[2].suspected_invalid);
    }

    #[test]
    fn rejects_empty_or_invalid_ids() {
        assert!(parse_efibootmgr("").is_err());
        assert!(!valid_linux_id("00000"));
        assert!(!valid_windows_id("{not-a-guid}"));
    }

    #[test]
    fn keeps_ascii_identifiers_when_field_labels_are_unreadable() {
        let output = "Firmware Application\n---------------------\n????                  {abcdefab-cdef-abcd-efab-cdefabcdefab}\n????                  \\EFI\\ubuntu\\shimx64.efi\n";
        let entries = parse_bcdedit(output).unwrap();
        assert_eq!(entries[0].id, "{abcdefab-cdef-abcd-efab-cdefabcdefab}");
        assert_eq!(
            entries[0].path.as_deref(),
            Some("\\EFI\\ubuntu\\shimx64.efi")
        );
    }

    #[test]
    fn parses_bcdedit_permanent_order_and_boot_sequence() {
        let output = "Firmware Boot Manager\nidentifier {fwbootmgr}\ndisplayorder {abcdefab-cdef-abcd-efab-cdefabcdefab}\n            {bootmgr}\nbootsequence {abcdefab-cdef-abcd-efab-cdefabcdefab}\n";
        let (order, next) = parse_bcdedit_boot_state(output);
        assert_eq!(
            order,
            vec!["{abcdefab-cdef-abcd-efab-cdefabcdefab}", "{bootmgr}"]
        );
        assert_eq!(
            next.as_deref(),
            Some("{abcdefab-cdef-abcd-efab-cdefabcdefab}")
        );
    }
}
