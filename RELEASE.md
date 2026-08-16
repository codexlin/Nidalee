# Nidalee 发布指南

Nidalee 使用两条独立的 GitHub Actions 流程：

- `Build and Test`：验证普通提交和 Pull Request，不创建 GitHub Release。
- `Release`：只在推送版本 Tag 时运行，构建安装包、生成自动更新清单并公开 Release。

## 触发规则

### 持续集成（CI）

以下操作会运行 `.github/workflows/build.yml`：

- 向 `main` 或 `release-v3-dev` 推送提交；
- 向这两个分支发起 Pull Request；
- 在 GitHub Actions 页面手动运行构建验证。

CI 会检查前端 lint、格式、类型、测试和构建，并检查 Rust 格式、Clippy、测试、MSRV 与 TypeScript 契约漂移。非 Pull Request 的 CI 还会生成 Windows MSI 工作流产物，但不会发布给用户。

### 正式发布（CD）

`.github/workflows/release.yml` 只有一个入口：推送版本 Tag。Tag 必须符合以下格式：

```text
v1.0.0
v1.0.1
v1.1.0-beta.1
```

Tag 指向的提交必须已经包含在 `main` 中，否则发布会被拒绝。普通提交、分支推送和手动点击 Actions 都不会创建正式 Release。

## 发布流程

1. 功能或修复先合并到 `release-v3-dev`。
2. 完成真实客户端验证及 CI 门禁。
3. 将 `release-v3-dev` 合并到 `main` 并推送。
4. 确认 `main` 的 CI 全部通过。
5. 在 `main` 当前发布提交上创建带注释的 Tag，并推送该 Tag：

```bash
git checkout main
git pull --ff-only origin main
git tag -a v1.0.0 -m "Nidalee v1.0.0"
git push origin v1.0.0
```

Tag 是正式安装包的版本来源。发布工作流会在构建环境中同步更新 `package.json`、`Cargo.toml` 和 `tauri.conf.json`，不需要为发布单独提交机械版本修改。

## 发布产物

发布工作流按以下顺序执行：

1. 完整质量门禁；
2. Windows x64 MSI；
3. macOS universal DMG（Intel 与 Apple Silicon）；
4. 合并各平台 updater 信息到 `latest.json`；
5. 两个平台均成功后，将草稿 Release 公开。

先使用草稿是为了防止用户看到只有一半产物的 Release。Windows 与 macOS 顺序上传时，`tauri-action` 会读取已有 `latest.json` 并保留其中的平台条目，因此最终清单同时包含两个平台。

## 自动更新

应用从以下地址读取更新清单：

```text
https://github.com/codexlin/Nidalee/releases/latest/download/latest.json
```

仓库必须配置：

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

这两个 Secret 用于 Tauri updater 的安装包完整性签名。它们不是 Windows Authenticode 证书，也不是 macOS Developer ID。

可选仓库变量：

- `WS_BASE_URL`：生产环境需要远程 WebSocket 服务时设置。

首次安装支持 updater 的正式版本需要用户手动下载安装。之后版本号递增时，应用才能从 `latest.json` 自动升级。版本号不能倒退；已安装的 `v2.x` 不会自动降级到重新开始的 `v1.0.0`。

## 当前平台状态

| 平台 | 产物 | 状态 |
| --- | --- | --- |
| Windows x64 | `.msi` | 主要支持平台，updater 已签名 |
| macOS universal | `.dmg` | 自动打包，但尚未 Developer ID 签名或公证 |
| Linux | — | 当前不发布 |

Windows 安装包目前没有 Authenticode 签名，因此系统可能显示“未知发布者”；这不会阻止安装或 Tauri 自动更新签名校验。macOS 未公证的构建可能需要用户手动允许打开。

## 发布失败处理

- 质量门禁失败：修复代码后创建新的版本 Tag；不要移动已经公开使用的 Tag。
- 某个平台构建失败：Release 会保持草稿，不会成为 `latest`。
- 签名缺失：检查两个 Tauri signing Secret，不能临时关闭 updater 签名绕过。
- `latest.json` 缺平台：不要手工编辑资产；修复工作流并重新发布新版本。
- 误推 Tag：如果 Release 尚未公开，可删除 Release 和 Tag 后重新处理；已经公开的版本应发布更高版本修正。

发布状态：

- [GitHub Actions](https://github.com/codexlin/Nidalee/actions)
- [GitHub Releases](https://github.com/codexlin/Nidalee/releases)
