# Nidalee 文档

这里仅保留需要长期维护、能够描述当前代码事实的文档。阶段性计划、完成报告和一次性排障记录不作为正式文档保存；历史可以从 Git 提交记录查询。

## 用户文档

- [中文项目说明](../README_ZH.md)
- [使用指南](user-guide-zh.md)
- [版本发布与自动更新](../RELEASE.md)
- [第三方许可](../THIRD_PARTY_NOTICES.md)

## 架构与开发

- [整体架构](ARCHITECTURE.md)
- [Rust 模块治理规范](RUST_MODULE_GOVERNANCE.md)
- [构建中心架构](BUILD_CENTER_ARCHITECTURE.md)
- [构建中心与符文系统](RUNE_SYSTEM_GUIDE.md)
- [贡献指南](../CONTRIBUTORS.md)
- [UI 设计语言](../DESIGN.md)

## 文档维护规则

1. 文档必须描述当前实现，不记录“以后可能做”的大型方案。
2. 文件路径、命令、版本和工作流触发条件发生变化时，同一提交内更新对应文档。
3. 临时调查结论应进入 Issue、PR 或提交说明，不在 `docs/` 长期堆积。
4. 架构文档描述所有权和数据流，不复制具体函数实现。
5. 版本发布以 [RELEASE.md](../RELEASE.md) 为唯一流程说明。
