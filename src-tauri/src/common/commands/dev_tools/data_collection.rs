use crate::http_client;
use crate::infrastructure::match_management::matches::service;
/// 开发期数据收集命令
///
/// 用于生成分析数据文件，帮助优化算法
use crate::shared::utils::lcu_get;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchPageProbeResult {
    pub first_range: String,
    pub second_range: String,
    pub first_ids: Vec<u64>,
    pub second_ids: Vec<u64>,
    pub overlap_ids: Vec<u64>,
    pub second_equals_first_prefix: bool,
}

/// 直接请求两段原生 LCU 战绩范围，用 gameId 验证 begIndex 是否生效。
#[tauri::command]
pub async fn probe_match_history_pages(
    first_begin: u32,
    first_end: u32,
    second_begin: u32,
    second_end: u32,
) -> Result<MatchPageProbeResult, String> {
    if first_begin > first_end || second_begin > second_end {
        return Err("起始索引不能大于结束索引".to_string());
    }
    if first_end > 99 || second_end > 99 {
        return Err("测试索引不能超过 99".to_string());
    }

    let client = http_client::get_lcu_client();
    let summoner: Value = lcu_get(client, "/lol-summoner/v1/current-summoner").await?;
    let puuid = summoner
        .get("puuid")
        .and_then(Value::as_str)
        .ok_or_else(|| "当前召唤师响应中没有 puuid".to_string())?;

    let fetch_ids = |begin: u32, end: u32| async move {
        let path = format!(
            "/lol-match-history/v1/products/lol/{}/matches?begIndex={}&endIndex={}",
            puuid, begin, end
        );
        let response: Value = lcu_get(client, &path).await?;
        Ok::<Vec<u64>, String>(
            response
                .get("games")
                .and_then(|games| games.get("games"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|game| game.get("gameId").and_then(Value::as_u64))
                .collect(),
        )
    };

    let first_ids = fetch_ids(first_begin, first_end).await?;
    let second_ids = fetch_ids(second_begin, second_end).await?;
    let first_set: HashSet<u64> = first_ids.iter().copied().collect();
    let overlap_ids = second_ids
        .iter()
        .copied()
        .filter(|id| first_set.contains(id))
        .collect::<Vec<_>>();
    let second_equals_first_prefix = second_ids == first_ids.iter().take(second_ids.len()).copied().collect::<Vec<_>>();

    Ok(MatchPageProbeResult {
        first_range: format!("{}-{}", first_begin, first_end),
        second_range: format!("{}-{}", second_begin, second_end),
        first_ids,
        second_ids,
        overlap_ids,
        second_equals_first_prefix,
    })
}

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

/// 原始LCU对局数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMatchData {
    /// 对局列表的原始JSON数据
    pub match_list_json: Value,
    /// 对局详情的原始JSON数据
    pub match_detail_json: Option<Value>,
    /// 对局时间线的原始JSON数据 🔥 新增
    pub match_timeline_json: Option<Value>,
    /// 游戏ID（用于快速访问）
    pub game_id: u64,
    /// 队列ID（用于快速过滤）
    pub queue_id: i32,
}

/// 原始数据收集结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RawDataCollectionResult {
    /// 原始对局数据
    pub raw_matches: Vec<RawMatchData>,
    /// 收集时间
    pub collection_time: String,
    /// 对局数量
    pub total_matches: usize,
    /// 队列分布
    pub queue_distribution: HashMap<String, usize>,
    /// 时间范围
    pub time_range: TimeRange,
}

/// 时间范围
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeRange {
    /// 最早对局时间
    pub earliest_match: i64,
    /// 最晚对局时间
    pub latest_match: i64,
    /// 时间跨度（天）
    pub span_days: f64,
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
    let queues_to_test = if let Some(queue_id) = queue_id {
        vec![queue_id]
    } else {
        vec![420, 440, 450, 700] // 单排、灵活、大乱斗、排位
    };

    for qid in queues_to_test {
        println!("📊 收集队列 {} 的数据...", qid);

        match service::get_match_history(client, game_count as usize, Some(qid), None).await {
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
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                median: 0.0,
                q25: 0.0,
                q75: 0.0,
                std_dev: 0.0,
            },
            kda_distribution: DistributionStats {
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                median: 0.0,
                q25: 0.0,
                q75: 0.0,
                std_dev: 0.0,
            },
            games_distribution: DistributionStats {
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                median: 0.0,
                q25: 0.0,
                q75: 0.0,
                std_dev: 0.0,
            },
            queue_stats: HashMap::new(),
        }
    };

    // 创建结果
    let total_points = data_points.len();
    let result = DataCollectionResult {
        data_points,
        collection_time: chrono::Utc::now().to_rfc3339(),
        total_points,
        summary,
    };

    // 保存到文件
    let filename = format!("analysis_data_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let filepath = format!("./{}", filename);

    let json_content = serde_json::to_string_pretty(&result).map_err(|e| format!("JSON序列化失败: {}", e))?;

    fs::write(&filepath, json_content).map_err(|e| format!("文件写入失败: {}", e))?;

    println!("📁 数据文件已保存: {}", filepath);
    println!("📊 收集了 {} 个数据点", result.total_points);

    Ok(format!(
        "数据文件已生成: {} ({} 个数据点)",
        filename, result.total_points
    ))
}

/// 生成数据统计摘要
fn generate_data_summary(data_points: &[AnalysisDataPoint]) -> DataSummary {
    let mut win_rates: Vec<f64> = data_points.iter().map(|dp| dp.win_rate).collect();
    let mut kdas: Vec<f64> = data_points.iter().map(|dp| dp.avg_kda).collect();
    let mut games: Vec<f64> = data_points.iter().map(|dp| dp.total_games as f64).collect();

    // 排序用于计算分位数
    win_rates.sort_by(f64::total_cmp);
    kdas.sort_by(f64::total_cmp);
    games.sort_by(f64::total_cmp);

    // 计算统计信息
    let win_rate_dist = calculate_distribution_stats(&win_rates);
    let kda_dist = calculate_distribution_stats(&kdas);
    let games_dist = calculate_distribution_stats(&games);

    // 按队列统计
    let mut queue_stats = HashMap::new();
    let mut queue_groups: HashMap<i32, Vec<&AnalysisDataPoint>> = HashMap::new();

    for dp in data_points {
        if let Some(qid) = dp.queue_id {
            queue_groups.entry(qid).or_default().push(dp);
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

        queue_stats.insert(
            qid.to_string(),
            QueueStats {
                queue_name,
                count: points.len(),
                avg_win_rate,
                avg_kda,
                avg_games,
            },
        );
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
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            median: 0.0,
            q25: 0.0,
            q75: 0.0,
            std_dev: 0.0,
        };
    }

    let min = values[0];
    let max = values[values.len() - 1];
    let mean = values.iter().sum::<f64>() / values.len() as f64;

    let median = if values.len().is_multiple_of(2) {
        (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
    } else {
        values[values.len() / 2]
    };

    let q25_idx = (values.len() as f64 * 0.25) as usize;
    let q75_idx = (values.len() as f64 * 0.75) as usize;
    let q25 = values[q25_idx.min(values.len() - 1)];
    let q75 = values[q75_idx.min(values.len() - 1)];

    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
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

    let content = fs::read_to_string(&file_path).map_err(|e| format!("读取文件失败: {}", e))?;

    let result: DataCollectionResult = serde_json::from_str(&content).map_err(|e| format!("JSON解析失败: {}", e))?;

    // 生成分析报告
    let mut report = String::new();
    report.push_str("📊 数据文件分析报告\n");
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

/// 收集原始LCU对局数据
#[tauri::command]
pub async fn collect_raw_match_data(
    count: Option<u32>,
    queue_id: Option<i32>,
    _include_timeline: Option<bool>,
) -> Result<String, String> {
    println!("🔬 开始收集原始LCU对局数据...");

    let client = http_client::get_lcu_client();
    let game_count = count.unwrap_or(50);

    // 第1步：获取当前召唤师信息
    println!("📍 第1步：获取当前召唤师信息");
    let summoner_data: Value = lcu_get(client, "/lol-summoner/v1/current-summoner")
        .await
        .map_err(|e| format!("获取召唤师信息失败: {}", e))?;

    let puuid = summoner_data
        .get("puuid")
        .and_then(|p| p.as_str())
        .ok_or_else(|| "未找到PUUID".to_string())?;

    let redacted = if puuid.len() > 8 {
        format!("{}…{}", &puuid[..4], &puuid[puuid.len() - 4..])
    } else {
        "***".to_string()
    };
    println!("🆔 提取到的PUUID: {}", redacted);

    // 第2步：单次获取指定数量的对局
    println!("📍 第2步：获取对局列表");
    let match_list_data =
        crate::infrastructure::match_management::matches::service::fetch_match_list(client, puuid, game_count as usize)
            .await
            .map_err(|e| format!("获取对局列表失败: {}", e))?;

    // 第3步：解析对局列表
    println!("📍 第3步：解析对局列表");
    let games_array = match_list_data
        .get("games")
        .and_then(|g| g.get("games"))
        .and_then(|g| g.as_array())
        .ok_or_else(|| "无法解析对局列表".to_string())?;

    println!("📊 找到 {} 场对局", games_array.len());

    // 第4步：收集原始数据
    let mut raw_matches = Vec::new();
    let mut queue_distribution: HashMap<String, usize> = HashMap::new();
    let mut earliest_match = i64::MAX;
    let mut latest_match = 0i64;

    for (index, game) in games_array.iter().enumerate() {
        if let Some(qid) = queue_id {
            let game_queue_id = game.get("queueId").and_then(|q| q.as_i64()).unwrap_or(0);
            if game_queue_id != qid as i64 {
                continue; // 跳过不符合队列过滤的对局
            }
        }

        println!("📋 处理第 {} 场对局...", index + 1);

        let game_id = game.get("gameId").and_then(|g| g.as_u64()).unwrap_or(0);
        let queue_id = game.get("queueId").and_then(|g| g.as_i64()).unwrap_or(0) as i32;

        // 获取对局详情数据
        let match_detail = if game_id > 0 {
            println!("🔍 获取对局详情: gameId={}", game_id);
            match get_match_detail(client, game_id).await {
                Ok(detail) => {
                    println!("✅ 对局详情获取成功: gameId={}", game_id);
                    Some(detail)
                }
                Err(e) => {
                    println!("⚠️ 对局详情获取失败: gameId={}, error={}", game_id, e);
                    None
                }
            }
        } else {
            None
        };

        // 🔥 获取对局时间线数据（关键！）
        let match_timeline = if game_id > 0 {
            println!("🔍 获取对局时间线: gameId={}", game_id);
            match get_match_timeline(client, game_id).await {
                Ok(timeline) => {
                    println!("✅ 对局时间线获取成功: gameId={}", game_id);
                    Some(timeline)
                }
                Err(e) => {
                    println!("⚠️ 对局时间线获取失败: gameId={}, error={}", game_id, e);
                    None
                }
            }
        } else {
            None
        };

        let raw_match = RawMatchData {
            match_list_json: game.clone(),       // 保存对局列表的原始JSON
            match_detail_json: match_detail,     // 保存对局详情的原始JSON
            match_timeline_json: match_timeline, // 🔥 保存对局时间线的原始JSON
            game_id,
            queue_id,
        };

        // 更新队列分布
        let queue_name = match raw_match.queue_id {
            420 => "单双排",
            440 => "灵活组排",
            450 => "大乱斗",
            700 => "排位",
            _ => "其他",
        };
        *queue_distribution.entry(queue_name.to_string()).or_insert(0) += 1;

        // 更新时间范围
        if let Some(game_creation) = raw_match.match_list_json.get("gameCreation").and_then(|c| c.as_i64()) {
            if game_creation > 0 {
                earliest_match = earliest_match.min(game_creation);
                latest_match = latest_match.max(game_creation);
            }
        }

        raw_matches.push(raw_match);
    }

    if raw_matches.is_empty() {
        return Err("没有收集到任何对局数据".to_string());
    }

    // 计算时间跨度
    let span_days = if earliest_match != i64::MAX && latest_match > 0 {
        (latest_match - earliest_match) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
    } else {
        0.0
    };

    // 创建结果
    let total_matches = raw_matches.len();
    let result = RawDataCollectionResult {
        raw_matches,
        collection_time: chrono::Utc::now().to_rfc3339(),
        total_matches,
        queue_distribution,
        time_range: TimeRange {
            earliest_match: if earliest_match == i64::MAX { 0 } else { earliest_match },
            latest_match,
            span_days,
        },
    };

    // 保存到文件
    let filename = format!("raw_match_data_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let filepath = format!("./{}", filename);

    let json_content = serde_json::to_string_pretty(&result).map_err(|e| format!("JSON序列化失败: {}", e))?;

    fs::write(&filepath, json_content).map_err(|e| format!("文件写入失败: {}", e))?;

    println!("📁 原始数据文件已保存: {}", filepath);
    println!("📊 收集了 {} 场原始对局", result.total_matches);
    println!("⏰ 时间跨度: {:.1} 天", result.time_range.span_days);
    println!("🎮 队列分布: {:?}", result.queue_distribution);

    Ok(format!(
        "原始数据文件已生成: {} ({} 场对局)",
        filename, result.total_matches
    ))
}

/// 获取单场对局的详细数据
async fn get_match_detail(client: &Client, game_id: u64) -> Result<Value, String> {
    let detail_url = format!("/lol-match-history/v1/games/{}", game_id);
    println!("🌐 请求对局详情URL: {}", detail_url);

    lcu_get(client, &detail_url)
        .await
        .map_err(|e| format!("获取对局详情失败: {}", e))
}

/// 🔥 获取单场对局的时间线数据（关键！）
async fn get_match_timeline(client: &Client, game_id: u64) -> Result<Value, String> {
    let timeline_url = format!("/lol-match-history/v1/game-timelines/{}", game_id);
    println!("🌐 请求对局时间线URL: {}", timeline_url);

    lcu_get(client, &timeline_url)
        .await
        .map_err(|e| format!("获取对局时间线失败: {}", e))
}

/// 分析原始对局数据的时间线特征
#[tauri::command]
pub async fn analyze_raw_match_timeline(file_path: String) -> Result<String, String> {
    println!("📊 分析原始对局数据的时间线特征: {}", file_path);

    let content = fs::read_to_string(&file_path).map_err(|e| format!("读取文件失败: {}", e))?;

    let result: RawDataCollectionResult = serde_json::from_str(&content).map_err(|e| format!("JSON解析失败: {}", e))?;

    // 生成时间线分析报告
    let mut report = String::new();
    report.push_str("📊 原始对局数据时间线分析报告\n");
    report.push_str(&format!("文件: {}\n", file_path));
    report.push_str(&format!("收集时间: {}\n", result.collection_time));
    report.push_str(&format!("对局数量: {}\n", result.total_matches));
    report.push_str(&format!("时间跨度: {:.1} 天\n\n", result.time_range.span_days));

    // 队列分布分析
    report.push_str("🎮 队列分布:\n");
    for (queue_name, count) in &result.queue_distribution {
        let percentage = (*count as f64 / result.total_matches as f64) * 100.0;
        report.push_str(&format!("  {}: {} 场 ({:.1}%)\n", queue_name, count, percentage));
    }

    // 游戏时长分析
    let mut durations: Vec<i32> = result
        .raw_matches
        .iter()
        .filter_map(|m| {
            m.match_list_json
                .get("gameDuration")
                .and_then(|d| d.as_i64())
                .map(|d| d as i32)
        })
        .collect();
    durations.sort();

    if !durations.is_empty() {
        let min_duration = durations[0];
        let max_duration = durations[durations.len() - 1];
        let avg_duration = durations.iter().sum::<i32>() as f64 / durations.len() as f64;
        let median_duration = if durations.len().is_multiple_of(2) {
            (durations[durations.len() / 2 - 1] + durations[durations.len() / 2]) as f64 / 2.0
        } else {
            durations[durations.len() / 2] as f64
        };

        report.push_str("\n⏱️ 游戏时长分析:\n");
        report.push_str(&format!(
            "  最短: {} 秒 ({:.1} 分钟)\n",
            min_duration,
            min_duration as f64 / 60.0
        ));
        report.push_str(&format!(
            "  最长: {} 秒 ({:.1} 分钟)\n",
            max_duration,
            max_duration as f64 / 60.0
        ));
        report.push_str(&format!(
            "  平均: {:.1} 秒 ({:.1} 分钟)\n",
            avg_duration,
            avg_duration / 60.0
        ));
        report.push_str(&format!(
            "  中位数: {:.1} 秒 ({:.1} 分钟)\n",
            median_duration,
            median_duration / 60.0
        ));
    }

    // 游戏模式分析
    let mut game_modes: HashMap<String, usize> = HashMap::new();
    let mut game_types: HashMap<String, usize> = HashMap::new();

    for match_data in &result.raw_matches {
        let game_mode = match_data
            .match_list_json
            .get("gameMode")
            .and_then(|m| m.as_str())
            .unwrap_or("未知")
            .to_string();
        let game_type = match_data
            .match_list_json
            .get("gameType")
            .and_then(|t| t.as_str())
            .unwrap_or("未知")
            .to_string();

        *game_modes.entry(game_mode).or_insert(0) += 1;
        *game_types.entry(game_type).or_insert(0) += 1;
    }

    report.push_str("\n🎯 游戏模式分析:\n");
    for (mode, count) in &game_modes {
        let percentage = (*count as f64 / result.total_matches as f64) * 100.0;
        report.push_str(&format!("  {}: {} 场 ({:.1}%)\n", mode, count, percentage));
    }

    report.push_str("\n🏷️ 游戏类型分析:\n");
    for (game_type, count) in &game_types {
        let percentage = (*count as f64 / result.total_matches as f64) * 100.0;
        report.push_str(&format!("  {}: {} 场 ({:.1}%)\n", game_type, count, percentage));
    }

    // 时间分布分析
    let mut hourly_distribution: HashMap<i32, usize> = HashMap::new();
    for match_data in &result.raw_matches {
        if let Some(game_creation) = match_data.match_list_json.get("gameCreation").and_then(|c| c.as_i64()) {
            if game_creation > 0 {
                let timestamp = game_creation / 1000; // 转换为秒
                let hour = (timestamp / 3600) % 24; // 获取小时
                *hourly_distribution.entry(hour as i32).or_insert(0) += 1;
            }
        }
    }

    report.push_str("\n🕐 游戏时间分布 (24小时):\n");
    for hour in 0..24 {
        let count = hourly_distribution.get(&hour).unwrap_or(&0);
        let percentage = (*count as f64 / result.total_matches as f64) * 100.0;
        let bar = "█".repeat((percentage / 2.0) as usize);
        report.push_str(&format!(
            "  {:02}:00 - {:02}:59: {} ({:.1}%) {}\n",
            hour, hour, count, percentage, bar
        ));
    }

    // 时间线数据分析
    analyze_timeline_data(&result.raw_matches, &mut report);

    println!("{}", report);
    Ok(report)
}

/// 分析时间线数据
fn analyze_timeline_data(raw_matches: &[RawMatchData], report: &mut String) {
    let mut timeline_available_count = 0;
    let mut total_creeps_per_min: Vec<f64> = Vec::new();
    let mut total_gold_per_min: Vec<f64> = Vec::new();
    let mut total_xp_per_min: Vec<f64> = Vec::new();
    let mut lane_distribution: HashMap<String, usize> = HashMap::new();
    let mut role_distribution: HashMap<String, usize> = HashMap::new();

    for match_data in raw_matches {
        if let Some(detail_json) = &match_data.match_detail_json {
            if let Some(participants) = detail_json.get("participants").and_then(|p| p.as_array()) {
                for participant in participants {
                    if let Some(timeline) = participant.get("timeline") {
                        timeline_available_count += 1;

                        // 分析位置信息
                        if let Some(lane) = timeline.get("lane").and_then(|l| l.as_str()) {
                            *lane_distribution.entry(lane.to_string()).or_insert(0) += 1;
                        }
                        if let Some(role) = timeline.get("role").and_then(|r| r.as_str()) {
                            *role_distribution.entry(role.to_string()).or_insert(0) += 1;
                        }

                        // 分析时间线数据
                        if let Some(creeps_per_min) = timeline.get("creepsPerMinDeltas").and_then(|c| c.as_object()) {
                            for (_, value) in creeps_per_min {
                                if let Some(creeps) = value.as_f64() {
                                    total_creeps_per_min.push(creeps);
                                }
                            }
                        }

                        if let Some(gold_per_min) = timeline.get("goldPerMinDeltas").and_then(|g| g.as_object()) {
                            for (_, value) in gold_per_min {
                                if let Some(gold) = value.as_f64() {
                                    total_gold_per_min.push(gold);
                                }
                            }
                        }

                        if let Some(xp_per_min) = timeline.get("xpPerMinDeltas").and_then(|x| x.as_object()) {
                            for (_, value) in xp_per_min {
                                if let Some(xp) = value.as_f64() {
                                    total_xp_per_min.push(xp);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 分析时间线API数据
    let mut timeline_frames_count = 0;
    let mut timeline_events_count = 0;
    let mut has_position_data = 0;
    let mut has_gold_data = 0;
    let mut has_xp_data = 0;
    let mut has_cs_data = 0;

    for match_data in raw_matches {
        if let Some(timeline_json) = &match_data.match_timeline_json {
            if let Some(frames) = timeline_json.get("frames").and_then(|f| f.as_array()) {
                timeline_frames_count += frames.len();

                for frame in frames {
                    // 统计事件
                    if let Some(events) = frame.get("events").and_then(|e| e.as_array()) {
                        timeline_events_count += events.len();
                    }

                    // 统计参与者帧数据
                    if let Some(participant_frames) = frame.get("participantFrames").and_then(|p| p.as_object()) {
                        for (_, pf) in participant_frames {
                            if pf.get("position").is_some() {
                                has_position_data += 1;
                            }
                            if pf.get("totalGold").is_some() {
                                has_gold_data += 1;
                            }
                            if pf.get("xp").is_some() {
                                has_xp_data += 1;
                            }
                            if pf.get("minionsKilled").is_some() {
                                has_cs_data += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    report.push_str("\n⏰ 时间线数据分析:\n");
    report.push_str(&format!("  有详情数据的参与者: {} 个\n", timeline_available_count));
    report.push_str("\n🔥 时间线API数据:\n");
    report.push_str(&format!("  时间线帧数: {} 个\n", timeline_frames_count));
    report.push_str(&format!("  游戏事件数: {} 个\n", timeline_events_count));
    report.push_str(&format!("  位置数据点: {} 个\n", has_position_data));
    report.push_str(&format!("  金币数据点: {} 个\n", has_gold_data));
    report.push_str(&format!("  经验数据点: {} 个\n", has_xp_data));
    report.push_str(&format!("  补刀数据点: {} 个\n", has_cs_data));

    if timeline_available_count > 0 {
        // 位置分布分析
        report.push_str("\n📍 位置分布:\n");
        for (lane, count) in &lane_distribution {
            let percentage = (*count as f64 / timeline_available_count as f64) * 100.0;
            report.push_str(&format!("  {}: {} 次 ({:.1}%)\n", lane, count, percentage));
        }

        report.push_str("\n🎭 角色分布:\n");
        for (role, count) in &role_distribution {
            let percentage = (*count as f64 / timeline_available_count as f64) * 100.0;
            report.push_str(&format!("  {}: {} 次 ({:.1}%)\n", role, count, percentage));
        }

        // 时间线统计
        if !total_creeps_per_min.is_empty() {
            let avg_creeps = total_creeps_per_min.iter().sum::<f64>() / total_creeps_per_min.len() as f64;
            report.push_str("\n📊 补刀数据:\n");
            report.push_str(&format!("  平均每分钟补刀: {:.2}\n", avg_creeps));
            report.push_str(&format!("  数据点数量: {}\n", total_creeps_per_min.len()));
        }

        if !total_gold_per_min.is_empty() {
            let avg_gold = total_gold_per_min.iter().sum::<f64>() / total_gold_per_min.len() as f64;
            report.push_str("\n💰 金币数据:\n");
            report.push_str(&format!("  平均每分钟金币: {:.2}\n", avg_gold));
            report.push_str(&format!("  数据点数量: {}\n", total_gold_per_min.len()));
        }

        if !total_xp_per_min.is_empty() {
            let avg_xp = total_xp_per_min.iter().sum::<f64>() / total_xp_per_min.len() as f64;
            report.push_str("\n⭐ 经验数据:\n");
            report.push_str(&format!("  平均每分钟经验: {:.2}\n", avg_xp));
            report.push_str(&format!("  数据点数量: {}\n", total_xp_per_min.len()));
        }
    } else {
        report.push_str("  ⚠️ 没有找到时间线数据，可能需要调用对局详情接口\n");
    }
}

/// 展示原始JSON数据结构
#[tauri::command]
pub async fn show_raw_json_structure(file_path: String, match_index: Option<usize>) -> Result<String, String> {
    println!("📊 展示原始JSON数据结构: {}", file_path);

    let content = fs::read_to_string(&file_path).map_err(|e| format!("读取文件失败: {}", e))?;

    let result: RawDataCollectionResult = serde_json::from_str(&content).map_err(|e| format!("JSON解析失败: {}", e))?;

    if result.raw_matches.is_empty() {
        return Err("没有找到任何对局数据".to_string());
    }

    let index = match_index.unwrap_or(0).min(result.raw_matches.len() - 1);
    let match_data = &result.raw_matches[index];

    let mut report = String::new();
    report.push_str("📊 原始LCU JSON数据结构分析\n");
    report.push_str(&format!("文件: {}\n", file_path));
    report.push_str(&format!("对局索引: {} / {}\n", index + 1, result.raw_matches.len()));
    report.push_str(&format!("游戏ID: {}\n", match_data.game_id));
    report.push_str(&format!("队列ID: {}\n\n", match_data.queue_id));

    // 展示对局列表JSON的顶级字段
    report.push_str("🔍 对局列表JSON顶级字段:\n");
    if let Some(obj) = match_data.match_list_json.as_object() {
        for (key, value) in obj {
            let value_type = match value {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(arr) => &format!("array[{}]", arr.len()),
                Value::Object(obj) => &format!("object[{}]", obj.len()),
            };
            report.push_str(&format!(
                "  {}: {} ({})\n",
                key,
                value_type,
                if value.is_string() {
                    value.as_str().unwrap_or("").chars().take(50).collect::<String>() + "..."
                } else {
                    value.to_string().chars().take(50).collect::<String>() + "..."
                }
            ));
        }
    }

    // 展示对局详情数据（如果有）
    if let Some(detail_json) = &match_data.match_detail_json {
        report.push_str("\n🎯 对局详情数据:\n");
        if let Some(obj) = detail_json.as_object() {
            for (key, value) in obj {
                let value_type = match value {
                    Value::Null => "null",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Array(arr) => &format!("array[{}]", arr.len()),
                    Value::Object(obj) => &format!("object[{}]", obj.len()),
                };
                report.push_str(&format!("  {}: {}\n", key, value_type));
            }
        }

        // 展示详情中的participants数组
        if let Some(participants) = detail_json.get("participants").and_then(|p| p.as_array()) {
            report.push_str(&format!("\n👥 详情参与者数据 ({} 个):\n", participants.len()));
            if let Some(first_participant) = participants.first() {
                if let Some(participant_obj) = first_participant.as_object() {
                    report.push_str("  第一个参与者的字段:\n");
                    for (key, value) in participant_obj {
                        let value_type = match value {
                            Value::Null => "null",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) => "number",
                            Value::String(_) => "string",
                            Value::Array(arr) => &format!("array[{}]", arr.len()),
                            Value::Object(obj) => &format!("object[{}]", obj.len()),
                        };
                        report.push_str(&format!("    {}: {}\n", key, value_type));
                    }
                }
            }
        }
    } else {
        report.push_str("\n⚠️ 没有对局详情数据\n");
    }

    // 展示对局列表中的participants数组
    if let Some(participants) = match_data
        .match_list_json
        .get("participants")
        .and_then(|p| p.as_array())
    {
        report.push_str(&format!("\n👥 参与者数据 ({} 个):\n", participants.len()));
        if let Some(first_participant) = participants.first() {
            if let Some(participant_obj) = first_participant.as_object() {
                report.push_str("  第一个参与者的字段:\n");
                for (key, value) in participant_obj {
                    let value_type = match value {
                        Value::Null => "null",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) => "number",
                        Value::String(_) => "string",
                        Value::Array(arr) => &format!("array[{}]", arr.len()),
                        Value::Object(obj) => &format!("object[{}]", obj.len()),
                    };
                    report.push_str(&format!("    {}: {}\n", key, value_type));
                }
            }
        }
    }

    // 展示teams数组的详细结构
    if let Some(teams) = match_data.match_list_json.get("teams").and_then(|t| t.as_array()) {
        report.push_str(&format!("\n🏆 队伍数据 ({} 个):\n", teams.len()));
        if let Some(first_team) = teams.first() {
            if let Some(team_obj) = first_team.as_object() {
                report.push_str("  第一个队伍的字段:\n");
                for (key, value) in team_obj {
                    let value_type = match value {
                        Value::Null => "null",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) => "number",
                        Value::String(_) => "string",
                        Value::Array(arr) => &format!("array[{}]", arr.len()),
                        Value::Object(obj) => &format!("object[{}]", obj.len()),
                    };
                    report.push_str(&format!("    {}: {}\n", key, value_type));
                }
            }
        }
    }

    // 展示完整的原始JSON（格式化）
    report.push_str("\n📄 完整对局列表JSON (格式化):\n");
    let pretty_json =
        serde_json::to_string_pretty(&match_data.match_list_json).map_err(|e| format!("JSON格式化失败: {}", e))?;
    report.push_str(&pretty_json);

    // 如果有对局详情数据，也展示
    if let Some(detail_json) = &match_data.match_detail_json {
        report.push_str("\n📄 完整对局详情JSON (格式化):\n");
        let detail_pretty_json =
            serde_json::to_string_pretty(detail_json).map_err(|e| format!("详情JSON格式化失败: {}", e))?;
        report.push_str(&detail_pretty_json);
    }

    // 🔥 如果有对局时间线数据，也展示
    if let Some(timeline_json) = &match_data.match_timeline_json {
        report.push_str("\n\n🔥 完整对局时间线JSON (格式化):\n");
        let timeline_pretty_json =
            serde_json::to_string_pretty(timeline_json).map_err(|e| format!("时间线JSON格式化失败: {}", e))?;
        report.push_str(&timeline_pretty_json);

        // 分析时间线数据结构
        report.push_str("\n\n🔍 时间线数据结构分析:\n");
        if let Some(frames) = timeline_json.get("frames").and_then(|f| f.as_array()) {
            report.push_str(&format!("  总帧数: {} 帧\n", frames.len()));

            if let Some(first_frame) = frames.first() {
                // 分析第一帧
                if let Some(participant_frames) = first_frame.get("participantFrames").and_then(|p| p.as_object()) {
                    report.push_str(&format!("  参与者数: {} 个\n", participant_frames.len()));

                    if let Some((_, first_pf)) = participant_frames.iter().next() {
                        report.push_str("\n  参与者帧数据字段:\n");
                        if let Some(obj) = first_pf.as_object() {
                            for (key, _) in obj {
                                report.push_str(&format!("    • {}\n", key));
                            }
                        }
                    }
                }

                if let Some(events) = first_frame.get("events").and_then(|e| e.as_array()) {
                    report.push_str(&format!("\n  第一帧事件数: {} 个\n", events.len()));
                }
            }
        }
    } else {
        report.push_str("\n\n⚠️ 没有对局时间线数据\n");
    }

    println!("{}", report);
    Ok(report)
}
