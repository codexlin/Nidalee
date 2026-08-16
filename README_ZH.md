<div align="center">
  <img src="src/assets/logo.svg" alt="Nidalee Logo" width="112" height="112">
  <h1>Nidalee</h1>
  <p><strong>基于 Tauri、Rust 与 Vue 的轻量英雄联盟桌面助手</strong></p>

  <p>
    <a href="https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode.zh-Hans"><img src="https://img.shields.io/badge/license-CC%20BY--NC--SA%204.0-orange.svg" alt="许可证"></a>
    <img src="https://img.shields.io/badge/Tauri-2-blue.svg" alt="Tauri 2">
    <img src="https://img.shields.io/badge/Vue-3.5-42b883.svg" alt="Vue 3.5">
    <img src="https://img.shields.io/badge/Rust_MSRV-1.88-orange.svg" alt="Rust MSRV 1.88">
    <img src="https://img.shields.io/badge/主要平台-Windows-blue.svg" alt="主要平台 Windows">
  </p>

  <p><a href="README.md">English</a> · <a href="docs/user-guide-zh.md">使用指南</a> · <a href="https://github.com/codexlin/Nidalee/releases">下载</a></p>
</div>

## 主要功能

- 当前账号仪表盘、排位概览与近期战绩。
- 基于 League Client 与 LiveClient 数据的实时双方队伍分析。
- 召唤师查询与对局详情。
- OP.GG 在线推荐与个人符文方案组成的构建中心。
- 对局接受辅助、英雄选禁辅助，以及受支持队列的符文配置辅助。
- 轻量的游戏内海克斯强化侧边栏。

Rust 后端负责英雄联盟客户端通信、会话状态与业务分析；Vue 前端只消费有类型约束的命令和事件，负责交互与展示。

## 下载安装

请从 [GitHub Releases](https://github.com/codexlin/Nidalee/releases) 下载最新 Windows MSI。Windows 是当前主要支持平台；发布流程也会生成实验性的 macOS universal DMG，但目前尚未完成 Apple 公证。

Windows 版本会请求管理员权限，以便可靠读取英雄联盟客户端连接参数。由于安装包暂未进行 Authenticode 签名，系统可能显示“未知发布者”。

## 本地开发

环境要求：

- Node.js 22.18 或更高版本
- pnpm 10.34.5
- Rust 1.88 或更高版本
- Windows WebView2 与 Tauri 2 系统依赖

```bash
git clone git@github.com:codexlin/Nidalee.git
cd Nidalee
pnpm install --frozen-lockfile

# 终端 A：前端
pnpm dev

# 终端 B：桌面应用
pnpm dev:app
```

常用检查：

```bash
pnpm build
cd src-tauri
cargo test --locked
```

开发者可继续阅读[架构说明](docs/ARCHITECTURE.md)、[文档索引](docs/README.md)和[发布指南](RELEASE.md)。

## 分支与发布

日常开发合并到 `release-v3-dev`，准备发布后再合并到 `main`。只有推送 `v1.0.0` 这类语义化版本 Tag 才会创建公开 Release；完整规则见 [RELEASE.md](RELEASE.md)。

## 许可证与免责声明

本项目使用 [CC BY-NC-SA 4.0](LICENSE) 许可，禁止商业用途，衍生作品必须采用相同许可。第三方许可见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

Nidalee 是独立开源项目，与 Riot Games 或腾讯无隶属或授权关系。应用只与本地英雄联盟客户端 API 通信，不注入、不修改、也不读取游戏内存。使用者仍需自行遵守游戏用户协议和所在地规则。
