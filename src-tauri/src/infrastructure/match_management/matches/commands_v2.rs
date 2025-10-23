/// v3.4: 新增多位置分组分析Command
///
/// 提供完整的多位置分析结果给前端

use crate::http_client;
use crate::shared::types::MultiPositionAnalysis;

/// 获取多位置分组分析的战绩数据
///
/// 与 get_match_history 的区别：
/// - 返回 MultiPositionAnalysis（包含分位置数据）
/// - 前端可以展示不同位置的详细统计
#[tauri::command]
pub async fn get_match_history_with_positions(
    count: Option<u32>,
    queue_id: Option<i32>,
) -> Result<MultiPositionAnalysis, String> {
    println!("🔢 ===== get_match_history_with_positions 命令被调用 =====");
    println!("📥 接收到的参数:");
    println!("   - count: {:?}", count);
    println!("   - queue_id: {:?}", queue_id);

    let client = http_client::get_lcu_client();

    // 第1步：获取当前召唤师信息
    println!("\n📍 第1步：获取当前召唤师信息");
    let summoner_data: serde_json::Value =
        crate::shared::utils::lcu_get(client, "/lol-summoner/v1/current-summoner").await?;

    let puuid = summoner_data
        .get("puuid")
        .and_then(|p| p.as_str())
        .ok_or_else(|| "未找到PUUID".to_string())?;
    println!("🆔 提取到的PUUID: {}", puuid);

    // 第2步：获取对局列表
    println!("\n📍 第2步：使用PUUID获取对局列表");
    let end_count: usize = count.unwrap_or(20) as usize;
    let safe_end = end_count.min(100);
    let actual_end_index = if safe_end > 0 { safe_end - 1 } else { 0 };

    let match_list_url = format!(
        "/lol-match-history/v1/products/lol/{}/matches?begIndex=0&endIndex={}",
        puuid, actual_end_index
    );
    println!("🌐 请求URL: {}", match_list_url);

    let match_list_data: serde_json::Value =
        crate::shared::utils::lcu_get(client, &match_list_url).await?;

    // 第3步：多位置分组分析
    println!("\n📍 第3步：多位置分组分析");
    let multi_position_result = super::position_analysis::analyze_with_position_grouping(
        match_list_data,
        puuid,
        queue_id,
    )?;

    println!("\n✅ ===== 多位置分析查询完成 =====");
    println!("📊 最终统计结果:");
    println!("   - 总对局: {}", multi_position_result.overall_stats.total_games);
    println!("   - 主要位置: {}", multi_position_result.main_position);
    println!("   - 位置数: {}", multi_position_result.position_stats.len());

    for pos_stat in &multi_position_result.position_stats {
        println!(
            "     • {}: {}场 (胜率{:.1}%，KDA {:.2})",
            pos_stat.position,
            pos_stat.games,
            pos_stat.win_rate,
            pos_stat.stats.avg_kda
        );
    }

    Ok(multi_position_result)
}

