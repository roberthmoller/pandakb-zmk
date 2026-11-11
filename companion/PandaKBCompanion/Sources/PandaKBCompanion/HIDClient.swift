import Foundation
import IOKit.hid

final class HIDClient: ObservableObject {
    @Published private(set) var connected = false
    @Published private(set) var lastError: String?

    private var manager: IOHIDManager?
    private var device: IOHIDDevice?

    func start() {
        let manager = IOHIDManagerCreate(kCFAllocatorDefault, IOOptionBits(kIOHIDOptionsTypeNone))
        self.manager = manager

        let matches: [[String: Any]] = [[
            kIOHIDDeviceUsagePageKey as String: PandaKBHID.usagePage,
            kIOHIDDeviceUsageKey as String: PandaKBHID.usage,
        ]]
        IOHIDManagerSetDeviceMatchingMultiple(manager, matches as CFArray)

        let ctx = Unmanaged.passUnretained(self).toOpaque()
        IOHIDManagerRegisterDeviceMatchingCallback(manager, { context, _, _, device in
            guard let context else { return }
            let client = Unmanaged<HIDClient>.fromOpaque(context).takeUnretainedValue()
            client.attach(device)
        }, ctx)
        IOHIDManagerRegisterDeviceRemovalCallback(manager, { context, _, _, device in
            guard let context else { return }
            let client = Unmanaged<HIDClient>.fromOpaque(context).takeUnretainedValue()
            client.detach(device)
        }, ctx)

        IOHIDManagerScheduleWithRunLoop(manager, CFRunLoopGetMain(), CFRunLoopMode.defaultMode.rawValue)
        let open = IOHIDManagerOpen(manager, IOOptionBits(kIOHIDOptionsTypeNone))
        if open != kIOReturnSuccess {
            lastError = "HID manager open failed: \(open)"
        }
    }

    func stop() {
        if let manager {
            IOHIDManagerClose(manager, IOOptionBits(kIOHIDOptionsTypeNone))
        }
        manager = nil
        device = nil
        connected = false
    }

    func setLayer(_ layer: StickyLayer) {
        send(command: PandaKBHID.cmdLayerTo, argument: layer.rawValue)
    }

    func setBadge(_ badge: GameBadge) {
        send(command: PandaKBHID.cmdSetBadge, argument: badge.rawValue)
    }

    private func attach(_ device: IOHIDDevice) {
        self.device = device
        DispatchQueue.main.async {
            self.connected = true
            self.lastError = nil
        }
    }

    private func detach(_ device: IOHIDDevice) {
        if self.device == device {
            self.device = nil
            DispatchQueue.main.async {
                self.connected = false
            }
        }
    }

    private func send(command: UInt8, argument: UInt8) {
        guard let device else {
            lastError = "Keyboard Raw HID not connected (plug in USB or pair BLE HID)"
            return
        }

        var report = [UInt8](repeating: 0, count: PandaKBHID.reportSize)
        report[0] = command
        report[1] = argument

        let result = report.withUnsafeMutableBufferPointer { buffer in
            IOHIDDeviceSetReport(
                device,
                kIOHIDReportTypeOutput,
                0,
                buffer.baseAddress!,
                buffer.count
            )
        }

        if result != kIOReturnSuccess {
            lastError = "HID send failed: \(result)"
        } else {
            lastError = nil
        }
    }
}
