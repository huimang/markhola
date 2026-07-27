# MarkHola v0.8.0 发布验证记录

## 候选制品

- DMG：`/Users/xiaolongdeng/Documents/self/ideas/markhola/dist/MarkHola-0.8.0.dmg`
- SHA-256：`b1cfd26d385ef84ffaa0dc7dc533af9a13b245a0ac70efc7d2dde4a5118fcce8`
- 格式：UDZO
- 对应提交：`010ec90554bee93c42cd3434f719912305d17776`

## 精确制品确认

- 挂载位置：`/Volumes/MarkHola 1`
- 隔离验证 App：`/tmp/markhola-release-validation-0.8.0.O33GVy/MarkHola.app`
- 验证进程：PID `8579`
- 实际进程路径：
  `/tmp/markhola-release-validation-0.8.0.O33GVy/MarkHola.app/Contents/MacOS/MarkHola`

启动日志证据：

```text
ts=2026-07-27T13:51:05.548 pid=8579 tid=main stage=app.start event_id=system msg="app run started" version=0.8.0 platform=macos/aarch64
ts=2026-07-27T13:51:05.788 pid=8579 tid=main stage=event_loop.init event_id=system msg="event loop init"
ts=2026-07-27T13:51:40.245 pid=8579 tid=main stage=send_event.end event_id=open-1 msg="OpenPath" source=tao-opened origin=tao-opened path=/Users/xiaolongdeng/Documents/self/ideas/markhola/examples/native-tabs.md result=ok
ts=2026-07-27T13:51:40.247 pid=8579 tid=main stage=open_document.end event_id=open-1 msg="open_document success" path=/Users/xiaolongdeng/Documents/self/ideas/markhola/examples/native-tabs.md
```

## 验证结果

- 自动回归：82 项通过，1 项因无头 WebView 环境按既有规则忽略。
- PDF、Mermaid PDF、HTML、打印准备和 Mermaid 打印页数 smoke test 均通过。
- 已确认候选 App 打开 `examples/native-tabs.md`。
- 已确认第二个文档加入同一 AppKit 原生 Tab 组。
- 已确认 Default 标题栏使用 MarkHola Green `#17B890`。
- 已确认原生 Tab 区域（包括选中 Tab）使用 Green Tint `#EAF9F5`，未跟随标题栏变为深绿。
- 已确认正文继续使用更淡的 Green Subtle，Footer 继续使用 Gray 色系。

本轮变更仅涉及标题栏与原生 Tab 的配色分离。按已确认的局部验证要求，打开、编辑、保存等未受影响流程复用同一版本前一轮真机验证结果，本轮不重复执行。

## 发布结论

候选 DMG 的运行路径、版本日志与目标视觉变更一致，可以作为 v0.8.0 发布制品。
