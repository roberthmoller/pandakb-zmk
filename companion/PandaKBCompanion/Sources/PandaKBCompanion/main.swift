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
        button.title = "KB \(model.status.summary)"
        rebuildMenu()
    }

    private func rebuildMenu() {
        let menu = NSMenu()

        let media = NSMenuItem(title: "Media: \(model.status.summary)", action: nil, keyEquivalent: "")
        media.isEnabled = false
        menu.addItem(media)

        // HID keyboard link UI hidden while CompanionModel.hidEnabled == false.
        if CompanionModel.hidEnabled {
            menu.addItem(.separator())
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
        } else {
            let hidOff = NSMenuItem(
                title: "Keyboard HID: disabled",
                action: nil,
                keyEquivalent: ""
            )
            hidOff.isEnabled = false
            menu.addItem(hidOff)
        }

        menu.addItem(.separator())

        let wizard = NSMenuItem(
            title: "Open Keymap Wizard",
            action: #selector(openWizard),
            keyEquivalent: "w"
        )
        wizard.target = self
        menu.addItem(wizard)

        let wizardBrowser = NSMenuItem(
            title: "Open Wizard in Browser",
            action: #selector(openWizardInBrowser),
            keyEquivalent: "W"
        )
        wizardBrowser.target = self
        menu.addItem(wizardBrowser)

        menu.addItem(.separator())
        let quit = NSMenuItem(title: "Quit", action: #selector(quitApp), keyEquivalent: "q")
        quit.target = self
        menu.addItem(quit)

        statusItem?.menu = menu
    }

    @objc private func openWizard() {
        WizardWindowController.shared.open()
    }

    @objc private func openWizardInBrowser() {
        WizardWindowController.shared.openInBrowser()
    }

    @objc private func quitApp() {
        WizardWindowController.shared.close()
        WizardServer.shared.stop()
        model.stop()
        NSApp.terminate(nil)
    }
}
