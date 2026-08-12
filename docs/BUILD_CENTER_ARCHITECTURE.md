# Build Center Architecture

构建中心同时承载在线推荐和用户方案，但两者的数据生命周期必须分开。

## 分层边界

| 层 | 职责 | 禁止事项 |
| --- | --- | --- |
| Query composables | 获取并缓存在线推荐 | 持久化用户方案、写入 LCU |
| 构建中心 UI | 浏览推荐、编辑个人方案 | 自行实现匹配规则或复制应用逻辑 |
| `BuildPresetStore` | 持久化方案与自动应用设置 | 请求在线推荐、调用 LCU |
| `useBuildApplication` | 校验并应用已经确定的符文 | 决定使用哪个方案 |
| `useAutoBuild` | 在锁定英雄后解析场景和唯一来源 | 实现数据源专用的 LCU 写入 |
| Rust `apply_rune_selection` | 再次校验 IPC 输入并更新当前可编辑符文页 | 请求 OP.GG、选择方案 |

## 状态归属

- 在线推荐属于服务端状态，保存在 Vue Query。
- 用户方案和自动应用设置属于用户状态，保存在 Pinia 与 Tauri Store。
- 选人会话属于运行时状态，保存在 `gameStore`，不复制到方案 Store。
- 单次应用进度属于操作状态，保存在 `useBuildApplication`。

## 方案目标

`BuildPreset.target` 使用精确的“英雄 + 场景”键：

- `ranked-top`
- `ranked-jungle`
- `ranked-mid`
- `ranked-adc`
- `ranked-support`
- `normal-sr`
- `aram`

同一英雄和场景最多只有一套 `autoUse` 方案。其他方案是手动备选，不参与自动匹配。这里不再使用“英雄通用、位置通用、越精确优先”等模糊打分规则。

## 自动应用合同

固定顺序只有一条：

1. 命中该英雄和场景已启用的个人方案时使用它。
2. 未命中时使用在线推荐。
3. 不支持的队列不修改客户端符文。

队列映射：

- 单双排 `420`、灵活排位 `440`：等待 LCU 提供有效位置，再映射到对应排位场景。
- 匹配峡谷 `400`、`430`、`490`：映射到 `normal-sr`；在线回退按英雄出场分布选择主流位置，不把该位置写入方案目标。
- 普通大乱斗 `450`：映射到 `aram`。
- 自定义和其他队列：跳过。

自动链路直接读取原始 Champ Select 会话中的 `queueId`、`isCustomGame`、本地玩家位置和锁定动作，不等待队伍深度分析。异步推荐返回后必须再次校验游戏阶段、英雄和场景，旧请求不得写入当前符文页。

## 单一应用路径

```text
在线推荐 ── 直接应用 ──────────────────┐
在线推荐 ── 保存快照 → BuildPresetStore ─┤
手动编辑 / 客户端导入 → BuildPresetStore ─┼→ useBuildApplication
自动场景解析 → 个人方案 / 在线推荐 ────────┘
                                      ↓
                            apply_rune_selection
                                      ↓
                              LCU 当前符文页
```

## 扩展装备时的约束

`BuildPreset.components` 是唯一扩展点。未来加入装备、召唤师技能或技能加点时：

1. 在同一方案里新增组件，不建立第二套 Store 或匹配器。
2. 在线数据先转换为统一快照，再决定直接应用或保存。
3. `useAutoBuild` 仍只解析一次目标方案。
4. 统一执行器按组件依次应用并返回明确结果。
