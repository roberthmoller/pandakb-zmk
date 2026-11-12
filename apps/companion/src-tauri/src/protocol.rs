//! PandaKB Raw HID widget protocol (mirrors modules/app-layer/PROTOCOL.md).

pub const USAGE_PAGE: u16 = 0xff60;
pub const USAGE: u16 = 0x61;
pub const VENDOR_ID: u16 = 0x1d50;
pub const PRODUCT_ID: u16 = 0x615e;
pub const REPORT_SIZE: usize = 32;

#[allow(dead_code)]
pub const CMD_SET_MEDIA: u8 = 0x03;
pub const CMD_SET_WIDGET: u8 = 0x10;
pub const CMD_LAYER_PREVIEW: u8 = 0x20;
pub const CMD_KEY_MATRIX: u8 = 0x21;
pub const CMD_STATUS: u8 = 0x22;
pub const CMD_ENTER_BOOTLOADER: u8 = 0x30;
pub const CMD_SET_LAYER: u8 = 0x31;
pub const CMD_GET_STATUS: u8 = 0x32;

/// Battery SoC not yet reported by the keyboard.
pub const BATTERY_UNKNOWN: u8 = 0xff;

pub const BOOTLOADER_SELF: u8 = 0;
pub const BOOTLOADER_LEFT: u8 = 1;
pub const BOOTLOADER_RIGHT: u8 = 2;

pub const LAYER_BASE: u8 = 0;
pub const LAYER_LOWER: u8 = 1;
pub const LAYER_RAISE: u8 = 2;
pub const LAYER_ADJUST: u8 = 3;
pub const LAYER_WOW: u8 = 4;
pub const LAYER_LOL: u8 = 5;

#[allow(dead_code)]
pub const LAYER_PREVIEW_HIDE: u8 = 0;
pub const LAYER_PREVIEW_SHOW: u8 = 1;

pub const KEY_MATRIX_BYTES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct LayerPreview {
    pub show: bool,
    pub layer: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct KeyMatrix {
    /// Pressed matrix positions (Lily58 indices).
    pub pressed: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct BoardStatus {
    /// Left (central) SoC 0–100, or [`BATTERY_UNKNOWN`].
    pub left_battery: u8,
    /// Right (peripheral) SoC 0–100, or [`BATTERY_UNKNOWN`].
    pub right_battery: u8,
    pub layer: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundReport {
    LayerPreview(LayerPreview),
    KeyMatrix(KeyMatrix),
    Status(BoardStatus),
}

fn strip_report_id(data: &[u8]) -> &[u8] {
    if data.first() == Some(&0) && data.len() > 1 {
        &data[1..]
    } else {
        data
    }
}

impl LayerPreview {
    fn parse(payload: &[u8]) -> Option<Self> {
        if payload.len() < 3 || payload[0] != CMD_LAYER_PREVIEW {
            return None;
        }
        Some(Self {
            show: payload[1] == LAYER_PREVIEW_SHOW,
            layer: payload[2],
        })
    }
}

impl KeyMatrix {
    fn parse(payload: &[u8]) -> Option<Self> {
        if payload.len() < 1 + KEY_MATRIX_BYTES || payload[0] != CMD_KEY_MATRIX {
            return None;
        }
        let mut pressed = Vec::new();
        for (byte_i, byte) in payload[1..1 + KEY_MATRIX_BYTES].iter().enumerate() {
            for bit in 0..8u8 {
                if byte & (1 << bit) != 0 {
                    pressed.push((byte_i as u8) * 8 + bit);
                }
            }
        }
        Some(Self { pressed })
    }
}

impl BoardStatus {
    fn parse(payload: &[u8]) -> Option<Self> {
        if payload.len() < 4 || payload[0] != CMD_STATUS {
            return None;
        }
        Some(Self {
            left_battery: payload[1],
            right_battery: payload[2],
            layer: payload[3],
        })
    }
}

impl InboundReport {
    /// Parse a Raw HID input report. Accepts with or without a leading report-id byte.
    pub fn from_report(data: &[u8]) -> Option<Self> {
        let payload = strip_report_id(data);
        match payload.first().copied()? {
            CMD_LAYER_PREVIEW => LayerPreview::parse(payload).map(Self::LayerPreview),
            CMD_KEY_MATRIX => KeyMatrix::parse(payload).map(Self::KeyMatrix),
            CMD_STATUS => BoardStatus::parse(payload).map(Self::Status),
            _ => None,
        }
    }
}

/// Short layer label for the menu bar (matches Lily58 companion keymap order).
pub fn layer_menu_name(layer: u8) -> &'static str {
    match layer {
        LAYER_BASE => "Base",
        LAYER_LOWER => "Lower",
        LAYER_RAISE => "Raise",
        LAYER_ADJUST => "Adjust",
        LAYER_WOW => "WoW",
        LAYER_LOL => "LoL",
        _ => "—",
    }
}

pub fn format_battery(soc: u8) -> String {
    if soc == BATTERY_UNKNOWN || soc > 100 {
        "–".into()
    } else {
        format!("{soc}%")
    }
}

pub const MODE_IDLE: u8 = 0;
pub const MODE_MEDIA: u8 = 1;
pub const MODE_LOL: u8 = 2;
pub const MODE_IDE: u8 = 3;

pub const FLAG_VALID: u8 = 0x01;
pub const LOL_EVENT_SHIFT: u8 = 1;
pub const LOL_EVENT_MASK: u8 = 0x07;

/// One-shot LoL OLED flash (Live Client Multikill / Ace / …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum LolEvent {
    #[default]
    None = 0,
    Kill = 1,
    Double = 2,
    Triple = 3,
    Quadra = 4,
    Penta = 5,
    Ace = 6,
    FirstBlood = 7,
}

impl LolEvent {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Kill => "+",
            Self::Double => "x2",
            Self::Triple => "x3",
            Self::Quadra => "x4",
            Self::Penta => "x5",
            Self::Ace => "ACE",
            Self::FirstBlood => "FB",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Playback {
    Stopped = 0,
    Playing = 1,
    Paused = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum IdeBuild {
    #[default]
    Idle = 0,
    Building = 1,
    Ok = 2,
    Fail = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Widget {
    Idle,
    Media {
        playback: Playback,
        volume: u8,
        title: String,
    },
    Lol {
        kills: u8,
        deaths: u8,
        assists: u8,
        cs: u16,
        level: u8,
        event: LolEvent,
        valid: bool,
    },
    Ide {
        errors: u8,
        warnings: u8,
        build: IdeBuild,
        tag: String,
    },
}

impl Widget {
    pub fn mode(&self) -> u8 {
        match self {
            Widget::Idle => MODE_IDLE,
            Widget::Media { .. } => MODE_MEDIA,
            Widget::Lol { .. } => MODE_LOL,
            Widget::Ide { .. } => MODE_IDE,
        }
    }

    pub fn to_report(&self) -> [u8; REPORT_SIZE] {
        let mut report = [0u8; REPORT_SIZE];
        report[0] = CMD_SET_WIDGET;
        report[1] = self.mode();

        match self {
            Widget::Idle => {}
            Widget::Media {
                playback,
                volume,
                title,
            } => {
                report[2] = *playback as u8;
                report[3] = (*volume).min(100);
                write_ascii(&mut report[4..9], title);
            }
            Widget::Lol {
                kills,
                deaths,
                assists,
                cs,
                level,
                event,
                valid,
            } => {
                report[2] = *kills;
                report[3] = *deaths;
                report[4] = *assists;
                report[5] = (*cs & 0xff) as u8;
                report[6] = ((*cs >> 8) & 0xff) as u8;
                report[7] = *level;
                let mut flags = if *valid { FLAG_VALID } else { 0 };
                flags |= (*event as u8 & LOL_EVENT_MASK) << LOL_EVENT_SHIFT;
                report[8] = flags;
            }
            Widget::Ide {
                errors,
                warnings,
                build,
                tag,
            } => {
                report[2] = *errors;
                report[3] = *warnings;
                report[4] = *build as u8;
                write_ascii(&mut report[5..9], tag);
            }
        }

        report
    }

    pub fn summary(&self) -> String {
        match self {
            Widget::Idle => "idle".into(),
            Widget::Media {
                playback,
                volume,
                title,
            } => {
                let p = match playback {
                    Playback::Playing => "playing",
                    Playback::Paused => "paused",
                    Playback::Stopped => "stopped",
                };
                if title.is_empty() {
                    format!("{p} {volume}%")
                } else {
                    format!("{p} {volume}% · {title}")
                }
            }
            Widget::Lol {
                kills,
                deaths,
                assists,
                cs,
                level,
                event,
                valid,
            } => {
                if *valid {
                    let flash = event.label();
                    if flash.is_empty() {
                        format!("LoL {kills}/{deaths}/{assists} CS{cs} Lv{level}")
                    } else {
                        format!("LoL {flash} {kills}/{deaths}/{assists} CS{cs}")
                    }
                } else {
                    "LoL …".into()
                }
            }
            Widget::Ide {
                errors,
                warnings,
                build,
                tag,
            } => {
                let b = match build {
                    IdeBuild::Idle => "idle",
                    IdeBuild::Building => "bld",
                    IdeBuild::Ok => "ok",
                    IdeBuild::Fail => "fail",
                };
                format!("IDE {tag} {b} E{errors} W{warnings}")
            }
        }
    }
}

fn write_ascii(dst: &mut [u8], text: &str) {
    for (i, b) in text.bytes().take(dst.len()).enumerate() {
        dst[i] = if (0x20..=0x7e).contains(&b) { b } else { b'?' };
    }
}
