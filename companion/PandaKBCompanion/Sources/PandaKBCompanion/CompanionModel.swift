import Combine
import Foundation

final class CompanionModel: ObservableObject {
    @Published var autoEnabled = true
    @Published var badge: GameBadge = .idle
    @Published var layer: StickyLayer = .base
    @Published var rules: [AppRule] = DefaultRules.all

    let hid = HIDClient()
    let focus = FocusMonitor()

    private var bag = Set<AnyCancellable>()

    func start() {
        hid.start()
        focus.start()

        focus.$frontmostBundleId
            .compactMap { $0 }
            .removeDuplicates()
            .sink { [weak self] bundleId in
                self?.applyFocus(bundleId: bundleId)
            }
            .store(in: &bag)
    }

    func stop() {
        focus.stop()
        hid.stop()
        bag.removeAll()
    }

    func applyFocus(bundleId: String) {
        guard autoEnabled else { return }

        if let rule = rules.first(where: { $0.bundleId == bundleId }) {
            let nextBadge = GameBadge(rawValue: rule.badge) ?? .idle
            setBadge(nextBadge)
            if rule.autoLayer, let nextLayer = StickyLayer(rawValue: rule.layer) {
                setLayer(nextLayer)
            }
        } else {
            setBadge(.idle)
            setLayer(.base)
        }
    }

    func setBadge(_ badge: GameBadge) {
        self.badge = badge
        hid.setBadge(badge)
    }

    func setLayer(_ layer: StickyLayer) {
        self.layer = layer
        hid.setLayer(layer)
    }
}
