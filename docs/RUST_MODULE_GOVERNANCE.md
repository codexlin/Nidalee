# Rust 模块治理规范

这份文档记录已经采用的长期规范，不再维护按阶段推进的重构计划。

## 模块职责

- `infrastructure/game_session`：客户端认证、连接、Lobby 与游戏会话。
- `infrastructure/real_time/websocket`：WS supervisor、传输、fallback snapshot、事件 reducer 与实时 enrichment。
- `infrastructure/match_management`：战绩抓取、详情映射和实时队伍数据编排。
- `domains/analysis`：只依赖输入数据的分析、证据、评分与策略。
- `shared/types`：跨边界 DTO 和 `ts-rs` 导出。
- `common/commands/dev_tools`：仅 Debug 构建可见的数据采集和实验命令。

对外模块使用小型 facade 和 re-export 保持调用路径稳定；内部按完整业务职责拆分，不按函数数量机械拆文件。

## 异步与状态

1. 网络 `await` 期间不持有全局写锁。
2. 长任务必须绑定 generation 或取消句柄，并在提交结果前再次校验当前会话。
3. cache 更新与对应事件发布需要保持明确、可审查的顺序。
4. 同一资源的并发刷新使用 singleflight；调用者取消不能让协调器永久停留在运行态。
5. WS 重连复用同一生命周期内的 handler，避免命令读取旧 cache。
6. fallback snapshot 与 WS 事件必须进入同一 reducer，不能维护第二套状态逻辑。
7. 断线清理应幂等；重复 stop 不得产生不同结果。

## 错误处理

- 内部函数优先使用结构化错误和 `Result`。
- 只在 Tauri IPC 或最终日志边界转换为用户可读字符串。
- 不用错误文本驱动业务分支。
- 可恢复失败要保留已有有效数据；没有有效数据时才进入 unavailable/error 状态。
- 禁止吞掉错误后仍记录或返回成功。

## 数据与契约

- Serde 字段和 `ts-rs` 导出属于公共契约；移动类型时保持名称、字段、可选性和导出路径。
- ID 解析兼容 LCU 合法的 number/string 表达，但进入领域层后使用规范类型。
- 队列分类只有一个权威来源；调用方不得维护自己的 420/440/450 白名单。
- 未知 team、英雄或身份不能被静默归入另一合法类别。
- 原始 LCU 数据只在必要的 Debug 采集路径落盘，不进入提交。

## 文件与测试

- 小型、高内聚单元测试保留在源文件相邻的 `tests` 模块。
- 大型 fixture 或集成场景放入相邻 `*_tests.rs` 或 `src-tauri/tests/`。
- 测试保护边界、竞态和降级语义，不为私有实现的每一行编写脆弱测试。
- 纯结构拆分与行为修改分开提交；结构提交应证明 API、事件名和请求顺序未变化。

## 必须通过的门禁

```bash
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --lib --locked
cargo test --test '*' --locked
```

公共 DTO 变化时还需要在仓库根目录执行：

```bash
pnpm types:check
```

CI 另外验证 Release library 与 Rust 1.88 MSRV。

## Review 清单

- 是否在网络等待期间持锁？
- 旧 generation 是否可能覆盖当前账号或当前对局？
- 取消首个调用者后 singleflight 是否仍能结束？
- 断线、重连和应用重启是否恢复同一状态？
- fallback 与实时事件是否走同一 reducer？
- 新 DTO 是否同步到 TypeScript？
- Release 是否排除了 Debug-only 命令？
- 日志是否避免 token、认证 header 和完整敏感载荷？
