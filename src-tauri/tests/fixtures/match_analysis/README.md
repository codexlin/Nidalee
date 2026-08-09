# 对局分析测试 fixtures

极小的**合成**对局数据，用于后续任务（fetcher / 解析 / Evidence）的确定性测试。

## 原则

- **不放真实数据**：不包含真实 PUUID、召唤师名、riotId、平台账号 ID 等任何可识别信息。
  PUUID 统一使用 `00000000-0000-4000-8000-0000000000xx` 形式的合成值。
- **只保留必需字段**：只包含 `analyzers/core/parser.rs` 解析所需的字段，
  不复制 LCU 返回的完整对局体（真实响应单局就有数百字段）。
- **一个文件一个场景**：文件名描述场景，便于测试按需组合。

## 文件

### 对局详情（`games.games[]` 元素）

| 文件 | 场景 | 说明 |
|------|------|------|
| `ranked_440_standard.json` | 标准灵活组排 | 30 分钟局，三阶段 timeline delta 齐全 |
| `ranked_440_short_game.json` | 短局 | 约 15 分钟，只有 `0-10` 阶段 delta |
| `ranked_440_missing_timeline.json` | 缺 timeline | `timeline` 只有 `role`/`lane`，没有任何 delta |
| `ranked_440_empty_deltas.json` | 空 delta | delta 对象存在但为空 `{}` |
| `ranked_440_ten_players.json` | 完整 10 人 | 上/野/中/ADC/辅助各两名，供对手识别与位置枚举测试 |

### 时间线（`/lol-match-history/v1/game-timelines/{id}` 响应）

配合 `ranked_440_ten_players.json` 使用，participantId 一一对应。

| 文件 | 场景 | 说明 |
|------|------|------|
| `timeline_440_ten_players_30min.json` | 标准 30 分钟 | 0 / 10 / 20 / 30 分钟四帧，三阶段齐全，含击杀、龙、先锋、大龙、虚空幼虫、建筑与 `killerId=0` 事件 |
| `timeline_440_short_15min.json` | 短局 | 只到 15 分钟，没有 late 阶段 |
| `timeline_440_remake_5min.json` | remake | 只到 5 分钟，只有 early 阶段 |
| `timeline_440_missing_frames.json` | 缺帧 | 后段帧里没有目标玩家的 `participantFrame` |

> **粒度提示**：上面几个时间线夹具是「阶段锚点级」的粗粒度数据（每 5~10 分钟一帧），
> 用来验证阶段划分与速率公式。**对手空间邻近**需要对线期有足够采样点，
> 粗粒度夹具剔除 t0 后只剩一帧，会（正确地）判定为证据不足。
> 这类场景请用 `tests/analysis_evidence.rs` 里的 `per_minute_timeline()`
> 程序化生成 0..=30 分钟、每分钟一帧的 31 帧时间线，
> 它可以按 `(participantId, minute)` 自定义坐标，用于构造 t0 泉水、
> 死亡回城坐标尖峰与离群支援帧。

## 数据结构约定

每个文件是**单个对局对象**（LCU `/lol-match-history/v1/products/lol/{puuid}/matches`
返回体中 `games.games[]` 的元素）。测试如需完整列表响应，请自行包裹：

```json
{ "games": { "games": [ <fixture> ] } }
```

`ranked_440_*` 系列每局固定 2 名参与者（本人 `teamId: 100`，对手 `teamId: 200`），
因此队伍聚合值等于本人数值 —— 这对策略/解析层测试足够；
需要真实队伍占比、对位匹配或 10 人位置枚举的测试请用 `ranked_440_ten_players.json`。

时间线文件是**单个时间线响应对象**，结构为 `{ "frames": [ { timestamp, events, participantFrames } ] }`，
`participantFrames` 的键是 `participantId` 的字符串形式。

## 注意

- 2 人 fixture 的目标玩家 PUUID 固定为 `00000000-0000-4000-8000-000000000001`。
- 10 人 fixture 里 `...0001` ~ `...0005` 依次是蓝方上/野/中/ADC/辅助，
  `...0006` ~ `...0010` 是红方对应位置，participantId 与末位数字一致。
- 若整体删除 participant 的 `timeline` 字段，`parser.rs` 会直接丢弃该局，
  因此「缺 timeline」场景保留空的 `timeline` 对象（只有 `role`/`lane`）。
