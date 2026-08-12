/// 阈值分析器
///
/// 基于真实对局数据统计KDA、DPM、CSPM等指标的分布，
/// 帮助优化算法阈值参数
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

/// 统计摘要
#[derive(Debug, Serialize, Deserialize)]
pub struct StatsSummary {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub p25: f64, // 25分位数
    pub p75: f64, // 75分位数
    pub p90: f64, // 90分位数
    pub p95: f64, // 95分位数
    pub std_dev: f64,
    pub count: usize,
}

/// 阈值分析结果
/// 分析原始数据文件，生成阈值建议
#[tauri::command]
pub async fn analyze_thresholds_from_raw_data(file_path: String) -> Result<String, String> {
    println!("📊 分析阈值参数: {}", file_path);

    // 读取原始数据文件
    let file_content = fs::read_to_string(&file_path).map_err(|e| format!("读取文件失败: {}", e))?;

    let data: Value = serde_json::from_str(&file_content).map_err(|e| format!("解析JSON失败: {}", e))?;

    let raw_matches = data["raw_matches"].as_array().ok_or("找不到raw_matches字段")?;

    println!("📊 找到 {} 场对局", raw_matches.len());

    // 按队列分组统计
    let mut queue_stats: HashMap<i32, Vec<ParticipantStats>> = HashMap::new();

    for match_data in raw_matches {
        let queue_id = match_data["queue_id"].as_i64().unwrap_or(0) as i32;

        // 获取对局详情数据
        if let Some(detail_json) = match_data.get("match_detail_json") {
            if let Some(participants) = detail_json.get("participants").and_then(|p| p.as_array()) {
                let game_duration_secs = detail_json
                    .get("gameDuration")
                    .and_then(|d| d.as_f64())
                    .unwrap_or(1800.0); // 默认30分钟
                let game_duration_mins = game_duration_secs / 60.0;

                if game_duration_mins < 1.0 {
                    continue; // 跳过时长异常的对局
                }

                for participant in participants {
                    if let Some(stats) = extract_participant_stats(participant, game_duration_mins) {
                        queue_stats.entry(queue_id).or_default().push(stats);
                    }
                }
            }
        }
    }

    // 生成报告
    let mut report = String::new();
    report.push_str("╔══════════════════════════════════════════════════════════════════════╗\n");
    report.push_str("║      📊 阈值分析报告（基于真实数据）                                ║\n");
    report.push_str("╚══════════════════════════════════════════════════════════════════════╝\n\n");

    for (queue_id, participants) in queue_stats.iter() {
        let queue_name = match *queue_id {
            420 => "单双排",
            440 => "灵活组排",
            450 => "大乱斗",
            900 => "极地大乱斗",
            430 => "匹配模式",
            _ => "其他",
        };

        report.push_str(&format!("\n═══════ {} (QueueId: {}) ═══════\n", queue_name, queue_id));
        report.push_str(&format!("参与者数量: {} 个\n\n", participants.len()));

        // 统计各项指标
        let kda_stats = calculate_stats(&participants.iter().map(|p| p.kda).collect::<Vec<_>>());
        let dpm_stats = calculate_stats(&participants.iter().map(|p| p.dpm).collect::<Vec<_>>());
        let cspm_stats = calculate_stats(&participants.iter().map(|p| p.cspm).collect::<Vec<_>>());
        let vspm_stats = calculate_stats(&participants.iter().map(|p| p.vspm).collect::<Vec<_>>());

        report.push_str("📊 KDA 分布:\n");
        report.push_str(&format!("  最小值: {:.2}\n", kda_stats.min));
        report.push_str(&format!("  25%分位: {:.2}\n", kda_stats.p25));
        report.push_str(&format!("  中位数: {:.2}\n", kda_stats.median));
        report.push_str(&format!("  平均值: {:.2}\n", kda_stats.mean));
        report.push_str(&format!("  75%分位: {:.2}\n", kda_stats.p75));
        report.push_str(&format!("  90%分位: {:.2}\n", kda_stats.p90));
        report.push_str(&format!("  95%分位: {:.2}\n", kda_stats.p95));
        report.push_str(&format!("  最大值: {:.2}\n", kda_stats.max));
        report.push_str(&format!("  标准差: {:.2}\n\n", kda_stats.std_dev));

        report.push_str("💥 DPM 分布 (每分钟伤害):\n");
        report.push_str(&format!("  最小值: {:.1}\n", dpm_stats.min));
        report.push_str(&format!("  25%分位: {:.1}\n", dpm_stats.p25));
        report.push_str(&format!("  中位数: {:.1}\n", dpm_stats.median));
        report.push_str(&format!("  平均值: {:.1}\n", dpm_stats.mean));
        report.push_str(&format!("  75%分位: {:.1}\n", dpm_stats.p75));
        report.push_str(&format!("  90%分位: {:.1}\n", dpm_stats.p90));
        report.push_str(&format!("  95%分位: {:.1}\n", dpm_stats.p95));
        report.push_str(&format!("  最大值: {:.1}\n", dpm_stats.max));
        report.push_str(&format!("  标准差: {:.1}\n\n", dpm_stats.std_dev));

        report.push_str("🌾 CSPM 分布 (每分钟补刀):\n");
        report.push_str(&format!("  最小值: {:.2}\n", cspm_stats.min));
        report.push_str(&format!("  25%分位: {:.2}\n", cspm_stats.p25));
        report.push_str(&format!("  中位数: {:.2}\n", cspm_stats.median));
        report.push_str(&format!("  平均值: {:.2}\n", cspm_stats.mean));
        report.push_str(&format!("  75%分位: {:.2}\n", cspm_stats.p75));
        report.push_str(&format!("  90%分位: {:.2}\n", cspm_stats.p90));
        report.push_str(&format!("  95%分位: {:.2}\n", cspm_stats.p95));
        report.push_str(&format!("  最大值: {:.2}\n", cspm_stats.max));
        report.push_str(&format!("  标准差: {:.2}\n\n", cspm_stats.std_dev));

        report.push_str("👁️ VSPM 分布 (每分钟视野):\n");
        report.push_str(&format!("  最小值: {:.3}\n", vspm_stats.min));
        report.push_str(&format!("  25%分位: {:.3}\n", vspm_stats.p25));
        report.push_str(&format!("  中位数: {:.3}\n", vspm_stats.median));
        report.push_str(&format!("  平均值: {:.3}\n", vspm_stats.mean));
        report.push_str(&format!("  75%分位: {:.3}\n", vspm_stats.p75));
        report.push_str(&format!("  90%分位: {:.3}\n", vspm_stats.p90));
        report.push_str(&format!("  95%分位: {:.3}\n", vspm_stats.p95));
        report.push_str(&format!("  最大值: {:.3}\n", vspm_stats.max));
        report.push_str(&format!("  标准差: {:.3}\n\n", vspm_stats.std_dev));

        // 生成阈值建议
        report.push_str("💡 建议的阈值设置:\n");
        report.push_str("  KDA:\n");
        report.push_str(&format!("    优秀: {:.1} (90%分位)\n", kda_stats.p90));
        report.push_str(&format!("    良好: {:.1} (75%分位)\n", kda_stats.p75));
        report.push_str(&format!("    一般: {:.1} (中位数)\n", kda_stats.median));
        report.push_str(&format!("    较差: {:.1} (25%分位)\n\n", kda_stats.p25));

        report.push_str("  DPM:\n");
        report.push_str(&format!("    优秀: {:.0} (90%分位)\n", dpm_stats.p90));
        report.push_str(&format!("    良好: {:.0} (75%分位)\n", dpm_stats.p75));
        report.push_str(&format!("    一般: {:.0} (中位数)\n", dpm_stats.median));
        report.push_str(&format!("    较差: {:.0} (25%分位)\n\n", dpm_stats.p25));

        report.push_str("  CSPM:\n");
        report.push_str(&format!("    优秀: {:.1} (90%分位)\n", cspm_stats.p90));
        report.push_str(&format!("    良好: {:.1} (75%分位)\n", cspm_stats.p75));
        report.push_str(&format!("    一般: {:.1} (中位数)\n", cspm_stats.median));
        report.push_str(&format!("    较差: {:.1} (25%分位)\n\n", cspm_stats.p25));

        report.push_str("  VSPM:\n");
        report.push_str(&format!("    优秀: {:.2} (90%分位)\n", vspm_stats.p90));
        report.push_str(&format!("    良好: {:.2} (75%分位)\n", vspm_stats.p75));
        report.push_str(&format!("    一般: {:.2} (中位数)\n", vspm_stats.median));
        report.push_str(&format!("    较差: {:.2} (25%分位)\n\n", vspm_stats.p25));
    }

    println!("{}", report);
    Ok(report)
}

/// 参与者统计数据
struct ParticipantStats {
    kda: f64,
    dpm: f64,
    cspm: f64,
    vspm: f64,
}

/// 从参与者数据中提取统计信息
fn extract_participant_stats(participant: &Value, game_duration_mins: f64) -> Option<ParticipantStats> {
    let stats = participant.get("stats")?;

    let kills = stats.get("kills")?.as_i64()? as f64;
    let deaths = stats.get("deaths")?.as_i64()? as f64;
    let assists = stats.get("assists")?.as_i64()? as f64;
    // KDA计算
    let kda = if deaths > 0.0 {
        (kills + assists) / deaths
    } else {
        kills + assists
    };

    // DPM计算
    let total_damage = stats.get("totalDamageDealtToChampions")?.as_i64()? as f64;
    let dpm = total_damage / game_duration_mins;

    // CSPM计算
    let total_cs = stats.get("totalMinionsKilled")?.as_i64()? as f64;
    let neutral_cs = stats.get("neutralMinionsKilled")?.as_i64().unwrap_or(0) as f64;
    let cspm = (total_cs + neutral_cs) / game_duration_mins;

    // VSPM计算
    let vision_score = stats.get("visionScore")?.as_i64()? as f64;
    let vspm = vision_score / game_duration_mins;

    Some(ParticipantStats { kda, dpm, cspm, vspm })
}

/// 计算统计摘要
fn calculate_stats(values: &[f64]) -> StatsSummary {
    if values.is_empty() {
        return StatsSummary {
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            median: 0.0,
            p25: 0.0,
            p75: 0.0,
            p90: 0.0,
            p95: 0.0,
            std_dev: 0.0,
            count: 0,
        };
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);

    let count = sorted.len();
    let sum: f64 = sorted.iter().sum();
    let mean = sum / count as f64;

    // 计算标准差
    let variance: f64 = sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
    let std_dev = variance.sqrt();

    // 计算分位数
    let percentile = |p: f64| -> f64 {
        let index = (p / 100.0 * (count - 1) as f64) as usize;
        sorted[index.min(count - 1)]
    };

    StatsSummary {
        min: sorted[0],
        max: sorted[count - 1],
        mean,
        median: percentile(50.0),
        p25: percentile(25.0),
        p75: percentile(75.0),
        p90: percentile(90.0),
        p95: percentile(95.0),
        std_dev,
        count,
    }
}
