//! 统一位置枚举
//!
//! 后端事实值只使用 ASCII 位置码（`TOP` / `JUNGLE` / ...），中文展示文案由前端负责，
//! 避免同一个概念在证据层出现两套写法。

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domains::analysis::queue_config::QueueType;

/// 统一位置
///
/// 声明顺序即排序顺序，聚合输出依赖它保持确定性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/EvidencePosition.ts",
    rename_all = "UPPERCASE"
)]
#[serde(rename_all = "UPPERCASE")]
pub enum EvidencePosition {
    Top,
    Jungle,
    Mid,
    Adc,
    Support,
    /// 大乱斗：无分路
    Aram,
    /// 娱乐/匹配模式下的灵活位置（无稳定分路语义）
    Flex,
    Unknown,
}

impl EvidencePosition {
    /// 后端事实值（与 serde 序列化结果一致）
    pub fn as_str(&self) -> &'static str {
        match self {
            EvidencePosition::Top => "TOP",
            EvidencePosition::Jungle => "JUNGLE",
            EvidencePosition::Mid => "MID",
            EvidencePosition::Adc => "ADC",
            EvidencePosition::Support => "SUPPORT",
            EvidencePosition::Aram => "ARAM",
            EvidencePosition::Flex => "FLEX",
            EvidencePosition::Unknown => "UNKNOWN",
        }
    }

    /// 是否为召唤师峡谷的固定位置（只有这类位置才允许做同位置对手匹配）
    pub fn is_lane_position(&self) -> bool {
        matches!(
            self,
            EvidencePosition::Top
                | EvidencePosition::Jungle
                | EvidencePosition::Mid
                | EvidencePosition::Adc
                | EvidencePosition::Support
        )
    }

    /// 是否允许用「对线期空间邻近」回退推断对手
    ///
    /// 只有**真正常驻某条线**的位置才有对线邻近语义：
    /// - 打野满地图游走，最近的敌人往往是被他 gank 的人，不是对位
    /// - ARAM 只有一条路，全员互为「最近」
    /// - FLEX / UNKNOWN 连自己的位置都没确定，谈不上对位
    ///
    /// 这些情况宁可返回 `None`，也不要编出一个会污染全部对线结论的对手。
    pub fn allows_spatial_opponent_fallback(&self) -> bool {
        matches!(
            self,
            EvidencePosition::Top | EvidencePosition::Mid | EvidencePosition::Adc | EvidencePosition::Support
        )
    }
}

impl std::fmt::Display for EvidencePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 是否为「无分路」的大乱斗类队列
///
/// 语义唯一来源是 [`QueueType`]，证据层不维护自己的队列白名单：
/// 一旦两边各存一份，新增队列时必然会漏改其中一份。
pub fn is_aram_queue(queue_id: i64) -> bool {
    QueueType::from_queue_id(queue_id as i32).is_aram()
}

/// 由 LCU 的 `timeline.role` / `timeline.lane` 推导统一位置
///
/// **五分路只服务排位（420/440）**：娱乐/匹配的 `role`/`lane` 常是峡谷残留字段
/// （海克斯填成 `DUO_SUPPORT/BOTTOM` 等），一律不得拆成上/野/中/ADC/辅。
///
/// 判定顺序：
/// 1. 大乱斗（450）→ `ARAM`
/// 2. 非排位 → `FLEX`（不读 role/lane）
/// 3. 排位 → 才解析 role/lane；认不出 → `UNKNOWN`
///
/// 兼容 LCU 的多种写法：`SOLO/TOP`、`NONE/JUNGLE`、`DUO_CARRY|CARRY|BOTTOM`、
/// `DUO_SUPPORT|SUPPORT|UTILITY`、`MIDDLE|MID`、`BOTTOM|BOT`。
pub fn position_from_role_lane(role: &str, lane: &str, queue_id: i64) -> EvidencePosition {
    if is_aram_queue(queue_id) {
        return EvidencePosition::Aram;
    }

    if !QueueType::from_queue_id(queue_id as i32).is_ranked() {
        return EvidencePosition::Flex;
    }

    let role = role.trim().to_ascii_uppercase();
    let lane = lane.trim().to_ascii_uppercase();
    position_from_lane(&role, &lane).unwrap_or(EvidencePosition::Unknown)
}

/// 纯 role/lane 推导（不掺入队列语义）
fn position_from_lane(role: &str, lane: &str) -> Option<EvidencePosition> {
    if role == "JUNGLE" || lane == "JUNGLE" {
        return Some(EvidencePosition::Jungle);
    }

    match lane {
        "TOP" => return Some(EvidencePosition::Top),
        "MIDDLE" | "MID" => return Some(EvidencePosition::Mid),
        "BOTTOM" | "BOT" => {
            return Some(match role {
                "DUO_SUPPORT" | "SUPPORT" | "UTILITY" => EvidencePosition::Support,
                _ => EvidencePosition::Adc,
            })
        }
        _ => {}
    }

    // 部分数据源只填 role（teamPosition 风格）
    match role {
        "TOP" => Some(EvidencePosition::Top),
        "MIDDLE" | "MID" => Some(EvidencePosition::Mid),
        "BOTTOM" | "DUO_CARRY" | "CARRY" => Some(EvidencePosition::Adc),
        "UTILITY" | "DUO_SUPPORT" | "SUPPORT" => Some(EvidencePosition::Support),
        _ => None,
    }
}
