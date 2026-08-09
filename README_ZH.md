<div align="center">
  <img src="src/assets/logo.svg" alt="Nidalee Logo" width="120" height="120">

  <h1>🎮 Nidalee</h1>
  <p><strong>高性能、体积小巧的智能英雄联盟游戏助手</strong></p>
  <p>Nidalee 是一款专为英雄联盟玩家设计的智能助手，集自动接受匹配、自动选/禁英雄、实时数据分析与个性化设置于一体，助你高效上分，安全合规无外挂风险。基于 Rust + Tauri，启动快、资源占用低，体积小巧。</p>

  <div>
    <a href="https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode.zh-Hans" target="_blank"><img src="https://img.shields.io/badge/license-CC%20BY--NC--SA%204.0-orange.svg" alt="License"/></a>
    <img src="https://img.shields.io/badge/platform-Windows-blue.svg" alt="Platform">
    <img src="https://img.shields.io/badge/tauri-2.0.0--alpha-green.svg" alt="Tauri">
    <img src="https://img.shields.io/badge/vue-3.x-brightgreen.svg" alt="Vue">
    <img src="https://img.shields.io/badge/rust-1.75-orange.svg" alt="Rust">
  </div>

  <br>

  <p>
    <a href="#-特性">✨ 特性</a> •
    <a href="#-下载安装">📦 下载</a> •
    <a href="#-开发">🚀 开发</a> •
    <a href="docs/user-guide-zh.md">📖 使用指南</a> •
    <a href="#-贡献">🤝 贡献</a>
  </p>

  <p>
    <a href="./README_ZH.md">简体中文</a> | <a href="./README.md">English</a>
  </p>
</div>

---

## ✨ 特性

- 🤖 **自动化功能**：自动接受匹配、自动选择/禁用英雄
- 📊 **数据分析**：实时游戏数据分析和统计
- 🎯 **个性化设置**：可自定义的游戏助手配置,生涯背景配置等等
- 🔒 **安全可靠**：直接与League Client API交互，无需第三方工具

## 📦 下载安装

前往 [Releases](../../releases) 页面下载最新 Windows 版本：

| 平台 | 下载文件 | 说明 |
|------|----------|------|
| **Windows** | `Nidalee_1.0.0_x64_en-US.msi` | Windows 64位安装程序 |

### 安装说明

#### Windows

1. 下载 `.msi` 文件
2. 双击运行安装程序
3. 按照向导完成安装

## 🚀 开发

### 环境要求

- **Node.js** 18+
- **pnpm** 8+
- **Rust** 1.70+
- **Tauri CLI** 2.0+

### 本地开发

```bash
# 克隆项目
git clone https://github.com/codexlin/Nidalee.git
cd Nidalee

# 安装依赖
pnpm install

# 开发模式
# 两个终端（beforeDevCommand 已清空，避免改 Rust 时把 Vite 一起崩掉）
pnpm dev          # 终端 A：Vite http://localhost:1422
pnpm dev:app      # 终端 B：Tauri（或 pnpm tauri dev）

# 构建生产版本
pnpm tauri build
```

### 项目结构

```
Nidalee/
├── src/                    # Vue.js 前端代码
├── src-tauri/             # Tauri Rust 后端代码
├── .github/workflows/     # GitHub Actions CI/CD
├── dist/                  # 构建输出
└── docs/                  # 项目文档
```

## 📋 功能清单

- [x] 基础架构搭建
- [x] League Client API 集成
- [x] CI/CD 自动化发布
- [x] 用户信息获取和展示
- [x] 召唤师特征分析
- [x] 自动接受匹配功能
- [x] 自动选择/禁用英雄
- [x] 游戏数据分析
- [x] 个性化设置界面
- [ ] 多语言支持

## 📖 使用指南

详细的使用说明请查看 [使用指南](docs/user-guide-zh.md)

## 🤝 贡献

欢迎提交 Pull Request 或 Issue！

### 发布流程

详见 [发布指南](RELEASE.md)

简要步骤：

1. 更新版本号
2. 创建 git tag: `git tag v1.0.1`
3. 推送 tag: `git push origin v1.0.1`
4. GitHub Actions 自动构建并发布

## 📄 许可证

本项目采用 [CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode.zh-Hans)（署名-非商业性使用-相同方式共享）国际许可协议。

- 允许自由复制、分发、演绎，但**禁止任何商业用途**。
- 衍生作品必须采用相同协议。
- 使用时请注明原作者及项目地址。

完整条款详见 LICENSE 文件。

## ⚠️ 免责声明

本项目 **Nidalee** 仅作为英雄联盟玩家的辅助工具，所有功能均基于 Riot Games 官方公开的 League Client API（LCU API）和客户端本地数据实现。

**本工具不进行任何游戏内存、进程、网络数据的篡改，不注入、不修改、不破解游戏客户端，也不具备任何外挂、作弊、加速、脚本等功能。**

- 本项目严格遵守英雄联盟用户协议及相关法律法规，仅供学习、研究和个人娱乐用途。
- 所有数据交互均通过官方API完成，未对游戏客户端、服务器或数据包进行任何非官方操作。
- 本工具不会收集、上传或泄露用户的任何隐私信息、账号密码等敏感数据。
- 本项目为开源软件，与 Riot Games（拳头公司）及腾讯公司无任何直接或间接关联，亦未获得其官方授权。
- 使用本工具所产生的任何后果（包括但不限于账号风险、数据丢失、功能异常等），开发者不承担任何法律责任和经济责任。
- **本项目及其所有衍生作品严禁任何商业用途，所有贡献和再分发须采用相同协议。**

**请用户在使用本工具前，务必确保自身行为符合英雄联盟用户协议及相关政策。如有疑问，请及时停止使用并咨询官方客服。**

---

**Built with ❤️ using [Tauri 2.0](https://tauri.app/) + [Vue.js](https://vuejs.org/)**
