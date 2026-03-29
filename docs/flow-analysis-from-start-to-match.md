# Nidalee 应用启动到对局完整流程分析

> 分析时间：2025-03-29
> 分析范围：应用启动 → 连接建立 → 游戏阶段流转 → 对局进行

---

## 一、应用启动阶段

### 1.1 前端初始化 (`src/main.ts`)

```
应用启动
  ↓
createApp(App)
  ↓
配置 TanStack Query
  - staleTime: 5分钟
  - gcTime: 24小时
  - refetchOnWindowFocus: false
  ↓
注册插件
  - VueQueryPlugin
  - stores (Pinia)
  - router
  ↓
挂载应用 (#app)
```

### 1.2 前端初始化逻辑 (`src/shared/composables/app/useAppInitialization.ts`)

```
initializeApp()
  ↓
├─ 1. 初始化主题
│   settingsStore.initTheme()
│
├─ 2. 初始化游戏版本
│   invoke('get_game_version')
│   → dataStore.setGameVersion(latestVersion)
│
└─ 3. 初始化连接状态
    connectionStore.checkConnection()
```

### 1.3 后端启动 (`src-tauri/src/app.rs`)

```
setup_app(app)
  ↓
├─ 1. 配置日志 (debug模式)
├─ 2. 设置系统托盘
├─ 3. 初始化 ConnectionManager
│    → 启动监控轮询 (已被 WS 优化)
├─ 4. 启动 WebSocket (异步)
│    tokio::spawn(start_ws(app_handle))
└─ 5. 异步加载游戏数据
     init_champion_data()
     init_summoner_spell_data()
```

---

## 二、连接建立阶段

### 2.1 WebSocket 连接流程

```
WebSocket 启动 (src-tauri/src/infrastructure/real_time/websocket/service.rs)
  ↓
┌─────────────────────────────────────────────────────────────┐
│  while WS_RUNNING:                                             │
│    ↓                                                          │
│  等待认证信息 (ensure_valid_auth_info)                       │
│    - 从进程命令行获取 token 和 port                            │
│    - 缓存 30 分钟                                              │
│    ↓                                                          │
│  连接 LCU WebSocket                                           │
│    wss://127.0.0.1:{port}/                                    │
│    Authorization: Basic {riot:token}                         │
│    ↓                                                          │
│  连接成功                                                      │
│    WS_CONNECTED = true  ✅                                    │
│    ↓                                                          │
│  订阅基础事件                                                  │
│    → /lol-gameflow/v1/gameflow-phase                          │
│    → /lol-gameflow/v1/session                                 │
│    → /lol-summoner/v1/current-summoner                         │
│    ↓                                                          │
│  进入事件接收循环                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 ConnectionManager 轮询流程

```
ConnectionManager.start_monitoring()
  ↓
monitor_loop()
  ↓
检查: is_ws_connected()?
  ↓
┌─────────────────┬─────────────────┐
│   YES (WS 已连接)  │   NO (WS 断开)    │
│   ↓                │   ↓               │
│ 跳过检查          │ 执行轮询检查      │
│ (30秒后重试)      │                   │
│                   │ → 检测进程       │
│                   │ → 获取认证       │
│                   │ → 验证 API        │
└─────────────────┴─────────────────┘
```

### 2.3 连接状态事件流

```
后端 → 前端
  ↓
connection-state-changed 事件
  → connectionStore.updateConnectionState(state)
  → 前端更新连接状态 UI
```

---

## 三、游戏阶段流转

### 3.1 阶段流转图

```
None (客户端主界面)
  ↓
Lobby (房间)
  ├─→ Matchmaking (匹配中)
  │      ↓
  │   ReadyCheck (找到对局，等待接受)
  │      ↓
  │   ChampSelect (英雄选择)
  │      ↓
  └─→ InProgress (游戏中)
         ↓
      WaitingForStats (等待结算)
         ↓
      EndOfGame (游戏结束)
         ↓
      None (返回大厅)
```

### 3.2 阶段变化事件流

```
WebSocket 推送
  ↓
/lol-gameflow/v1/gameflow-phase 变化
  ↓
WsEventHandler.handle_event()
  ↓
app.emit("gameflow-phase-change", &phase)
  ↓
前端 useAppEvents.startListening()
  ↓
handleGameFlowPhaseChange()
  ↓
GamePhaseManager.handleGamePhaseChange(phase)
```

### 3.3 各阶段处理 (`useGamePhaseManager.ts`)

| 阶段 | 处理动作 |
|------|---------|
| **None** | 返回客户端主界面，清理游戏状态 |
| **Lobby** | 更新大厅信息，记录活动日志 |
| **Matchmaking** | 跳转对局分析页面，启动自动接受 |
| **ReadyCheck** | 触发自动接受对局（如果启用） |
| **ChampSelect** | 记录活动日志，准备自动选人 |
| **InProgress** | 记录游戏开始 |
| **WaitingForStats** | 记录游戏结束，清理状态，更新战绩 |

---

## 四、英雄选择阶段

### 4.1 动态订阅机制

```
进入 ChampSelect 阶段
  ↓
WebSocket 检测到 phase 变化
  ↓
动态订阅新事件
  → /lol-champ-select/v1/session
  → /lol-lobby/v2/lobby (取消订阅)
  → /lol-matchmaking/v1/search (取消订阅)
```

### 4.2 选人事件流

```
WebSocket 推送
  ↓
/lol-champ-select/v1/session 变化
  ↓
WsEventHandler.handle_event()
  ↓
app.emit("champ-select-session-changed", &session)
  ↓
前端 useAppEvents.startListening()
  ↓
ChampSelectManager.handleChampSelectChange(session)
  ↓
gameStore.updateChampSelectSession(session)
  ↓
触发自动选人检查 (useAutoRune, useAutoChampion)
```

### 4.3 自动选人逻辑

```
handleChampSelectChange()
  ↓
检查: 是否轮到我选人？
  ↓
┌─────────────────┬─────────────────┐
│  是              │  否                │
│  ↓                │  ↓               │
│ 检查配置          │ 等待下次轮次    │
│ - 自动选人启用？   │                  │
│ - 英雄列表        │                  │
│ - 备选英雄        │                  │
│  ↓                │                  │
│ 执行选人          │                  │
│ invoke('pick_champion') │
└─────────────────┴─────────────────┘
```

---

## 五、对局进行阶段

### 5.1 游戏开始流程

```
InProgress 阶段
  ↓
WebSocket 可能推送事件
  - /liveclient-data (实时数据)
  - /lol-gameflow/v1/session 更新
  ↓
前端处理
  - 游戏数据展示
  - 实时统计
```

### 5.2 数据缓存机制

```
版本化数据缓存
  ↓
┌─────────────────────────────────────────────────────────┐
│  静态数据 (版本驱动)                                   │
│  - queryKey: ['static', 'champions', version]           │
│  - staleTime: Infinity                                 │
│  - gcTime: Infinity                                    │
│  - 版本变化时自动失效                                  │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  战绩分析数据 (会话缓存)                                │
│  - team_analysis_data (EventCache)                     │
│  - 由 WS 事件或 HTTP Fallback 更新                        │
└─────────────────────────────────────────────────────────┘
```

---

## 六、关键优化点

### 6.1 WS 驱动的轮询优化

```
┌─────────────────────────────────────────────────────────┐
│  ConnectionManager 轮询逻辑                            │
│                                                          │
│  if is_ws_connected():                                  │
│      → 跳过轮询 (30秒后重试)                             │
│  else:                                                   │
│      → 执行 check_connection_state()                      │
└─────────────────────────────────────────────────────────┘

效果：WS 正常时，ConnectionManager 几乎不消耗资源
```

### 6.2 认证信息缓存

```
ensure_valid_auth_info()
  ↓
缓存检查 (30 分钟)
  ↓
┌────────┬────────┐
│ 存在   │ 不存在  │
│ ↓      │ ↓      │
│ 返回   │ 重新   │
│ 缓存   │ 获取    │
└────────┴────────┘
```

### 6.3 进程检测优化

```
has_lol_process()
  ↓
快速路径: 检查缓存的 PID
  → 存在 → 验证进程是否仍运行
  ↓
慢速路径: 扫描所有进程
  → 查找 LeagueClientUx.exe
  → 缓存 PID 和进程名
```

---

## 七、事件订阅时序图

```
时间线
  │
  │  应用启动
  ├─→ WebSocket 连接建立
  │   └─→ 订阅: /lol-gameflow/v1/gameflow-phase
  │   └─→ 订阅: /lol-gameflow/v1/session
  │   └─→ 订阅: /lol-summoner/v1/current-summoner
  │
  ├─→ [进入 Lobby]
  │   └─→ WS推送: gameflow-phase-change → "Lobby"
  │   └─→ 动态订阅: /lol-lobby/v2/lobby
  │   └─→ 动态订阅: /lol-matchmaking/v1/search
  │
  ├─→ [开始匹配]
  │   └─→ WS推送: matchmaking-state-changed
  │
  ├─→ [找到对局]
  │   └─→ WS推送: gameflow-phase-change → "ReadyCheck"
  │   └─→ 自动接受对局 (如果启用)
  │
  ├─→ [进入选人]
  │   └─→ WS推送: gameflow-phase-change → "ChampSelect"
  │   └─→ 动态订阅: /lol-champ-select/v1/session
  │   └─→ 取消订阅: /lol-lobby/v2/lobby
  │   └─→ 取消订阅: /lol-matchmaking/v1/search
  │   └─→ WS推送: champ-select-session-changed (持续更新)
  │   └─→ 触发自动选人
  │
  ├─→ [游戏开始]
  │   └─→ WS推送: gameflow-phase-change → "InProgress"
  │   └─→ 取消订阅: /lol-champ-select/v1/session
  │
  └─→ [游戏结束]
      └─→ WS推送: gameflow-phase-change → "EndOfGame"
      └─→ 订阅恢复: /lol-gameflow/v1/gameflow-phase
```

---

## 八、数据流图

```
┌─────────────────────────────────────────────────────────────┐
│                        LCU 客户端                          │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  LCU API (HTTPS)        WebSocket (WAMP)                │  │
│  │  /lol-gameflow/v1/*     OnJsonApiEvent               │  │
│  │  /lol-champ-select/*    ┌──────────────────────────┐   │  │
│  │  /lol-lobby/v2/*        │ phase: "ChampSelect"     │   │  │
│  │  /lol-summoner/v1/*      │ data: {...}                │   │  │
│  │                         │ lobby: {...}               │   │  │
│  │                         │ summoner: {...}            │   │  │
│  │  └────────────────────┴───────────────────────────┘   │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                         │
                         │ WebSocket + HTTPS
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                     Rust 后端 (Tauri)                       │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  WebSocket 服务层                                     │  │
│  │  - start_ws() - 启动和重连                            │  │
│  │  - connect_and_run_ws() - 接收事件                      │  │
│  │  - WsEventHandler - 处理事件并转发前端                │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  ConnectionManager (轮询，备用)                         │  │
│  │  - check_connection_state() - 检查进程和认证            │  │
│  │  - is_ws_connected() - WS 连接时跳过轮询               │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Tauri Commands (前端调用)                            │  │
│  │  - get_current_summoner                               │  │
│  │  - pick_champion                                     │  │
│  │  - get_match_history                                 │  │
│  │  - get_opgg_champion_build                           │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                         │
                         │ Tauri Events (invoke/event)
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                     Vue 前端                                │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  初始化                                              │  │
│  │  - useAppInitialization → 初始化主题、版本、连接           │  │
│  │  - TanStack Query → 配置缓存策略                          │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  事件监听 (useAppEvents)                                 │  │
│  │  - startListening() → 监听所有后端事件                  │  │
│  │    - gameflow-phase-change                             │  │
│  │    - lobby-change                                      │  │
│  │    - champ-select-session-changed                     │  │
│  │    - matchmaking-state-changed                        │  │
│  │    - team-analysis-data                                │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  业务处理 Composables                                   │  │
│  │  - useGamePhaseManager → 处理游戏阶段变化                │  │
│  │  - useChampSelectManager → 处理英雄选择                  │  │
│  │  - useAutoRune → 自动符文                               │  │
│  │  - useAutoChampion → 自动选人                            │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  状态管理 (Pinia Stores)                                 │  │
│  │  - gameStore → 游戏状态、英雄选择、大厅信息               │  │
│  │  - connectionStore → 连接状态                            │  │
│  │  - autoFunctionStore → 自动功能配置                     │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 九、时间线总结

从启动到一局游戏的完整时间线：

| 阶段 | 时间点 | 关键动作 |
|------|--------|----------|
| **应用启动** | T=0s | 初始化主题、版本、连接 |
| **WS 连接** | T≈2s | 获取认证、连接 WS、订阅基础事件 |
| **游戏数据加载** | T≈5s | 并行加载英雄和技能数据 |
| **进入大厅** | 用户操作 | WS 推送 `Lobby` 阶段 |
| **大厅订阅** | Lobby | 动态订阅 `/lol-lobby/v2/lobby` |
| **开始匹配** | 用户操作 | WS 推送匹配状态变化 |
| **接受对局** | ReadyCheck | 自动接受（如果启用） |
| **进入选人** | ChampSelect | 动态订阅 `/lol-champ-select/v1/session` |
| **自动选人** | 轮到我 | 检查配置 → 执行选人 |
| **游戏开始** | InProgress | 取消英雄选择订阅 |
| **游戏结束** | EndOfGame | 清理状态、更新战绩 |

---

## 十、关键文件索引

| 模块 | 文件路径 | 职责 |
|------|---------|------|
| **应用启动** | `src-tauri/src/app.rs` | 后端启动入口 |
| **前端初始化** | `src/shared/composables/app/useAppInitialization.ts` | 前端初始化 |
| **WebSocket 服务** | `src-tauri/src/infrastructure/real_time/websocket/service.rs` | WS 连接管理 |
| **WS 事件处理** | `src-tauri/src/infrastructure/real_time/websocket/event_handler.rs` | 事件转发前端 |
| **连接管理** | `src-tauri/src/infrastructure/game_session/connection/service.rs` | 进程/认证监控 |
| **阶段管理** | `src/shared/composables/game/useGamePhaseManager.ts` | 阶段变化处理 |
| **选人管理** | `src/shared/composables/game/useChampSelectManager.ts` | 选人事件处理 |
| **事件监听** | `src/shared/composables/app/useAppEvents.ts` | 前端事件注册 |
| **自动功能** | `src/shared/stores/features/autoFunctionStore.ts` | 自动选人/禁用配置 |
| **数据缓存** | `src/shared/composables/data/useVersionedData.ts` | 版本化缓存策略 |

---

*本文档记录应用从启动到进行一局游戏的完整流程，作为代码理解和后续优化的参考。*
