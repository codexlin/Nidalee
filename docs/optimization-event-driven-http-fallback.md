# 优化提案：事件驱动的 HTTP Fallback

## 背景

当前实现中，WebSocket 有一个 10 秒空闲的 HTTP Fallback 机制：

```rust
tokio::select! {
    Some(msg_result) = ws_stream.next() => { /* 处理 WS 消息 */ }
    _ = tokio::time::sleep(Duration::from_secs(10)) => {
        // 10 秒空闲 → 全量 HTTP 拉取
        fallback_fetch_and_emit(app, current_phase.as_deref()).await;
    }
}
```

## 问题

1. **盲目拉取**：不管是否需要都拉取所有数据
2. **资源浪费**：即使 WS 正常工作也在拉取
3. **不可预测**：前端不知道后端何时会拉取数据

## 优化方案

### 核心思想

**完全依赖 WebSocket，事件驱动的按需补偿**

```
WS 收到事件 → 分析缺失数据 → 只请求缺失的部分
```

### 实现逻辑

```rust
async fn handle_ws_event(event: LcuEvent) {
    match event {
        // 游戏阶段变化
        Event::GameflowPhaseChanged { phase } => {
            // 1. 发送阶段变化事件
            emit("gameflow-phase-change", &phase);

            // 2. 根据新阶段，补充拉取缺失的数据
            match phase {
                "ChampSelect" => {
                    // 阶段变了，但面板数据可能还没推送
                    // 主动拉取一次确保数据完整
                    let session = get_champ_select_session().await?;
                    emit("champ-select-session-changed", &session);
                }
                "Lobby" => {
                    let lobby = get_lobby_info().await?;
                    emit("lobby-change", &lobby);
                }
                "Matchmaking" => {
                    let state = get_matchmaking_state().await?;
                    emit("matchmaking-state-changed", &state);
                }
            }
        }

        // 大厅更新（WS 可能只推送增量）
        Event::LobbyChanged { data } => {
            // 检查数据是否完整，不完整才补充
            if needs_completion(&data) {
                let full_lobby = get_lobby_info().await?;
                emit("lobby-change", &full_lobby);
            } else {
                emit("lobby-change", &data);
            }
        }
    }
}
```

### 事件驱动的补偿策略

| WS 事件 | 可能缺失 | 补偿动作 |
|---------|---------|---------|
| `gameflow-phase-change` | 完整面板数据 | 拉取对应阶段的 session |
| `lobby-change` (增量) | 成员列表/队伍信息 | 拉取完整 lobby 数据 |
| `champ-select-session-changed` (增量) | 其他玩家数据 | 按需补充 |
| `matchmaking-state-changed` | 队列信息 | 按需补充 |

## 实现步骤

### Phase 1: 移除定时 Fallback
- 移除 `tokio::time::sleep(Duration::from_secs(10))` 的 fallback
- 只依赖 WS 事件驱动

### Phase 2: 事件处理器增强
- 在 `WsEventHandler` 中添加事件分析逻辑
- 判断每个事件是否需要补充数据

### Phase 3: 按需补偿
- 实现各事件的补偿逻辑
- 避免重复拉取已有数据

## 预期效果

| 指标 | 当前 | 优化后 |
|------|------|--------|
| HTTP 请求数 | 每 10 秒一次 | 按需（大幅减少） |
| 数据新鲜度 | 10 秒延迟 | 事件驱动（更及时） |
| 资源消耗 | 持续请求 | 按需请求 |
| 可预测性 | 低 | 高（随事件触发） |

## 备注

- 保留 ConnectionManager 的轮询作为最后兜底（检测进程存活）
- 完全信任 WebSocket 推送的游戏事件
- 只在明确缺失数据时才发起 HTTP 补偿
