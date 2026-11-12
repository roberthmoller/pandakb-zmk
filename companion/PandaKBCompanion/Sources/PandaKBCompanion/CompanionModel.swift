import Combine
import Foundation

final class CompanionModel: ObservableObject {
    @Published var autoEnabled = true
    @Published var badge: GameBadge = .idle
    @Published var status = GameStatus.badgeOnly(.idle)
    @Published var layer: StickyLayer = .base
    @Published var rules: [AppRule] = DefaultRules.all
    @Published var lolLiveLine: String = "LoL live: —"

    let hid = HIDClient()
    let focus = FocusMonitor()
    let lol = LolLiveClient()

    private var bag = Set<AnyCancellable>()
    private var focusBadge: GameBadge = .idle

    func start() {
        hid.start()
        focus.start()
        lol.start()

        focus.$frontmostBundleId
            .compactMap { $0 }
            .removeDuplicates()
            .sink { [weak self] bundleId in
                self?.applyFocus(bundleId: bundleId)
            }
            .store(in: &bag)

        lol.$status
            .combineLatest(lol.$inGame)
            .receive(on: DispatchQueue.main)
            .sink { [weak self] live, inGame in
                self?.applyLolLive(live: live, inGame: inGame)
            }
            .store(in: &bag)
    }

    func stop() {
        lol.stop()
        focus.stop()
        hid.stop()
        bag.removeAll()
    }

    func applyFocus(bundleId: String) {
        guard autoEnabled else { return }

        if let rule = rules.first(where: { $0.bundleId == bundleId }) {
            let nextBadge = GameBadge(rawValue: rule.badge) ?? .idle
            focusBadge = nextBadge
            if rule.autoLayer, let nextLayer = StickyLayer(rawValue: rule.layer) {
                setLayer(nextLayer)
            }
            // Prefer live LoL stats when available and LoL is focused.
            if nextBadge == .lol, let live = lol.status, lol.inGame {
                pushStatus(live)
            } else {
                pushStatus(.badgeOnly(nextBadge))
            }
        } else {
            focusBadge = .idle
            pushStatus(.badgeOnly(.idle))
            setLayer(.base)
        }
    }

    private func applyLolLive(live: GameStatus?, inGame: Bool) {
        if inGame, let live {
            lolLiveLine = "LoL live: \(live.summary)"
        } else {
            lolLiveLine = "LoL live: not in game"
        }

        guard autoEnabled, focusBadge == .lol else { return }
        if inGame, let live {
            pushStatus(live)
        } else if badge == .lol {
            pushStatus(.badgeOnly(.lol))
        }
    }

    func setBadge(_ badge: GameBadge) {
        focusBadge = badge
        if badge == .lol, let live = lol.status, lol.inGame {
            pushStatus(live)
        } else {
            pushStatus(.badgeOnly(badge))
        }
    }

    func setLayer(_ layer: StickyLayer) {
        guard layer != self.layer else { return }
        self.layer = layer
        hid.setLayer(layer)
    }

    private func pushStatus(_ next: GameStatus) {
        // Avoid HID spam when LoL stats are unchanged.
        if next == status, badge == next.badge { return }
        status = next
        badge = next.badge
        hid.setStatus(next)
    }
}
