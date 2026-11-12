use serde_json::Value;

use crate::protocol::Widget;

const LOL_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";

pub async fn poll_lol(client: &reqwest::Client) -> Option<Widget> {
    let resp = client.get(LOL_URL).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: Value = resp.json().await.ok()?;
    parse_all_game_data(&json)
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
                p.get("summonerName").and_then(|s| s.as_str()) == Some(active_name)
                    || p.get("riotIdGameName").and_then(|s| s.as_str()) == Some(active_name)
            }
        })
        .or_else(|| players.first())?;

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

    Some(Widget::Lol {
        kills,
        deaths,
        assists,
        cs,
        level,
        valid: true,
    })
}

pub fn insecure_lol_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_millis(1500))
        .build()
        .expect("reqwest client")
}
