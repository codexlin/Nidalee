# 领域层

领域层只包含与 UI、Tauri 命令和网络传输无关的业务规则。外部数据由
`infrastructure` 获取并转换后，再交给领域层处理。

## 模块

- `analysis/`：对局证据提取、分析策略、分析管线和结果模型。
- `ai_analysis/`：发送给 AI 的分析上下文和结果模型。

## 对局分析主链路

```text
Tauri command
  -> matches::analysis_service
  -> MatchFetcher
  -> MatchBundle / evidence
  -> analysis::pipeline::orchestrate_analysis
  -> MatchAnalysisResult
  -> frontend store / UI
```

约束：

- `domains` 不直接请求 LCU 或公网 API。
- `infrastructure` 负责认证、缓存、重试和 DTO 转换。
- 同一次分析只允许一条抓取链路，避免重复获取战绩与时间线。
- 新分析能力优先扩展 evidence 和 pipeline，不再创建平行的旧式分析器。
