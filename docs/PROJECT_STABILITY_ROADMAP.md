# Nidalee 稳定性与代码治理路线图

> 最后更新：2026-08-11
>
> 开发基线：`release-v3-dev`
>
> 当前开发分支：`feature/build-workbench`

这份文档用于记录当前稳定性改造之后仍需处理的事项。它不是“看到大文件就拆”的重构清单，而是按用户可感知风险、数据正确性、依赖关系和实施成本排序的执行路线图。

## 一、实施原则

1. **行为正确优先于代码美化**：先修旧响应覆盖、重复请求和生命周期问题，再移动文件或改模块名。
2. **一个状态只有一个所有者**：后端负责 LCU 数据获取与业务计算；Pinia Store 负责前端会话状态；组件只负责展示和用户交互。
3. **事件和请求必须有生命周期**：监听器、定时器、异步请求都必须能取消，并在写入状态前确认自己仍属于当前页面、动作或连接。
4. **局部机制足够时不引入全局框架**：优先使用局部请求序号、重连清理和 Store 单例，不预先引入复杂的全局 `accountSessionGeneration`。
5. **结构重构不改变行为**：拆文件的提交只移动职责并保持公共接口，不能同时加入新业务逻辑。
6. **生成文件不能手工维护**：Rust → TypeScript 类型、自动导入声明和组件声明必须由脚本生成，并由 CI 检查漂移。
7. **每批可独立验证、独立回滚**：一个提交只解决一个主题，不执行 `git add src` 之类的大范围暂存。

## 二、当前状态快照

### 已完成

| 批次 | 提交 | 结果 |
| --- | --- | --- |
| LCU 实时链路稳定化 | `cd676f2` | 统一认证恢复、WS supervisor 生命周期、快照恢复、断线清理和旧任务取消 |
| 重连数据流协调 | `39d97e4` | 前端监听就绪后启动 WS；重连时清理旧账号/战绩数据并重新拉取 |
| 前端游戏生命周期 | `eed48c2` | `gameStore` 不再持久化运行态；自动符文改为应用级单例；去掉重复监听与旧配置源 |
| 自动操作生命周期 | `e9b0101` | 自动接受、自动选人和自动禁用统一取消与执行前校验，用户手动操作优先 |
| 对局分析单一数据流 | `0edf48c` | 应用事件层唯一恢复后端缓存，页面只读取 Store，实时事件不会被旧缓存覆盖 |
| 前端旧响应保护 | `deb98d8` | 搜索、召唤师详情和对局详情只允许最新请求更新界面，并清理历史拼写 |
| 静态数据查询收敛 | `bd253b7` | 英雄列表与详情统一到版本化数据入口，移除重复请求和陈旧自动导入声明 |
| Rust → TypeScript 契约门禁 | `bf1e528` | 清空并稳定生成 111 份后端类型，CI 会阻止 `global.d.ts` 契约漂移 |
| Rust 格式基线 | `84c2a48` | 使用仓库 `.rustfmt.toml` 统一现有 Rust 排版，不混入行为修改 |
| Rust 格式门禁 | `9bbfc75` | CI 与发布预检固定安装 rustfmt，并执行 `cargo fmt --all -- --check` |
| Dashboard 与样式批次 | 多个独立提交 | HarmonyOS 字体、Dashboard 设计、海报导出、过程复盘 UI 等已分批落地 |
| Rust 静态目录权威化 | `a5d6d8b` 等 | 英雄/技能由 `static_catalog` 按版本落盘；离线回退、Windows 覆盖、singleflight、前端禁止 `unknown` 版本兜底 |
| 特殊模式英雄身份 | `7446470` | Jade 等变体身份解析与元数据支持 |
| 匿名玩家 vs bot | `40890b4` | 选人隐名与自定义局机器人分类边界 |

### 当前执行位置

阶段 0 至阶段 3 已完成；**静态目录所有权重构已闭环**（含并发刷新回归测试、旧 `init_*` 入口移除）。阶段 6 的契约/格式门禁已完成。

下一批建议：从 `GameDetailDialog.vue` 开始阶段 4 前端职责拆分。阶段 5 的 Rust 大模块拆分仍暂缓；阶段 8 发布平台决策在非公开发布阶段保持 P2。

本轮静态目录已知可接受遗留：Windows `replace_file` 为备份—替换—回滚，非严格系统级 crash-atomic（缓存可重拉）。

## 三、推荐实施顺序

## 阶段 0：收口当前自动操作批次

**状态：已完成（`e9b0101`）**

### 目标

确认所有延迟自动操作都不会在状态已经变化后继续执行，也不会因为路由切换或监听重建而重复执行。

### 手工验证

- ReadyCheck 出现后能按设置自动接受。
- 自动接受倒计时期间离开 ReadyCheck，不再调用接受接口。
- 进入 Champ Select 后，自动 Hover/选择/禁用按设置执行。
- 用户已经手动 Hover 或选择英雄时，自动任务不覆盖用户操作。
- 退出 Champ Select、断开客户端或退出应用后，不再残留定时任务。
- 多次切换页面不会增加自动操作次数。

### 提交边界

只提交当前 7 个自动操作相关文件。建议提交信息：

```text
fix-cancel-stale-auto-actions
```

---

## 阶段 1：个人分析数据改为单一入口

**状态：已完成（`0edf48c`）**

### 已确认问题

当前存在两处 `get_cached_analysis_data` 调用：

- `src/shared/composables/app/useAppEvents.ts` 在应用监听初始化时恢复缓存。
- `src/features/match-analysis/MatchAnalysis.vue` 在页面挂载时再次恢复缓存。

这会让路由组件和应用事件层同时拥有数据恢复职责，容易产生重复请求、重复写入和后续维护分叉。

### 推荐结构

```text
Rust 事件 / 后端缓存
        ↓
useAppEvents（唯一接收与恢复入口）
        ↓
matchAnalysisStore（唯一前端状态源）
        ↓
MatchAnalysis.vue（只展示和触发明确的用户操作）
```

### 实施内容

- 移除 `MatchAnalysis.vue` 的路由挂载恢复请求。
- 保留 `useAppEvents` 的缓存恢复和实时事件处理。
- 页面仅从 `matchAnalysisStore` 读取数据。
- 明确断线、游戏结束和新 Champ Select 时各自清理哪些字段。

### 验收标准

- 应用启动、路由来回切换和 LCU 重连都不会重复恢复分析数据。
- 进入分析页不会额外发起后端缓存请求。
- 后端推送新分析结果后，页面无需重挂载即可更新。

建议提交信息：

```text
refactor-centralize-match-analysis-state
```

---

## 阶段 2：修复前端旧响应覆盖

**状态：已完成（`deb98d8`）**

### 已确认位置

1. `src/shared/composables/game/useSearchMatches.ts`
   - 连续搜索两个召唤师时，较早发出的慢请求可能最后返回并覆盖新搜索。
   - 战绩请求也存在同类问题。
   - `currentRestult`、`cunrrentIndex` 等历史拼写应渐进迁移。
2. `src/features/dashboard/components/GameDetailDialog.vue`
   - 快速切换对局时，旧详情请求可能晚返回并覆盖当前选择。

### 推荐实现

- 每个独立请求所有者维护一个递增的 `requestId`。
- 发出请求时记录本次 ID；返回时只有 ID 仍是最新值才允许写 Store 或组件状态。
- 新请求开始时取消或废弃旧请求；组件卸载时使当前 ID 失效。
- `loading` 和 `error` 同样只能由最新请求更新。
- 对于一次性的用户操作请求，不为了形式强行迁移到 Vue Query；局部序号足够解决竞态。

### 明确不做

暂不引入全局 `accountSessionGeneration`。现有重连清理已经覆盖账号断开/重连，先用局部请求序号解决已经确认的竞态。只有以后真实复现“跨账号旧响应仍能覆盖”的场景，再评估全局 generation。

### 验收标准

- 快速搜索 A、B，最终界面只能显示 B。
- 快速打开对局 A、B，最终详情只能显示 B。
- 失败的旧请求不能清空或覆盖新请求的成功结果。
- 修正拼写时保留短期兼容别名，并在消费者迁移完成后删除。

建议提交信息：

```text
fix-guard-stale-frontend-requests
```

---

## 阶段 3：统一静态游戏数据与图片查询

**状态：已完成（FE 查询收敛 `bd253b7`；Rust 目录权威化 `a5d6d8b` 等）**

### 已确认问题

`useChampionQuery.ts` 与 `useVersionedData.ts` 都在管理以下查询：

- `['gameVersion']`
- `['static', 'champions', version]`

同一类数据存在两个入口后，缓存参数、错误处理和刷新策略会逐渐不一致。Vue Query 只能复用相同 query key 的数据请求，它不能阻止图片节点销毁后的浏览器重新解码；Dashboard 的 `<KeepAlive>` 已解决主要的路由销毁问题。

### 实施内容

- 以 `useVersionedData.ts` 作为版本化静态数据的唯一入口。
- 迁移 `useChampionQuery.ts` 的消费者；确认无独特职责后删除该重复封装。
- 建立统一 query key 工厂，避免各文件手写数组。
- 统一 `staleTime`、`gcTime`、重试和版本变更失效策略。
- 区分三类缓存：后端数据缓存、Vue Query 内存缓存、浏览器 HTTP 图片缓存。
- **后续补强（已完成）**：英雄摘要与召唤师技能改由 Rust `static_catalog` 权威加载并按版本落盘；前端仅 IPC hydrate；Connected 时 `refresh_static_catalogs`（true singleflight）；删除旧 `init_champion_data` / `init_summoner_spell_data`。

### 验收标准

- 英雄、符文、技能和图标不存在重复 query 定义。
- 路由切换不重复请求版本化数据。
- 版本变化时能统一失效，而不是逐个页面刷新。
- 身份解析（分析 / 对局详情补名）只认 Rust 目录；离线可回退磁盘完整包。

建议提交信息：

```text
refactor-consolidate-static-data-queries
refactor(static-catalog): centralize versioned game metadata
```

---

## 阶段 4：前端职责拆分与代码规范收敛

**优先级：P1 / P2，按功能逐批执行**

**状态：部分进行** — `src/lib/index.ts` 已大幅瘦身；下一批优先 `GameDetailDialog.vue`（请求生命周期 / 队伍 / 参赛者展示）。

### 推荐职责边界

| 层 | 应负责 | 不应负责 |
| --- | --- | --- |
| Vue 组件 | 展示、输入、触发用户动作 | 维护跨页面会话、直接编排多条后端流程 |
| Composable | 一个用例或一种副作用的生命周期 | 充当无边界工具箱、创建多份全局 watcher |
| Pinia Store | 前端唯一状态源和同步状态转换 | 隐藏网络请求、持久化实时游戏状态 |
| 前端 API 层 | 参数转换、调用 Tauri command、统一错误映射 | 复制业务规则或保存页面状态 |
| Rust Application/Service | 用例编排、LCU 数据聚合 | UI 展示规则 |
| Rust Domain | 可测试的业务规则和计算 | Tauri、网络和进程扫描 |
| Rust Infrastructure | LCU、WS、文件、系统接口 | 业务评分和页面状态 |

### 优先检查的大文件

以下文件达到“需要审查职责”的规模，但不能仅按行数机械拆分：

- `src/lib/dataApi.ts`：按静态数据、图标 URL、版本和外部数据源拆分。
- `src/lib/index.ts`：只保留稳定的公共导出；实现逻辑移动到对应领域模块。
- `GameDetailDialog.vue`：把请求生命周期、队伍区块和参赛者展示拆开。
- `SummonerCard.vue`：拆为身份、排位、升级进度和会话统计区块。
- `GameStats.vue`：分离筛选状态、列表数据和空状态展示。
- `RunePerkPicker.vue`：分离符文选择规则与视觉列表。
- `DataCollectionTestView.vue`：确认只在开发环境可达，避免测试工具进入正式用户路径。

### 拆分规则

- 超过约 400 行只是“审查触发条件”，不是必须拆分的硬限制。
- 先找变化原因：如果两个区块总是一起改，拆开不会提高维护性。
- Composable 名称应体现用例，例如 `useChampSelectAutomation`，不要继续扩大 `useGame` 之类的泛化模块。
- `index.ts` 应以 re-export 为主，不能继续成为杂项实现文件。
- 禁止组件直接创建应用级监听器；应用级监听由 `useAppEvents` 或明确的单例 Store 持有。

建议按功能分别提交，不做一次性“大前端重构”。

---

## 阶段 5：Rust 大模块结构治理

**优先级：P2；实时链路真实验证稳定后再做**

### 候选模块

1. `websocket/event_handler.rs`
   - 拆分为事件缓存、状态 reducer、Champ Select 富化、队伍分析和前端 emit。
   - 保持一个公开入口，不能让 transport 直接依赖多个业务细节。
2. `shared/types/types/mod.rs`
   - 按账号、对局、分析、Champ Select、设置等领域拆成子模块。
   - 通过 `pub use` 保持外部接口稳定，避免一次性修改所有调用方。
3. `matches/service.rs`
   - 分离原始 LCU 拉取、数据规范化、详情组装和分析证据构建。
4. `common/commands/data_collection.rs`
   - 明确其开发测试性质，移到 dev tooling 或通过开发特性/构建条件隔离。

### 实施约束

- 先为当前公共行为补回归测试，再移动代码。
- 一次只拆一个模块。
- 结构提交不调整算法、超时或重试策略。
- 每次拆分前后运行同一组 Rust 测试，确保只是结构变化。

---

## 阶段 6：质量门禁与测试基线

**优先级：P1**

**状态：部分完成（契约门禁：`bf1e528`；格式基线：`84c2a48`；格式门禁：`9bbfc75`）**

### Rust

- [x] 单独创建一次 `cargo fmt` 基线提交，避免格式变化污染业务提交。
- [x] 基线清理完成后在 CI 和发布预检增加 `cargo fmt --check`。
- 分阶段处理 Clippy 告警，再增加 `cargo clippy --all-targets --locked -- -D warnings`；不要在现有大量告警未清理时直接阻塞所有开发。
- 核心实时链路继续保留 reducer、认证恢复、重连和旧任务取消测试。

### 前端

- 为纯逻辑调度器、请求序号和筛选函数补最小单元测试。
- 再决定是否引入 Vitest；不为了覆盖率数字一次性给所有 Vue 组件补浅层测试。
- 统一日志策略：生产逻辑使用项目 logger，清理长期存在的 `console.log` lint 警告。
- 对“失败后仍记录成功”这一类错误吞噬做一次全局搜索和修正。

### Rust → TypeScript 契约

- [x] 提供稳定命令完成“清空旧生成物 → 生成类型 → 同步目标文件 → 检查 git diff”。
- [x] CI 和发布预检执行相同流程，生成结果变化时直接失败。
- [x] `src/types/global.d.ts` 由 Rust `ts-rs` 测试和同步脚本生成，不再保留手写补丁。
- [ ] 在项目文档治理阶段补充 `types/auto-imports.d.ts` 和组件声明的生成来源。

建议拆为多个提交：

```text
style-establish-rustfmt-baseline
chore-enforce-rust-quality-gates
test-cover-frontend-request-lifecycle
chore-check-generated-type-contracts
```

---

## 阶段 7：项目文档与遗留说明清理

**优先级：P1**

### 已确认过时内容

`CLAUDE.md` 与当前项目存在多处偏差，包括：

- Node 版本仍写 `>=20`，实际工程已使用 Node 22 基线。
- 仍描述已不存在的 ESLint/Prettier/旧 lint 脚本。
- Tauri 版本和测试数量是旧值。
- 一边写 Windows only，一边存在 macOS 构建流程和 DMG 配置。
- 动态数据仍描述为 5–15 秒轮询，没有准确反映当前 WS + 健康快照机制。
- Rust → TypeScript 类型生成流程需要与实际脚本和 CI 对齐。

### 实施内容

- 以实际 `package.json`、`Cargo.toml`、workflow 和源码更新 `CLAUDE.md`。
- `docs/ARCHITECTURE.md` 只保留当前架构，历史设计移动到归档目录。
- 合并 `domains/analysis/docs` 下大量阶段性完成报告，保留一个当前 README 和必要的算法说明。
- 检查 `CHART_IMPLEMENTATION_PLAN.md` 的编码和失效示例；不要直接提交已损坏内容。
- 修复 Tauri/Cargo 配置中疑似乱码的应用描述、版权和窗口标题，并以 UTF-8 复核。
- 文档中的命令必须在 CI 或本地真实执行一次。

建议提交信息：

```text
docs-align-project-guidance-with-current-architecture
```

---

## 阶段 8：明确发布平台与签名策略

**优先级：公开发布前为 P0；日常开发阶段为 P2**

目前项目同时出现“Windows only”说明、Windows 注册表/原生 keyring 实现，以及 macOS workflow/DMG 配置。必须先做产品决策，再继续维护构建矩阵。

### 方案 A：明确仅支持 Windows

- 移除或暂停 macOS 发布任务和 DMG target。
- 文档明确 Windows 版本与最低系统要求。
- 配置 Windows Authenticode 证书和时间戳服务。
- 在干净 Windows 环境验证 MSI 安装、启动、退出和升级。

### 方案 B：正式支持 macOS

- 修复 `.exe`、注册表、进程发现和路径等 Windows 假设。
- 为 macOS 配置可持久化的 keyring backend。
- 明确构建单架构还是 universal binary；workflow 必须实际传入相应 target，而不只是安装 target。
- 配置 Developer ID 签名和 Apple notarization。
- 在 Intel 与 Apple Silicon 或 universal 产物上完成真实测试。

### 共同事项

- 区分 Tauri updater 签名与 Windows/macOS 操作系统代码签名；两者不是一回事。
- `createUpdaterArtifacts` 当前关闭，需要决定是否真正启用自动更新。
- 统一 `package.json`、Cargo、Tauri 和 tag 的版本号来源。
- 决定 `rust-version` 的真实最低版本，避免 manifest 与 CI 工具链长期不一致。

---

## 阶段 9：可观测性与真实客户端回归矩阵

**优先级：P1，贯穿每个稳定性批次**

实时链路已经完成较大调整，下一步应以真实日志和回归场景证明稳定，而不是继续无证据地重写。

### 日志应能回答

- 当前处于第几次连接尝试。
- 认证来自缓存、重新发现还是 401/403 后刷新。
- WS 为什么断开、为什么重连。
- HTTP 健康快照为什么触发、耗时多少、是否被新 WS 事件废弃。
- 当前召唤师、战绩和游戏会话在什么时点被清理及重新填充。
- 自动操作为什么执行或为什么被取消。

日志不能包含 token、密码、完整认证 header 或其他敏感信息。

### 手工回归矩阵

| 场景 | 预期结果 |
| --- | --- |
| 先启动 Nidalee，后启动 LoL | 连接后清理占位状态，再恢复召唤师、排位和战绩 |
| LoL 已运行，再启动 Nidalee | 监听就绪后连接，立即恢复当前会话，不丢首个快照 |
| 关闭 LoL | 先显示未连接，再清理账号、战绩和当前游戏状态 |
| 不退出 Nidalee，重新启动 LoL | 新连接不显示上一会话数据，并自动重新拉取 |
| Lobby → ReadyCheck → ChampSelect | 阶段顺序正确，自动接受和自动选人最多执行一次 |
| ChampSelect 中退出/秒退 | 所有延迟自动任务取消，不继续调用旧会话接口 |
| 快速切换搜索对象/对局详情 | 旧响应不能覆盖最新选择 |
| 快速切换 Dashboard 路由 | 图片节点和主要数据不重复初始化，手动刷新仍有效 |

## 四、代码规范清单

后续评审可直接使用以下检查项：

- [ ] 同一后端数据是否只有一个前端恢复入口？
- [ ] Store 是否是该状态的唯一写入所有者？
- [ ] 组件卸载后，异步响应是否仍可能写状态？
- [ ] 新请求是否能使旧请求失效？
- [ ] 每个 `setTimeout` / debounce / watcher / listener 是否有对应取消？
- [ ] 延迟动作执行前是否重新检查当前阶段和用户设置？
- [ ] 运行态是否被错误持久化到磁盘？
- [ ] 是否重复定义 Vue Query key、缓存时间或静态数据请求？
- [ ] 是否存在吞掉错误后仍输出“成功”的路径？
- [ ] Rust 网络 `await` 期间是否持有全局写锁？
- [ ] 生成声明是否由脚本更新，而不是手工修改？
- [ ] 新文件是否属于明确的 feature/domain，而不是继续塞进 `lib/index.ts` 或 `mod.rs`？
- [ ] 提交是否只包含一个主题，并且没有日志、缓存、原始对局 JSON 或本地构建目录？

## 五、明确暂不处理

为避免再次扩大范围，以下事项当前不作为后续主线：

- 不优化最终安装包体积和字体体积；用户已明确暂不需要。
- 不因为理论上的风险引入完整全局 `accountSessionGeneration`。
- 不继续重写已经稳定的 WS/认证链路；除非真实日志复现新问题。
- 不按行数一次性拆完所有大文件。
- 不在功能提交中顺手升级全部依赖。
- 不在未决定平台策略前宣称 macOS 已正式支持。

## 六、推荐的下一批提交

按当前状态，最合理的连续执行顺序是：

1. `test-cover-frontend-request-lifecycle`
2. 分模块清理 Clippy 告警，再决定何时启用严格门禁

完成这些批次后，再根据是否临近公开发布，在“文档治理 / 平台发布”和“大文件结构拆分”之间选择下一阶段。公开发布临近时优先处理发布平台、签名和 CI；否则优先清理质量基线和前端数据所有权。
