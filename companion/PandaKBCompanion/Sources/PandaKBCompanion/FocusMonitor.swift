import AppKit
import Foundation

struct AppRule: Codable, Equatable, Identifiable {
    var id: String { bundleId }
    var bundleId: String
    var displayName: String
    var badge: UInt8
    var layer: UInt8
    var autoLayer: Bool
}

final class FocusMonitor: ObservableObject {
    @Published private(set) var frontmostBundleId: String?
    @Published private(set) var frontmostName: String?

    private var observer: NSObjectProtocol?

    func start() {
        observer = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] note in
            let app = note.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
            self?.update(app)
        }
        update(NSWorkspace.shared.frontmostApplication)
    }

    func stop() {
        if let observer {
            NSWorkspace.shared.notificationCenter.removeObserver(observer)
        }
        observer = nil
    }

    private func update(_ app: NSRunningApplication?) {
        frontmostBundleId = app?.bundleIdentifier
        frontmostName = app?.localizedName
    }
}

enum DefaultRules {
    static let all: [AppRule] = [
        AppRule(
            bundleId: "com.blizzard.worldofwarcraft",
            displayName: "World of Warcraft",
            badge: GameBadge.wow.rawValue,
            layer: StickyLayer.wow.rawValue,
            autoLayer: true
        ),
        AppRule(
            bundleId: "com.blizzard.worldofwarcraft.classic",
            displayName: "WoW Classic",
            badge: GameBadge.wow.rawValue,
            layer: StickyLayer.wow.rawValue,
            autoLayer: true
        ),
        AppRule(
            bundleId: "com.riotgames.LeagueofLegends.GameClient",
            displayName: "League of Legends",
            badge: GameBadge.lol.rawValue,
            layer: StickyLayer.lol.rawValue,
            autoLayer: true
        ),
        AppRule(
            bundleId: "com.riotgames.LeagueofLegends.LeagueClient",
            displayName: "League Client",
            badge: GameBadge.lol.rawValue,
            layer: StickyLayer.lol.rawValue,
            autoLayer: false
        ),
    ]
}
