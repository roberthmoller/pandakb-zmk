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

#[allow(dead_code)]
pub const LAYER_PREVIEW_HIDE: u8 = 0;
pub const LAYER_PREVIEW_SHOW: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct LayerPreview {
    pub show: bool,
    pub layer: u8,
}

impl LayerPreview {
    /// Parse a Raw HID input report. Accepts with or without a leading report-id byte.
    pub fn from_report(data: &[u8]) -> Option<Self> {
        let payload = if data.first() == Some(&CMD_LAYER_PREVIEW) {
            data
        } else if data.len() > 1 && data[0] == 0 && data[1] == CMD_LAYER_PREVIEW {
            &data[1..]
        } else {
            return None;
        };

        if payload.len() < 3 || payload[0] != CMD_LAYER_PREVIEW {
            return None;
        }

        Some(Self {
            show: payload[1] == LAYER_PREVIEW_SHOW,
            layer: payload[2],
        })
    }
}

pub const MODE_IDLE: u8 = 0;
pub const MODE_MEDIA: u8 = 1;
pub const MODE_LOL: u8 = 2;
pub const MODE_IDE: u8 = 3;

pub const FLAG_VALID: u8 = 0x01;

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
                valid,
            } => {
                report[2] = *kills;
                report[3] = *deaths;
                report[4] = *assists;
                report[5] = (*cs & 0xff) as u8;
                report[6] = ((*cs >> 8) & 0xff) as u8;
                report[7] = *level;
                report[8] = if *valid { FLAG_VALID } else { 0 };
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
                valid,
            } => {
                if *valid {
                    format!("LoL {kills}/{deaths}/{assists} CS{cs} Lv{level}")
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
