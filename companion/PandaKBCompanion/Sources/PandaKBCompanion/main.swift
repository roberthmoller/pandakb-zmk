import AppKit

let app = NSApplication.shared
app.setActivationPolicy(.accessory)

let model = CompanionModel()
model.start()

let delegate = AppDelegate(model: model)
app.delegate = delegate
app.run()

final class AppDelegate: NSObject, NSApplicationDelegate {
    private let model: CompanionModel
    private var statusItem: NSStatusItem?

    init(model: CompanionModel) {
        self.model = model
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = item.button {
            button.title = "KB"
            button.toolTip = "PandaKB Companion"
        }
        statusItem = item
        rebuildMenu()

        Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            self?.refreshTitle()
        }
    }

    private func refreshTitle() {
        guard let button = statusItem?.button else { return }
        let badge = model.badge == .idle ? "·" : model.badge.title
        let link = model.hid.connected ? "" : "!"
        button.title = "KB \(badge)\(link)"
        rebuildMenu()
    }

    private func rebuildMenu() {
        let menu = NSMenu()

        let status = NSMenuItem(
            title: model.hid.connected ? "Keyboard: connected" : "Keyboard: not connected",
            action: nil,
            keyEquivalent: ""
        )
        status.isEnabled = false
        menu.addItem(status)

        if let err = model.hid.lastError {
            let errorItem = NSMenuItem(title: err, action: nil, keyEquivalent: "")
            errorItem.isEnabled = false
            menu.addItem(errorItem)
        }

        let focusName = model.focus.frontmostName ?? "—"
        let focusItem = NSMenuItem(title: "Focus: \(focusName)", action: nil, keyEquivalent: "")
        focusItem.isEnabled = false
        menu.addItem(focusItem)

        menu.addItem(.separator())

        let auto = NSMenuItem(
            title: "Auto switch from focus",
            action: #selector(toggleAuto),
            keyEquivalent: "a"
        )
        auto.state = model.autoEnabled ? .on : .off
        auto.target = self
        menu.addItem(auto)

        menu.addItem(.separator())

        let badgeMenu = NSMenuItem(title: "Badge", action: nil, keyEquivalent: "")
        let badgeSub = NSMenu()
        for badge in GameBadge.allCases {
            let item = NSMenuItem(
                title: badge.title,
                action: #selector(selectBadge(_:)),
                keyEquivalent: ""
            )
            item.representedObject = badge.rawValue
            item.state = model.badge == badge ? .on : .off
            item.target = self
            badgeSub.addItem(item)
        }
        badgeMenu.submenu = badgeSub
        menu.addItem(badgeMenu)

        let layerMenu = NSMenuItem(title: "Layer", action: nil, keyEquivalent: "")
        let layerSub = NSMenu()
        for layer in StickyLayer.allCases {
            let item = NSMenuItem(
                title: layer.title,
                action: #selector(selectLayer(_:)),
                keyEquivalent: ""
            )
            item.representedObject = layer.rawValue
            item.state = model.layer == layer ? .on : .off
            item.target = self
            layerSub.addItem(item)
        }
        layerMenu.submenu = layerSub
        menu.addItem(layerMenu)

        menu.addItem(.separator())
        let quit = NSMenuItem(title: "Quit", action: #selector(quitApp), keyEquivalent: "q")
        quit.target = self
        menu.addItem(quit)

        statusItem?.menu = menu
    }

    @objc private func toggleAuto() {
        model.autoEnabled.toggle()
        rebuildMenu()
    }

    @objc private func selectBadge(_ sender: NSMenuItem) {
        guard let raw = sender.representedObject as? UInt8,
              let badge = GameBadge(rawValue: raw) else { return }
        model.setBadge(badge)
        rebuildMenu()
    }

    @objc private func selectLayer(_ sender: NSMenuItem) {
        guard let raw = sender.representedObject as? UInt8,
              let layer = StickyLayer(rawValue: raw) else { return }
        model.setLayer(layer)
        rebuildMenu()
    }

    @objc private func quitApp() {
        model.stop()
        NSApp.terminate(nil)
    }
}
