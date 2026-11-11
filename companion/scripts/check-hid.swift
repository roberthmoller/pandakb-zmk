#!/usr/bin/env swift
import Foundation
import IOKit.hid

let usagePage: Int = 0xFF60
let usage: Int = 0x61

let manager = IOHIDManagerCreate(kCFAllocatorDefault, IOOptionBits(kIOHIDOptionsTypeNone))
let matches: [[String: Any]] = [[
    kIOHIDDeviceUsagePageKey as String: usagePage,
    kIOHIDDeviceUsageKey as String: usage,
]]
IOHIDManagerSetDeviceMatchingMultiple(manager, matches as CFArray)
IOHIDManagerOpen(manager, IOOptionBits(kIOHIDOptionsTypeNone))

guard let devices = IOHIDManagerCopyDevices(manager) as? Set<IOHIDDevice> else {
    print("No Raw HID devices matched usage page 0xFF60 / usage 0x61")
    print("Flash the new left firmware (raw_hid_adapter) and reconnect USB.")
    exit(1)
}

print("Found \(devices.count) Raw HID device(s):")
for device in devices {
    let product = IOHIDDeviceGetProperty(device, kIOHIDProductKey as CFString) as? String ?? "?"
    let vid = IOHIDDeviceGetProperty(device, kIOHIDVendorIDKey as CFString) as? Int ?? 0
    let pid = IOHIDDeviceGetProperty(device, kIOHIDProductIDKey as CFString) as? Int ?? 0
    print(String(format: "- %@  VID=0x%04X PID=0x%04X", product, vid, pid))
}
exit(0)
