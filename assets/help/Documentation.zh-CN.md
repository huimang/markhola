# MarkHola 文档

Current version: `v0.9.2`

## MarkHola 是什么

MarkHola 是一款基于 AI 构建的免费 Markdown 阅读与编辑器。

## 主要功能

- 新建、打开、阅读、编辑和保存本地 Markdown 文档
- 使用 `Command + /` 在只读与编辑模式之间切换
- 使用 macOS 原生标签页打开多个文档
- 在 English 和简体中文界面之间切换
- 跨应用重启记住语言、主题和文档字号
- 在只读模式查找文本，在编辑模式查找或替换文本
- 在只读模式打开右侧文档大纲
- 渲染 Mermaid（包括字面 `\n` 换行）、公式、表格、链接、图片和高亮代码
- 将当前文档导出为 PNG、PDF 或 HTML，成功后显示导出路径与“打开”操作，并支持打印
- 通过一次性的应用二进制命令，将规范化绝对路径的 Markdown 源文件导出为 PNG、PDF
  或 HTML，而不打开常规应用工作区
- 点击主窗口左上角红色关闭按钮退出 MarkHola，标签页关闭命令仍只关闭当前文档
- 为 macOS 14.0 或更高版本分别提供 Apple Silicon 与 Intel 架构专用应用

## 菜单

### 文件

- 新建
- 打开
- 保存
- 另存为
- 打印
- 导出 > PNG / PDF / HTML
- 关闭
- 退出

### 编辑

- 切换模式
- 撤销 / 重做
- 查找
- 剪切 / 复制 / 粘贴
- 全选

### 标签页

- 下一个标签页
- 上一个标签页
- 关闭标签页
- 关闭其他标签页
- 关闭所有标签页

### 视图

- 主题 > 跟随系统 / 浅色 / 深色
- 语言 > English / 简体中文
- 字号 > 放大 / 缩小 / 重置
- 大纲
- 切换全屏

### 帮助

- 文档

## 离线 CLI 导出

每个进程只执行一个命令，请使用 `MarkHola.app` 内的可执行文件：

```bash
MARKHOLA_BIN="/absolute/path/to/MarkHola.app/Contents/MacOS/MarkHola"
"$MARKHOLA_BIN" export-png \
  --source=/absolute/input.md \
  --target=/absolute/output.png \
  --theme=light \
  --json
```

公开命令包括 `export-png`、`export-pdf`、`export-html`、`version` 和 `help`。`--source`
与 `--target` 必须使用绝对、规范化且已 canonicalize 的路径。默认主题为 `light`；
`--theme=dark` 选择深色主题。除非明确提供 `--overwrite`，否则不会覆盖已有输出。
JSON 响应使用 `schema_version: 1`。

稳定退出码如下：

- `0`：成功
- `2`：命令、参数或 schema 无效
- `3`：源文件无效或不可读
- `4`：目标路径不安全、不可用或存在冲突
- `5`：渲染或导出失败
- `6`：资源限制或超时
- `7`：内部错误或隐藏 runtime 失败

## 说明

- 切换界面语言会立即生效，不会重新加载当前文档。
- Markdown 正文、文件名和导出内容不会被翻译。
- 操作系统对话框继续使用 macOS 提供的语言。
- 打开文档并处于只读模式时，大纲菜单可用。
- 使用 `Command + 1` 到 `Command + 9` 按可见顺序选择原生标签页。
- 跟随系统会在 macOS 外观变化时自动切换浅色或深色主题。
- 渲染后的 Markdown 链接不显示下划线。
- 浅色与深色主题使用各自的护眼代码色板。
- 浅色空白状态不显示状态提示；PNG、PDF 或 HTML 导出成功后显示输出路径与“打开”操作。
- MarkHola 为 macOS 14.0 或更高版本分别提供带明确标记的 Apple Silicon 与 Intel 下载。
