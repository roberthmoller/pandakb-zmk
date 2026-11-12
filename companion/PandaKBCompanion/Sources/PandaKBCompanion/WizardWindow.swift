import AppKit
import WebKit

/// Hosts ZmkWiz in a WKWebView window opened from the menu bar.
final class WizardWindowController: NSObject, NSWindowDelegate {
    static let shared = WizardWindowController()

    private var window: NSWindow?
    private var webView: WKWebView?
    private var opening = false

    func open() {
        if opening { return }
        opening = true

        WizardServer.shared.ensureRunning { [weak self] result in
            guard let self else { return }
            self.opening = false
            switch result {
            case .success(let url):
                self.show(url: url)
            case .failure(let error):
                self.presentError(error)
            }
        }
    }

    func openInBrowser() {
        WizardServer.shared.ensureRunning { result in
            switch result {
            case .success(let url):
                NSWorkspace.shared.open(url)
            case .failure(let error):
                self.presentError(error)
            }
        }
    }

    func close() {
        window?.close()
    }

    // MARK: - Window

    private func show(url: URL) {
        if window == nil {
            let config = WKWebViewConfiguration()
            config.preferences.setValue(true, forKey: "developerExtrasEnabled")

            let webView = WKWebView(frame: NSRect(x: 0, y: 0, width: 1280, height: 860), configuration: config)
            self.webView = webView

            let window = NSWindow(
                contentRect: NSRect(x: 0, y: 0, width: 1280, height: 860),
                styleMask: [.titled, .closable, .miniaturizable, .resizable],
                backing: .buffered,
                defer: false
            )
            window.title = "PandaKB Keymap Wizard"
            window.contentView = webView
            window.delegate = self
            window.center()
            window.setFrameAutosaveName("PandaKBWizard")
            self.window = window
        }

        webView?.load(URLRequest(url: url))
        window?.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    private func presentError(_ error: Error) {
        let alert = NSAlert()
        alert.messageText = "Could not open Keymap Wizard"
        alert.informativeText = error.localizedDescription
        alert.alertStyle = .warning
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }

    func windowWillClose(_ notification: Notification) {
        // Keep the local server warm for quick re-open; CompanionModel.stop() kills it on quit.
    }
}
