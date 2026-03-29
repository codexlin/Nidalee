use crate::infrastructure::game_session::auth::service::{
    ensure_valid_auth_info, invalidate_auth_info, validate_auth_connection,
};
use crate::shared::types::{ConnectionState, LcuAuthInfo};
use crate::shared::utils::OptimizedPollingManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/ConnectionInfo.ts")]
pub struct ConnectionInfo {
    pub state: ConnectionState,
    #[ts(skip)]
    #[serde(skip)]
    pub auth_info: Option<LcuAuthInfo>,
    #[ts(skip)]
    #[serde(skip)]
    pub last_successful_connection: Option<Instant>,
    pub consecutive_failures: u32,
    pub error_message: Option<String>,
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            auth_info: None,
            last_successful_connection: None,
            consecutive_failures: 0,
            error_message: None,
        }
    }
}

pub struct ConnectionManager {
    info: Arc<RwLock<ConnectionInfo>>,
    app: AppHandle,
    polling_manager: OptimizedPollingManager,
}

impl ConnectionManager {
    pub fn new(app: AppHandle) -> Self {
        let polling_manager = OptimizedPollingManager::new(app.clone());
        Self {
            info: Arc::new(RwLock::new(ConnectionInfo::default())),
            app,
            polling_manager,
        }
    }

    /// 启动连接监控
    pub async fn start_monitoring(&self) {
        let info = self.info.clone();
        let app = self.app.clone();
        let polling_manager = self.polling_manager.clone();

        tokio::spawn(async move {
            let mut manager = ConnectionManager {
                info,
                app: app.clone(),
                polling_manager: polling_manager.clone(),
            };
            manager.monitor_loop().await;
        });

        // ❌ 禁用 OptimizedPollingManager（功能已被 WebSocket 完全覆盖）
        // 原因：
        // 1. 它发送的 summoner-change 事件前端不监听
        // 2. 它的连接检查与 ConnectionManager 重复
        // 3. 召唤师信息由前端主动调用 invoke('get_current_summoner') 获取
        // 预期效果：CPU 降低 2-3%，内存降低 20-30MB
        // self.polling_manager.start().await;

        log::info!(
            target: "connection::service",
            "Connection monitoring started (OptimizedPollingManager disabled, using WebSocket + manual fetch)"
        );
    }

    async fn monitor_loop(&mut self) {
        let mut last_state = ConnectionState::Disconnected;

        loop {
            // ⚡ 优化：如果 WebSocket 已连接，跳过轮询检查
            // WebSocket 连接正常说明客户端肯定在运行，不需要额外验证
            use crate::infrastructure::real_time::websocket::service::is_ws_connected;
            if is_ws_connected() {
                log::debug!(
                    target: "connection::service",
                    "WebSocket is connected, skipping connection check (redundant)"
                );
                // WS 连接时，降低检查频率（30秒检查一次 WS 状态）
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }

            // WebSocket 未连接，执行正常的轮询检查
            let current_state = self.check_connection_state().await;

            // 状态变化时通知前端
            if current_state != last_state {
                self.handle_state_change(&last_state, &current_state).await;
                last_state = current_state.clone();
            }

            // 根据当前状态决定下次检查间隔
            let sleep_duration = self.get_check_interval(&current_state).await;
            tokio::time::sleep(sleep_duration).await;
        }
    }

    pub async fn check_connection_state(&self) -> ConnectionState {
        // 1. 尝试获取认证信息
        let auth_info = ensure_valid_auth_info();

        if auth_info.is_none() {
            // 无认证信息，检查是否有进程
            if self.has_lol_process().await {
                self.update_info(|info| {
                    info.state = ConnectionState::ProcessFound;
                    info.auth_info = None;
                    info.consecutive_failures += 1;
                    info.error_message = Some("客户端进程已启动但无法获取认证信息".to_string());
                })
                .await;
                return ConnectionState::ProcessFound;
            } else {
                self.update_info(|info| {
                    info.state = ConnectionState::Disconnected;
                    info.auth_info = None;
                    info.consecutive_failures = 0;
                    info.error_message = Some("未检测到客户端进程".to_string());
                })
                .await;
                return ConnectionState::Disconnected;
            }
        }

        let auth = auth_info.unwrap();

        // 2. 验证连接是否真正可用
        // ✅ 优化：如果上次连接成功且时间不长，跳过验证（WebSocket 已在监控）
        let should_validate = {
            let info = self.info.read().await;
            info.state != ConnectionState::Connected
                || info
                    .last_successful_connection
                    .map_or(true, |last| last.elapsed() > Duration::from_secs(60))
        };

        let connection_valid = if should_validate {
            validate_auth_connection(&auth).await
        } else {
            // 跳过验证，信任 WebSocket 的监控
            log::debug!(
                target: "connection::service",
                "Skipping validation (recently verified, WebSocket active)"
            );
            true
        };

        if connection_valid {
            self.update_info(|info| {
                info.state = ConnectionState::Connected;
                info.auth_info = Some(auth.clone());
                info.last_successful_connection = Some(Instant::now());
                info.consecutive_failures = 0;
                info.error_message = None;
            })
            .await;

            ConnectionState::Connected
        } else {
            // 连接失败，需要再次检查进程是否还存在
            if !self.has_lol_process().await {
                // 进程已经不存在，说明客户端已退出
                self.update_info(|info| {
                    info.state = ConnectionState::Disconnected;
                    info.auth_info = None;
                    info.consecutive_failures = 0;
                    info.error_message = Some("客户端进程已退出".to_string());
                })
                .await;
                return ConnectionState::Disconnected;
            }

            // 进程存在但连接失败，判断是网络问题还是认证过期
            let current_failures = {
                let info = self.info.read().await;
                info.consecutive_failures
            };

            if current_failures > 5 {
                self.update_info(|info| {
                    info.state = ConnectionState::AuthExpired;
                    info.consecutive_failures += 1;
                    info.error_message = Some("认证信息可能已过期，需要重新获取".to_string());
                })
                .await;
                ConnectionState::AuthExpired
            } else {
                self.update_info(|info| {
                    info.state = ConnectionState::Unstable;
                    info.consecutive_failures += 1;
                    info.error_message = Some("网络连接不稳定或API服务未就绪".to_string());
                })
                .await;
                ConnectionState::Unstable
            }
        }
    }

    /// 检测 LoL 进程是否存在（优化版：记录进程名快速查找）
    async fn has_lol_process(&self) -> bool {
        use once_cell::sync::Lazy;
        use std::sync::{Mutex, RwLock};
        use sysinfo::{ProcessRefreshKind, Pid, RefreshKind, System};
        use std::collections::HashMap;

        // 缓存：上次找到的 LoL 进程名 -> PID
        struct CachedProcess {
            name: String,
            pid: Option<u32>,
        }
        static CACHED_PROCESS: Lazy<RwLock<Option<CachedProcess>>> = Lazy::new(|| RwLock::new(None));

        // 快速路径：检查缓存的进程是否仍然存在
        {
            let cache = CACHED_PROCESS.read().unwrap();
            if let Some(ref cached) = *cache {
                if let Some(pid) = cached.pid {
                    let mut system = PROCESS_SYSTEM.lock().unwrap();
                    // 刷新进程信息
                    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                    // 进程仍然存在，验证名称
                    if let Some(process) = system.process(Pid::from_u32(pid)) {
                        let name = process.name().to_string_lossy();
                        if name.eq_ignore_ascii_case(&cached.name) {
                            log::trace!("[连接管理] 使用缓存的 LoL 进程: {} (PID: {})", name, pid);
                            return true;
                        }
                    }
                }
                // 缓存失效，清除
                drop(cache);
            }
        }

        // 慢速路径：缓存未命中，执行完整扫描
        log::debug!("[连接管理] 扫描 LoL 进程...");
        static PROCESS_SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));

        let mut system = PROCESS_SYSTEM.lock().unwrap();
        system.refresh_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));

        let possible_names = ["LeagueClientUx.exe", "LeagueClient.exe", "LeagueOfLegends.exe"];
        let mut found_info: Option<(String, u32)> = None;

        for (pid, process) in system.processes() {
            let process_name = process.name().to_string_lossy();
            if let Some(matched_name) = possible_names
                .iter()
                .find(|name| process_name.eq_ignore_ascii_case(name))
            {
                found_info = Some((matched_name.to_string(), pid.as_u32()));
                break;
            }
        }

        if let Some((name, pid)) = found_info {
            // 缓存找到的进程信息
            let mut cache = CACHED_PROCESS.write().unwrap();
            *cache = Some(CachedProcess {
                name: name.clone(),
                pid: Some(pid),
            });
            log::debug!("[连接管理] 找到 LoL 进程: {} (PID: {})", name, pid);
            true
        } else {
            // 清除缓存
            let mut cache = CACHED_PROCESS.write().unwrap();
            *cache = None;
            false
        }
    }

    async fn handle_state_change(&self, old_state: &ConnectionState, new_state: &ConnectionState) {
        let message = match (old_state, new_state) {
            (_, ConnectionState::Disconnected) => "客户端已断开连接",
            (ConnectionState::Disconnected, ConnectionState::ProcessFound) => "检测到客户端启动，正在建立连接...",
            (ConnectionState::Unstable, ConnectionState::Connected) => "连接已恢复稳定",
            (_, ConnectionState::Connected) => "客户端连接成功",
            (ConnectionState::Connected, ConnectionState::Unstable) => "连接变得不稳定，正在重试...",
            (_, ConnectionState::AuthExpired) => "认证信息已过期，正在重新获取...",
            _ => "连接状态发生变化",
        };

        log::info!("[连接管理] 状态变化: {:?} -> {:?}, {}", old_state, new_state, message);

        // 发送状态变化事件到前端
        let info = self.info.read().await;
        let _ = self.app.emit("connection-state-changed", &*info);

        // 根据状态变化触发相应的处理
        match new_state {
            ConnectionState::AuthExpired => {
                // 认证过期时清除缓存，强制重新获取
                invalidate_auth_info();
            }
            _ => {
                // 其他状态变化已通过 connection-state-changed 事件通知前端
            }
        }
    }

    async fn get_check_interval(&self, state: &ConnectionState) -> Duration {
        use crate::infrastructure::real_time::websocket::service::is_ws_running;

        // ⚡ 优化：WebSocket 运行时，大幅降低 HTTP 检查频率
        // WebSocket 已经在实时监控连接状态，HTTP 检查只是回退方案
        let ws_running = is_ws_running();

        match state {
            ConnectionState::Connected => {
                // WebSocket 运行时：几乎不需要检查
                // WebSocket 断开时：保持较低频率的回退检查
                if ws_running {
                    Duration::from_secs(120) // 2 分钟（从 60s 增加）
                } else {
                    Duration::from_secs(30)  // WS 未运行时的回退检查
                }
            }
            ConnectionState::Disconnected => {
                // WebSocket 未运行时需要更频繁的检查来发现连接
                if !ws_running && self.consecutive_failures() <= 10 {
                    Duration::from_secs(5)   // 初期：5 秒
                } else {
                    Duration::from_secs(30)  // 长期断开：30 秒
                }
            }
            ConnectionState::ProcessFound => Duration::from_secs(2), // 进程启动，等待认证就绪
            ConnectionState::Unstable => Duration::from_secs(15),
            ConnectionState::AuthExpired => Duration::from_secs(10),
        }
    }

    /// 获取连续失败次数（内部辅助方法）
    fn consecutive_failures(&self) -> u32 {
        // 注意：这里是同步读取，可能略有延迟，但在循环中可以接受
        if let Ok(info) = self.info.try_read() {
            info.consecutive_failures
        } else {
            0
        }
    }

    async fn update_info<F>(&self, updater: F)
    where
        F: FnOnce(&mut ConnectionInfo),
    {
        let mut info = self.info.write().await;
        updater(&mut *info);
    }

    /// 获取当前连接信息
    pub async fn get_connection_info(&self) -> ConnectionInfo {
        self.info.read().await.clone()
    }

    /// 强制重新检查连接
    pub async fn force_refresh(&self) {
        log::info!("[连接管理] 收到强制刷新请求");
        // 清除认证缓存
        invalidate_auth_info();

        // 重置连续失败计数
        self.update_info(|info| {
            info.consecutive_failures = 0;
            info.error_message = None;
        })
        .await;
    }
}
