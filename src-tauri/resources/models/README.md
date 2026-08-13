# PaddleOCR 模型文件

本目录存放 **PaddleOCRv5 mobile** 推理模型，用于游戏内海克斯增强的实时 OCR 识别。

## 模型规格

| 文件 | 大小 | 说明 |
|------|------|------|
| `paddleocr/det/inference.onnx` | ~4.7 MB | 文字区域检测模型 |
| `paddleocr/rec/inference.onnx` | ~15.8 MB | 文字识别模型（PP-OCRv5_mobile_rec） |
| `paddleocr/rec/inference.yml` | ~166 KB | 字符字典（18426 字符，含中文 / 英文 / emoji） |

**版本**：PaddleOCRv5 mobile（v4 升级版，预处理与 v4 略有差异）

## 模型来源

参考 **ARAMGG 助手**（[github.com/valkia/aramgg_client](https://github.com/valkia/aramgg_client)）的实际生产部署，
沿用其打包的同款模型，确保中文识别精度与社区一致。

> **License**: PaddleOCR 模型 / 代码遵循 **Apache-2.0**，字符字典与配置数据另行检查。
> 详细许可声明见 [`THIRD_PARTY_NOTICES.md`](../../../THIRD_PARTY_NOTICES.md)（如有）。

## 首次安装

模型文件在 `.gitignore` 中，**不会**通过 git 提交。首次构建时：

#### 方式 A：复用 ARAMGG 已下载的模型（官方推荐）

PP-OCRv5 mobile 模型的官方 BCE 路径经常变化（验证时大部分 404），
最可靠的获取方式是从 ARAMGG 仓库（已经验证 hash 一致）：

```bash
# 假设你已经克隆了 ARAMGG 仓库：
#   git clone https://github.com/valkia/aramgg_client.git ../aramgg_client

cp -r ../aramgg_client/resources/paddleocr/* src-tauri/resources/models/paddleocr/
```

#### 方式 B（备用）：从 PaddleOCR 官方下载

```bash
# PP-OCRv5 mobile Chinese models（官方百度云 BCE 路径）
DET_URL="https://paddleocr.bj.bcebos.com/PP-OCRv5/chinese/ch_PP-OCRv5_mobile_det_train.tar"
REC_URL="https://paddleocr.bj.bcebos.com/PP-OCRv5/chinese/ch_PP-OCRv5_mobile_rec_train.tar"

mkdir -p src-tauri/resources/models/paddleocr/{det,rec}
curl -L "$DET_URL" | tar x -C src-tauri/resources/models/paddleocr/det --strip 1
curl -L "$REC_URL" | tar x -C src-tauri/resources/models/paddleocr/rec --strip 1
```

> ⚠️ **2026-08 实测**：PP-OCRv5 官方路径全部 404，**强烈建议走方式 A**。

## 验证

```bash
# 模型版本校验（应该输出 PP-OCRv5_mobile_rec）
head -3 src-tauri/resources/models/paddleocr/rec/inference.yml

# SHA256 校验（与 ARAMGG 仓库一致）
sha256sum src-tauri/resources/models/paddleocr/{det,rec}/inference.onnx
# det:  1eb7b4f7ab657ebd1c66d5f79bca7497f29768a2e3c15e52daecbba1a8e4a039
# rec:  f2fb81dc0cf6bf07736e7422bab38c6636e776bc8b5bc8c8d3c7d7322cd8f3a9
```

## 加载时机

- **不会**随应用启动加载（避免 200MB 内存占用）
- 仅在 `gameflow-phase-change` 事件变为 `InProgress` 时懒加载
- 离开对局阶段立即释放
