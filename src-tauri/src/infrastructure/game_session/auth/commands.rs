use crate::infrastructure::game_session::auth::service::{AUTH_INFO, verify_lockfile_vs_cmdline as do_verify_lockfile_vs_cmdline};
use crate::shared::types::LcuAuthInfo;

#[tauri::command]
pub fn get_auth_info() -> Option<LcuAuthInfo> {
    let auth = AUTH_INFO.read().unwrap();
    auth.as_ref().cloned()
}

/// 🔍 验证命令：对比 lockfile 和进程命令行获取的信息
/// 前端调用：invoke('verify_lockfile_vs_cmdline')
#[tauri::command]
pub fn verify_lockfile_vs_cmdline() {
    do_verify_lockfile_vs_cmdline();
}
