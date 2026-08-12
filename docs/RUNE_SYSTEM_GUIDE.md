# 构建中心与符文系统

## 面向普通用户

普通用户只需要在“设置 > 符文设置”开启自动构建。锁定英雄后，系统会按固定规则工作：

1. 有匹配的个人自动方案：使用个人方案。
2. 没有个人自动方案：使用在线推荐。
3. 当前游戏模式不受支持：保持客户端现有符文，不做修改。

设置页只保留自动开关、在线推荐参考段位和结果通知。没有“智能、仅推荐、仅保存”三种策略。

当前自动支持：单双排、灵活排位、匹配峡谷和普通大乱斗。自定义、轮换模式、竞技场等不会自动修改符文。

## 面向高级用户

个人方案位于“构建中心 > 我的方案”。创建流程是：

1. 选择英雄。
2. 选择游戏场景；只有排位需要再选择位置。
3. 手动选择符文，或从 OP.GG / 当前客户端载入。
4. 决定是否自动使用并保存。

场景只有七种：排位五位置、匹配峡谷、普通大乱斗。同一英雄和场景可以保存多个手动备选，但最多启用一套自动方案。

“保存推荐”创建的是独立快照，默认不自动使用；需要时在“我的方案”中开启。导入文件同样默认是手动方案，避免悄悄替换现有自动方案。

## 数据模型

```text
BuildPreset
├─ id / name
├─ target
│  ├─ championId / championName
│  └─ scenario
├─ components
│  └─ runes: RuneSelection
├─ source
├─ autoUse
└─ createdAt / updatedAt / usageCount
```

`RuneSelection` 是前后端唯一的符文输入：主系、副系和恰好 9 个互不重复的正整数符文 ID。前端保存及调用前校验，Rust 命令再次校验。

持久化和导入导出格式版本为 v2，使用 `build-presets-v2.json`。旧实验结构不迁移、不兼容；旧文件可以直接删除。

## 应用与失败语义

- 直接应用、手动应用和自动应用最终都调用同一个 `useBuildApplication`。
- Rust 端只有一个 `apply_rune_selection` 写入口。
- Store 先成功写盘，再提交 Pinia 内存；写盘失败不得显示成功状态。
- 自动请求在网络返回后会复核当前英雄、场景和选人阶段；旧请求直接丢弃。
- 排位缺少位置时等待，不再静默当成中路。
- 匹配峡谷没有可靠分路，在线推荐按该英雄主流位置解析，但个人方案仍只属于 `normal-sr`。

## 关键代码

- 模型：`src/shared/models/buildPreset.ts`
- 场景解析：`src/shared/models/buildContext.ts`
- 推荐位置解析：`src/shared/models/opggRecommendation.ts`
- 持久化：`src/shared/stores/features/buildPresetStore.ts`
- 统一应用：`src/shared/composables/game/useBuildApplication.ts`
- 自动应用：`src/shared/composables/game/useAutoBuild.ts`
- 构建中心：`src/features/build-center/BuildCenter.vue`
- Rust 写入口：`src-tauri/src/infrastructure/champion_selection/perks/commands.rs`
