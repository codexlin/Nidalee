# 构建中心与符文系统

## 产品边界

Nidalee 只有一个构建入口：**构建中心**。

- **推荐方案**展示 OP.GG 等外部数据源的即时查询结果。查询结果不是用户资产，不直接持久化。
- **我的方案**保存用户拥有的构建快照，可创建、编辑、导入、导出、手动应用或参与自动匹配。
- **设置 > 符文设置**只管理自动应用开关和选择策略，不再承担方案编辑职责。

当前构建方案只包含符文。未来加入装备、召唤师技能或技能加点时，应扩展同一个 `BuildPreset.components`，不能再建立平行的配置 Store 或应用命令。

## 用户流程

### 使用推荐方案

1. 进入“构建中心 > 推荐方案”。
2. 选择英雄、位置和模式。
3. 可以直接应用当前推荐，也可以保存为“我的方案”。
4. 保存动作创建独立快照；后续推荐数据变化不会修改用户保存的内容。

### 管理我的方案

1. 进入“构建中心 > 我的方案”。
2. 新建方案，或编辑由推荐保存的快照。
3. 设置适用范围：英雄 + 位置、英雄通用、位置通用。
4. 手动选择符文，或从 OP.GG / 当前客户端载入完整符文后再保存。

### 自动应用

设置页提供三种策略：

- **智能模式**：优先匹配“我的方案”，没有匹配时才使用推荐方案。
- **仅推荐方案**：始终使用当前 OP.GG 推荐。
- **仅我的方案**：只应用匹配的已保存方案；没有匹配时不做任何修改。

自动应用只在本地玩家锁定英雄、分析数据确认同一个英雄且游戏阶段为 `ChampSelect` 时触发。离开选人阶段会清理本局应用标记，下一局可以重新应用。

## 数据模型

`BuildPreset` 是唯一持久化模型：

```text
BuildPreset
├─ id / name
├─ applicability
│  ├─ scope
│  ├─ championId / championName
│  └─ position
├─ components
│  └─ runes: RuneSelection
├─ source
└─ isDefault / createdAt / updatedAt / usageCount
```

`RuneSelection` 是前后端唯一符文输入：主系、副系和恰好 9 个不重复的正整数符文 ID。前端在保存和调用前校验，Rust 命令再次校验，客户端服务只接收已验证的数据。

方案匹配顺序固定为：

1. 英雄 + 位置；
2. 英雄通用；
3. 位置通用；
4. 同一级先选默认方案，再按更新时间和 ID 稳定排序。

## 单一数据流

```text
OP.GG Query ──┬── 直接应用 ───────────────┐
              └── 保存快照 → BuildPresetStore ─┤
手动编辑 / 客户端导入 ───→ BuildPresetStore ─┤
                                                ↓
                               useBuildApplication
                                                ↓
                                  apply_rune_selection
                                                ↓
                                    LCU 当前可编辑符文页
```

直接应用、我的方案手动应用和自动应用最终都调用 `useBuildApplication`，Rust 端只有 `apply_rune_selection` 一个写入口。禁止重新增加按数据源区分的应用命令。

## 所有权与失败语义

- `BuildPresetStore` 是保存方案和自动应用策略的唯一前端所有者。
- Store 变更先写入磁盘，成功后再提交到 Pinia 内存；写盘失败不会显示假成功状态。
- Store 写操作串行执行，避免快速连续编辑互相覆盖。
- 已成功写入英雄联盟客户端后，“使用次数”保存失败只记录警告，不得把应用结果改判为失败或再次应用另一套符文。
- 导入文件必须是当前版本结构；不提供旧结构兼容迁移。

## 关键代码

- 模型：`src/shared/models/buildPreset.ts`
- 持久化：`src/shared/stores/features/buildPresetStore.ts`
- 统一应用：`src/shared/composables/game/useBuildApplication.ts`
- 自动选择：`src/shared/composables/game/useAutoBuild.ts`
- 构建中心：`src/features/opgg/Opgg.vue`
- 我的方案：`src/features/opgg/components/presets/`
- Rust 命令：`src-tauri/src/infrastructure/champion_selection/perks/commands.rs`

## 扩展装备时的约束

加入出装构建时：

1. 在 `BuildPreset.components` 增加可选 `items`，保持方案 ID、适用条件、来源和生命周期不变。
2. 推荐数据先转换为统一快照，再决定直接使用或保存；页面不得持久化 Query 原始对象。
3. 应用层按已存在的组件执行，返回每一部分的明确结果；不要复制新的“OP.GG 应用”和“自定义应用”路径。
4. 自动应用策略仍只解析一次方案，随后由统一执行器应用符文、装备等组件。
