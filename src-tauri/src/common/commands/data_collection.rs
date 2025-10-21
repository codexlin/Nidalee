/// 数据收集测试命令
///
/// 用于生成分析数据文件，帮助优化算法
use crate::shared::types::PlayerMatchStats;
use crate::infrastructure::match_management::matches::service;
use crate::http_client;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 分析数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisDataPoint {
    /// 时间戳
    pub timestamp: i64,
    /// 总游戏场次
    pub total_games: u32,
    /// 胜场
    pub wins: u32,
    /// 败场
    pub losses: u32,
    /// 胜率
    pub win_rate: f64,
    /// 平均击杀
    pub avg_kills: f64,
    /// 平均死亡
    pub avg_deaths: f64,
    /// 平均助攻
    pub avg_assists: f64,
    /// 平均KDA
    pub avg_kda: f64,
    /// 今日游戏场次
    pub today_games: u32,
    /// 今日胜场
    pub today_wins: u32,
    /// 每分钟伤害
    pub dpm: f64,
    /// 每分钟补刀
    pub cspm: f64,
    /// 每分钟视野得分
    pub vspm: f64,
    /// 常用英雄数量
    pub favorite_champions_count: usize,
    /// 最近表现场次
    pub recent_performance_count: usize,
    /// 特质数量
    pub traits_count: usize,
    /// 建议数量
    pub advice_count: usize,
    /// 队列ID（用于分析不同模式）
    pub queue_id: Option<i32>,
    /// 玩家段位（如果有的话）
    pub player_tier: Option<String>,
}

/// 数据收集结果
#[derive(Debug, Serialize, Deserialize)]
pub struct DataCollectionResult {
    /// 收集的数据点
    pub data_points: Vec<AnalysisDataPoint>,
    /// 收集时间
    pub collection_time: String,
    /// 数据点数量
    pub total_points: usize,
    /// 统计摘要
    pub summary: DataSummary,
}

/// 数据统计摘要
#[derive(Debug, Serialize, Deserialize)]
pub struct DataSummary {
    /// 胜率分布
    pub win_rate_distribution: DistributionStats,
    /// KDA分布
    pub kda_distribution: DistributionStats,
    /// 游戏场次分布
    pub games_distribution: DistributionStats,
    /// 不同队列的统计
    pub queue_stats: HashMap<String, QueueStats>,
}

/// 分布统计
#[derive(Debug, Serialize, Deserialize)]
pub struct DistributionStats {
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
    /// 平均值
    pub mean: f64,
    /// 中位数
    pub median: f64,
    /// 25%分位数
    pub q25: f64,
    /// 75%分位数
    pub q75: f64,
    /// 标准差
    pub std_dev: f64,
}

/// 队列统计
#[derive(Debug, Serialize, Deserialize)]
pub struct QueueStats {
    /// 队列名称
    pub queue_name: String,
    /// 数据点数量
    pub count: usize,
    /// 平均胜率
    pub avg_win_rate: f64,
    /// 平均KDA
    pub avg_kda: f64,
    /// 平均游戏场次
    pub avg_games: f64,
}

/// 生成测试数据文件
#[tauri::command]
pub async fn generate_test_data_file(
    count: Option<u32>,
    queue_id: Option<i32>,
    include_summary: Option<bool>,
) -> Result<String, String> {
    println!("🔬 开始生成测试数据文件...");

    let client = http_client::get_lcu_client();
    let game_count = count.unwrap_or(20);
    let include_summary = include_summary.unwrap_or(true);

    // 收集数据
    let mut data_points = Vec::new();

    // 尝试收集多个队列的数据
    let queues_to_test = if queue_id.is_some() {
        vec![queue_id.unwrap()]
    } else {
        vec![420, 440, 450, 700] // 单排、灵活、大乱斗、排位
    };

    for qid in queues_to_test {
        println!("📊 收集队列 {} 的数据...", qid);

        match service::get_match_history(client, game_count as usize, Some(qid)).await {
            Ok(stats) => {
                let data_point = AnalysisDataPoint {
                    timestamp: chrono::Utc::now().timestamp(),
                    total_games: stats.total_games,
                    wins: stats.wins,
                    losses: stats.losses,
                    win_rate: stats.win_rate,
                    avg_kills: stats.avg_kills,
                    avg_deaths: stats.avg_deaths,
                    avg_assists: stats.avg_assists,
                    avg_kda: stats.avg_kda,
                    today_games: stats.today_games,
                    today_wins: stats.today_wins,
                    dpm: stats.dpm,
                    cspm: stats.cspm,
                    vspm: stats.vspm,
                    favorite_champions_count: stats.favorite_champions.len(),
                    recent_performance_count: stats.recent_performance.len(),
                    traits_count: stats.traits.len(),
                    advice_count: stats.advice.len(),
                    queue_id: Some(qid),
                    player_tier: None, // 可以后续添加段位信息
                };

                data_points.push(data_point);
                println!("✅ 队列 {} 数据收集完成", qid);
            }
            Err(e) => {
                println!("⚠️ 队列 {} 数据收集失败: {}", qid, e);
            }
        }
    }

    if data_points.is_empty() {
        return Err("没有收集到任何数据".to_string());
    }

    // 生成统计摘要
    let summary = if include_summary {
        generate_data_summary(&data_points)
    } else {
        DataSummary {
            win_rate_distribution: DistributionStats {
                min: 0.0, max: 0.0, mean: 0.0, median: 0.0,
                q25: 0.0, q75: 0.0, std_dev: 0.0,
            },
            kda_distribution: DistributionStats {
                min: 0.0, max: 0.0, mean: 0.0, median: 0.0,
                q25: 0.0, q75: 0.0, std_dev: 0.0,
            },
            games_distribution: DistributionStats {
                min: 0.0, max: 0.0, mean: 0.0, median: 0.0,
                q25: 0.0, q75: 0.0, std_dev: 0.0,
            },
            queue_stats: HashMap::new(),
        }
    };

    // 创建结果
    let result = DataCollectionResult {
        data_points,
        collection_time: chrono::Utc::now().to_rfc3339(),
        total_points: data_points.len(),
        summary,
    };

    // 保存到文件
    let filename = format!("analysis_data_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let filepath = format!("./{}", filename);

    let json_content = serde_json::to_string_pretty(&result)
        .map_err(|e| format!("JSON序列化失败: {}", e))?;

    fs::write(&filepath, json_content)
        .map_err(|e| format!("文件写入失败: {}", e))?;

    println!("📁 数据文件已保存: {}", filepath);
    println!("📊 收集了 {} 个数据点", result.total_points);

    Ok(format!("数据文件已生成: {} ({} 个数据点)", filename, result.total_points))
}

/// 生成数据统计摘要
fn generate_data_summary(data_points: &[AnalysisDataPoint]) -> DataSummary {
    let mut win_rates: Vec<f64> = data_points.iter().map(|dp| dp.win_rate).collect();
    let mut kdas: Vec<f64> = data_points.iter().map(|dp| dp.avg_kda).collect();
    let mut games: Vec<f64> = data_points.iter().map(|dp| dp.total_games as f64).collect();

    // 排序用于计算分位数
    win_rates.sort_by(|a, b| a.partial_cmp(b).unwrap());
    kdas.sort_by(|a, b| a.partial_cmp(b).unwrap());
    games.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // 计算统计信息
    let win_rate_dist = calculate_distribution_stats(&win_rates);
    let kda_dist = calculate_distribution_stats(&kdas);
    let games_dist = calculate_distribution_stats(&games);

    // 按队列统计
    let mut queue_stats = HashMap::new();
    let mut queue_groups: HashMap<i32, Vec<&AnalysisDataPoint>> = HashMap::new();

    for dp in data_points {
        if let Some(qid) = dp.queue_id {
            queue_groups.entry(qid).or_insert_with(Vec::new).push(dp);
        }
    }

    for (qid, points) in queue_groups {
        let queue_name = match qid {
            420 => "单双排".to_string(),
            440 => "灵活组排".to_string(),
            450 => "大乱斗".to_string(),
            700 => "排位".to_string(),
            _ => format!("队列{}", qid),
        };

        let avg_win_rate = points.iter().map(|p| p.win_rate).sum::<f64>() / points.len() as f64;
        let avg_kda = points.iter().map(|p| p.avg_kda).sum::<f64>() / points.len() as f64;
        let avg_games = points.iter().map(|p| p.total_games as f64).sum::<f64>() / points.len() as f64;

        queue_stats.insert(qid.to_string(), QueueStats {
            queue_name,
            count: points.len(),
            avg_win_rate,
            avg_kda,
            avg_games,
        });
    }

    DataSummary {
        win_rate_distribution: win_rate_dist,
        kda_distribution: kda_dist,
        games_distribution: games_dist,
        queue_stats,
    }
}

/// 计算分布统计
fn calculate_distribution_stats(values: &[f64]) -> DistributionStats {
    if values.is_empty() {
        return DistributionStats {
            min: 0.0, max: 0.0, mean: 0.0, median: 0.0,
            q25: 0.0, q75: 0.0, std_dev: 0.0,
        };
    }

    let min = values[0];
    let max = values[values.len() - 1];
    let mean = values.iter().sum::<f64>() / values.len() as f64;

    let median = if values.len() % 2 == 0 {
        (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
    } else {
        values[values.len() / 2]
    };

    let q25_idx = (values.len() as f64 * 0.25) as usize;
    let q75_idx = (values.len() as f64 * 0.75) as usize;
    let q25 = values[q25_idx.min(values.len() - 1)];
    let q75 = values[q75_idx.min(values.len() - 1)];

    let variance = values.iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    let std_dev = variance.sqrt();

    DistributionStats {
        min,
        max,
        mean,
        median,
        q25,
        q75,
        std_dev,
    }
}

/// 分析现有数据文件
#[tauri::command]
pub async fn analyze_data_file(file_path: String) -> Result<String, String> {
    println!("📊 分析数据文件: {}", file_path);

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let result: DataCollectionResult = serde_json::from_str(&content)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    // 生成分析报告
    let mut report = String::new();
    report.push_str(&format!("📊 数据文件分析报告\n"));
    report.push_str(&format!("文件: {}\n", file_path));
    report.push_str(&format!("收集时间: {}\n", result.collection_time));
    report.push_str(&format!("数据点数量: {}\n\n", result.total_points));

    // 胜率分析
    report.push_str("🎯 胜率分析:\n");
    let wr = &result.summary.win_rate_distribution;
    report.push_str(&format!("  范围: {:.1}% - {:.1}%\n", wr.min, wr.max));
    report.push_str(&format!("  平均: {:.1}%\n", wr.mean));
    report.push_str(&format!("  中位数: {:.1}%\n", wr.median));
    report.push_str(&format!("  25%-75%分位: {:.1}% - {:.1}%\n\n", wr.q25, wr.q75));

    // KDA分析
    report.push_str("⚔️ KDA分析:\n");
    let kda = &result.summary.kda_distribution;
    report.push_str(&format!("  范围: {:.2} - {:.2}\n", kda.min, kda.max));
    report.push_str(&format!("  平均: {:.2}\n", kda.mean));
    report.push_str(&format!("  中位数: {:.2}\n", kda.median));
    report.push_str(&format!("  25%-75%分位: {:.2} - {:.2}\n\n", kda.q25, kda.q75));

    // 队列分析
    report.push_str("🎮 队列分析:\n");
    for (qid, stats) in &result.summary.queue_stats {
        report.push_str(&format!("  {} ({}):\n", stats.queue_name, qid));
        report.push_str(&format!("    数据点: {}\n", stats.count));
        report.push_str(&format!("    平均胜率: {:.1}%\n", stats.avg_win_rate));
        report.push_str(&format!("    平均KDA: {:.2}\n", stats.avg_kda));
        report.push_str(&format!("    平均场次: {:.0}\n", stats.avg_games));
    }

    println!("{}", report);
    Ok(report)
}
