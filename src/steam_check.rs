use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;
use steam_vent::{Connection, ConnectionError, ConnectionTrait, EResult, LoginError, ServerList};
use steam_vent_proto::steammessages_auth_steamclient::CAuthentication_AccessToken_GenerateForApp_Request;

#[derive(Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Liveness {
    Valid,
    Invalid,
    Unknown,
}

#[derive(Default, Clone, Serialize)]
pub struct DeepData {
    pub premier_rating: Option<i64>,
    pub premier_wins: Option<i64>,
    pub wingman_rank: Option<i64>,
    pub cooldown_expires: Option<i64>,
}

pub struct DeepOutcome {
    pub liveness: Liveness,
    pub data: Option<DeepData>,
    pub steamid: Option<u64>,
}

pub type ServerListCell = tokio::sync::OnceCell<ServerList>;

pub async fn server_list(cell: &ServerListCell) -> Option<&ServerList> {
    cell.get_or_try_init(|| async { ServerList::discover().await })
        .await
        .ok()
}

fn is_dead_error(err: &ConnectionError) -> bool {
    match err {
        ConnectionError::AccessToken(_) => true,
        ConnectionError::LoginError(le) => match le {
            LoginError::InvalidCredentials
            | LoginError::UnavailableAccount
            | LoginError::InvalidSteamId => true,
            LoginError::Unknown(er) => matches!(
                er,
                EResult::Expired
                    | EResult::Revoked
                    | EResult::InvalidPassword
                    | EResult::AccessDenied
            ),
            _ => false,
        },
        _ => false,
    }
}

async fn mint_access_token(conn: &Connection, raw_token: &str, steamid: u64) -> Option<String> {
    let req = CAuthentication_AccessToken_GenerateForApp_Request {
        refresh_token: Some(raw_token.to_string()),
        steamid: Some(steamid),
        renewal_type: None,
        ..Default::default()
    };
    let resp = conn.service_method(req).await.ok()?;
    if resp
        .refresh_token
        .as_deref()
        .map(|s| !s.is_empty())
        .unwrap_or(false)
    {
        return None;
    }
    resp.access_token.filter(|s| !s.is_empty())
}

pub async fn deep_check(server_list: &ServerList, account: &str, raw_token: &str) -> DeepOutcome {
    match Connection::access(server_list, account, raw_token).await {
        Ok(conn) => {
            let steamid: u64 = conn.steam_id().into();
            let access = mint_access_token(&conn, raw_token, steamid).await;
            drop(conn);
            let data = match access {
                Some(a) => tokio::task::spawn_blocking(move || scrape_gcpd(steamid, &a))
                    .await
                    .ok()
                    .flatten(),
                None => None,
            };
            DeepOutcome {
                liveness: Liveness::Valid,
                data,
                steamid: Some(steamid),
            }
        }
        Err(err) => DeepOutcome {
            liveness: if is_dead_error(&err) {
                Liveness::Invalid
            } else {
                Liveness::Unknown
            },
            data: None,
            steamid: None,
        },
    }
}

fn scrape_gcpd(steamid: u64, access_token: &str) -> Option<DeepData> {
    let cookie_value = urlencoding::encode(&format!("{steamid}||{access_token}")).into_owned();
    let url =
        format!("https://steamcommunity.com/profiles/{steamid}/gcpd/730?tab=matchmaking&l=english");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .ok()?;
    let resp = client
        .get(&url)
        .header("Cookie", format!("steamLoginSecure={cookie_value}"))
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let html = resp.text().ok()?;
    if looks_like_login_page(&html) || !looks_like_gcpd_page(&html) {
        return None;
    }
    Some(parse_matchmaking(&html))
}

fn looks_like_login_page(html: &str) -> bool {
    html.contains("g_steamID = false") || html.contains("<title>Sign In")
}
fn looks_like_gcpd_page(html: &str) -> bool {
    html.contains("generic_kv_table") || html.contains("Personal Game Data")
}

const COOLDOWN_PERMANENT: i64 = 2_000_000_000;

fn re(pat: &str, cell: &'static OnceLock<Regex>) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).unwrap())
}

fn strip_tags(s: &str) -> String {
    static TAG: OnceLock<Regex> = OnceLock::new();
    let no_tags = re(r"<[^>]*>", &TAG).replace_all(s, "");
    let mut out = no_tags.replace(['\n', '\r', '\t'], "");
    for (from, to) in [
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#39;", "'"),
        ("&nbsp;", " "),
    ] {
        out = out.replace(from, to);
    }
    out.trim().to_string()
}

fn to_int(s: &str, dflt: i64) -> i64 {
    let cleaned: String = s.chars().filter(|c| !matches!(c, ',' | ' ')).collect();
    static NUM: OnceLock<Regex> = OnceLock::new();
    match re(r"^[+-]?\d+", &NUM).find(&cleaned) {
        Some(m) => m.as_str().parse().unwrap_or(dflt),
        None => dflt,
    }
}

fn civil_to_unix(y: i64, m: i64, d: i64, hh: i64, mm: i64, ss: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    days * 86400 + hh * 3600 + mm * 60 + ss
}

fn parse_timestamp(s: &str) -> i64 {
    static TS: OnceLock<Regex> = OnceLock::new();
    let r = re(r"\s*(\d+)-(\d+)-(\d+)\s+(\d+):(\d+):(\d+)", &TS);
    match r.captures(s) {
        Some(c) => {
            let g = |i: usize| c.get(i).and_then(|m| m.as_str().parse::<i64>().ok()).unwrap_or(0);
            civil_to_unix(g(1), g(2), g(3), g(4), g(5), g(6))
        }
        None => 0,
    }
}

fn tables(html: &str) -> Vec<Vec<Vec<String>>> {
    static TABLE: OnceLock<Regex> = OnceLock::new();
    static ROW: OnceLock<Regex> = OnceLock::new();
    static CELL: OnceLock<Regex> = OnceLock::new();
    let table_re = re(
        r#"(?is)<table[^>]*class\s*=\s*"[^"]*generic_kv_table[^"]*"[^>]*>(.*?)</table>"#,
        &TABLE,
    );
    let row_re = re(r"(?is)<tr[^>]*>(.*?)</tr>", &ROW);
    let cell_re = re(r"(?is)<t[dh][^>]*>(.*?)</t[dh]>", &CELL);
    let mut out = Vec::new();
    for tm in table_re.captures_iter(html) {
        let body = tm.get(1).map(|m| m.as_str()).unwrap_or("");
        let mut rows = Vec::new();
        for rm in row_re.captures_iter(body) {
            let rbody = rm.get(1).map(|m| m.as_str()).unwrap_or("");
            let cells: Vec<String> = cell_re
                .captures_iter(rbody)
                .filter_map(|cm| cm.get(1).map(|m| strip_tags(m.as_str())))
                .filter(|c| !c.is_empty())
                .collect();
            if !cells.is_empty() {
                rows.push(cells);
            }
        }
        out.push(rows);
    }
    out
}

fn ci(hay: &str, needle: &str) -> bool {
    hay.to_lowercase().contains(&needle.to_lowercase())
}

fn parse_matchmaking(html: &str) -> DeepData {
    let mut out = DeepData::default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let mut cooldown: i64 = 0;

    for tbl in tables(html) {
        if tbl.len() < 2 {
            continue;
        }
        let header = &tbl[0];
        if header.is_empty() {
            continue;
        }

        if ci(&header[0], "Cooldown") {
            for row in &tbl[1..] {
                if row.is_empty() {
                    continue;
                }
                let ts = parse_timestamp(&row[0]);
                if ts == 0 {
                    if !row[0].is_empty() && row.len() > 1 && to_int(&row[1], 0) >= 1 {
                        cooldown = COOLDOWN_PERMANENT;
                    }
                    continue;
                }
                if cooldown == COOLDOWN_PERMANENT {
                    continue;
                }
                if ts > now && (cooldown == 0 || ts < cooldown) {
                    cooldown = ts;
                }
            }
            continue;
        }

        if ci(&header[0], "Matchmaking Mode") {
            if header.iter().any(|c| {
                ["Map", "Mappa", "Mapa", "Carte", "Karte"].iter().any(|n| ci(c, n))
            }) {
                continue;
            }
            let mut skill_col: i64 = -1;
            let mut wins_col: i64 = -1;
            for (c, cell) in header.iter().enumerate() {
                if skill_col < 0 && ci(cell, "Skill") {
                    skill_col = c as i64;
                }
                if wins_col < 0 && ci(cell, "Wins") {
                    wins_col = c as i64;
                }
            }
            if skill_col < 0 {
                skill_col = 4;
            }
            if wins_col < 0 {
                wins_col = 1;
            }
            for row in &tbl[1..] {
                if row.is_empty() {
                    continue;
                }
                let skill = row.get(skill_col as usize).map(|s| to_int(s, -1)).unwrap_or(-1);
                let wins = row.get(wins_col as usize).map(|s| to_int(s, -1)).unwrap_or(-1);
                if skill < 0 && wins < 0 {
                    continue;
                }
                if ci(&row[0], "Premier") {
                    if skill > 0 {
                        out.premier_rating = Some(skill);
                    }
                    if wins >= 0 {
                        out.premier_wins = Some(wins);
                    }
                } else if ci(&row[0], "Wingman") {
                    if skill > 0 {
                        out.wingman_rank = Some(skill);
                    }
                }
            }
            continue;
        }
    }

    if cooldown > 0 {
        out.cooldown_expires = Some(cooldown);
    }
    out
}

#[derive(Clone, Serialize, Default)]
pub struct ProfileInfo {
    pub persona: Option<String>,
    pub avatar: Option<String>,
    pub state: i32,
    pub private: bool,
    pub in_game: Option<String>,
}

fn xml_field(html: &str, tag: &str) -> Option<String> {
    let pat = format!(r"(?is)<{tag}>(?:<!\[CDATA\[)?(.*?)(?:\]\]>)?</{tag}>");
    let re = Regex::new(&pat).ok()?;
    let cap = re.captures(html)?;
    let val = cap.get(1)?.as_str().trim().to_string();
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}

pub fn fetch_profile(steamid: u64) -> Option<ProfileInfo> {
    let url = format!("https://steamcommunity.com/profiles/{steamid}/?xml=1");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body = resp.text().ok()?;
    let mut info = ProfileInfo {
        persona: xml_field(&body, "steamID"),
        avatar: xml_field(&body, "avatarFull"),
        ..Default::default()
    };
    let privacy = xml_field(&body, "privacyState").unwrap_or_default();
    if privacy != "public" {
        info.private = true;
        info.state = 0;
        return Some(info);
    }
    match xml_field(&body, "onlineState").as_deref() {
        Some("in-game") => {
            info.state = 6;
            info.in_game = xml_field(&body, "stateMessage");
        }
        Some("online") => info.state = 1,
        _ => info.state = 0,
    }
    Some(info)
}

#[derive(Clone, Serialize, Default)]
pub struct BanInfo {
    pub vac: bool,
    pub vac_count: u32,
    pub game_bans: u32,
    pub community: bool,
}

pub fn fetch_bans(api_key: &str, ids: &[String]) -> HashMap<String, BanInfo> {
    let mut out = HashMap::new();
    if api_key.is_empty() || ids.is_empty() {
        return out;
    }
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return out,
    };
    for chunk in ids.chunks(100) {
        let url = format!(
            "https://api.steampowered.com/ISteamUser/GetPlayerBans/v1/?key={}&steamids={}",
            api_key,
            chunk.join(",")
        );
        let Ok(resp) = client.get(&url).send() else { continue };
        if !resp.status().is_success() {
            continue;
        }
        let Ok(json) = resp.json::<serde_json::Value>() else { continue };
        if let Some(players) = json.get("players").and_then(|p| p.as_array()) {
            for p in players {
                let Some(sid) = p.get("SteamId").and_then(|v| v.as_str()) else { continue };
                out.insert(
                    sid.to_string(),
                    BanInfo {
                        vac: p.get("VACBanned").and_then(|v| v.as_bool()).unwrap_or(false),
                        vac_count: p.get("NumberOfVACBans").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        game_bans: p.get("NumberOfGameBans").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        community: p
                            .get("CommunityBanned")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                    },
                );
            }
        }
    }
    out
}

pub fn fetch_level(api_key: &str, steamid: u64) -> Option<u32> {
    if api_key.is_empty() {
        return None;
    }
    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetSteamLevel/v1/?key={api_key}&steamid={steamid}"
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;
    let resp = client.get(&url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json = resp.json::<serde_json::Value>().ok()?;
    json.get("response")
        .and_then(|r| r.get("player_level"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
}

pub fn fetch_inventory_value(steamid: u64) -> Option<f64> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .ok()?;
    let inv_url =
        format!("https://steamcommunity.com/inventory/{steamid}/730/2?l=english&count=2000");
    let resp = client
        .get(&inv_url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Referer", format!("https://steamcommunity.com/profiles/{steamid}/inventory"))
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json = resp.json::<serde_json::Value>().ok()?;
    let descriptions = json.get("descriptions")?.as_array()?;
    let mut names: HashMap<String, u32> = HashMap::new();
    for d in descriptions {
        let marketable = d.get("marketable").and_then(|v| v.as_u64()).unwrap_or(0);
        if marketable == 0 {
            continue;
        }
        if let Some(name) = d
            .get("market_hash_name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            *names.entry(name.to_string()).or_insert(0) += 1;
        }
    }
    if names.is_empty() {
        return Some(0.0);
    }
    let mut total = 0.0f64;
    let mut looked = 0;
    for (name, count) in names.iter() {
        if looked >= 80 {
            break;
        }
        looked += 1;
        if let Some(price) = market_price(&client, name) {
            total += price * (*count as f64);
        }
        std::thread::sleep(Duration::from_millis(120));
    }
    Some((total * 100.0).round() / 100.0)
}

fn market_price(client: &reqwest::blocking::Client, name: &str) -> Option<f64> {
    let url = "https://steamcommunity.com/market/priceoverview/";
    let resp = client
        .get(url)
        .query(&[
            ("appid", "730"),
            ("currency", "1"),
            ("market_hash_name", name),
        ])
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json = resp.json::<serde_json::Value>().ok()?;
    if !json.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
        return None;
    }
    let raw = json
        .get("median_price")
        .or_else(|| json.get("lowest_price"))
        .and_then(|v| v.as_str())?;
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    cleaned.parse::<f64>().ok()
}
