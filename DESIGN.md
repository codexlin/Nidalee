# Nidalee Design Language

气质：**暗色竞技桌面** — 磨砂悬浮（macOS / iOS toolbar 感），克制、可读、分层清晰。

## 颜色（OKLCH only）

对齐 **Tailwind CSS v4** 与 **shadcn-vue** 官方：

| 项 | 约定 |
|----|------|
| 真源文件 | `src/styles/theme.css` |
| 色值格式 | 完整 `oklch(...)`，写入 `:root` / `.theme-*` / `.dark` |
| `@theme inline` | `--color-*: var(--*)`，**禁止**再包 `hsl()` |
| JS / Canvas | 用 `themeColor()` / `themeColors()`（`src/lib/themeColor.ts`） |
| 禁止 | 裸 HSL 通道、`hsl(var(--token))`、双轨主题文件 |

胜负语义后续收拢为 `--win` / `--loss`；当前可用语义色或 Tailwind 语义 token。

## 圆角约定

| 角色 | Token | 值 | Tailwind |
|------|-------|-----|----------|
| 次级浮钮 / 工具控件 | `--radius-float` | `1rem` | `rounded-2xl` |
| 主内容卡片 / chip / inset | `--radius-raised` | `0.75rem` | `rounded-xl` |

小控件更圆、大卡片略收，避免巨型「气球」面。

## 字体

| 项 | 约定 |
|----|------|
| 族名 | `HarmonyOS Sans SC`（`--font-display`） |
| 嵌入字重 | Regular `400` / Medium `500` / Bold `700`（三档 woff2；首屏只预载 Regular） |
| 官方没有 | SemiBold `600`、ExtraBold `800`；Black `900` 有文件但不嵌入（收益小） |
| 文件 | `src/assets/fonts/harmonyos-sans-sc/*.woff2` + `LICENSE.txt` |
| `@font-face` | `src/styles/fonts.css` |
| 回退 | `Microsoft YaHei`, `sans-serif` |
| 授权 | 须在软件内显著声明使用；不可改字体；保留协议；不可单独售卖字体 |

`style.css` `@theme`：`thin`–`light` → 400，`semibold`/`extrabold`/`black` → 700；`font-synthesis: none`。新代码优先写 `font-normal` / `font-medium` / `font-bold`。

## 字号层级

默认栈：HarmonyOS Sans SC（见 `--font-display`）。数字优先 `tabular-nums`。

| 角色 | Tailwind | 重量 | 示例 |
|------|----------|------|------|
| 标签 / 辅助说明 | `text-xs` | normal，多用 `text-muted-foreground` | 「等级」「胜率」标签、样本提示、相对时间 |
| 正文 / 控件 / 状态值 | `text-sm` | medium / bold | 昵称、会话时长、等级值、下拉、特征行内文 |
| 区块小标题 | `text-base` | medium / bold | 「召唤师特征」「常用英雄」「最近对局」 |
| 页内主标题 / 次级 KPI | `text-lg` | medium / bold | 「游戏统计」、段位名、KDA、特征胜率、今日对局数 |
| 主 KPI | `text-2xl` | medium / bold | 概览总胜率 |

原则：

- **标签 xs + 数值更大**；同级信息字号对齐（如 header 状态条等级与会话同为 `sm`）  
- 一屏主焦点最多一个 `2xl`；次级数字用 `lg`，勿到处 `xl`/`2xl`  
- 小标题须写明 `text-base`，避免依赖继承  

## 表面层级

| 类名 | 用途 |
|------|------|
| `surface-float` | Header 通知/刷新、文字次级操作（如「加载更多」） |
| `surface-chip` | Header 状态条（连接信息）；与 float 同材料，**无 hover 抬升** |
| `surface-raised` | Dashboard / 设置等主内容卡 |
| `surface-inset` | 主卡内部的分区（概览条、特征、英雄格） |
| `surface-inset-interactive` | 可点子区（最近对局）；hover 弱浮，弱于 float |
| `surface-overlay` | Popover / 浮层面板 |

厚度：`inset` ≈ `chip` < `float` < `raised` < `overlay`。

### 分层原则

- **外壳 raised**：整块模块浮于页面底  
- **内层 inset**：模块内分区，不要再套通知那种强磨砂悬浮  
- **工具 float**：独立可点的小控件；文字操作用 `pill`，图标用 `icon`  
- **状态 chip**：展示信息，不是按钮，不要 hover 抬升  

## Border 约定

### 职责拆分

| 手段 | 职责 |
|------|------|
| `border` | **唯一**常驻描边（轮廓 / 分区 / hover 变实） |
| `box-shadow` | **只做海拔**（托起）；禁止再用 `0 0 0 1px` 当假描边 |
| `ring` / `focus-visible:ring-*` | **只做焦点态**（键盘 focus），不当卡片造型描边 |

### 各层描边

| 层 | 描边 | 阴影 |
|----|------|------|
| `float` | 1px，`border` 50% 透明 → hover 实色 | `--shadow-float` |
| `chip` | 1px，`border` 50% 透明 | `--shadow-float`（静息；无 hover 抬升） |
| `raised` | 1px，实色 `var(--border)` | `--shadow-raised` |
| `inset` | 1px，`border` 70% 透明 | 无 |
| `overlay` | 1px，`border` 40% 透明 | `--shadow-overlay` |

业务层不要再额外加 `border border-border` 叠在已有 surface 上。

### `border-2` 使用范围

**默认全站 1px。`border-2` 仅用于语义强调，不当日常造型。**

| 可用 | 示例 |
|------|------|
| 危险 / 未连接强提示 | 状态条 `border-destructive`（必要时升 2px） |
| 表单 invalid | 与 shadcn `aria-invalid` 一致 |
| 拖拽投放区 | 虚线 + 2px |
| 实时进行中态（若有） | 录制中 / 对局中 |

同一屏尽量只有一处粗边在「喊」。普通选中用底色 / primary 淡底，不要靠 `border-2`。

## Focus ring

可交互控件须支持键盘焦点环，与 shadcn 一致：

```text
focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]
```

- 已覆盖：Button、Input、Select、Checkbox、Switch、Sidebar 等  
- 浮钮：`FloatIconButton` 必须带上述 focus-visible ring  
- **不要**用常驻 ring / `0 0 0 1px` 代替卡片 border  

## 组件

| 场景 | 用法 |
|------|------|
| 图标浮钮（通知、刷新） | `FloatIconButton`（默认 `variant="icon"`） |
| 文字浮钮（加载更多） | `FloatIconButton variant="pill"` |
| Header 连接状态条 | `ConnectionStatus` → `surface-chip` |
| 主内容卡 | `Card` → 默认 `surface-raised` |
| 游戏统计子块 | `surface-inset` / `surface-inset-interactive` |
| 分享海报入口 | `FloatIconButton`（图标或 pill「导出海报」）；入口在 Dashboard，不进侧栏 |

## Dashboard 分享海报（长图）

气质与 Dashboard 一致：**独立海报稿**，不是整页截图。固定画布导出 PNG，便于微信 / QQ 分享。

### 原则

| 项 | 约定 |
|----|------|
| 形态 | **专用海报组件**（offscreen / 隐藏挂载），复用当前 Dashboard 数据，不截侧栏、Header、筛选与刷新控件 |
| 画布宽 | **720px**（导出像素宽；高度随内容，形成长图） |
| 缩放 | `devicePixelRatio` 建议 **2**，保证清晰；宽高比随内容自然拉长 |
| 主题 | 跟当前应用主题（含 `.theme-*` / dark）；导出瞬间冻结样式，避免闪切 |
| 字体 | 与产品一致：HarmonyOS Sans SC；导出前确保 Regular / Medium / Bold 已可用 |

### 内容结构（自上而下）

1. **品牌条**：`Nidalee` + 一句极短副标（如「战绩海报」）；不抢召唤师名  
2. **召唤师头卡**：头像、昵称 `#tag`、等级、单双 / 灵活段位与 LP、今日对局与胜率（与 CompactProfile 信息同级，布局可略收成竖向更适合长图）  
3. **概览 KPI**：总胜率（主 KPI `2xl`）+ 场均 KDA / 场次等次级 `lg`  
4. **召唤师特征**（有数据才渲染；无则整块省略）  
5. **常用英雄**（按场次，有限数量，如 Top 5）  
6. **最近对局**：与 Dashboard 同款信息密度；右下角 **自研评级衬底字**（S+～D）；不含「点击查看详情」等交互文案  
7. **页脚**：导出时间 +「Nidalee · 自研评级非官方」一行 `xs` muted  

**明确不进海报**：AI 解读面板、模式/场数 Select、「记住选择」、刷新按钮、连接态 Badge、侧栏与右侧工具条、空态大图标区。

### 表面与装饰

- 海报外框：整张画布用 `surface-raised` 材料或等价（底 + 1px border + raised 影）；内部区块用 `surface-inset`  
- 对局行：视觉对齐 `surface-inset`，**导出态无 hover**  
- 评级衬底字：大号倾斜半透明字母；色阶与 Dashboard 一致（S+ 橙金 / S 紫 / … / C 石灰）；S+ 字号略大于 S，其余同级  
- 胜负色：固定 emerald / rose（不跟主题 hue）  
- 禁止：霓虹 glow、多图层假描边、把整页 Dashboard 当背景糊进去  

### 导出行为

| 动作 | 约定 |
|------|------|
| 保存 | 系统「另存为」PNG（`tauri-plugin-dialog` + 写文件） |
| 剪贴板 | 同时写入图片到系统剪贴板，便于直接粘贴 |
| 成功反馈 | 轻量 toast（如「已保存并复制」）；取消保存对话框不报错打扰 |
| 失败 | 可读错误（字体未就绪 / 图标跨域失败 / 写盘失败） |

实现要点（产品约定，非强制 API 名）：前端用 DOM→PNG（如 `html-to-image`）渲染专用节点；跨域头像/英雄图需可绘制（CORS 或已缓存 blob）。

### 交互入口

- 仅在 Dashboard **已连接且有可展示战绩数据**时启用  
- 导出中禁用二次点击；可短暂展示「生成中…」  
- 不提供「截当前页面」备用路径，避免两套观感
