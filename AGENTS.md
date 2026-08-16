# Agent Guidelines

## Git 分支工作流（必须遵守）

开发基线分支是 **`main`**。

### 命名

| 类型 | 分支名 | 说明 |
|------|--------|------|
| 新功能 | `feature/<简短英文描述>` | 例如 `feature/lobby-chat` |
| Bug 修复 | `fixbug/<简短英文描述>` | 例如 `fixbug/today-winrate` |

### 流程

1. 从最新的 `main` 拉取并切出新分支：
   ```bash
   git checkout main
   git pull --ff-only origin main
   git checkout -b feature/xxx   # 或 fixbug/xxx
   ```
2. 在 feature / fixbug 分支上开发、提交。
3. 完成后 **merge 回 `main`**（优先使用 Pull Request，也可按用户要求本地合并）。
4. 未经用户明确要求，不要直接在 `main` 上堆功能提交；改动应落在对应 feature/fixbug 分支。

### 例外

- 用户明确说「就在当前分支改 / 直接提交到 main」时可例外。
- 文档-only、紧急热修等若用户另有指示，以用户当次指令为准。

## 更多项目约定

- 架构、命令与数据分层见 [`CLAUDE.md`](./CLAUDE.md)
- UI 设计语言见 [`DESIGN.md`](./DESIGN.md)（浮钮 `rounded-2xl` / 主卡 `rounded-xl`）
