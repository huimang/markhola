# v0.7.9 发布验证记录

## 候选产物

- 候选 DMG：`/Users/xiaolongdeng/Documents/self/ideas/markhola/dist/MarkHola-0.7.9.dmg`
- DMG SHA-256：`ae09e25685e8b8a7238352c679155bfb75c39f8fb5e9f959de33a83d072b1ade`
- 镜像格式：`UDZO`
- 挂载卷：`/Volumes/MarkHola`
- 验证应用副本：`/tmp/markhola-release-validation-0.7.9/MarkHola.app`
- 验证应用版本：`0.7.9`
- 包内与复制后可执行文件 SHA-256：`f0b98ca32d228e2b42ca34ec2c0eb483f3c6ed4f4857f01ab221466e7ed579cb`

## 运行目标证据

- 验证进程 PID：`36580`
- 实际进程路径：`/tmp/markhola-release-validation-0.7.9/MarkHola.app/Contents/MacOS/MarkHola`
- 启动日志：

```text
ts=2026-07-26T11:27:01.753 pid=36580 tid=main stage=app.start event_id=system msg="app run started" version=0.7.9 platform=macos/aarch64
ts=2026-07-26T11:27:02.028 pid=36580 tid=main stage=event_loop.init event_id=system msg="building event loop"
ts=2026-07-26T11:27:02.298 pid=36580 tid=main stage=ipc.received event_id=system msg="ipc payload received" payload={"kind":"shell-ready"}
ts=2026-07-26T11:27:02.298 pid=36580 tid=main stage=send_event.end event_id=system msg="ShellReady" source=ipc result=ok
```

验证期间未运行 `/Applications/MarkHola.app` 或其他冲突的 MarkHola 进程。

## 自动验证

- `./scripts/release_regression.sh --with-package`：通过
- `cargo test`：77 个通过，0 个失败，1 个忽略
- PDF 导出冒烟测试：通过
- Mermaid PDF 导出冒烟测试：通过
- HTML 导出冒烟测试：通过
- 打印准备冒烟测试：通过
- DMG 打包：通过，产物为 `dist/MarkHola-0.7.9.dmg`

## 真实界面验证

1. 从候选 DMG 复制出的应用副本启动，并确认实际运行进程路径匹配验证副本。
2. 通过原生文件选择器打开隔离验证文件 `/tmp/markhola-release-validation-0.7.9/angle-bracket-links-validation.md`。
3. 验证只读渲染中的尖括号链接生效：
   - `<README.md>`
   - `<docs/v0.7.9-angle-bracket-links-design.md>`
   - `<https://example.com>`
4. 验证反引号内和代码块内的 `<README.md>` / `<https://example.com>` 保持字面量，不被转成链接。
5. 验证 HTML 类标签 `<b>bold-like tag</b>` 不被识别为链接。
6. 使用 `Command + /` 切换到 Writable，追加如下内容并保存：

```markdown
## Release Sandbox Save Check

<saved-check.md>
```

7. 使用 `Command + S` 保存后，确认状态显示 `Saved.`，并在磁盘文件中看到新增内容。
8. 切回 Readonly 后，确认新增段落正确渲染，`saved-check.md` 以链接形式显示。
9. 打开 `Help > Documentation`，确认英文帮助文档显示 `v0.7.9`。
10. 切换界面语言到 `简体中文`，确认原生菜单切换为 `文件 / 编辑 / 标签页 / 视图 / 帮助`。
11. 打开 `帮助 > 文档`，确认中文帮助文档显示 `v0.7.9`。

## 结论

候选 DMG 中的应用已通过发布前验证，可以作为 GitHub `v0.7.9` Release 的上传产物。
