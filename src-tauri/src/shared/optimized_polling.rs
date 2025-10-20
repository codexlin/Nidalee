// 优化的轮询管理器 - 与 WebSocket 配合使用
use crate::infrastructure::game_session::auth::service::ensure_valid_auth_info;
use crate::infrastructure::data_services::summoner::service::get_current_summoner;
use crate::shared::types::{LcuAuthInfo, SummonerInfo};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct OptimizedPollingManager {
    app: AppHandle,
    client: reqwest::Client,
    state: Arc<RwLock<OptimizedPollingState>>,
    is_running: Arc<RwLock<bool>>,
}

#[derive(Default, Clone)]
struct OptimizedPollingState {
    // 连接状态
    is_connected: bool,
    auth_info: Option<LcuAuthInfo>,

    // 召唤师信息（仍然需要轮询，因为变化频率低）
    current_summoner: Option<SummonerInfo>,
    last_summoner_check: Option<std::time::Instant>,
}

impl OptimizedPollingManager {
    pub fn new(app: AppHandle) -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(8))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            app,
            client,
            state: Arc::new(RwLock::new(OptimizedPollingState::default())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) {
        let mut running = self.is_running.write().await;
        if *running {
            log::warn!("[优化轮询] 轮询已在运行，跳过启动");
            return;
        }
        *running = true;
        drop(running);

        log::info!("[优化轮询] 启动优化轮询管理器（与 WebSocket 配合）");

        let manager = self.clone();
        tokio::spawn(async move {
            manager.run_main_loop().await;
        });
    }

    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
        log::info!("[优化轮询] 停止轮询管理器");
    }

    async fn run_main_loop(&self) {
        let mut connection_check_counter = 0;

        loop {
            // 检查是否应该停止
            if !*self.is_running.read().await {
                break;
            }

            // 每5次循环检查一次连接状态（减少频率）
            connection_check_counter += 1;
            if connection_check_counter >= 5 {
                self.check_connection_status().await;
                connection_check_counter = 0;
            }

            // 只有在连接时才进行业务轮询
            let is_connected = self.state.read().await.is_connected;
            if is_connected {
                self.poll_essential_data().await;
            } else {
                // 未连接时减少轮询频率
                log::debug!("[优化轮询] 未连接，跳过业务轮询");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }

            // 使用更长的轮询间隔，因为大部分数据通过 WebSocket 获取
            tokio::time::sleep(Duration::from_secs(30)).await;
        }

        log::info!("[优化轮询] 主循环已退出");
    }

    async fn check_connection_status(&self) {
        let auth_info = ensure_valid_auth_info();
        let is_connected = auth_info.is_some();

        let mut state = self.state.write().await;

        // 连接状态变化处理
        if state.is_connected != is_connected {
            if is_connected {
                log::info!("[优化轮询] 检测到连接建立");
                state.is_connected = true;
                state.auth_info = auth_info;

                // 连接建立时立即获取召唤师信息
                drop(state);
                self.fetch_summoner_info().await;
            } else {
                log::info!("[优化轮询] 检测到连接断开，清理所有状态");
                state.is_connected = false;
                state.auth_info = None;

                // 清理所有游戏状态
                self.clear_all_state(&mut state).await;
            }
        }
    }

    async fn poll_essential_data(&self) {
        // 只轮询必要的数据，其他数据通过 WebSocket 获取
        self.check_summoner_info().await;
    }

    async fn check_summoner_info(&self) {
        let now = std::time::Instant::now();
        let should_check = {
            let state = self.state.read().await;
            state.current_summoner.is_none()
                || state
                    .last_summoner_check
                    .map_or(true, |last| now.duration_since(last) > Duration::from_secs(60))
        };

        if should_check {
            self.fetch_summoner_info().await;
        }
    }

    async fn fetch_summoner_info(&self) {
        match retry(|| get_current_summoner(&self.client), 2, 1000).await {
            Ok(summoner) => {
                let mut state = self.state.write().await;
                if state.current_summoner.as_ref() != Some(&summoner) {
                    log::info!("[优化轮询] 召唤师信息更新: {}", summoner.display_name);
                    state.current_summoner = Some(summoner.clone());
                    state.last_summoner_check = Some(std::time::Instant::now());
                    let _ = self.app.emit("summoner-change", &Some(summoner));
                }
            }
            Err(e) => {
                log::debug!("[优化轮询] 获取召唤师信息失败: {}", e);
            }
        }
    }

    async fn clear_all_state(&self, state: &mut OptimizedPollingState) {
        // 清理状态
        state.current_summoner = None;
        state.last_summoner_check = None;

        // 发送清理事件
        let _ = self.app.emit("summoner-change", &None::<SummonerInfo>);

        log::info!("[优化轮询] 所有状态已清理并通知前端");
    }
}

// 通用重试工具
async fn retry<F, Fut, T, E>(mut f: F, retries: u32, delay_ms: u64) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut last_err = None;
    for _ in 0..retries {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                last_err = Some(e);
                if retries > 1 {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }
    Err(last_err.unwrap())
}
