import AppKit
import ApplicationServices
import CoreGraphics
import Foundation

struct ProbePayload: Encodable {
    let mode: String
    let pid: Int32
    let windowOwnerPID: Int32?
    let axTrusted: Bool
    let visibleWindow: Bool
    let windowOwner: String?
    let windowNames: [String]
    let error: String?
}

func writeJSON(_ payload: ProbePayload) throws {
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
    let data = try encoder.encode(payload)
    FileHandle.standardOutput.write(data)
    FileHandle.standardOutput.write(Data("\n".utf8))
}

func windows(for pid: pid_t) -> (visible: Bool, ownerPID: Int32?, owner: String?, titles: [String]) {
    guard let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID)
            as? [[String: Any]]
    else {
        return (false, nil, nil, [])
    }

    let filtered = list.filter { entry in
        guard let ownerPid = entry[kCGWindowOwnerPID as String] as? Int else {
            return false
        }
        return ownerPid == Int(pid)
    }

    let titles = filtered.compactMap { entry in
        entry[kCGWindowName as String] as? String
    }.filter { !$0.isEmpty }

    let owner = filtered.compactMap { entry in
        entry[kCGWindowOwnerName as String] as? String
    }.first

    let ownerPID = filtered.compactMap { entry in
        (entry[kCGWindowOwnerPID as String] as? Int).map(Int32.init)
    }.first

    let visible = filtered.contains { entry in
        let layer = entry[kCGWindowLayer as String] as? Int ?? Int.max
        let alpha = entry[kCGWindowAlpha as String] as? Double ?? 0
        return layer == 0 && alpha > 0
    }

    return (visible, ownerPID, owner, titles)
}

func runLoop(seconds: TimeInterval) {
    let deadline = Date().addingTimeInterval(seconds)
    while Date() < deadline {
        RunLoop.current.run(mode: .default, before: deadline)
    }
}

func runProbeWindow() throws {
    let app = NSApplication.shared
    app.setActivationPolicy(.regular)

    let window = NSWindow(
        contentRect: NSRect(x: 80, y: 80, width: 480, height: 240),
        styleMask: [.titled, .closable, .miniaturizable, .resizable],
        backing: .buffered,
        defer: false
    )
    window.title = "MarkHola Intel G4 Probe"
    window.center()
    window.makeKeyAndOrderFront(nil)
    app.activate(ignoringOtherApps: true)

    runLoop(seconds: 1.5)

    let visible = windows(for: getpid())
    try writeJSON(
        ProbePayload(
            mode: "create-probe-window",
            pid: getpid(),
            windowOwnerPID: visible.ownerPID,
            axTrusted: AXIsProcessTrusted(),
            visibleWindow: visible.visible,
            windowOwner: visible.owner,
            windowNames: visible.titles,
            error: nil
        )
    )
}

func inspectExistingPID(_ pid: pid_t) throws {
    let visible = windows(for: pid)
    try writeJSON(
        ProbePayload(
            mode: "inspect-existing-pid",
            pid: Int32(pid),
            windowOwnerPID: visible.ownerPID,
            axTrusted: AXIsProcessTrusted(),
            visibleWindow: visible.visible,
            windowOwner: visible.owner,
            windowNames: visible.titles,
            error: nil
        )
    )
}

func fail(_ message: String) -> Never {
    do {
        try writeJSON(
            ProbePayload(
                mode: "error",
                pid: getpid(),
                windowOwnerPID: nil,
                axTrusted: AXIsProcessTrusted(),
                visibleWindow: false,
                windowOwner: nil,
                windowNames: [],
                error: message
            )
        )
    } catch {
        fputs("{\"mode\":\"error\",\"error\":\"\(message)\"}\n", stderr)
    }
    exit(1)
}

let arguments = CommandLine.arguments
do {
    if arguments.count == 3 && arguments[1] == "--inspect-pid" {
        guard let pid = Int32(arguments[2]) else {
            fail("invalid pid")
        }
        try inspectExistingPID(pid)
    } else if arguments.count == 1 {
        try runProbeWindow()
    } else {
        fail("unsupported arguments")
    }
} catch {
    fail(String(describing: error))
}
