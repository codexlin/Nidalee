use super::super::builder::AdviceBuilder;
use super::super::types::{AdviceCategory, AdvicePerspective, GameAdvice};
/// 协作建议策略（对队友）⭐ 用户创新
///
/// 职责：
/// - 生成如何配合队友的建议
/// - 识别队友的强弱，提供协作方案
/// - 措辞：第三人称（"该队友"/"他"）
/// - 目标：团队协作，提高胜率
use super::base::{AdviceStrategy, ProblemData, ProblemType};

pub struct CollaborationStrategy;

impl AdviceStrategy for CollaborationStrategy {
    fn generate_advice(&self, problem_type: ProblemType, data: &ProblemData) -> Option<GameAdvice> {
        match problem_type {
            ProblemType::LaningCsDeficit => self.create_cs_deficit_advice(data),
            ProblemType::LaningDominated => self.create_dominated_advice(data),
            ProblemType::MidGameDecline => self.create_midgame_decline_advice(data),
            ProblemType::PoorFarming => self.create_poor_farming_advice(data),
            ProblemType::LowKillParticipation => self.create_low_kp_advice(data),
            ProblemType::LowTeamfightParticipation => self.create_low_teamfight_advice(data),
            ProblemType::HighDeathRate => self.create_high_death_advice(data),
            ProblemType::PoorPositioning => self.create_positioning_advice(data),
            ProblemType::LowVisionScore => self.create_vision_advice(data),
            ProblemType::ChampionPoolNarrow => None, // 英雄池不影响协作
            ProblemType::ChampionDependency => None,
        }
    }

    fn name(&self) -> &str {
        "团队协作策略"
    }

    fn perspective(&self) -> AdvicePerspective {
        AdvicePerspective::Collaboration
    }
}

impl CollaborationStrategy {
    /// 队友补刀弱 → 如何帮助
    fn create_cs_deficit_advice(&self, data: &ProblemData) -> Option<GameAdvice> {
        let teammate = data.target_name.as_deref().unwrap_or("该队友");

        AdviceBuilder::new()
            .title(format!("队友{}对线期需要帮助", data.role))
            .problem(format!(
                "{}对线期补刀能力弱，容易被压制（平均落后{:.1}刀）",
                teammate, -data.value
            ))
            .evidence("该队友对线期容易落后，需要支援".to_string())
            .suggestion(format!("🛡️ 打野：多去{}路反蹲，保护发育", data.role))
            .suggestion(format!("👁️ 辅助/中单：帮{}路做视野，避免被gank", data.role))
            .suggestion("⚠️ 不要过度依赖：该路可能无法carry，做好备案")
            .suggestion(format!("🤝 中单：6级后可以游走{}路帮忙缓解压力", data.role))
            .priority(3)
            .category(AdviceCategory::Laning)
            .perspective(self.perspective())
            .target_player(data.target_name.as_deref().unwrap_or("队友"))
            .affected_role(data.role.clone())
            .build()
    }

    /// 队友被压制 → 如何保护
    fn create_dominated_advice(&self, data: &ProblemData) -> Option<GameAdvice> {
        let teammate = data.target_name.as_deref().unwrap_or("该队友");

        AdviceBuilder::new()
            .title(format!("队友{}前期需要重点保护", data.role))
            .problem(format!("{}前期容易被击杀，抗压能力弱", teammate))
            .evidence("该队友对线期经常死亡，需要团队支援".to_string())
            .suggestion(format!("🛡️ 打野：前期常驻{}半区，多反蹲保护", data.role))
            .suggestion(format!("👁️ 全队：帮{}路做视野，减少被gank机会", data.role))
            .suggestion("⚠️ 降低期待：不要指望该路carry，求稳即可")
            .suggestion("🎯 资源分配：其他路发育好了，带动该路")
            .suggestion(format!("⏰ 及时支援：看到{}路被抓，立即TP/游走支援", data.role))
            .priority(4)
            .category(AdviceCategory::Laning)
            .perspective(self.perspective())
            .target_player(data.target_name.as_deref().unwrap_or("队友"))
            .affected_role(data.role.clone())
            .build()
    }

    /// 队友中期弱 → 如何弥补
    fn create_midgame_decline_advice(&self, data: &ProblemData) -> Option<GameAdvice> {
        let teammate = data.target_name.as_deref().unwrap_or("该队友");

        AdviceBuilder::new()
            .title(format!("队友{}中期发育效率低", data.role))
            .problem(format!(
                "{}中期经济效率下降{:.0}%，发育节奏有问题",
                teammate,
                data.severity * 100.0
            ))
            .evidence("该队友中期容易落后，需要团队弥补".to_string())
            .suggestion("💰 让资源：适当让野怪和兵线给该队友")
            .suggestion("🎯 打野：清完野怪后，把小野怪让给该队友")
            .suggestion("⏰ 避免无意义团战：不要在该队友装备未成型时强开团")
            .suggestion("🛡️ 保护发育：给该队友时间发育，等装备成型")
            .priority(2)
            .category(AdviceCategory::Farming)
            .perspective(self.perspective())
            .target_player(data.target_name.as_deref().unwrap_or("队友"))
            .build()
    }

    /// 队友补刀效率低 → 如何应对
    fn create_poor_farming_advice(&self, data: &ProblemData) -> Option<GameAdvice> {
        let teammate = data.target_name.as_deref().unwrap_or("该队友");

        AdviceBuilder::new()
            .title("队友发育能力较弱")
            .problem(format!("{}补刀效率偏低（{:.1}/分钟）", teammate, data.value))
            .evidence("该队友发育较慢，后期可能装备落后".to_string())
            .suggestion("💰 其他路多发育：该路可能无法carry，其他路要挑大梁")
            .suggestion("⏰ 加快节奏：前中期尽量结束游戏，不要拖后期")
            .suggestion("🎯 保护发育：不要让对面打野针对该路")
            .priority(2)
            .category(AdviceCategory::Farming)
            .perspective(self.perspective())
            .target_player(data.target_name.as_deref().unwrap_or("队友"))
            .build()
    }

    /// 队友团战参与度低 → 调整策略
    fn create_low_teamfight_advice(&self, data: &ProblemData) -> Option<GameAdvice> {
        let teammate = data.target_name.as_deref().unwrap_or("该队友");

        let adaptation = match data.role.as_str() {
            "打野" => "打野参团少可能在反野发育，其他人可以主动控龙，用信号提前通知",
            "中单" => "中单不游走说明想发育，可以让他守中推线，其他人主动找机会",
            "上单" => "上单参团慢很正常，打小规模团战就行，别指望他每次都能TP",
            "ADC" => "ADC参团少可能在发育装备，前期可以让他安心补刀，中期再抱团",
            "辅助" => "辅助不游走可能在保护ADC，可以让他们下路发育，其他路自己小心",
            _ => "队友参团意识弱，需要调整战术不依赖他",
        };

        AdviceBuilder::new()
            .title(format!("团队适应：{}参团较少", data.role))
            .problem(format!("{}的助攻仅{:.1}次/场，团战中经常缺席", teammate, data.value))
            .evidence(
                data.extra_info
                    .clone()
                    .unwrap_or_else(|| "该队友偏向发育/分带，不喜欢抱团".to_string()),
            )
            .suggestion(format!("💡 理解打法：{}", adaptation))
            .suggestion("⏰ 提前60秒信号：打龙/团战至少提前1分钟标记，给队友充足准备时间")
            .suggestion("📍 战场选择：尽量在该队友附近的区域开团，减少他的赶路时间")
            .suggestion("🚫 避免强开：看到该队友位置很远时，不要强行开团，容易4v5")
            .suggestion("🎯 利用分带：如果他在单带，可以利用他吸引敌人注意力，其他人偷龙或推塔")
            .suggestion("💬 友善沟通：语音或打字说「来龙坑集合」，别责怪他不来")
            .priority(3)
            .category(AdviceCategory::Teamfight)
            .perspective(self.perspective())
            .affected_role(data.role.clone())
            .target_player(data.target_name.as_deref().unwrap_or("队友"))
            .build()
    }

    /// 队友死亡过多 → 保护策略
    fn create_high_death_advice(&self, data: &ProblemData) -> Option<GameAdvice> {
        let teammate = data.target_name.as_deref().unwrap_or("该队友");

        let protection = match data.role.as_str() {
            "ADC" => vec![
                "🛡️ 全力保护：辅助和坦克要贴身保护，别让刺客切到他",
                "👁️ 视野深度：在该ADC活动区域提供大量视野，避免被抓",
                "⚡ 优先救援：看到他被开立即用技能救他，ADC死了团战必输",
                "💬 温柔沟通：提醒他「站后面」「别太激进」，但语气要友好",
            ],
            "中单" | "法师" => vec![
                "🛡️ 开团保护：先手开团时要考虑他的位置，别让他跟不上",
                "👁️ 河道视野：多在中路河道做视野，避免他被Gank",
                "💰 让资源发育：野怪、兵线适当让给他，让他快点出装备",
                "📢 危险信号：看到对面刺客missing立即发信号",
            ],
            "打野" => vec![
                "🌳 反野支援：他反野时要提供视野和支援，别让他单独行动",
                "🐉 控龙协助：打龙时帮他清视野，避免被偷袭",
                "💬 路线提醒：看到对方入侵立即发信号「小心」",
                "🎯 减少依赖：该打野容易死，前期别太依赖他带节奏",
            ],
            "上单" | "坦克" => vec![
                "🎭 控制开团：别让他无脑开团，用信号告诉他「等我们」",
                "⚡ 及时跟进：他开团时立即跟进，别让他一个人扛伤害",
                "📢 位置提醒：团战前告诉他「别太激进」「站中间就行」",
                "💬 鼓励为主：他死多了心态可能不好，多鼓励别责怪",
            ],
            "辅助" => vec![
                "👀 做视野协助：他插眼时跟着去，避免被抓",
                "🛡️ 双辅助思维：如果可以，出点肉装保护团队",
                "📢 危险提醒：他探草或深入时立即发信号",
                "💬 沟通配合：提醒他「别一个人走」「跟紧ADC」",
            ],
            _ => vec![
                "🛡️ 多加保护：该队友容易死，需要额外照顾",
                "👁️ 视野支持：在他活动区域多做视野",
                "⚡ 及时救援：看到他被抓立即支援",
                "💬 友善沟通：提醒但不要责怪",
            ],
        };

        let mut builder = AdviceBuilder::new()
            .title(format!("保护队友：{}生存能力弱", data.role))
            .problem(format!("{}场均死亡{:.1}次，是团队的薄弱环节", teammate, data.value))
            .evidence(
                data.extra_info
                    .clone()
                    .unwrap_or_else(|| "该队友频繁阵亡，需要团队保护".to_string()),
            );

        for suggestion in protection {
            builder = builder.suggestion(suggestion);
        }

        builder
            .suggestion("🌟 心态关键：别责怪他「又送了」，多鼓励「没事稳住」，心态崩了更容易送")
            .priority(4)
            .category(AdviceCategory::Teamfight)
            .perspective(self.perspective())
            .affected_role(data.role.clone())
            .target_player(data.target_name.as_deref().unwrap_or("队友"))
            .build()
    }

    /// 队友参团率低 → 如何应对
    fn create_low_kp_advice(&self, data: &ProblemData) -> Option<GameAdvice> {
        let teammate = data.target_name.as_deref().unwrap_or("该队友");

        AdviceBuilder::new()
            .title("队友参团意识较弱")
            .problem(format!("{}参团率仅{:.0}%，经常缺席团战", teammate, data.value * 100.0))
            .evidence("该队友可能更倾向于单带或刷野".to_string())
            .suggestion("⚠️ 调整战术：不要期待该队友每次都参团")
            .suggestion("🔔 多发信号：提前标记团战位置，给该队友足够时间赶来")
            .suggestion("🎯 利用分带：如果该队友单带，其他人牵制正面")
            .suggestion("⏰ 避免强开：确认该队友位置后再开团")
            .priority(3)
            .category(AdviceCategory::Teamfight)
            .perspective(self.perspective())
            .target_player(data.target_name.as_deref().unwrap_or("队友"))
            .build()
    }

    fn create_positioning_advice(&self, data: &ProblemData) -> Option<GameAdvice> {
        let teammate = data.target_name.as_deref().unwrap_or("该队友");

        AdviceBuilder::new()
            .title("队友团战容易暴毙")
            .problem(format!("{}团战站位激进，容易被先手击杀", teammate))
            .evidence("该队友团战存活能力弱".to_string())
            .suggestion("🛡️ 辅助：优先保护该队友，及时给盾/治疗")
            .suggestion("⏰ 不要指望该队友先手：让坦克先手，该队友跟输出")
            .suggestion("🎯 团战目标：优先击杀对方威胁，保护该队友输出空间")
            .suggestion("⚠️ 备案：如果该队友先死，立即撤退")
            .priority(3)
            .category(AdviceCategory::Positioning)
            .perspective(self.perspective())
            .target_player(data.target_name.as_deref().unwrap_or("队友"))
            .affected_role(data.role.clone())
            .build()
    }

    fn create_vision_advice(&self, data: &ProblemData) -> Option<GameAdvice> {
        let teammate = data.target_name.as_deref().unwrap_or("该队友");

        AdviceBuilder::new()
            .title("队友视野控制不足")
            .problem(format!(
                "{}视野得分低（{:.1}/分钟），需要团队弥补",
                teammate, data.value
            ))
            .evidence("该队友做视野意识弱，团队视野压力大".to_string())
            .suggestion("👁️ 其他人多买眼：弥补该队友的视野不足")
            .suggestion("🔍 打野/辅助：承担更多视野责任")
            .suggestion("📍 关键位置：优先保证龙坑、野区入口视野")
            .suggestion("⚠️ 提醒队友：团战前提醒该队友一起做视野")
            .priority(2)
            .category(AdviceCategory::Vision)
            .perspective(self.perspective())
            .target_player(data.target_name.as_deref().unwrap_or("队友"))
            .build()
    }
}
