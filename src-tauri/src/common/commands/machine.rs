#[tauri::command]
pub async fn get_machine_hash() -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::process::Command;
    #[cfg(target_os = "windows")]
    fn get_board_sn() -> Option<String> {
        let output = Command::new("wmic")
            .args(["baseboard", "get", "serialnumber"])
            .output()
            .ok()?;
        String::from_utf8(output.stdout)
            .ok()?
            .lines()
            .nth(1)
            .map(|s| s.trim().to_string())
    }
    #[cfg(target_os = "linux")]
    fn get_board_sn() -> Option<String> {
        let output = Command::new("cat")
            .arg("/sys/class/dmi/id/board_serial")
            .output()
            .ok()?;
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    #[cfg(target_os = "macos")]
    fn get_board_sn() -> Option<String> {
        let output = Command::new("ioreg").args(&["-l"]).output().ok()?;
        let out = String::from_utf8_lossy(&output.stdout);
        out.lines()
            .find(|line| line.contains("IOPlatformSerialNumber"))
            .and_then(|line| line.split('=').nth(1))
            .map(|s| s.replace("\"", "").trim().to_string())
    }
    fn get_mac() -> Option<String> {
        use mac_address::get_mac_address;
        match get_mac_address() {
            Ok(Some(ma)) => Some(ma.to_string()),
            _ => None,
        }
    }
    let board_sn = get_board_sn().unwrap_or_default();
    let mac = get_mac().unwrap_or_default();
    if board_sn.is_empty() && mac.is_empty() {
        return Err("无法获取主板序列号和MAC地址".to_string());
    }
    let raw = format!("{}-{}", board_sn, mac);
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = format!("{:x}", hash);
    Ok(hash_hex)
}
