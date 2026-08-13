//! 从当前三张卡里标出 T 级最高、同档再比胜率最高的一张。

use super::types::OverlayAugment;

#[derive(Debug, Clone)]
pub struct OfferScore {
    pub slot: usize,
    pub id: Option<i32>,
    pub tier: Option<i32>,
    pub win_rate: Option<f64>,
    pub sampled: bool,
}

/// T1 最好，未知档最后。
pub fn tier_rank(tier: Option<i32>) -> i32 {
    match tier {
        Some(1) => 1,
        Some(2) => 2,
        Some(3) => 3,
        Some(4) => 4,
        _ => 9,
    }
}

pub fn pick_best_offer(offers: &[OfferScore]) -> Option<usize> {
    offers
        .iter()
        .filter(|offer| offer.id.is_some())
        .min_by(|left, right| {
            tier_rank(left.tier)
                .cmp(&tier_rank(right.tier))
                .then_with(|| match (left.sampled, right.sampled) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    (true, true) => right
                        .win_rate
                        .partial_cmp(&left.win_rate)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    (false, false) => std::cmp::Ordering::Equal,
                })
        })
        .map(|offer| offer.slot)
}

pub fn mark_recommended(augments: &mut [OverlayAugment], scores: &[OfferScore]) {
    let best = pick_best_offer(scores);
    for augment in augments.iter_mut() {
        augment.recommended = best == Some(augment.detected_slot as usize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer(slot: usize, id: i32, tier: Option<i32>, win_rate: Option<f64>, sampled: bool) -> OfferScore {
        OfferScore {
            slot,
            id: Some(id),
            tier,
            win_rate,
            sampled,
        }
    }

    #[test]
    fn prefers_t1_over_higher_win_rate_t2() {
        let offers = [
            offer(0, 1, Some(2), Some(0.70), true),
            offer(1, 2, Some(1), Some(0.51), true),
            offer(2, 3, Some(3), Some(0.80), true),
        ];
        assert_eq!(pick_best_offer(&offers), Some(1));
    }

    #[test]
    fn same_tier_prefers_higher_win_rate() {
        let offers = [
            offer(0, 1, Some(1), Some(0.51), true),
            offer(1, 2, Some(1), Some(0.62), true),
            offer(2, 3, Some(1), Some(0.55), true),
        ];
        assert_eq!(pick_best_offer(&offers), Some(1));
    }

    #[test]
    fn sampled_beats_missing_win_rate_in_same_tier() {
        let offers = [
            offer(0, 1, Some(1), None, false),
            offer(1, 2, Some(1), Some(0.54), true),
        ];
        assert_eq!(pick_best_offer(&offers), Some(1));
    }

    #[test]
    fn ignores_unmatched_slots() {
        let offers = [OfferScore {
            slot: 0,
            id: None,
            tier: Some(1),
            win_rate: Some(0.9),
            sampled: true,
        }];
        assert_eq!(pick_best_offer(&offers), None);
    }
}
