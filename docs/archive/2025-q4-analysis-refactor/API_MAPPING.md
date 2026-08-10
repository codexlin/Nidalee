# LCU API 字段映射验证

## 📋 **数据来源验证**

本文档验证 `parser.rs` 中的字段映射是否与 `api.md` (LCU API 文档) 完全一致。

---

## ✅ **游戏基础信息映射**

| API 字段 (api.md) | ParsedGame 字段 | 代码位置 | 状态 |
|------------------|----------------|---------|------|
| `gameId` (line 15) | `game_id: u64` | parser.rs:80 | ✅ 正确 |
| `queueId` (line 20) | `queue_id: i64` | parser.rs:81 | ✅ 正确 |
| `gameDuration` (line 19) | `game_duration: i32` | parser.rs:82 | ✅ 正确 |
| `gameCreation` (line 17) | `game_creation: i64` | parser.rs:83 | ✅ 正确 |
| `gameMode` (line 24) | ❌ 未包含 | - | ⚠️ 可选 |

**说明**:
- `gameMode` 未包含在 `ParsedGame` 中，因为当前分析不需要
- 在 `MatchPerformance` 中有 `game_mode` 字段，设为 `None` 是合理的
- 如需要，可以在 `ParsedGame` 中添加此字段

---

## ✅ **参与者基础信息映射**

| API 字段 (api.md) | ParsedGame 字段 | 代码位置 | 状态 |
|------------------|----------------|---------|------|
| `participants[].participantId` (line 56) | 用于查找 | parser.rs:69 | ✅ 正确 |
| `participants[].teamId` (line 57) | 用于队伍数据 | parser.rs:75 | ✅ 正确 |
| `participants[].championId` (line 58) | `champion_id: i32` | parser.rs:130 | ✅ 正确 |

---

## ✅ **参与者统计数据映射 (stats)**

### **核心 KDA 数据**

| API 字段 (api.md) | ParsedPlayerData 字段 | 代码位置 | 状态 |
|------------------|---------------------|---------|------|
| `stats.win` (line 64) | `win: bool` | parser.rs:114 | ✅ 正确 |
| `stats.kills` (line 72) | `kills: i32` | parser.rs:93 | ✅ 正确 |
| `stats.deaths` (line 73) | `deaths: i32` | parser.rs:94 | ✅ 正确 |
| `stats.assists` (line 74) | `assists: i32` | parser.rs:95 | ✅ 正确 |

**KDA 计算**:
```rust
// parser.rs:97-101
kda = if deaths > 0 {
    (kills + assists) as f64 / deaths as f64
} else {
    (kills + assists) as f64
}
```
✅ 公式正确

---

### **伤害数据**

| API 字段 (api.md) | ParsedPlayerData 字段 | 代码位置 | 状态 |
|------------------|---------------------|---------|------|
| `stats.totalDamageDealtToChampions` (line 89) | `damage_to_champions: i64` | parser.rs:119 | ✅ 正确 |
| `stats.totalDamageTaken` (line 95) | `damage_taken: i64` | parser.rs:120 | ✅ 正确 |
| `stats.damageDealtToObjectives` (line 129) | `damage_to_objectives: i64` | parser.rs:127 | ✅ 正确 |
| `stats.damageDealtToTurrets` (line 130) | `damage_to_turrets: i64` | parser.rs:128 | ✅ 正确 |

---

### **经济数据**

| API 字段 (api.md) | ParsedPlayerData 字段 | 代码位置 | 状态 |
|------------------|---------------------|---------|------|
| `stats.goldEarned` (line 99) | `gold_earned: i64` | parser.rs:121 | ✅ 正确 |

---

### **视野数据**

| API 字段 (api.md) | ParsedPlayerData 字段 | 代码位置 | 状态 |
|------------------|---------------------|---------|------|
| `stats.visionScore` (line 131) | `vision_score: i32` | parser.rs:122 | ✅ 正确 |
| `stats.wardsPlaced` (line 111) | `wards_placed: i32` | parser.rs:123 | ✅ 正确 |
| `stats.wardsKilled` (line 112) | `wards_killed: i32` | parser.rs:124 | ✅ 正确 |

---

### **补刀数据**

| API 字段 (api.md) | ParsedPlayerData 字段 | 代码位置 | 状态 |
|------------------|---------------------|---------|------|
| `stats.totalMinionsKilled` (line 103) | `cs` (part) | parser.rs:125 | ✅ 正确 |
| `stats.neutralMinionsKilled` (line 104) | `cs` (part) | parser.rs:126 | ✅ 正确 |

**CS 计算**:
```rust
// parser.rs:125-126
cs: (stats["totalMinionsKilled"].as_i64().unwrap_or(0)
    + stats["neutralMinionsKilled"].as_i64().unwrap_or(0)) as i32
```
✅ 公式正确：总补刀 = 兵线补刀 + 野怪补刀

---

### **控制数据**

| API 字段 (api.md) | ParsedPlayerData 字段 | 代码位置 | 状态 |
|------------------|---------------------|---------|------|
| `stats.timeCCingOthers` (line 132) | `time_cc_others: i32` | parser.rs:129 | ✅ 正确 |

---

## ✅ **位置信息映射 (timeline)**

| API 字段 (api.md) | ParsedPlayerData 字段 | 代码位置 | 状态 |
|------------------|---------------------|---------|------|
| `timeline.role` (line 180) | `role: String` | parser.rs:106 | ✅ 正确 |
| `timeline.lane` (line 181) | `lane: String` | parser.rs:107 | ✅ 正确 |

**位置识别逻辑**:
```rust
// parser.rs:176-183
match (role, lane) {
    ("DUO_CARRY", _) => "ADC",
    ("DUO_SUPPORT", _) => "辅助",
    ("SOLO", "TOP") => "上单",
    ("SOLO", "MIDDLE") | ("SOLO", "MID") => "中单",
    ("NONE", "JUNGLE") | ("JUNGLE", _) => "打野",
    _ => "未知",
}
```
✅ 逻辑正确，涵盖所有位置

---

## ✅ **玩家身份信息映射 (participantIdentities)**

| API 字段 (api.md) | 使用位置 | 代码位置 | 状态 |
|------------------|---------|---------|------|
| `participantIdentities[].participantId` (line 222) | 查找玩家 | parser.rs:58 | ✅ 正确 |
| `participantIdentities[].player.puuid` (line 224) | 匹配当前玩家 | parser.rs:61 | ✅ 正确 |

**查找逻辑**:
```rust
// parser.rs:58-63
let participant_id = game["participantIdentities"]
    .as_array()?
    .iter()
    .find(|id| id["player"]["puuid"].as_str() == Some(puuid))?
    ["participantId"]
    .as_i64()?;
```
✅ 逻辑正确：通过 puuid 找到 participant_id，再用 participant_id 找到玩家数据

---

## ✅ **队伍数据聚合**

| 聚合数据 | ParsedTeamData 字段 | 计算逻辑 | 状态 |
|---------|-------------------|---------|------|
| 队伍总击杀 | `team_total_kills` | 累加同队所有 `kills` | ✅ 正确 |
| 队伍总伤害 | `team_total_damage` | 累加同队所有 `totalDamageDealtToChampions` | ✅ 正确 |
| 队伍总承伤 | `team_total_damage_taken` | 累加同队所有 `totalDamageTaken` | ✅ 正确 |
| 队伍总视野 | `team_total_vision_score` | 累加同队所有 `visionScore` | ✅ 正确 |

**队伍数据计算**:
```rust
// parser.rs:145-152
for p in participants {
    if p["teamId"].as_i64() == Some(team_id) {
        let stats = &p["stats"];
        team_total_kills += stats["kills"].as_i64().unwrap_or(0) as i32;
        team_total_damage += stats["totalDamageDealtToChampions"].as_i64().unwrap_or(0);
        team_total_damage_taken += stats["totalDamageTaken"].as_i64().unwrap_or(0);
        team_total_vision_score += stats["visionScore"].as_i64().unwrap_or(0) as i32;
    }
}
```
✅ 逻辑正确：通过 `teamId` 过滤同队玩家，累加数据

---

## 📊 **数据类型验证**

### **整数类型**

| 字段 | API 类型 (api.md) | Parser 类型 | 正确性 |
|------|------------------|------------|--------|
| `gameId` | number | `u64` | ✅ 正确 (游戏ID无符号) |
| `queueId` | number | `i64` | ✅ 正确 |
| `gameDuration` | number | `i32` | ✅ 正确 (秒数) |
| `gameCreation` | number | `i64` | ✅ 正确 (时间戳) |
| `kills` | number | `i32` | ✅ 正确 |
| `deaths` | number | `i32` | ✅ 正确 |
| `assists` | number | `i32` | ✅ 正确 |
| `totalDamageDealtToChampions` | number | `i64` | ✅ 正确 (大数值) |
| `goldEarned` | number | `i64` | ✅ 正确 (大数值) |
| `visionScore` | number | `i32` | ✅ 正确 |
| `championId` | number | `i32` | ✅ 正确 |

### **布尔类型**

| 字段 | API 类型 (api.md) | Parser 类型 | 正确性 |
|------|------------------|------------|--------|
| `win` | boolean | `bool` | ✅ 正确 |

### **字符串类型**

| 字段 | API 类型 (api.md) | Parser 类型 | 正确性 |
|------|------------------|------------|--------|
| `role` | string | `String` | ✅ 正确 |
| `lane` | string | `String` | ✅ 正确 |
| `puuid` | string | 查询参数 | ✅ 正确 |

### **浮点类型**

| 字段 | 计算公式 | Parser 类型 | 正确性 |
|------|---------|------------|--------|
| `kda` | (K+A)/D 或 K+A | `f64` | ✅ 正确 |

---

## 🔍 **容错处理验证**

所有字段都使用了 `.unwrap_or()` 提供默认值，确保 API 字段缺失时不会崩溃：

```rust
// 示例：parser.rs
stats["kills"].as_i64().unwrap_or(0) as i32
stats["win"].as_bool().unwrap_or(false)
timeline["role"].as_str().unwrap_or("NONE")
```

✅ 容错处理完善

---

## 🎯 **数据流验证**

```
LCU API JSON (api.md 定义)
    ↓
parse_games(games, puuid)  [parser.rs:164]
    ↓
    for each game:
        1. find participant_id by puuid  [parser.rs:58-63]
        2. find participant by participant_id  [parser.rs:66-69]
        3. parse_player_data(participant)  [parser.rs:89-134]
           ✓ 提取 stats.*
           ✓ 提取 timeline.role/lane
           ✓ 计算 KDA
           ✓ 计算 CS
        4. parse_team_data(game, team_id)  [parser.rs:136-161]
           ✓ 遍历同队玩家
           ✓ 累加队伍数据
        5. 组装 ParsedGame  [parser.rs:79-86]
    ↓
Vec<ParsedGame>
```

✅ 数据流完整且正确

---

## ⚠️ **可选优化建议**

### **1. 添加 gameMode 字段**

```rust
// ParsedGame 可以添加：
pub struct ParsedGame {
    // ...
    pub game_mode: String,  // "CLASSIC", "ARAM", etc.
}

// 在 parse_game 中：
game_mode: game["gameMode"].as_str().unwrap_or("UNKNOWN").to_string(),
```

**影响**: 可以在分析时区分游戏模式（虽然现在通过 queueId 也能判断）

**优先级**: 🔵 低（可选）

---

### **2. 添加游戏版本字段**

```rust
pub struct ParsedGame {
    // ...
    pub game_version: String,  // "14.20.586.1234"
}
```

**影响**: 可以分析不同版本的数据

**优先级**: 🔵 低（可选）

---

## ✅ **最终验证结果**

| 验证项 | 状态 | 说明 |
|--------|------|------|
| **字段映射完整性** | ✅ 通过 | 所有需要的字段都已正确映射 |
| **数据类型正确性** | ✅ 通过 | 类型选择合理（i32/i64/f64/bool/String） |
| **计算公式正确性** | ✅ 通过 | KDA、CS 计算公式正确 |
| **容错处理** | ✅ 通过 | 所有字段都有默认值处理 |
| **查找逻辑** | ✅ 通过 | puuid → participant_id → participant 正确 |
| **队伍数据聚合** | ✅ 通过 | 同队玩家数据累加正确 |
| **位置识别** | ✅ 通过 | role + lane → 位置名称映射正确 |

---

## 🎉 **总结**

### **✅ 优点**

1. **字段映射准确**：所有关键字段都正确映射到 API 文档
2. **类型选择合理**：i32/i64 区分，避免溢出
3. **容错完善**：所有字段都有 `unwrap_or` 默认值
4. **逻辑清晰**：查找、解析、聚合逻辑清晰
5. **易于维护**：所有 API 交互集中在 `parser.rs`

### **📝 建议**

- `gameMode` 字段是可选的，当前不影响功能
- 如果未来需要按游戏模式（CLASSIC/ARAM）做更细致的分析，可以添加此字段

### **🎯 结论**

**parser.rs 的数据来源和数据结构与 api.md 完全一致，没有问题！** ✅

---

*验证日期: 2025-10-17*
*验证文件: parser.rs vs api.md*
*验证结果: ✅ 通过*

