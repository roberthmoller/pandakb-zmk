use std::sync::Mutex;
use std::time::{Duration, Instant};

use hidapi::{HidApi, HidDevice};
use log::{debug, warn};

use crate::protocol::{self, InboundReport, Widget};

pub struct HidClient {
    api: Mutex<HidApi>,
    device: Mutex<Option<HidDevice>>,
    last_path: Mutex<Option<String>>,
    last_sent: Mutex<Option<Widget>>,
    last_open_attempt: Mutex<Instant>,
}

impl HidClient {
    pub fn new() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| format!("hidapi init: {e}"))?;
        Ok(Self {
            api: Mutex::new(api),
            device: Mutex::new(None),
            last_path: Mutex::new(None),
            last_sent: Mutex::new(None),
            last_open_attempt: Mutex::new(Instant::now() - Duration::from_secs(60)),
        })
    }

    pub fn connected(&self) -> bool {
        self.device.lock().unwrap().is_some()
    }

    pub fn ensure_connected(&self) -> bool {
        if self.connected() {
            return true;
        }

        let mut last_attempt = self.last_open_attempt.lock().unwrap();
        if last_attempt.elapsed() < Duration::from_secs(2) {
            return false;
        }
        *last_attempt = Instant::now();
        drop(last_attempt);

        let mut api = self.api.lock().unwrap();
        if let Err(e) = api.refresh_devices() {
            warn!("hid refresh failed: {e}");
            return false;
        }

        let path = api.device_list().find_map(|info| {
            if info.vendor_id() == protocol::VENDOR_ID
                && info.product_id() == protocol::PRODUCT_ID
                && info.usage_page() == protocol::USAGE_PAGE
                && info.usage() == protocol::USAGE
            {
                Some(info.path().to_owned())
            } else {
                None
            }
        });

        let Some(path) = path else {
            return false;
        };

        match api.open_path(&path) {
            Ok(dev) => {
                if let Err(e) = dev.set_blocking_mode(false) {
                    warn!("HID non-blocking mode failed: {e}");
                }
                debug!("opened Raw HID {}", path.to_string_lossy());
                *self.last_path.lock().unwrap() = Some(path.to_string_lossy().into_owned());
                *self.device.lock().unwrap() = Some(dev);
                *self.last_sent.lock().unwrap() = None;
                true
            }
            Err(e) => {
                warn!("open Raw HID failed: {e}");
                false
            }
        }
    }

    fn write_report(&self, payload: &[u8; protocol::REPORT_SIZE]) -> Result<(), String> {
        if !self.ensure_connected() {
            return Err("Keyboard Raw HID not connected (plug in left half USB)".into());
        }

        // hidapi expects a leading report ID byte (0 for this device).
        let mut out = vec![0u8; protocol::REPORT_SIZE + 1];
        out[1..].copy_from_slice(payload);

        let mut guard = self.device.lock().unwrap();
        let Some(dev) = guard.as_mut() else {
            return Err("HID device missing".into());
        };

        match dev.write(&out) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("HID write failed: {e}");
                *guard = None;
                Err(format!("HID send failed: {e}"))
            }
        }
    }

    pub fn push_if_changed(&self, widget: &Widget) -> Result<bool, String> {
        {
            let last = self.last_sent.lock().unwrap();
            if last.as_ref() == Some(widget) {
                return Ok(false);
            }
        }

        self.write_report(&widget.to_report())?;
        *self.last_sent.lock().unwrap() = Some(widget.clone());
        Ok(true)
    }

    /// Ask the keyboard to soft-enter UF2 bootloader (protocol 0x30).
    pub fn enter_bootloader(&self, half: u8) -> Result<(), String> {
        let mut payload = [0u8; protocol::REPORT_SIZE];
        payload[0] = protocol::CMD_ENTER_BOOTLOADER;
        payload[1] = half;
        // Drop cached widget — device will disconnect.
        *self.last_sent.lock().unwrap() = None;
        self.write_report(&payload)?;
        // Device resets; drop the handle so we reopen cleanly later.
        *self.device.lock().unwrap() = None;
        Ok(())
    }

    /// Sticky layer switch on the keyboard (protocol 0x31, `&to` semantics).
    pub fn set_layer(&self, layer: u8) -> Result<(), String> {
        let mut payload = [0u8; protocol::REPORT_SIZE];
        payload[0] = protocol::CMD_SET_LAYER;
        payload[1] = layer;
        self.write_report(&payload)
    }

    /// Ask the keyboard to push a STATUS (0x22) report.
    pub fn request_status(&self) -> Result<(), String> {
        let mut payload = [0u8; protocol::REPORT_SIZE];
        payload[0] = protocol::CMD_GET_STATUS;
        self.write_report(&payload)
    }

    /// Drain available input reports (LAYER_PREVIEW / KEY_MATRIX / STATUS).
    pub fn poll_inbound(&self) -> Vec<InboundReport> {
        if !self.ensure_connected() {
            return Vec::new();
        }

        let mut guard = self.device.lock().unwrap();
        let Some(dev) = guard.as_mut() else {
            return Vec::new();
        };

        let mut out = Vec::new();
        let mut buf = [0u8; protocol::REPORT_SIZE + 1];
        loop {
            match dev.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if let Some(report) = InboundReport::from_report(&buf[..n]) {
                        out.push(report);
                    }
                }
                Err(e) => {
                    let msg = e.to_string().to_lowercase();
                    // Would-block / timeout — nothing pending.
                    if msg.contains("timed out")
                        || msg.contains("would block")
                        || msg.contains("resource temporarily unavailable")
                    {
                        break;
                    }
                    warn!("HID read failed: {e}");
                    *guard = None;
                    break;
                }
            }
        }
        out
    }
}
