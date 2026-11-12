import CoreAudio
import Darwin
import Foundation

/// Polls macOS now-playing (MediaRemote) and output volume. Change-only publishing.
final class MediaMonitor: ObservableObject {
    @Published private(set) var status = MediaStatus(playback: .stopped, volume: 0)

    private var timer: Timer?
    private var getNowPlayingInfo: MRMediaRemoteGetNowPlayingInfoFunction?

    private typealias MRMediaRemoteGetNowPlayingInfoFunction =
        @convention(c) (DispatchQueue, @escaping ([AnyHashable: Any]) -> Void) -> Void

    func start() {
        loadMediaRemote()
        stop()
        let t = Timer.scheduledTimer(withTimeInterval: 2.0, repeats: true) { [weak self] _ in
            self?.poll()
        }
        RunLoop.main.add(t, forMode: .common)
        timer = t
        poll()
    }

    func stop() {
        timer?.invalidate()
        timer = nil
    }

    private func loadMediaRemote() {
        let path = "/System/Library/PrivateFrameworks/MediaRemote.framework/MediaRemote"
        guard let handle = dlopen(path, RTLD_LAZY) else { return }
        typealias Fn = MRMediaRemoteGetNowPlayingInfoFunction
        if let sym = dlsym(handle, "MRMediaRemoteGetNowPlayingInfo") {
            getNowPlayingInfo = unsafeBitCast(sym, to: Fn.self)
        }
    }

    private func poll() {
        let volume = Self.systemOutputVolumePercent()
        guard let getNowPlayingInfo else {
            publish(MediaStatus(playback: .stopped, volume: volume))
            return
        }

        getNowPlayingInfo(DispatchQueue.main) { [weak self] info in
            guard let self else { return }
            let playback = Self.playback(from: info)
            self.publish(MediaStatus(playback: playback, volume: volume))
        }
    }

    private func publish(_ next: MediaStatus) {
        if next != status {
            status = next
        }
    }

    private static func playback(from info: [AnyHashable: Any]) -> MediaPlayback {
        // kMRMediaRemoteNowPlayingInfoPlaybackRate
        let rateKey = "kMRMediaRemoteNowPlayingInfoPlaybackRate"
        let rate = (info[rateKey] as? Double)
            ?? (info[rateKey] as? NSNumber)?.doubleValue
            ?? 0

        let hasTitle = (info["kMRMediaRemoteNowPlayingInfoTitle"] as? String)?.isEmpty == false
            || (info["Title"] as? String)?.isEmpty == false

        if rate > 0.01 {
            return .playing
        }
        if hasTitle {
            return .paused
        }
        return .stopped
    }

    private static func systemOutputVolumePercent() -> UInt8 {
        var deviceID = AudioObjectID(kAudioObjectSystemObject)
        var address = AudioObjectPropertyAddress(
            mSelector: kAudioHardwarePropertyDefaultOutputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )
        var size = UInt32(MemoryLayout<AudioObjectID>.size)
        guard AudioObjectGetPropertyData(deviceID, &address, 0, nil, &size, &deviceID) == noErr else {
            return 0
        }

        // Prefer virtual master volume; fall back to channel-1 scalar.
        let selectors: [AudioObjectPropertySelector] = [
            AudioObjectPropertySelector(FourCharCode(0x766D7663)), // 'vmvc' virtual master volume
            kAudioDevicePropertyVolumeScalar,
        ]

        for selector in selectors {
            address = AudioObjectPropertyAddress(
                mSelector: selector,
                mScope: kAudioDevicePropertyScopeOutput,
                mElement: kAudioObjectPropertyElementMain
            )
            var volume: Float32 = 0
            size = UInt32(MemoryLayout<Float32>.size)
            if AudioObjectGetPropertyData(deviceID, &address, 0, nil, &size, &volume) == noErr {
                let clamped = max(0, min(1, volume))
                return UInt8((clamped * 100).rounded())
            }
        }
        return 0
    }
}
