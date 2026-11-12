use std::sync::Mutex;

use serde_json::Value;

use crate::protocol::{LolEvent, Widget};

const LOL_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";

struct LolTracker {
    last_event_id: i64,
}

impl Default for LolTracker {
    fn default() -> Self {
        Self { last_event_id: -1 }
    }
}

static TRACKER: Mutex<LolTracker> = Mutex::new(LolTracker {
    last_event_id: -1,
});

pub async fn poll_lol(client: &reqwest::Client) -> Option<Widget> {
    let resp = client.get(LOL_URL).send().await.ok()?;
    if !resp.status().is_success() {
        reset_tracker();
        return None;
    }
    let json: Value = resp.json().await.ok()?;
    match parse_all_game_data(&json) {
        Some(widget) => Some(widget),
        None => {
            reset_tracker();
            None
        }
    }
}

fn reset_tracker() {
    if let Ok(mut t) = TRACKER.lock() {
        *t = LolTracker::default();
    }
}

fn parse_all_game_data(json: &Value) -> Option<Widget> {
    let active = json.get("activePlayer")?;
    let active_name = active
        .get("summonerName")
        .or_else(|| active.get("riotIdGameName"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut level = active
        .get("level")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .min(255) as u8;

    let players = json.get("allPlayers")?.as_array()?;
    let me = players
        .iter()
        .find(|p| {
            if active_name.is_empty() {
                p.get("isBot").and_then(|b| b.as_bool()) != Some(true)
            } else {
                player_name_matches(p, active_name)
            }
        })
        .or_else(|| players.first())?;

    let my_names = player_names(me, active_name);

    let scores = me.get("scores")?;
    let kills = scores.get("kills").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
    let deaths = scores.get("deaths").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
    let assists = scores.get("assists").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
    let cs = scores
        .get("creepScore")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .min(u16::MAX as u64) as u16;

    if level == 0 {
        level = me
            .get("level")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            .min(255) as u8;
    }

    let event = take_local_event(json, &my_names);

    Some(Widget::Lol {
        kills,
        deaths,
        assists,
        cs,
        level,
        event,
        valid: true,
    })
}

fn player_names(player: &Value, active_name: &str) -> Vec<String> {
    let mut names = Vec::new();
    for key in ["summonerName", "riotIdGameName", "riotId"] {
        if let Some(s) = player.get(key).and_then(|v| v.as_str()) {
            if !s.is_empty() {
                names.push(s.to_string());
            }
        }
    }
    if !active_name.is_empty() && !names.iter().any(|n| n == active_name) {
        names.push(active_name.to_string());
    }
    names
}

fn player_name_matches(player: &Value, active_name: &str) -> bool {
    player
        .get("summonerName")
        .and_then(|s| s.as_str())
        .is_some_and(|s| s == active_name)
        || player
            .get("riotIdGameName")
            .and_then(|s| s.as_str())
            .is_some_and(|s| s == active_name)
}

fn name_is_local(name: &str, my_names: &[String]) -> bool {
    my_names.iter().any(|n| n == name)
}

/// Diff Live Client events since the last poll; return a one-shot flash code.
fn take_local_event(json: &Value, my_names: &[String]) -> LolEvent {
    let Ok(mut tracker) = TRACKER.lock() else {
        return LolEvent::None;
    };

    let Some(events) = json
        .pointer("/events/Events")
        .or_else(|| json.pointer("/Events"))
        .and_then(|v| v.as_array())
    else {
        return LolEvent::None;
    };

    let mut max_id = tracker.last_event_id;
    for ev in events {
        if let Some(id) = ev.get("EventID").and_then(|v| v.as_i64()) {
            max_id = max_id.max(id);
        }
    }

    // First sight of a game: seed cursor so mid-match join does not replay history.
    if tracker.last_event_id < 0 {
        tracker.last_event_id = max_id;
        return LolEvent::None;
    }

    let mut best = LolEvent::None;

    for ev in events {
        let Some(id) = ev.get("EventID").and_then(|v| v.as_i64()) else {
            continue;
        };
        if id <= tracker.last_event_id {
            continue;
        }

        let Some(name) = ev.get("EventName").and_then(|v| v.as_str()) else {
            continue;
        };

        let candidate = match name {
            "Multikill" => {
                let killer = ev.get("KillerName").and_then(|v| v.as_str()).unwrap_or("");
                if !name_is_local(killer, my_names) {
                    LolEvent::None
                } else {
                    match ev.get("KillStreak").and_then(|v| v.as_u64()).unwrap_or(0) {
                        2 => LolEvent::Double,
                        3 => LolEvent::Triple,
                        4 => LolEvent::Quadra,
                        n if n >= 5 => LolEvent::Penta,
                        _ => LolEvent::Kill,
                    }
                }
            }
            "ChampionKill" => {
                let killer = ev.get("KillerName").and_then(|v| v.as_str()).unwrap_or("");
                if name_is_local(killer, my_names) {
                    LolEvent::Kill
                } else {
                    LolEvent::None
                }
            }
            "Ace" => {
                let acer = ev.get("Acer").and_then(|v| v.as_str()).unwrap_or("");
                if name_is_local(acer, my_names) {
                    LolEvent::Ace
                } else {
                    LolEvent::None
                }
            }
            "FirstBlood" => {
                let recipient = ev.get("Recipient").and_then(|v| v.as_str()).unwrap_or("");
                if name_is_local(recipient, my_names) {
                    LolEvent::FirstBlood
                } else {
                    LolEvent::None
                }
            }
            _ => LolEvent::None,
        };

        if event_priority(candidate) > event_priority(best) {
            best = candidate;
        }
    }

    tracker.last_event_id = max_id;
    best
}

fn event_priority(event: LolEvent) -> u8 {
    match event {
        LolEvent::None => 0,
        LolEvent::Kill => 1,
        LolEvent::FirstBlood => 2,
        LolEvent::Double => 3,
        LolEvent::Triple => 4,
        LolEvent::Quadra => 5,
        LolEvent::Ace => 6,
        LolEvent::Penta => 7,
    }
}

pub fn insecure_lol_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_millis(1500))
        .build()
        .expect("reqwest client")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_game(events: Value) -> Value {
        json!({
            "activePlayer": { "level": 11, "summonerName": "Panda" },
            "allPlayers": [{
                "summonerName": "Panda",
                "isBot": false,
                "level": 11,
                "scores": { "kills": 5, "deaths": 0, "assists": 1, "creepScore": 124 }
            }],
            "events": { "Events": events }
        })
    }

    #[test]
    fn seeds_history_then_flashes_new_penta_once() {
        reset_tracker();
        let baseline = sample_game(json!([
            { "EventID": 1, "EventName": "ChampionKill", "KillerName": "Panda", "VictimName": "A" },
            { "EventID": 2, "EventName": "Multikill", "KillerName": "Panda", "KillStreak": 5 }
        ]));
        match parse_all_game_data(&baseline).unwrap() {
            Widget::Lol { event, cs, .. } => {
                assert_eq!(event, LolEvent::None, "first poll must not replay history");
                assert_eq!(cs, 124);
            }
            _ => panic!("expected lol"),
        }

        let with_new = sample_game(json!([
            { "EventID": 1, "EventName": "ChampionKill", "KillerName": "Panda", "VictimName": "A" },
            { "EventID": 2, "EventName": "Multikill", "KillerName": "Panda", "KillStreak": 5 },
            { "EventID": 3, "EventName": "ChampionKill", "KillerName": "Panda", "VictimName": "B" },
            { "EventID": 4, "EventName": "Multikill", "KillerName": "Panda", "KillStreak": 5 }
        ]));
        match parse_all_game_data(&with_new).unwrap() {
            Widget::Lol { event, .. } => assert_eq!(event, LolEvent::Penta),
            _ => panic!("expected lol"),
        }
        match parse_all_game_data(&with_new).unwrap() {
            Widget::Lol { event, .. } => assert_eq!(event, LolEvent::None),
            _ => panic!("expected lol"),
        }
    }
}
