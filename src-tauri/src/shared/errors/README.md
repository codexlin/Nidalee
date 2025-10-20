# 统一错误处理指南

## 📌 概述

Nidalee 项目采用 [thiserror](https://github.com/dtolnay/thiserror) 实现统一的错误处理策略，参考顶级 Rust 项目（tokio, serde, actix-web）的最佳实践。

### 设计原则

1. **结构化**：每种错误都有明确的类型和上下文
2. **可组合**：使用 `#[from]` 自动转换第三方错误
3. **可追踪**：包含详细的错误信息和上下文
4. **类型安全**：编译时捕获错误处理问题

---

## 🚀 快速开始

### 1. 基础使用

```rust
use crate::shared::errors::{NidaleeError, Result};

// 旧方式（不推荐）
pub async fn get_summoner_old() -> std::result::Result<Summoner, String> {
    Err("召唤师未找到".to_string())
}

// 新方式（推荐）✅
pub async fn get_summoner() -> Result<Summoner> {
    Err(NidaleeError::not_found("召唤师未找到"))
}
```

### 2. 返回不同类型的错误

```rust
use crate::shared::errors::{NidaleeError, Result};

pub async fn fetch_match_data(match_id: i64) -> Result<MatchData> {
    // LCU 连接错误
    if !is_lcu_connected() {
        return Err(NidaleeError::LcuNotFound);
    }

    // HTTP 请求错误（自动转换）
    let response = client.get(url).send().await?;  // reqwest::Error 自动转为 NidaleeError

    // JSON 解析错误（自动转换）
    let data: MatchData = response.json().await?;  // serde_json::Error 自动转为 NidaleeError

    // 数据验证错误
    if data.participants.is_empty() {
        return Err(NidaleeError::Validation(
            "对局数据缺少参与者信息".to_string()
        ));
    }

    Ok(data)
}
```

### 3. 使用便捷构造函数

```rust
// ✅ 推荐：使用便捷构造函数
return Err(NidaleeError::lcu_connection("连接超时"));
return Err(NidaleeError::not_found("未找到英雄数据"));
return Err(NidaleeError::analysis("数据不足，无法分析"));

// ⚪ 也可以：直接构造
return Err(NidaleeError::LcuConnection("连接超时".to_string()));
return Err(NidaleeError::NotFound("未找到英雄数据".to_string()));
```

### 4. 错误上下文

```rust
// 提供详细的错误上下文
pub async fn call_lcu_api(endpoint: &str) -> Result<Value> {
    let response = client.get(endpoint).send().await.map_err(|e| {
        NidaleeError::lcu_api_call(endpoint, e.to_string())
    })?;

    Ok(response.json().await?)
}

// 错误信息：LCU API call failed: /lol-summoner/v1/current-summoner - connection timeout
```

---

## 📚 错误类型完整列表

### LCU 相关错误

```rust
NidaleeError::LcuConnection(String)           // LCU 连接失败
NidaleeError::LcuNotFound                     // LCU 未启动
NidaleeError::LcuAuth(String)                 // 认证失败
NidaleeError::LcuWebSocket(String)            // WebSocket 错误
NidaleeError::lcu_api_call(endpoint, message) // API 调用失败
```

### 网络相关错误

```rust
NidaleeError::Http(reqwest::Error)            // HTTP 错误（自动转换）
NidaleeError::Timeout(String)                 // 超时
NidaleeError::NetworkUnavailable              // 网络不可用
```

### 数据相关错误

```rust
NidaleeError::Json(serde_json::Error)         // JSON 错误（自动转换）
NidaleeError::Parse(String)                   // 解析错误
NidaleeError::Validation(String)              // 验证错误
NidaleeError::not_found(msg)                  // 数据未找到
```

### 游戏状态错误

```rust
NidaleeError::invalid_game_state(expected, actual)  // 游戏状态不正确
NidaleeError::NotInGame                             // 不在游戏中
NidaleeError::OperationTimeout(String)              // 操作超时
```

### 分析相关错误

```rust
NidaleeError::analysis(msg)                   // 分析错误
NidaleeError::InsufficientData(String)        // 数据不足
```

### 通用错误

```rust
NidaleeError::internal(msg)                   // 内部错误
NidaleeError::NotImplemented(String)          // 未实现
NidaleeError::Other(String)                   // 其他错误（向后兼容）
```

---

## 🔄 迁移指南

### 从 `Result<T, String>` 迁移

#### 步骤 1：修改函数签名

```rust
// 之前
pub async fn get_summoner_info(client: &Client) -> Result<SummonerInfo, String> {
    // ...
}

// 之后
use crate::shared::errors::Result;  // 注意：这是我们的 Result，不是 std::result::Result

pub async fn get_summoner_info(client: &Client) -> Result<SummonerInfo> {
    // ...
}
```

#### 步骤 2：更新错误返回

```rust
// 之前
return Err("LCU 连接失败".to_string());
return Err(format!("未找到召唤师: {}", name));

// 之后（推荐方式）
return Err(NidaleeError::lcu_connection("LCU 连接失败"));
return Err(NidaleeError::not_found(format!("未找到召唤师: {}", name)));

// 之后（简便方式，向后兼容）
return Err("LCU 连接失败".into());  // 自动转为 NidaleeError::Other
```

#### 步骤 3：处理第三方库错误

```rust
// 之前
let response = client.get(url)
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

// 之后（自动转换）
let response = client.get(url)
    .send()
    .await?;  // reqwest::Error 自动转为 NidaleeError::Http

// 之后（添加上下文）
let response = client.get(url)
    .send()
    .await
    .map_err(|e| NidaleeError::lcu_connection(format!("请求失败: {}", e)))?;
```

#### 步骤 4：更新错误匹配

```rust
// 之前
match get_summoner_info().await {
    Ok(info) => println!("召唤师: {}", info.name),
    Err(msg) => eprintln!("错误: {}", msg),
}

// 之后（仍可使用，向后兼容）
match get_summoner_info().await {
    Ok(info) => println!("召唤师: {}", info.name),
    Err(e) => eprintln!("错误: {}", e),  // NidaleeError 实现了 Display
}

// 之后（类型匹配，推荐）
match get_summoner_info().await {
    Ok(info) => println!("召唤师: {}", info.name),
    Err(NidaleeError::LcuNotFound) => eprintln!("LCU 未启动"),
    Err(NidaleeError::NotFound(msg)) => eprintln!("未找到: {}", msg),
    Err(e) => eprintln!("其他错误: {}", e),
}
```

### Tauri 命令迁移

Tauri 命令仍然返回 `Result<T, String>`，但我们的 `NidaleeError` 可以自动转换：

```rust
use crate::shared::errors::Result;

// 内部函数使用 NidaleeError
async fn get_summoner_internal(client: &Client) -> Result<SummonerInfo> {
    // ...
}

// Tauri 命令自动转换
#[tauri::command]
pub async fn get_summoner(app: AppHandle) -> std::result::Result<SummonerInfo, String> {
    let client = get_client(&app)?;

    // NidaleeError 自动转为 String
    get_summoner_internal(&client)
        .await
        .map_err(|e| e.to_string())
}
```

---

## ✅ 最佳实践

### 1. 优先使用具体的错误类型

```rust
// ❌ 不好
return Err(NidaleeError::Other("连接失败".to_string()));

// ✅ 好
return Err(NidaleeError::lcu_connection("连接失败"));
```

### 2. 提供有用的错误上下文

```rust
// ❌ 不好
return Err(NidaleeError::NotFound("未找到".to_string()));

// ✅ 好
return Err(NidaleeError::not_found(format!(
    "未找到召唤师 {} 的战绩数据",
    summoner_name
)));
```

### 3. 使用 `?` 操作符简化代码

```rust
// ❌ 不好
let response = match client.get(url).send().await {
    Ok(r) => r,
    Err(e) => return Err(NidaleeError::Http(e)),
};

// ✅ 好
let response = client.get(url).send().await?;
```

### 4. 为特定场景添加新的错误变体

如果发现某个错误场景经常出现，考虑添加专门的错误变体：

```rust
#[derive(Error, Debug)]
pub enum NidaleeError {
    // ... 现有变体

    /// 英雄池分析错误
    #[error("Champion pool analysis error: {0}")]
    ChampionPoolAnalysis(String),
}
```

---

## 🔍 调试技巧

### 1. 打印完整的错误链

```rust
match some_operation().await {
    Err(e) => {
        eprintln!("错误: {}", e);
        eprintln!("详细: {:?}", e);  // Debug 输出，包含完整错误信息
    }
    _ => {}
}
```

### 2. 在日志中记录错误

```rust
use log::error;

if let Err(e) = risky_operation().await {
    error!("操作失败: {}", e);
    // 可以继续处理或返回
}
```

---

## 📖 参考资料

- [thiserror 文档](https://docs.rs/thiserror)
- [Rust 错误处理最佳实践](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [tokio 错误处理](https://github.com/tokio-rs/tokio/blob/master/tokio/src/io/error.rs)
- [serde 错误处理](https://github.com/serde-rs/serde/blob/master/serde/src/de/mod.rs)

---

## ❓ 常见问题

### Q: 为什么不使用 `anyhow`？

A: `anyhow` 适合应用程序，但 `thiserror` 更适合库代码。由于 Nidalee 的核心逻辑可能被其他模块复用，使用 `thiserror` 提供更好的类型安全性。

### Q: 需要立即迁移所有代码吗？

A: 不需要。`NidaleeError` 实现了 `From<String>` 和 `From<&str>`，因此旧代码可以继续工作。建议渐进式迁移：
1. 新代码使用 `NidaleeError`
2. 修改旧代码时顺便迁移
3. 核心模块优先迁移

### Q: 如何处理不确定的错误？

A: 使用 `NidaleeError::Other` 或 `NidaleeError::internal`：

```rust
// 临时占位
return Err("暂未实现".into());  // 转为 NidaleeError::Other

// 明确是内部错误
return Err(NidaleeError::internal("不应该到达这里"));
```

