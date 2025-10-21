/// 阈值配置模块
///
/// 职责：
/// - 集中管理所有分析阈值
/// - 便于后续调整和实验
/// - 独立于策略模式，可以单独维护

/// 胜率阈值
pub mod win_rate {
    pub const EXCELLENT_RANKED: f64 = 65.0; // 排位大神
    pub const EXCELLENT_OTHER: f64 = 60.0; // 其他模式大神
    pub const GOOD: f64 = 55.0;
    pub const POOR: f64 = 45.0;
}

/// KDA阈值
pub mod kda {
    pub const EXCELLENT_RANKED: f64 = 4.0; // 排位优秀
    pub const EXCELLENT_OTHER: f64 = 3.5; // 其他模式优秀
    pub const GOOD: f64 = 2.5;
    pub const POOR: f64 = 1.5;

    // 单场表现等级
    pub const S_GRADE: f64 = 6.0; // S级：超神表现
    pub const A_GRADE: f64 = 4.0; // A级：优秀表现
    pub const B_GRADE: f64 = 2.5; // B级：良好表现
    pub const D_GRADE: f64 = 1.5; // D级：崩盘表现

    // 死亡阈值
    pub const DEATH_TOO_MANY: f64 = 6.0; // 场均死亡超过6次
    pub const DEATH_EXCELLENT: f64 = 3.0; // 场均死亡低于3次
}

/// 参团率阈值 (KP%)
pub mod kill_participation {
    pub const HIGH: f64 = 0.70;
    pub const LOW: f64 = 0.40;
}

/// 伤害占比阈值
pub mod damage_share {
    pub const HIGH: f64 = 0.30;
    pub const LOW: f64 = 0.15;

    /// 根据位置获取伤害阈值
    pub fn for_role(role: &str) -> (f64, f64) {
        match role {
            "ADC" | "中单" => (0.30, 0.22),
            "上单" => (0.25, 0.16),
            "打野" => (0.22, 0.12),
            "辅助" => (0.0, 0.0),
            _ => (HIGH, LOW),
        }
    }
}

/// 视野得分阈值（每分钟）
pub mod vision {
    pub const HIGH: f64 = 2.0;
    pub const LOW: f64 = 1.0;

    /// 根据位置获取视野阈值
    pub fn for_role(role: &str) -> (f64, f64) {
        match role {
            "辅助" => (2.5, 1.5),
            "打野" => (1.8, 1.0),
            "中单" => (1.5, 0.8),
            "上单" | "ADC" => (1.2, 0.6),
            _ => (HIGH, LOW),
        }
    }
}

/// 连胜/连败阈值
pub mod streak {
    pub const WIN_STREAK_GOOD: i32 = 5;
    pub const WIN_STREAK_EXCELLENT: i32 = 8;
    pub const LOSS_STREAK_BAD: i32 = -3;
    pub const LOSS_STREAK_TERRIBLE: i32 = -5;
}

/// 稳定性阈值（方差）
pub mod stability {
    pub const KDA_VARIANCE_STABLE: f64 = 1.0;
    pub const KDA_VARIANCE_UNSTABLE: f64 = 3.0;
}

/// 分布阈值
pub mod distribution {
    pub const S_GRADE_THRESHOLD: f64 = 0.30; // 30%以上S级表现
    pub const D_GRADE_THRESHOLD: f64 = 0.25; // 25%以上D级表现
}

/// CS阈值（每分钟）
pub mod cs {
    pub const EXCELLENT: f64 = 8.0;
    pub const GOOD: f64 = 6.0;
    pub const POOR: f64 = 4.0;
}

/// 伤害阈值（每分钟）
pub mod damage_per_minute {
    pub const EXCELLENT: f64 = 800.0;
    pub const GOOD: f64 = 600.0;
    pub const POOR: f64 = 400.0;
}

/// 时间线阈值（对线期）⭐ NEW
pub mod laning_phase {
    // CS差阈值
    pub const CS_DIFF_DOMINATE: f64 = 15.0; // 压制级（领先15+刀）
    pub const CS_DIFF_ADVANTAGE: f64 = 8.0; // 优势级（领先8-15刀）
    pub const CS_DIFF_NEUTRAL_HIGH: f64 = 5.0; // 均势上限
    pub const CS_DIFF_NEUTRAL_LOW: f64 = -5.0; // 均势下限
    pub const CS_DIFF_DISADVANTAGE: f64 = -8.0; // 劣势级（落后8-15刀）
    pub const CS_DIFF_SUPPRESSED: f64 = -15.0; // 被压制（落后15+刀）

    // 经验差阈值
    pub const XP_DIFF_ADVANTAGE: f64 = 300.0; // 经验优势（领先300+）
    pub const XP_DIFF_DISADVANTAGE: f64 = -300.0; // 经验劣势（落后300+）

    // 金币效率阈值（每分钟）
    pub const GOLD_PER_MIN_EXCELLENT: f64 = 450.0;
    pub const GOLD_PER_MIN_GOOD: f64 = 400.0;
    pub const GOLD_PER_MIN_POOR: f64 = 350.0;
}

/// 发育曲线阈值 ⭐ NEW
pub mod growth {
    // 成长率阈值
    pub const MID_GAME_BOOST: f64 = 1.15; // 中期经济提升15%（游走型）
    pub const MID_GAME_DECLINE: f64 = 0.85; // 中期经济下降15%（节奏问题）

    // 后期成长
    pub const LATE_GAME_BOOST: f64 = 1.10; // 后期继续提升

    // 稳定发育标准
    pub const STABLE_GOLD_EARLY: f64 = 400.0; // 对线期金币标准
    pub const STABLE_GOLD_MID: f64 = 380.0; // 中期金币标准
}
