# 对局分析领域

本目录只保留一条生产分析链：

```text
LCU 对局列表/详情/时间线
  -> analyzers/core/parser（适配输入形状）
  -> evidence（提取可验证事实、位置与对手）
  -> pipeline/orchestrator（队列策略、降级、有限并发）
  -> pipeline/insights / trait_strategies（展示结论）
  -> MatchAnalysisResult
```

## 边界

- `analyzers/core`：输入解析、基础统计和分析模式；不访问网络。
- `analyzers/traits`：由 pipeline 调用的确定性特征规则；不访问网络。
- `evidence`：从详情和时间线提取事实，处理缺失时间线与对位识别。
- `pipeline`：唯一业务编排入口，决定队列能力、分析深度和降级诊断。
- `services`：领域级组合服务；不得绕过 pipeline 创建第二套分析流程。
- `thresholds` / `queue_config`：集中管理阈值和队列差异。

## 约束

1. 网络和 LCU 调用位于 `infrastructure/match_management`，领域层只接收数据。
2. 新结论必须先有 evidence，再由 insight/trait 生成展示文本。
3. 缺失时间线必须显式降级，不得用零值伪造完整证据。
4. 不恢复已删除的旧 timeline bridge、独立 opponent/teammate/self analyzer。
5. 新增规则时优先写确定性单元测试，再接入 orchestrator。
