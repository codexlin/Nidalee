//! 队列特定的配置和阈值
//!
//! 根据不同的队列类型（排位、大乱斗等）提供不同的分析策略和阈值。
//!
//! 这里是**队列语义的唯一来源**：其他模块（含 `evidence`）不得自建队列白名单。
//! 只有 catalog 里语义明确的队列才允许进入 [`QueueType`]，语义不明的一律落到
//! [`QueueType::Other`]，宁可判成「未知」也不要猜错模式。

use serde::{Deserialize, Serialize};

/// 队列类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueType {
    /// 单双排位 (420)
    SoloRanked,
    /// 灵活组排 (440)
    FlexRanked,
    /// 大乱斗 (450)
    Aram,
    /// 极地大乱斗 (900)
    Urf,
    /// 匹配模式 (430)
    Normal,
    /// 其他模式
    Other,
}

impl QueueType {
    /// 从队列ID获取队列类型
    pub fn from_queue_id(queue_id: i32) -> Self {
        match queue_id {
            420 => QueueType::SoloRanked,
            440 => QueueType::FlexRanked,
            450 => QueueType::Aram,
            900 => QueueType::Urf,
            430 => QueueType::Normal,
            _ => QueueType::Other,
        }
    }

    /// 获取队列名称
    pub fn name(&self) -> &'static str {
        match self {
            QueueType::SoloRanked => "单双排",
            QueueType::FlexRanked => "灵活组排",
            QueueType::Aram => "大乱斗",
            QueueType::Urf => "极地大乱斗",
            QueueType::Normal => "匹配模式",
            QueueType::Other => "其他模式",
        }
    }

    /// 是否为排位赛
    #[allow(dead_code)]
    pub fn is_ranked(&self) -> bool {
        matches!(self, QueueType::SoloRanked | QueueType::FlexRanked)
    }

    /// 是否为娱乐模式
    #[allow(dead_code)]
    pub fn is_fun_mode(&self) -> bool {
        matches!(self, QueueType::Aram | QueueType::Urf)
    }

    /// 是否为「无分路」的大乱斗类地图
    ///
    /// 判定标准是**地图没有分路语义**，不是「娱乐模式」：极地大乱斗（URF）在
    /// 召唤师峡谷进行，仍然有上中下野辅，因此不属于这一类。
    /// 队列语义只在这里定义一次，`evidence` 层通过 `is_aram_queue` 复用。
    pub fn is_aram(&self) -> bool {
        matches!(self, QueueType::Aram)
    }
}

/// 队列特定的KDA阈值
pub struct QueueKdaThresholds {
    pub excellent: f64,
    pub good: f64,
    pub average: f64,
    pub poor: f64,
}

impl QueueKdaThresholds {
    /// 获取队列特定的KDA阈值
    pub fn for_queue(queue_type: QueueType) -> Self {
        match queue_type {
            // 排位赛：KDA要求较高
            QueueType::SoloRanked | QueueType::FlexRanked => Self {
                excellent: 4.0, // 优秀
                good: 3.0,      // 良好
                average: 2.0,   // 一般
                poor: 1.5,      // 较差
            },
            // 大乱斗：KDA普遍较高
            QueueType::Aram => Self {
                excellent: 3.5, // 大乱斗KDA普遍高一些
                good: 2.8,
                average: 2.0,
                poor: 1.5,
            },
            // 极地大乱斗：击杀频繁
            QueueType::Urf => Self {
                excellent: 3.0,
                good: 2.5,
                average: 2.0,
                poor: 1.5,
            },
            // 匹配模式：较为宽松
            QueueType::Normal => Self {
                excellent: 3.5,
                good: 2.5,
                average: 2.0,
                poor: 1.5,
            },
            // 其他模式：默认值
            QueueType::Other => Self {
                excellent: 3.5,
                good: 2.5,
                average: 2.0,
                poor: 1.5,
            },
        }
    }
}

/// 队列特定的伤害阈值 (DPM - Damage Per Minute)
pub struct QueueDamageThresholds {
    pub excellent: f64,
    pub good: f64,
    pub average: f64,
    pub poor: f64,
}

impl QueueDamageThresholds {
    /// 获取队列特定的伤害阈值（每分钟伤害）
    pub fn for_queue(queue_type: QueueType) -> Self {
        match queue_type {
            // 排位赛：标准阈值
            QueueType::SoloRanked | QueueType::FlexRanked => Self {
                excellent: 700.0,
                good: 550.0,
                average: 400.0,
                poor: 300.0,
            },
            // 大乱斗：伤害较高（频繁团战）
            QueueType::Aram => Self {
                excellent: 900.0,
                good: 700.0,
                average: 500.0,
                poor: 350.0,
            },
            // 极地大乱斗：伤害极高
            QueueType::Urf => Self {
                excellent: 1200.0,
                good: 900.0,
                average: 650.0,
                poor: 450.0,
            },
            // 匹配模式：标准阈值
            QueueType::Normal => Self {
                excellent: 650.0,
                good: 500.0,
                average: 380.0,
                poor: 280.0,
            },
            // 其他模式：默认值
            QueueType::Other => Self {
                excellent: 650.0,
                good: 500.0,
                average: 380.0,
                poor: 280.0,
            },
        }
    }
}

/// 队列特定的补刀阈值 (CSPM - CS Per Minute)
pub struct QueueCsThresholds {
    pub excellent: f64,
    pub good: f64,
    pub average: f64,
    pub poor: f64,
}

impl QueueCsThresholds {
    /// 获取队列特定的补刀阈值（每分钟补刀）
    pub fn for_queue(queue_type: QueueType) -> Self {
        match queue_type {
            // 排位赛：标准补刀要求
            QueueType::SoloRanked | QueueType::FlexRanked => Self {
                excellent: 7.5,
                good: 6.0,
                average: 5.0,
                poor: 4.0,
            },
            // 大乱斗：没有补刀概念（使用击杀数作为替代）
            QueueType::Aram => Self {
                excellent: 0.0,
                good: 0.0,
                average: 0.0,
                poor: 0.0,
            },
            // 极地大乱斗：补刀较少
            QueueType::Urf => Self {
                excellent: 5.0,
                good: 4.0,
                average: 3.0,
                poor: 2.0,
            },
            // 匹配模式：标准补刀要求
            QueueType::Normal => Self {
                excellent: 7.0,
                good: 5.5,
                average: 4.5,
                poor: 3.5,
            },
            // 其他模式：默认值
            QueueType::Other => Self {
                excellent: 6.5,
                good: 5.0,
                average: 4.0,
                poor: 3.0,
            },
        }
    }
}

/// 队列特定的视野阈值 (VSPM - Vision Score Per Minute)
pub struct QueueVisionThresholds {
    pub excellent: f64,
    pub good: f64,
    pub average: f64,
    pub poor: f64,
}

impl QueueVisionThresholds {
    /// 获取队列特定的视野阈值（每分钟视野得分）
    pub fn for_queue(queue_type: QueueType) -> Self {
        match queue_type {
            // 排位赛：视野很重要
            QueueType::SoloRanked | QueueType::FlexRanked => Self {
                excellent: 1.5,
                good: 1.2,
                average: 0.8,
                poor: 0.5,
            },
            // 大乱斗：视野不重要
            QueueType::Aram => Self {
                excellent: 0.0,
                good: 0.0,
                average: 0.0,
                poor: 0.0,
            },
            // 极地大乱斗：视野较重要
            QueueType::Urf => Self {
                excellent: 1.2,
                good: 0.9,
                average: 0.6,
                poor: 0.4,
            },
            // 匹配模式：视野重要
            QueueType::Normal => Self {
                excellent: 1.4,
                good: 1.1,
                average: 0.7,
                poor: 0.5,
            },
            // 其他模式：默认值
            QueueType::Other => Self {
                excellent: 1.3,
                good: 1.0,
                average: 0.7,
                poor: 0.5,
            },
        }
    }
}

/// 获取队列特定的配置
pub fn get_queue_config(queue_id: i32) -> QueueConfig {
    let queue_type = QueueType::from_queue_id(queue_id);
    QueueConfig {
        queue_type,
        kda_thresholds: QueueKdaThresholds::for_queue(queue_type),
        damage_thresholds: QueueDamageThresholds::for_queue(queue_type),
        cs_thresholds: QueueCsThresholds::for_queue(queue_type),
        vision_thresholds: QueueVisionThresholds::for_queue(queue_type),
    }
}

/// 队列配置
pub struct QueueConfig {
    pub queue_type: QueueType,
    pub kda_thresholds: QueueKdaThresholds,
    pub damage_thresholds: QueueDamageThresholds,
    pub cs_thresholds: QueueCsThresholds,
    pub vision_thresholds: QueueVisionThresholds,
}

impl QueueConfig {
    /// 评估KDA等级
    pub fn evaluate_kda(&self, kda: f64) -> &'static str {
        if kda >= self.kda_thresholds.excellent {
            "优秀"
        } else if kda >= self.kda_thresholds.good {
            "良好"
        } else if kda >= self.kda_thresholds.average {
            "一般"
        } else if kda >= self.kda_thresholds.poor {
            "较差"
        } else {
            "糟糕"
        }
    }

    /// 评估伤害等级
    pub fn evaluate_damage(&self, dpm: f64) -> &'static str {
        if dpm >= self.damage_thresholds.excellent {
            "优秀"
        } else if dpm >= self.damage_thresholds.good {
            "良好"
        } else if dpm >= self.damage_thresholds.average {
            "一般"
        } else if dpm >= self.damage_thresholds.poor {
            "较差"
        } else {
            "糟糕"
        }
    }

    /// 评估补刀等级
    pub fn evaluate_cs(&self, cspm: f64) -> &'static str {
        // 大乱斗没有补刀概念
        if self.queue_type == QueueType::Aram {
            return "不适用";
        }

        if cspm >= self.cs_thresholds.excellent {
            "优秀"
        } else if cspm >= self.cs_thresholds.good {
            "良好"
        } else if cspm >= self.cs_thresholds.average {
            "一般"
        } else if cspm >= self.cs_thresholds.poor {
            "较差"
        } else {
            "糟糕"
        }
    }

    /// 评估视野等级
    pub fn evaluate_vision(&self, vspm: f64) -> &'static str {
        // 大乱斗视野不重要
        if self.queue_type == QueueType::Aram {
            return "不适用";
        }

        if vspm >= self.vision_thresholds.excellent {
            "优秀"
        } else if vspm >= self.vision_thresholds.good {
            "良好"
        } else if vspm >= self.vision_thresholds.average {
            "一般"
        } else if vspm >= self.vision_thresholds.poor {
            "较差"
        } else {
            "糟糕"
        }
    }
}
