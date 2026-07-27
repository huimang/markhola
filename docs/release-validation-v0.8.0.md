# MarkHola v0.8.0 发布验证记录

## 候选制品

- DMG：`/Users/xiaolongdeng/Documents/self/ideas/markhola/dist/MarkHola-0.8.0.dmg`
- SHA-256：`b65a8149e2aa735283741fa33249c8ca1b6788e4ddcd794fcf859e91d34a0840`
- 格式：UDZO
- 制品实现提交：`2e9b2d2812d06e08794c141fae0818c2fc0df1df`

## 精确制品确认

- 挂载位置：`/Volumes/MarkHola 1`
- 隔离验证 App：`/tmp/markhola-release-validation-0.8.0-final.YCuTow/MarkHola.app`
- 验证进程：PID `37041`
- 实际进程路径：
  `/tmp/markhola-release-validation-0.8.0-final.YCuTow/MarkHola.app/Contents/MacOS/MarkHola`

启动日志证据：

```text
ts=2026-07-27T14:12:03.230 pid=37041 tid=main stage=app.start event_id=system msg="app run started" version=0.8.0 platform=macos/aarch64
ts=2026-07-27T14:12:03.474 pid=37041 tid=main stage=event_loop.init event_id=system msg="event loop init"
ts=2026-07-27T14:12:11.061 pid=37041 tid=main stage=send_event.end event_id=open-1 msg="OpenPath" source=tao-opened origin=tao-opened path=/Users/xiaolongdeng/Documents/self/ideas/markhola/examples/native-tabs.md result=ok
ts=2026-07-27T14:12:11.063 pid=37041 tid=main stage=open_document.end event_id=open-1 msg="open_document success" path=/Users/xiaolongdeng/Documents/self/ideas/markhola/examples/native-tabs.md
```

## 验证结果

- 自动回归：85 项通过，1 项因无头 WebView 环境按既有规则忽略。
- PDF、Mermaid PDF、HTML、打印准备和 Mermaid 打印页数 smoke test 均通过。
- 已确认候选 App 打开 `examples/native-tabs.md`。
- 已确认第二、第三个文档加入同一 AppKit 原生 Tab 组。
- 已通过无障碍树和真实截图确认三个原生 Tab 右侧依次显示 `⌘1`、`⌘2`、`⌘3`。
- 已确认 Default 标题栏使用低透明度 Gray Tint 的系统 `titlebar` 毛玻璃材质。
- 已确认选中原生 Tab 使用 Green Tint `#EAF9F5`，与灰色标题栏保持区分。
- 已确认 Footer 仅显示左侧路径和右侧 Words、Lines，不展示 Readonly、Writable 或 status。
- 已确认正文继续使用更淡的 Green Subtle，阅读区最大宽度为 960 px。
- 已确认 `examples/basic.md` 的表格使用直角边框。

本轮仅验证最终候选包中受近期局部视觉变更影响的区域。按已确认的局部验证要求，打开、编辑、保存等未受影响流程复用同一版本前一轮真机验证结果，本轮不重复执行。

## 发布结论

候选 DMG 的运行路径、版本日志与目标视觉变更一致，可以作为 v0.8.0 发布制品。
