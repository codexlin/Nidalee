//! 错误处理使用示例
//!
//! 本文件展示如何在实际代码中使用统一的错误处理

use super::{NidaleeError, Result};
use reqwest::Client;
use serde_json::Value;

// ============== 示例 1: 基础错误处理 ==============

/// 获取召唤师信息（基础示例）
pub async fn get_summoner_info_basic(client: &Client, summoner_id: i64) -> Result<Value> {
    // 检查客户端状态
    if summoner_id <= 0 {
        return Err(NidaleeError::Validation(
            "召唤师ID必须大于0".to_string()
        ));
    }

    // HTTP 请求（自动错误转换）
    let response = client
        .get(format!("https://example.com/summoner/{}", summoner_id))
        .send()
        .await?;  // reqwest::Error 自动转为 NidaleeError::Http

    // JSON 解析（自动错误转换）
    let data: Value = response.json().await?;  // serde_json::Error 自动转为 NidaleeError::Json

    Ok(data)
}

// ============== 示例 2: 使用便捷构造函数 ==============

/// 连接 LCU 客户端
pub async fn connect_to_lcu() -> Result<String> {
    // 模拟检查 LCU 进程
    let is_running = check_lcu_process();

    if !is_running {
        return Err(NidaleeError::LcuNotFound);
    }

    // 模拟认证
    let auth_token = authenticate_lcu()
        .ok_or_else(|| NidaleeError::lcu_connection("认证失败"))?;

    Ok(auth_token)
}

// ============== 示例 3: API 调用错误 ==============

/// 调用 LCU API
pub async fn call_lcu_api(client: &Client, endpoint: &str) -> Result<Value> {
    let url = format!("https://127.0.0.1:2999{}", endpoint);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| NidaleeError::lcu_api_call(endpoint, e.to_string()))?;

    if !response.status().is_success() {
        return Err(NidaleeError::lcu_api_call(
            endpoint,
            format!("HTTP {}", response.status())
        ));
    }

    let data = response.json().await?;
    Ok(data)
}

// ============== 示例 4: 游戏状态检查 ==============

/// 检查游戏状态
pub fn check_game_state(current_state: &str, required_state: &str) -> Result<()> {
    if current_state != required_state {
        return Err(NidaleeError::invalid_game_state(
            required_state,
            current_state
        ));
    }
    Ok(())
}

/// 确保玩家在游戏中
pub fn ensure_in_game(game_flow: &str) -> Result<()> {
    match game_flow {
        "InProgress" | "GameStart" => Ok(()),
        _ => Err(NidaleeError::NotInGame),
    }
}

// ============== 示例 5: 数据分析错误 ==============

/// 分析玩家数据
pub fn analyze_player_data(games: &[Value]) -> Result<AnalysisResult> {
    // 检查数据是否足够
    if games.len() < 5 {
        return Err(NidaleeError::InsufficientData(
            format!("至少需要5场对局，当前只有{}场", games.len())
        ));
    }

    // 执行分析
    let result = perform_analysis(games)
        .map_err(|e| NidaleeError::analysis(format!("分析失败: {}", e)))?;

    Ok(result)
}

// ============== 示例 6: 缓存操作 ==============

/// 从缓存获取数据
pub fn get_from_cache(key: &str) -> Result<Value> {
    let cache = get_cache();

    cache.get(key)
        .ok_or_else(|| NidaleeError::CacheMiss(key.to_string()))
}

// ============== 示例 7: 错误传播和转换 ==============

/// 复杂操作，涉及多个子操作
pub async fn complex_operation(client: &Client) -> Result<ComplexResult> {
    // 连接 LCU
    let auth = connect_to_lcu().await?;

    // 获取召唤师信息
    let summoner = get_summoner_info_basic(client, 12345).await?;

    // 检查游戏状态
    let game_flow = summoner["gameFlowPhase"].as_str().unwrap_or("None");
    ensure_in_game(game_flow)?;

    // 获取对局数据
    let matches = fetch_matches(client).await?;

    // 分析数据
    let analysis = analyze_player_data(&matches)?;

    Ok(ComplexResult {
        summoner,
        analysis,
    })
}

// ============== 示例 8: Tauri 命令集成 ==============

/// Tauri 命令示例
#[allow(dead_code)]
pub async fn tauri_command_example() -> std::result::Result<String, String> {
    // 内部使用 NidaleeError
    let result = internal_operation().await;

    // 转换为 Tauri 需要的 Result<T, String>
    result.map_err(|e| e.to_string())
}

async fn internal_operation() -> Result<String> {
    // 使用统一的错误处理
    Ok("success".to_string())
}

// ============== 示例 9: 错误匹配和处理 ==============

/// 根据不同错误类型采取不同行动
pub async fn handle_errors(client: &Client) -> Result<()> {
    match call_lcu_api(client, "/lol-summoner/v1/current-summoner").await {
        Ok(_) => println!("✅ 成功"),
        Err(NidaleeError::LcuNotFound) => {
            println!("⚠️ LCU 未启动，请启动游戏客户端");
            // 可以尝试重新连接
        }
        Err(NidaleeError::LcuApiCall { endpoint, message }) => {
            println!("❌ API 调用失败: {} - {}", endpoint, message);
            // 可以重试
        }
        Err(NidaleeError::Http(e)) => {
            println!("❌ 网络错误: {}", e);
            // 检查网络连接
        }
        Err(e) => {
            println!("❌ 未知错误: {}", e);
        }
    }

    Ok(())
}

// ============== 辅助类型和函数（仅用于示例） ==============

#[derive(Debug)]
#[allow(dead_code)]
struct AnalysisResult {
    win_rate: f64,
    kda: f64,
}

#[derive(Debug)]
#[allow(dead_code)]
struct ComplexResult {
    summoner: Value,
    analysis: AnalysisResult,
}

fn check_lcu_process() -> bool {
    // 模拟实现
    true
}

fn authenticate_lcu() -> Option<String> {
    // 模拟实现
    Some("auth_token".to_string())
}

fn perform_analysis(_games: &[Value]) -> std::result::Result<AnalysisResult, String> {
    // 模拟实现
    Ok(AnalysisResult {
        win_rate: 0.55,
        kda: 3.2,
    })
}

fn get_cache() -> std::collections::HashMap<String, Value> {
    // 模拟实现
    std::collections::HashMap::new()
}

async fn fetch_matches(_client: &Client) -> Result<Vec<Value>> {
    // 模拟实现
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let result = check_game_state("Lobby", "ChampSelect");
        assert!(result.is_err());

        match result {
            Err(NidaleeError::InvalidGameState { expected, actual }) => {
                assert_eq!(expected, "ChampSelect");
                assert_eq!(actual, "Lobby");
            }
            _ => panic!("Expected InvalidGameState error"),
        }
    }

    #[test]
    fn test_insufficient_data() {
        let games = vec![]; // 空数组
        let result = analyze_player_data(&games);
        assert!(result.is_err());

        if let Err(NidaleeError::InsufficientData(msg)) = result {
            assert!(msg.contains("至少需要5场"));
        } else {
            panic!("Expected InsufficientData error");
        }
    }
}

