/// 队列感知的特征分析
///
/// 根据不同队列类型（排位、大乱斗等）提供不同的评估标准和建议
use crate::domains::analysis::queue_config::{get_queue_config, QueueType};
use crate::shared::types::{PlayerMatchStats, SummonerTrait};

/// 队列感知的特征分析
pub fn analyze_queue_aware_traits(stats: &PlayerMatchStats, queue_id: Option<i32>) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    // 如果没有队列ID或数据太少，跳过
    if queue_id.is_none() || stats.total_games < 3 {
        return traits;
    }

    let queue_id = queue_id.unwrap();
    let config = get_queue_config(queue_id);

    // 1. 队列感知的KDA评估
    analyze_queue_kda(&config, stats, &mut traits);

    // 2. 队列感知的伤害评估
    analyze_queue_damage(&config, stats, &mut traits);

    // 3. 队列感知的补刀评估
    analyze_queue_cs(&config, stats, &mut traits);

    // 4. 队列感知的视野评估
    analyze_queue_vision(&config, stats, &mut traits);

    // 5. 队列特定的建议
    generate_queue_specific_advice(&config, stats, &mut traits);

    traits
}

/// 队列感知的KDA评估
fn analyze_queue_kda(
    config: &crate::domains::analysis::queue_config::QueueConfig,
    stats: &PlayerMatchStats,
    traits: &mut Vec<SummonerTrait>,
) {
    let evaluation = config.evaluate_kda(stats.avg_kda);
    let queue_name = config.queue_type.name();

    match evaluation {
        "优秀" => {
            traits.push(SummonerTrait {
                name: format!("{}高手", queue_name),
                description: format!("在{}中KDA优秀（{:.2}），表现出色", queue_name, stats.avg_kda),
                score: (stats.avg_kda * 10.0) as i32,
                trait_type: "good".to_string(),
            });
        }
        "良好" => {
            traits.push(SummonerTrait {
                name: format!("{}熟手", queue_name),
                description: format!("在{}中KDA良好（{:.2}），稳定发挥", queue_name, stats.avg_kda),
                score: (stats.avg_kda * 8.0) as i32,
                trait_type: "neutral".to_string(),
            });
        }
        "较差" | "糟糕" => {
            if stats.avg_deaths >= 7.0 {
                traits.push(SummonerTrait {
                    name: format!("{}磨练中", queue_name),
                    description: format!("在{}中KDA偏低（{:.2}），需要提升", queue_name, stats.avg_kda),
                    score: stats.avg_deaths as i32,
                    trait_type: "bad".to_string(),
                });
            }
        }
        _ => {}
    }
}

/// 队列感知的伤害评估
fn analyze_queue_damage(
    config: &crate::domains::analysis::queue_config::QueueConfig,
    stats: &PlayerMatchStats,
    traits: &mut Vec<SummonerTrait>,
) {
    let evaluation = config.evaluate_damage(stats.dpm);
    let queue_name = config.queue_type.name();

    match evaluation {
        "优秀" => {
            traits.push(SummonerTrait {
                name: "高输出".to_string(),
                description: format!("在{}中输出优秀（{:.0} DPM），carry能力强", queue_name, stats.dpm),
                score: (stats.dpm / 10.0) as i32,
                trait_type: "good".to_string(),
            });
        }
        "良好" => {
            traits.push(SummonerTrait {
                name: "输出稳定".to_string(),
                description: format!("在{}中输出良好（{:.0} DPM）", queue_name, stats.dpm),
                score: (stats.dpm / 12.0) as i32,
                trait_type: "neutral".to_string(),
            });
        }
        "较差" | "糟糕" => {
            traits.push(SummonerTrait {
                name: "输出不足".to_string(),
                description: format!("在{}中输出偏低（{:.0} DPM），需要提升", queue_name, stats.dpm),
                score: (stats.dpm / 10.0) as i32,
                trait_type: "bad".to_string(),
            });
        }
        _ => {}
    }
}

/// 队列感知的补刀评估
fn analyze_queue_cs(
    config: &crate::domains::analysis::queue_config::QueueConfig,
    stats: &PlayerMatchStats,
    traits: &mut Vec<SummonerTrait>,
) {
    let evaluation = config.evaluate_cs(stats.cspm);

    // 大乱斗不评估补刀
    if evaluation == "不适用" {
        return;
    }

    let queue_name = config.queue_type.name();

    match evaluation {
        "优秀" => {
            traits.push(SummonerTrait {
                name: "补刀大师".to_string(),
                description: format!("在{}中补刀优秀（{:.1} CSPM），发育能力强", queue_name, stats.cspm),
                score: (stats.cspm * 10.0) as i32,
                trait_type: "good".to_string(),
            });
        }
        "较差" | "糟糕" => {
            traits.push(SummonerTrait {
                name: "补刀不足".to_string(),
                description: format!("在{}中补刀偏低（{:.1} CSPM），影响发育", queue_name, stats.cspm),
                score: (stats.cspm * 5.0) as i32,
                trait_type: "bad".to_string(),
            });
        }
        _ => {}
    }
}

/// 队列感知的视野评估
fn analyze_queue_vision(
    config: &crate::domains::analysis::queue_config::QueueConfig,
    stats: &PlayerMatchStats,
    traits: &mut Vec<SummonerTrait>,
) {
    let evaluation = config.evaluate_vision(stats.vspm);

    // 大乱斗不评估视野
    if evaluation == "不适用" {
        return;
    }

    let queue_name = config.queue_type.name();

    match evaluation {
        "优秀" => {
            traits.push(SummonerTrait {
                name: "视野大师".to_string(),
                description: format!("在{}中视野优秀（{:.2} VSPM），团队意识强", queue_name, stats.vspm),
                score: (stats.vspm * 50.0) as i32,
                trait_type: "good".to_string(),
            });
        }
        "较差" | "糟糕" => {
            traits.push(SummonerTrait {
                name: "视野不足".to_string(),
                description: format!("在{}中视野偏低（{:.2} VSPM），容易被偷袭", queue_name, stats.vspm),
                score: (stats.vspm * 30.0) as i32,
                trait_type: "bad".to_string(),
            });
        }
        _ => {}
    }
}

/// 生成队列特定的建议
fn generate_queue_specific_advice(
    config: &crate::domains::analysis::queue_config::QueueConfig,
    stats: &PlayerMatchStats,
    traits: &mut Vec<SummonerTrait>,
) {
    let queue_type = config.queue_type;

    match queue_type {
        // 排位赛特定建议
        QueueType::SoloRanked | QueueType::FlexRanked => {
            if stats.win_rate < 50.0 && stats.total_games >= 10 {
                traits.push(SummonerTrait {
                    name: "排位建议".to_string(),
                    description: "胜率偏低，建议专精1-2个英雄，提升对位理解".to_string(),
                    score: 50,
                    trait_type: "neutral".to_string(),
                });
            }

            if stats.cspm < 5.0 && stats.total_games >= 5 {
                traits.push(SummonerTrait {
                    name: "发育建议".to_string(),
                    description: "补刀偏低，建议训练模式练习补刀，提升经济".to_string(),
                    score: 45,
                    trait_type: "neutral".to_string(),
                });
            }

            if stats.vspm < 0.8 && stats.total_games >= 5 {
                traits.push(SummonerTrait {
                    name: "视野建议".to_string(),
                    description: "视野得分偏低，多买真眼，注意做视野和排眼".to_string(),
                    score: 40,
                    trait_type: "neutral".to_string(),
                });
            }
        }

        // 大乱斗特定建议
        QueueType::Aram => {
            if stats.avg_kda < 2.0 && stats.total_games >= 5 {
                traits.push(SummonerTrait {
                    name: "大乱斗建议".to_string(),
                    description: "注意站位，减少无谓死亡，多利用回血包".to_string(),
                    score: 40,
                    trait_type: "neutral".to_string(),
                });
            }

            if stats.avg_deaths >= 10.0 {
                traits.push(SummonerTrait {
                    name: "大乱斗提醒".to_string(),
                    description: "死亡过多，注意不要过于激进，学会拉扯".to_string(),
                    score: 45,
                    trait_type: "neutral".to_string(),
                });
            }
        }

        // 极地大乱斗特定建议
        QueueType::Urf => {
            if stats.avg_kills < 5.0 {
                traits.push(SummonerTrait {
                    name: "URF建议".to_string(),
                    description: "击杀偏少，多利用无限火力的技能优势".to_string(),
                    score: 35,
                    trait_type: "neutral".to_string(),
                });
            }
        }

        _ => {}
    }
}
