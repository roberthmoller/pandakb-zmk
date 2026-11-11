import Foundation

enum PandaKBHID {
    static let usagePage: UInt16 = 0xFF60
    static let usage: UInt16 = 0x61
    /// ZMK / nice!nano USB identifiers
    static let vendorId: Int = 0x1D50
    static let productId: Int = 0x615E
    static let reportSize = 32

    static let cmdLayerTo: UInt8 = 0x01
    static let cmdSetBadge: UInt8 = 0x02

    static let layerBase: UInt8 = 0
    static let layerWow: UInt8 = 4
    static let layerLol: UInt8 = 5

    static let badgeIdle: UInt8 = 0
    static let badgeWow: UInt8 = 1
    static let badgeLol: UInt8 = 2

    static let statusFlagValid: UInt8 = 0x01
}

enum GameBadge: UInt8, CaseIterable, Identifiable {
    case idle = 0
    case wow = 1
    case lol = 2

    var id: UInt8 { rawValue }

    var title: String {
        switch self {
        case .idle: return "Buddy"
        case .wow: return "WoW"
        case .lol: return "LoL"
        }
    }
}

struct GameStatus: Equatable {
    var badge: GameBadge
    var kills: UInt8 = 0
    var deaths: UInt8 = 0
    var assists: UInt8 = 0
    var cs: UInt16 = 0
    var level: UInt8 = 0
    var statsValid: Bool = false

    static func badgeOnly(_ badge: GameBadge) -> GameStatus {
        GameStatus(badge: badge)
    }

    var summary: String {
        guard statsValid else { return badge.title }
        switch badge {
        case .lol:
            return "\(kills)/\(deaths)/\(assists) CS\(cs)"
        case .wow:
            return "Lv \(level)"
        case .idle:
            return badge.title
        }
    }
}

enum StickyLayer: UInt8, CaseIterable, Identifiable {
    case base = 0
    case wow = 4
    case lol = 5

    var id: UInt8 { rawValue }

    var title: String {
        switch self {
        case .base: return "Base"
        case .wow: return "WoW"
        case .lol: return "LoL"
        }
    }
}
