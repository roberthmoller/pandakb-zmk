import Combine
import Foundation

final class CompanionModel: ObservableObject {
    @Published var status = MediaStatus(playback: .stopped, volume: 0)

    let hid = HIDClient()
    let media = MediaMonitor()

    private var bag = Set<AnyCancellable>()
    private var lastSent = MediaStatus(playback: .stopped, volume: 255) // force first push

    func start() {
        hid.start()
        media.start()

        media.$status
            .receive(on: DispatchQueue.main)
            .sink { [weak self] next in
                self?.pushIfChanged(next)
            }
            .store(in: &bag)

        hid.$connected
            .receive(on: DispatchQueue.main)
            .sink { [weak self] connected in
                guard let self, connected else { return }
                // Resync when the keyboard reconnects.
                self.lastSent = MediaStatus(playback: .stopped, volume: 255)
                self.pushIfChanged(self.media.status)
            }
            .store(in: &bag)
    }

    func stop() {
        media.stop()
        hid.stop()
        bag.removeAll()
    }

    private func pushIfChanged(_ next: MediaStatus) {
        status = next
        guard next != lastSent else { return }
        lastSent = next
        hid.setMedia(next)
    }
}
