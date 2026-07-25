# v0.7.8 发布验证记录

## 候选产物

- 候选 DMG：`/Users/xiaolongdeng/Documents/self/ideas/markhola/dist/MarkHola-0.7.8.dmg`
- DMG SHA-256：`22eb40d1e9f9626769a34303aa13166485e03b52fcebe21584fe3e904a9d65cd`
- 镜像格式：`UDZO`（只读 zlib 压缩）
- 隔离挂载路径：`/tmp/markhola-0.7.8-release-validation.elOfop/mount`
- 验证应用副本：`/tmp/markhola-0.7.8-release-validation.elOfop/MarkHola.app`
- 验证应用版本：`0.7.8`
- 包内与复制后可执行文件 SHA-256：`ca0d4dba6733ca449b8b2ec63f9bb816072b3482b36bbc3c16a554167f380f90`

## 运行目标证据

- 验证进程 PID：`21646`
- 实际进程路径：`/tmp/markhola-0.7.8-release-validation.elOfop/MarkHola.app/Contents/MacOS/MarkHola`
- 启动日志：

```text
ts=2026-07-25T22:51:13.149 pid=21646 tid=main stage=app.start event_id=system msg="app run started" version=0.7.8 platform=macos/aarch64
ts=2026-07-25T22:51:13.608 pid=21646 tid=main stage=ipc.received event_id=system msg="ipc payload received" payload={"kind":"shell-ready"}
ts=2026-07-25T22:51:13.609 pid=21646 tid=main stage=send_event.end event_id=system msg="ShellReady" source=ipc result=ok
```

验证期间未运行 `/Applications/MarkHola.app` 或其他 MarkHola 进程。

## 自动验证

- `cargo test`：64 个通过，0 个失败，1 个因无头 WKWebView 环境忽略
- Release 构建：通过
- HTML 导出冒烟测试：通过
- PDF、Mermaid PDF 和打印准备测试：遇到既有的沙箱 WKWebView JavaScript 返回类型限制，发布回归脚本按既定规则记录为警告
- 应用签名完整性：`codesign --verify --deep --strict` 通过

## 真实界面验证

1. 从候选应用的原生文件选择器打开隔离 Markdown 文件。
2. 使用 `Command + /` 切换到 Writable。
3. 修改正文并确认窗口和 Tab 显示未保存状态。
4. 使用 `Command + S` 保存，确认未保存状态消失，并在磁盘文件中找到新内容。
5. 切回 Readonly，确认保存后的标题和正文正确渲染。
6. 确认原生底栏按顺序只读显示路径、Words、Lines、Readonly/Writable 和状态内容，且没有 Mode、Status 前缀、字号或 Outline 按钮。
7. 通过 `View > Outline` 打开右侧标题面板，并通过面板关闭按钮关闭。
8. 通过 `View > Size` 调整文档字号，确认状态更新，最后重置为 `100%`。

## 结论

候选 DMG 中的应用已通过发布前验证，可以作为 GitHub `v0.7.8` Release 的上传产物。

发布地址：`https://github.com/huimang/markhola/releases/tag/v0.7.8`
