#!/usr/bin/env swift
import Foundation
import IOKit.hid

let usagePage: Int = 0xFF60
let usage: Int = 0x61

let manager = IOHIDManagerCreate(kCFAllocatorDefault, IOOptionBits(kIOHIDOptionsTypeNone))
let matches: [[String: Any]] = [[
    kIOHIDVendorIDKey as String: 0x1D50,
    kIOHIDProductIDKey as String: 0x615E,
    kIOHIDDeviceUsagePageKey as String: usagePage,
    kIOHIDDeviceUsageKey as String: usage,
]]
IOHIDManagerSetDeviceMatchingMultiple(manager, matches as CFArray)
IOHIDManagerOpen(manager, IOOptionBits(kIOHIDOptionsTypeNone))

guard let devices = IOHIDManagerCopyDevices(manager) as? Set<IOHIDDevice>, !devices.isEmpty else {
    print("No ZMK Raw HID device (VID 0x1D50 / PID 0x615E, page 0xFF60).")
    print("Flash Lily58_left_oled.uf2, plug the left half over USB, then retry.")
    exit(1)
}

print("Found \(devices.count) ZMK Raw HID device(s):")
for device in devices {
    let product = IOHIDDeviceGetProperty(device, kIOHIDProductKey as CFString) as? String ?? "?"
    let vid = IOHIDDeviceGetProperty(device, kIOHIDVendorIDKey as CFString) as? Int ?? 0
    let pid = IOHIDDeviceGetProperty(device, kIOHIDProductIDKey as CFString) as? Int ?? 0
    print(String(format: "- %@  VID=0x%04X PID=0x%04X", product, vid, pid))
}
exit(0)
