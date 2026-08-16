# Nidalee 架构

Nidalee 是 Tauri 2 桌面应用。Rust 后端负责 League Client 通信、会话状态、数据归一化和分析；Vue 前端负责交互、视图状态和展示。

## 核心原则

1. 后端提供事实与分析结果，前端不重复实现业务算法。
2. 实时数据只有一条主链：LCU/LiveClient → Rust reducer/cache → Tauri event → Pinia → UI。
3. 请求型数据使用明确的 query key 和生命周期，旧响应不能覆盖当前账号、筛选或详情。
4. 运行时会话不持久化；客户端断开时清理账号、游戏和实时分析状态。
5. Rust 公共 DTO 由 `ts-rs` 生成 TypeScript 契约，禁止手工维护重复类型。

## 前端边界

```text
src/
├─ views/          路由壳层，只组合功能
├─ features/       按业务功能组织的页面、组件和局部逻辑
├─ shared/         跨功能模型、Store、composable 和工具
├─ components/     通用 UI 与布局组件
├─ router/         路由名称、路径和窗口隔离
├─ types/          生成的 Rust 契约与前端声明
└─ styles/         设计 token、字体和全局样式
```

依赖方向：

```text
views → features → shared
views/features → components
shared 不依赖 features
```

Pinia 只保存客户端状态和跨页面会话状态。在线推荐、可缓存请求等服务端状态优先交给 Vue Query。组件不得复制 Store 中已经存在的状态。

## Rust 边界

```text
src-tauri/src/
├─ infrastructure/  LCU、LiveClient、WebSocket、持久化和外部服务适配
├─ domains/         无 UI、无传输细节的分析与业务规则
├─ shared/          DTO、错误、请求基础设施和跨域能力
├─ common/          应用级命令与 Debug-only 工具
└─ lib.rs / app.rs  Tauri 组装、插件和命令注册
```

`infrastructure` 可以调用 `domains`，但 `domains` 不反向依赖 Tauri、HTTP 或 UI。网络 `await` 期间不得持有全局写锁；Tauri command 是错误字符串化与权限检查的边界。

## 实时会话数据流

```text
League Client process / lockfile
  → AuthInfo
  → WebSocket supervisor
  → transport decoder
  → WsEventHandler reducer
  → EventCache
  → identity/history enrichment
  → TeamAnalysisData
  → team-analysis-data event
  → useAppEvents
  → matchAnalysisStore
  → MatchAnalysis UI
```

- 前端先注册监听，再调用 `start_lcu_ws`，避免启动快照丢失。
- WS 是主通道；无事件时的 HTTP snapshot 通过相同 reducer 校准状态。
- generation 与取消句柄阻止旧选人、旧对局和旧网络请求回写。
- Champ Select 可以先发布基础 roster，再异步补齐身份、段位和战绩。
- 断线 reducer 先发布未连接状态，再清理依赖该连接的数据。

## 请求型数据流

Dashboard 与战绩查询复用相同的后端分析能力，但由各自页面持有筛选条件。请求 key 至少包含目标 PUUID 与分析范围；query function 使用 key 中的快照，而不是在重试时读取最新响应式值。

对局详情使用显式目标身份和 latest-request guard。打开新玩家或新对局后，旧响应即使稍后返回也不能更新当前弹层。

## 静态目录

英雄、召唤师技能、符文、装备与队列元数据通过版本化静态目录提供：

- Rust 负责下载、完整性检查、磁盘缓存和离线回退；
- 前端将目录安装到响应式内存映射；
- UI 只按 ID 查询名称和图片信息；
- 未知版本不得写入磁盘缓存。

## 类型契约

Rust DTO 使用 `ts-rs` 导出到 `src/types/generated/`，随后由脚本合并到 `src/types/global.d.ts`：

```bash
pnpm types
```

CI 会重新生成并检查 `global.d.ts` 漂移。修改公共 Rust 类型时必须同步生成文件，不能直接编辑合并结果。

Vue 自动导入声明由 Vite 插件生成到 `types/auto-imports.d.ts` 和 `types/components.d.ts`。只有生成配置变化或扫描结果变化时才更新这些文件。

## 模块新增规则

- 新路由：在 `views/` 建路由壳，在 `features/` 建业务实现，并更新 `router/appRoutes.ts`。
- 新 Tauri 命令：放入对应 `infrastructure/<area>/commands.rs`，在 `lib.rs` 注册。
- 新分析规则：优先放入 `domains/analysis`，通过稳定 facade 暴露。
- 两个模块的相似代码不自动等于公共抽象；只有所有权和失败语义一致时才提取。
- Debug 数据采集命令必须留在 `common/commands/dev_tools` 并受 `debug_assertions` 隔离。

更细的 Rust 约束见 [RUST_MODULE_GOVERNANCE.md](RUST_MODULE_GOVERNANCE.md)。
