import Combine
import Foundation

final class CompanionModel: ObservableObject {
    /// Legacy Swift driver — prefer the Tauri companion in `companion/zmkwiz`.
    /// Keep HIDClient around for reference; HID is owned by Tauri now.
    static let hidEnabled = false

    @Published var status = MediaStatus(playback: .stopped, volume: 0)

    let hid = HIDClient()
    let media = MediaMonitor()

    private var bag = Set<AnyCancellable>()
    private var lastSent = MediaStatus(playback: .stopped, volume: 255)

    func start() {
        if Self.hidEnabled {
            hid.start()
        }
        media.start()

        media.$status
            .receive(on: DispatchQueue.main)
            .sink { [weak self] next in
                self?.onMedia(next)
            }
            .store(in: &bag)

        if Self.hidEnabled {
            hid.$connected
                .receive(on: DispatchQueue.main)
                .sink { [weak self] connected in
                    guard let self, connected else { return }
                    self.lastSent = MediaStatus(playback: .stopped, volume: 255)
                    self.pushIfChanged(self.media.status)
                }
                .store(in: &bag)
        }
    }

    func stop() {
        media.stop()
        if Self.hidEnabled {
            hid.stop()
        }
        bag.removeAll()
    }

    private func onMedia(_ next: MediaStatus) {
        status = next
        guard Self.hidEnabled else { return }
        pushIfChanged(next)
    }

    private func pushIfChanged(_ next: MediaStatus) {
        guard next != lastSent else { return }
        lastSent = next
        hid.setMedia(next)
    }
}
