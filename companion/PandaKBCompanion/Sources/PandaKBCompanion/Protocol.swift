import Foundation

enum PandaKBHID {
    static let usagePage: UInt16 = 0xFF60
    static let usage: UInt16 = 0x61
    /// ZMK / nice!nano USB identifiers
    static let vendorId: Int = 0x1D50
    static let productId: Int = 0x615E
    static let reportSize = 32

    static let cmdSetMedia: UInt8 = 0x03
}

enum MediaPlayback: UInt8, CaseIterable, Identifiable {
    case stopped = 0
    case playing = 1
    case paused = 2

    var id: UInt8 { rawValue }

    var title: String {
        switch self {
        case .stopped: return "Stopped"
        case .playing: return "Playing"
        case .paused: return "Paused"
        }
    }
}

struct MediaStatus: Equatable {
    var playback: MediaPlayback
    var volume: UInt8 // 0–100

    var summary: String {
        "\(playback.title) \(volume)%"
    }
}
