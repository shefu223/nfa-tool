#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod steam_check;

use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use steam_check::ServerListCell;
use tauri::State;
use tokio::sync::Semaphore;
use winreg::RegKey;

const WARRANTY_API_URL: &str = "https://shefu223.shop/api/warranty";
const APP_VERSION: &str = "1";
const VERSION_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/shefu223/nfa-tool/main/latest.json";

#[derive(Clone, Serialize, Deserialize)]
struct Account {
    username: String,
    token: String,
    steamid: String,
    #[serde(default = "unix_now")]
    added_at: u64,
    #[serde(default)]
    no_warranty: bool,
    #[serde(default)]
    warranty_expiry: Option<u64>,
}

#[derive(Clone, Serialize)]
struct AccountView {
    username: String,
    steamid: String,
    added_at: u64,
    no_warranty: bool,
    warranty_expiry: Option<u64>,
    token_expiry: Option<u64>,
}

impl From<&Account> for AccountView {
    fn from(a: &Account) -> Self {
        AccountView {
            username: a.username.clone(),
            steamid: a.steamid.clone(),
            added_at: a.added_at,
            no_warranty: a.no_warranty,
            warranty_expiry: a.warranty_expiry,
            token_expiry: jwt_expiry(&a.token),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct Settings {
    #[serde(default)]
    api_key: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct Snapshot {
    steamid: String,
    #[serde(default)]
    token_state: Option<String>,
    #[serde(default)]
    persona: Option<String>,
    #[serde(default)]
    avatar: Option<String>,
    #[serde(default)]
    profile_state: i32,
    #[serde(default)]
    private: bool,
    #[serde(default)]
    in_game: Option<String>,
    #[serde(default)]
    premier_rating: Option<i64>,
    #[serde(default)]
    premier_wins: Option<i64>,
    #[serde(default)]
    wingman_rank: Option<i64>,
    #[serde(default)]
    cooldown_expires: Option<i64>,
    #[serde(default)]
    vac: Option<bool>,
    #[serde(default)]
    vac_count: Option<u32>,
    #[serde(default)]
    game_bans: Option<u32>,
    #[serde(default)]
    level: Option<u32>,
    #[serde(default)]
    inv_value: Option<f64>,
    #[serde(default)]
    public_checked_at: Option<u64>,
    #[serde(default)]
    deep_checked_at: Option<u64>,
    #[serde(default)]
    inv_checked_at: Option<u64>,
}

#[derive(Serialize)]
struct Bootstrap {
    accounts: Vec<AccountView>,
    snapshots: Vec<Snapshot>,
    active_user: Option<String>,
    elevated: bool,
    has_api_key: bool,
}

#[derive(Serialize)]
struct WarrantyView {
    expiry: Option<u64>,
    no_warranty: bool,
}

#[derive(Deserialize)]
struct WarrantyResponse {
    warranty_expires: Option<u64>,
    #[serde(default)]
    no_warranty: bool,
}

#[derive(Deserialize)]
struct VersionManifest {
    version: String,
    #[serde(default)]
    download_url: Option<String>,
}

#[derive(Serialize)]
struct VersionInfo {
    current: String,
    latest: String,
    url: Option<String>,
    update_available: bool,
}

struct AppData {
    accounts: Mutex<Vec<Account>>,
    settings: Mutex<Settings>,
    snapshots: Mutex<HashMap<String, Snapshot>>,
    server_list: ServerListCell,
    deep_gate: Semaphore,
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[tauri::command]
fn bootstrap(state: State<AppData>) -> Bootstrap {
    let accounts = state.accounts.lock().unwrap().iter().map(AccountView::from).collect();
    let snapshots = state.snapshots.lock().unwrap().values().cloned().collect();
    let has_api_key = !state.settings.lock().unwrap().api_key.trim().is_empty();
    Bootstrap {
        accounts,
        snapshots,
        active_user: read_autologin_user(),
        elevated: is_elevated(),
        has_api_key,
    }
}

#[tauri::command]
fn add_account(line: String, state: State<AppData>) -> Result<AccountView, String> {
    let acc = build_account(&line)?;
    let mut accounts = state.accounts.lock().unwrap();
    if let Some(existing) = accounts.iter_mut().find(|a| a.steamid == acc.steamid) {
        existing.username = acc.username.clone();
        existing.token = acc.token.clone();
    } else {
        accounts.push(acc.clone());
    }
    save_accounts(&accounts);
    let view = accounts.iter().find(|a| a.steamid == acc.steamid).map(AccountView::from).unwrap();
    Ok(view)
}

#[tauri::command]
fn remove_account(steamid: String, state: State<AppData>) -> Result<(), String> {
    let mut accounts = state.accounts.lock().unwrap();
    accounts.retain(|a| a.steamid != steamid);
    save_accounts(&accounts);
    Ok(())
}

#[tauri::command]
fn rename_account(steamid: String, name: String, state: State<AppData>) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Name can't be empty.".into());
    }
    let mut accounts = state.accounts.lock().unwrap();
    let acc = accounts.iter_mut().find(|a| a.steamid == steamid).ok_or("Account not found.")?;
    acc.username = name;
    save_accounts(&accounts);
    Ok(())
}

#[tauri::command]
fn clear_all(state: State<AppData>) -> Result<(), String> {
    let mut accounts = state.accounts.lock().unwrap();
    accounts.clear();
    save_accounts(&accounts);
    Ok(())
}

#[tauri::command]
fn active_user() -> Option<String> {
    read_autologin_user()
}

#[tauri::command]
fn restart_admin() {
    if relaunch_as_admin() {
        std::process::exit(0);
    }
}

#[tauri::command]
async fn login(steamid: String, state: State<'_, AppData>) -> Result<String, String> {
    let acc = {
        let accounts = state.accounts.lock().unwrap();
        accounts.iter().find(|a| a.steamid == steamid).cloned()
    }
    .ok_or("Account not found.")?;
    tauri::async_runtime::spawn_blocking(move || login_account(&acc))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn logout() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(logout_steam).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn clear_steam() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(clear_steam_data).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn warranty(steamid: String, state: State<'_, AppData>) -> Result<Option<WarrantyView>, String> {
    let cred = {
        let accounts = state.accounts.lock().unwrap();
        accounts.iter().find(|a| a.steamid == steamid).map(|a| format!("{}----{}", a.username, a.token))
    };
    let Some(cred) = cred else { return Ok(None) };
    let info = tauri::async_runtime::spawn_blocking(move || fetch_warranty(&cred))
        .await
        .map_err(|e| e.to_string())?;
    let Some(info) = info else { return Ok(None) };
    {
        let mut accounts = state.accounts.lock().unwrap();
        if let Some(a) = accounts.iter_mut().find(|a| a.steamid == steamid) {
            a.warranty_expiry = info.warranty_expires;
            a.no_warranty = info.no_warranty;
        }
        save_accounts(&accounts);
    }
    Ok(Some(WarrantyView { expiry: info.warranty_expires, no_warranty: info.no_warranty }))
}

#[tauri::command]
fn view_token(steamid: String, state: State<AppData>) -> Result<String, String> {
    let accounts = state.accounts.lock().unwrap();
    accounts
        .iter()
        .find(|a| a.steamid == steamid)
        .map(|a| format!("{}----{}", a.username, a.token))
        .ok_or_else(|| "Account not found.".into())
}

#[tauri::command]
fn set_api_key(key: String, state: State<AppData>) -> Result<bool, String> {
    let mut s = state.settings.lock().unwrap();
    s.api_key = key.trim().to_string();
    save_settings(&s);
    Ok(!s.api_key.is_empty())
}

#[tauri::command]
async fn public_check(steamid: String, state: State<'_, AppData>) -> Result<Snapshot, String> {
    let (sid_u64, api_key) = {
        let accounts = state.accounts.lock().unwrap();
        let acc = accounts
            .iter()
            .find(|a| a.steamid == steamid)
            .ok_or("Account not found.")?;
        let sid: u64 = acc.steamid.parse().map_err(|_| "Bad SteamID.".to_string())?;
        (sid, state.settings.lock().unwrap().api_key.clone())
    };

    let profile = tauri::async_runtime::spawn_blocking(move || steam_check::fetch_profile(sid_u64))
        .await
        .map_err(|e| e.to_string())?;
    let level = if api_key.is_empty() {
        None
    } else {
        let k = api_key.clone();
        tauri::async_runtime::spawn_blocking(move || steam_check::fetch_level(&k, sid_u64))
            .await
            .ok()
            .flatten()
    };
    let ban = if api_key.is_empty() {
        None
    } else {
        let k = api_key.clone();
        let sid_str = steamid.clone();
        tauri::async_runtime::spawn_blocking(move || steam_check::fetch_bans(&k, &[sid_str]))
            .await
            .ok()
            .and_then(|m| m.into_values().next())
    };

    let got_profile = profile.is_some();
    let mut snaps = state.snapshots.lock().unwrap();
    let snap = snaps.entry(steamid.clone()).or_default();
    snap.steamid = steamid.clone();
    if let Some(p) = profile {
        snap.persona = p.persona;
        snap.avatar = p.avatar;
        snap.profile_state = p.state;
        snap.private = p.private;
        snap.in_game = p.in_game;
    }
    if let Some(l) = level {
        snap.level = Some(l);
    }
    if let Some(b) = ban {
        snap.vac = Some(b.vac);
        snap.vac_count = Some(b.vac_count);
        snap.game_bans = Some(b.game_bans);
    }
    if got_profile {
        snap.public_checked_at = Some(unix_now());
    }
    let out = snap.clone();
    save_snapshots(&snaps);
    Ok(out)
}

#[tauri::command]
async fn deep_check(steamid: String, state: State<'_, AppData>) -> Result<Snapshot, String> {
    let (username, token) = {
        let accounts = state.accounts.lock().unwrap();
        let acc = accounts
            .iter()
            .find(|a| a.steamid == steamid)
            .ok_or("Account not found.")?;
        (acc.username.clone(), acc.token.clone())
    };

    let _permit = state
        .deep_gate
        .acquire()
        .await
        .map_err(|_| "Busy, try again in a moment.".to_string())?;
    let server_list = steam_check::server_list(&state.server_list)
        .await
        .ok_or("Can't check right now. Check your internet and try again.")?;
    let outcome = steam_check::deep_check(server_list, &username, &token).await;

    let mut snaps = state.snapshots.lock().unwrap();
    let snap = snaps.entry(steamid.clone()).or_default();
    snap.steamid = steamid.clone();
    snap.token_state = Some(
        match outcome.liveness {
            steam_check::Liveness::Valid => "valid",
            steam_check::Liveness::Invalid => "invalid",
            steam_check::Liveness::Unknown => "unknown",
        }
        .to_string(),
    );
    if let Some(d) = outcome.data {
        snap.premier_rating = d.premier_rating.or(snap.premier_rating);
        snap.premier_wins = d.premier_wins.or(snap.premier_wins);
        snap.wingman_rank = d.wingman_rank.or(snap.wingman_rank);
        snap.cooldown_expires = d.cooldown_expires;
    }
    snap.deep_checked_at = Some(unix_now());
    let out = snap.clone();
    save_snapshots(&snaps);
    Ok(out)
}

#[tauri::command]
async fn inventory_value(steamid: String, state: State<'_, AppData>) -> Result<f64, String> {
    let sid_u64: u64 = steamid.parse().map_err(|_| "Bad SteamID.".to_string())?;
    let val =
        tauri::async_runtime::spawn_blocking(move || steam_check::fetch_inventory_value(sid_u64))
            .await
            .map_err(|e| e.to_string())?;
    let val = val
        .ok_or("Can't check the inventory right now. It may be private, or try again later.")?;
    let mut snaps = state.snapshots.lock().unwrap();
    let snap = snaps.entry(steamid.clone()).or_default();
    snap.steamid = steamid.clone();
    snap.inv_value = Some(val);
    snap.inv_checked_at = Some(unix_now());
    save_snapshots(&snaps);
    Ok(val)
}

#[tauri::command]
async fn version_info() -> Option<VersionInfo> {
    tauri::async_runtime::spawn_blocking(fetch_version_info).await.ok().flatten()
}

#[tauri::command]
fn open_url(url: String) {
    if !url.starts_with("https://") {
        return;
    }
    let target = HSTRING::from(url);
    unsafe {
        ShellExecuteW(
            HWND::default(),
            w!("open"),
            PCWSTR(target.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
    }
}

fn main() {
    if !is_elevated() && relaunch_as_admin() {
        return;
    }
    tauri::Builder::default()
        .manage(AppData {
            accounts: Mutex::new(load_accounts()),
            settings: Mutex::new(load_settings()),
            snapshots: Mutex::new(load_snapshots()),
            server_list: ServerListCell::new(),
            deep_gate: Semaphore::new(1),
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            add_account,
            remove_account,
            rename_account,
            clear_all,
            active_user,
            restart_admin,
            login,
            logout,
            clear_steam,
            warranty,
            view_token,
            set_api_key,
            public_check,
            deep_check,
            inventory_value,
            version_info,
            open_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn fetch_warranty(license_key: &str) -> Option<WarrantyResponse> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .ok()?;
    let resp = client
        .post(WARRANTY_API_URL)
        .json(&serde_json::json!({ "license_key": license_key }))
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json::<WarrantyResponse>().ok()
}

fn fetch_version_info() -> Option<VersionInfo> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
        .ok()?;
    let resp = client.get(VERSION_MANIFEST_URL).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let manifest = resp.json::<VersionManifest>().ok()?;
    let latest = manifest.version.trim().to_string();
    if latest.is_empty() {
        return None;
    }
    Some(VersionInfo {
        current: APP_VERSION.to_string(),
        update_available: latest != APP_VERSION,
        latest,
        url: manifest.download_url,
    })
}

fn nfa_dir() -> PathBuf {
    let base = std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."));
    base.join("shefu223-nfa")
}
fn accounts_path() -> PathBuf {
    nfa_dir().join("accounts.json")
}
fn settings_path() -> PathBuf {
    nfa_dir().join("settings.json")
}
fn snapshots_path() -> PathBuf {
    nfa_dir().join("snapshots.json")
}
fn load_accounts() -> Vec<Account> {
    let Ok(text) = fs::read_to_string(accounts_path()) else {
        return Vec::new();
    };
    serde_json::from_str(&text).unwrap_or_default()
}
fn save_accounts(accounts: &[Account]) {
    let path = accounts_path();
    if let Some(p) = path.parent() {
        let _ = fs::create_dir_all(p);
    }
    if let Ok(text) = serde_json::to_string_pretty(accounts) {
        let _ = fs::write(&path, text);
    }
}
fn load_settings() -> Settings {
    fs::read_to_string(settings_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}
fn save_settings(settings: &Settings) {
    let path = settings_path();
    if let Some(p) = path.parent() {
        let _ = fs::create_dir_all(p);
    }
    if let Ok(text) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(&path, text);
    }
}
fn load_snapshots() -> HashMap<String, Snapshot> {
    fs::read_to_string(snapshots_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}
fn save_snapshots(snapshots: &HashMap<String, Snapshot>) {
    let path = snapshots_path();
    if let Some(p) = path.parent() {
        let _ = fs::create_dir_all(p);
    }
    if let Ok(text) = serde_json::to_string_pretty(snapshots) {
        let _ = fs::write(&path, text);
    }
}
fn jwt_expiry(token: &str) -> Option<u64> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&payload).ok()?;
    json.get("exp").and_then(|v| v.as_u64())
}
fn build_account(line: &str) -> Result<Account, String> {
    let (username, token) = parse_credential(line)?;
    let steamid = extract_steamid_from_jwt(&token)?;
    Ok(Account { username, token, steamid, added_at: unix_now(), no_warranty: false, warranty_expiry: None })
}

fn login_account(acc: &Account) -> Result<String, String> {
    let steam_path = get_steam_path()?;
    let config_dir = Path::new(&steam_path).join("config");
    check_steam_config_files(&config_dir)?;
    kill_steam_process()?;
    inject_account_into_config(&config_dir.join("config.vdf"), &acc.username, &acc.steamid)?;
    update_loginusers_vdf(&config_dir.join("loginusers.vdf"), &acc.username, &acc.steamid)?;
    write_local_vdf(&acc.username, &acc.token)?;
    write_localconfig_vdf(&steam_path, &acc.steamid)?;
    disable_user_chooser(&config_dir.join("config.vdf"));
    write_autologin_user(&acc.username)?;
    write_remember_password();
    launch_steam(&steam_path);
    Ok(format!("Logged in as '{}'. Steam is starting.", acc.username))
}
fn logout_steam() -> Result<String, String> {
    let steam_path = get_steam_path()?;
    let config_dir = Path::new(&steam_path).join("config");
    kill_steam_process()?;
    clear_autologin_user()?;
    let loginusers = config_dir.join("loginusers.vdf");
    if loginusers.exists() {
        if let Ok(content) = fs::read_to_string(&loginusers) {
            let _ = fs::write(&loginusers, content.replace("\"MostRecent\"\t\t\"1\"", "\"MostRecent\"\t\t\"0\""));
        }
    }
    Ok("Logged out. Steam will ask which account to use.".into())
}
fn clear_steam_data() -> Result<String, String> {
    let steam_path = get_steam_path()?;
    let config_dir = Path::new(&steam_path).join("config");
    let base_path = Path::new(&std::env::var("LOCALAPPDATA").unwrap_or_default()).join("Steam");
    kill_steam_process()?;
    delete_steam_files_and_folder(&config_dir, &base_path)?;
    Ok("Steam data cleared.".into())
}
fn launch_steam(steam_path: &str) {
    let _ = Command::new(Path::new(steam_path).join("steam.exe")).spawn();
}

fn parse_credential(input: &str) -> Result<(String, String), String> {
    let mut parts = input.trim().split("----");
    let username = parts.next().unwrap_or("").trim().to_string();
    let token = parts.next().ok_or("Invalid format, expected username----token.")?.trim().to_string();
    if username.is_empty() || token.is_empty() {
        return Err("Invalid format, expected username----token.".into());
    }
    Ok((username, token))
}
fn extract_steamid_from_jwt(jwt: &str) -> Result<String, String> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return Err("That token doesn't look like a valid login token.".into());
    }
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| "Could not read the login token.".to_string())?;
    let json: serde_json::Value =
        serde_json::from_slice(&payload).map_err(|_| "Could not read the login token.".to_string())?;
    json.get("sub")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or("Token is missing the SteamID.".into())
}
fn get_steam_path() -> Result<String, String> {
    let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey("SOFTWARE\\Valve\\Steam")
        .map_err(|_| "Steam not found. Is it installed?".to_string())?;
    key.get_value("SteamPath").map_err(|_| "Could not read the Steam install path.".to_string())
}
fn check_steam_config_files(config_dir: &Path) -> Result<(), String> {
    if !config_dir.join("config.vdf").exists() || !config_dir.join("loginusers.vdf").exists() {
        return Err("Open Steam and sign into any account once first.".into());
    }
    Ok(())
}
fn inject_account_into_config(path: &Path, username: &str, steamid: &str) -> Result<(), String> {
    let mut content = fs::read_to_string(path).map_err(|e| io_msg("read config.vdf", &e))?;
    if content.contains(&format!("\"SteamID\"\t\t\"{}\"", steamid)) {
        return Ok(());
    }
    let block = format!(
        "\n\t\t\t\t\t\"{}\"\n\t\t\t\t\t{{\n\t\t\t\t\t\t\"SteamID\"\t\t\"{}\"\n\t\t\t\t\t}}\n",
        username, steamid
    );
    let pos = content
        .rfind("\"Accounts\"")
        .and_then(|i| content[i..].find('{').map(|o| i + o + 1))
        .ok_or("Could not find the Accounts block in config.vdf.")?;
    content.insert_str(pos, &block);
    fs::write(path, content).map_err(|e| io_msg("write config.vdf", &e))
}
fn update_loginusers_vdf(path: &Path, username: &str, steamid: &str) -> Result<(), String> {
    let mut content = fs::read_to_string(path).map_err(|e| io_msg("read loginusers.vdf", &e))?;
    content = content.replace("\"MostRecent\"\t\t\"1\"", "\"MostRecent\"\t\t\"0\"");
    content = if content.contains(&format!("\"{}\"", steamid)) {
        update_existing_user(&content, username, steamid)?
    } else {
        insert_new_user(&content, username, steamid)?
    };
    fs::write(path, content).map_err(|e| io_msg("write loginusers.vdf", &e))
}
fn current_timestamp() -> String {
    unix_now().to_string()
}
fn update_existing_user(content: &str, username: &str, steamid: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        result.push_str(line);
        result.push('\n');
        if line.contains(&format!("\"{}\"", steamid)) {
            let mut seen_allow = false;
            let mut seen_remember = false;
            let mut seen_offline = false;
            let mut seen_skip = false;
            let mut seen_most = false;
            let mut seen_ts = false;
            for inner in lines.by_ref() {
                if inner.trim() == "}" {
                    if !seen_most {
                        result.push_str("\t\t\t\"MostRecent\"\t\t\"1\"\n");
                    }
                    if !seen_allow {
                        result.push_str("\t\t\t\"AllowAutoLogin\"\t\t\"1\"\n");
                    }
                    if !seen_remember {
                        result.push_str("\t\t\t\"RememberPassword\"\t\t\"1\"\n");
                    }
                    if !seen_offline {
                        result.push_str("\t\t\t\"WantsOfflineMode\"\t\t\"0\"\n");
                    }
                    if !seen_skip {
                        result.push_str("\t\t\t\"SkipOfflineModeWarning\"\t\t\"0\"\n");
                    }
                    if !seen_ts {
                        result.push_str(&format!(
                            "\t\t\t\"Timestamp\"\t\t\"{}\"\n",
                            current_timestamp()
                        ));
                    }
                    result.push_str(inner);
                    result.push('\n');
                    break;
                }
                if inner.contains("\"AccountName\"") {
                    result.push_str(&format!("\t\t\t\"AccountName\"\t\t\"{}\"\n", username));
                } else if inner.contains("\"PersonaName\"") {
                    result.push_str(&format!("\t\t\t\"PersonaName\"\t\t\"{}\"\n", username));
                } else if inner.contains("\"MostRecent\"") {
                    seen_most = true;
                    result.push_str("\t\t\t\"MostRecent\"\t\t\"1\"\n");
                } else if inner.contains("\"AllowAutoLogin\"") {
                    seen_allow = true;
                    result.push_str("\t\t\t\"AllowAutoLogin\"\t\t\"1\"\n");
                } else if inner.contains("\"RememberPassword\"") {
                    seen_remember = true;
                    result.push_str("\t\t\t\"RememberPassword\"\t\t\"1\"\n");
                } else if inner.contains("\"WantsOfflineMode\"") {
                    seen_offline = true;
                    result.push_str("\t\t\t\"WantsOfflineMode\"\t\t\"0\"\n");
                } else if inner.contains("\"SkipOfflineModeWarning\"") {
                    seen_skip = true;
                    result.push_str("\t\t\t\"SkipOfflineModeWarning\"\t\t\"0\"\n");
                } else if inner.contains("\"Timestamp\"") {
                    seen_ts = true;
                    result.push_str(&format!(
                        "\t\t\t\"Timestamp\"\t\t\"{}\"\n",
                        current_timestamp()
                    ));
                } else {
                    result.push_str(inner);
                    result.push('\n');
                }
            }
        }
    }
    Ok(result)
}

fn disable_user_chooser(config_path: &Path) {
    let Ok(content) = fs::read_to_string(config_path) else {
        return;
    };
    if !content.contains("AlwaysShowUserChooser") {
        return;
    }
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r#"("AlwaysShowUserChooser"\s+)"[^"]*""#).unwrap()
    });
    let new_content = re.replace_all(&content, r#"${1}"0""#);
    if new_content != content {
        let _ = fs::write(config_path, new_content.as_ref());
    }
}
fn insert_new_user(content: &str, username: &str, steamid: &str) -> Result<String, String> {
    let block = format!(
        r#"
	"{steamid}"
	{{
		"AccountName"		"{username}"
		"PersonaName"		"{username}"
		"RememberPassword"		"1"
		"WantsOfflineMode"		"0"
		"SkipOfflineModeWarning"		"0"
		"AllowAutoLogin"		"1"
		"MostRecent"		"1"
		"Timestamp"		"{timestamp}"
	}}
"#,
        steamid = steamid,
        username = username,
        timestamp = current_timestamp()
    );
    let pos = content.rfind('}').ok_or("loginusers.vdf is malformed.")?;
    let mut out = content.to_string();
    out.insert_str(pos, &block);
    Ok(out)
}
fn compute_crc32(data: &str) -> String {
    let v = crc32fast::hash(data.as_bytes());
    let hex = format!("{:08x}", v);
    let trimmed = hex.trim_start_matches('0');
    if trimmed.is_empty() {
        "01".to_string()
    } else {
        format!("{}1", trimmed)
    }
}
use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};
fn steam_encrypt(token: &str, account_name: &str) -> Result<String, String> {
    let data_bytes = token.as_bytes();
    let name_bytes = account_name.as_bytes();
    let data_in = CRYPT_INTEGER_BLOB { cbData: data_bytes.len() as u32, pbData: data_bytes.as_ptr() as *mut u8 };
    let entropy = CRYPT_INTEGER_BLOB { cbData: name_bytes.len() as u32, pbData: name_bytes.as_ptr() as *mut u8 };
    let desc = "BObfuscateBuffer\0";
    let desc_wide: Vec<u16> = desc.encode_utf16().collect();
    let mut data_out = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptProtectData(&data_in, windows::core::PCWSTR(desc_wide.as_ptr()), Some(&entropy), None, None, 0x11, &mut data_out)
            .map_err(|_| "Encryption failed.".to_string())?;
        let slice = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize);
        let hex: String = slice.iter().map(|b| format!("{:02x}", b)).collect();
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn LocalFree(hmem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        }
        LocalFree(data_out.pbData as *mut std::ffi::c_void);
        Ok(hex)
    }
}
fn write_local_vdf(username: &str, token: &str) -> Result<(), String> {
    let crc = compute_crc32(username);
    let encrypted = steam_encrypt(token, username)?;
    let base = Path::new(&std::env::var("LOCALAPPDATA").unwrap_or_default()).join("Steam");
    let path = base.join("local.vdf");
    fs::create_dir_all(&base).map_err(|e| io_msg("create Steam folder", &e))?;
    let content = if path.exists() {
        inject_connect_cache(&fs::read_to_string(&path).map_err(|e| io_msg("read local.vdf", &e))?, &crc, &encrypted)?
    } else {
        create_new_local_vdf(&crc, &encrypted)
    };
    fs::write(path, content).map_err(|e| io_msg("write local.vdf", &e))
}
fn inject_connect_cache(content: &str, crc: &str, encrypted: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut lines = content.lines().peekable();
    let mut in_cc = false;
    let mut depth = 0;
    let mut replaced = false;
    while let Some(line) = lines.next() {
        let t = line.trim();
        if t == "\"ConnectCache\"" {
            in_cc = true;
            depth = 0;
            output.push_str(line);
            output.push('\n');
            continue;
        }
        if in_cc {
            if t.starts_with('{') {
                depth += 1;
            } else if t.starts_with('}') {
                depth -= 1;
                if depth == 0 && !replaced {
                    output.push_str(&format!("\t\t\t\t\t\"{}\"\t\t\"{}\"\n", crc, encrypted));
                    replaced = true;
                }
            }
            if t.starts_with(&format!("\"{}\"", crc)) {
                output.push_str(&format!("\t\t\t\t\t\"{}\"\t\t\"{}\"\n", crc, encrypted));
                replaced = true;
                continue;
            }
        }
        output.push_str(line);
        output.push('\n');
    }
    if !replaced {
        return Ok(create_new_local_vdf(crc, encrypted));
    }
    Ok(output)
}
fn create_new_local_vdf(crc: &str, encrypted: &str) -> String {
    format!(
        r#""MachineUserConfigStore"
{{
	"Software"
	{{
		"Valve"
		{{
			"Steam"
			{{
				"ConnectCache"
				{{
					"{crc}"		"{encrypted}"
				}}
			}}
		}}
	}}
}}
"#,
        crc = crc,
        encrypted = encrypted
    )
}
fn steamid64_to_steamid3(id64: &str) -> Result<String, String> {
    let v: u64 = id64.parse().map_err(|_| "Invalid SteamID64.")?;
    if v < 76561197960265728 {
        return Err("SteamID64 too small.".into());
    }
    Ok((v - 76561197960265728).to_string())
}
fn write_localconfig_vdf(steam_path: &str, steamid64: &str) -> Result<(), String> {
    let sid3 = steamid64_to_steamid3(steamid64)?;
    let content = format!(
        r#""UserLocalConfigStore"
{{
	"friends"
	{{
		"SignIntoFriends" "1"
	}}
	"WebStorage"
	{{
		"FriendStoreLocalPrefs_{sid3}" "{{\"ePersonaState\":7,\"strNonFriendsAllowedToMsg\":\"\"}}"
	}}
}}
"#,
        sid3 = sid3
    );
    let path = Path::new(steam_path).join("userdata").join(&sid3).join("config").join("localconfig.vdf");
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| io_msg("create userdata folder", &e))?;
    fs::write(path, content).map_err(|e| io_msg("write localconfig.vdf", &e))
}
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ,
};
fn read_autologin_user() -> Option<String> {
    let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let key = hkcu.open_subkey("SOFTWARE\\Valve\\Steam").ok()?;
    let val: String = key.get_value("AutoLoginUser").ok()?;
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}
fn write_autologin_user(name: &str) -> Result<(), String> {
    unsafe {
        let subkey: Vec<u16> = "SOFTWARE\\Valve\\Steam\0".encode_utf16().collect();
        let val_name: Vec<u16> = "AutoLoginUser\0".encode_utf16().collect();
        let name_wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let mut hkey = HKEY::default();
        let err = RegOpenKeyExW(HKEY_CURRENT_USER, windows::core::PCWSTR(subkey.as_ptr()), 0, KEY_SET_VALUE, &mut hkey);
        if err.0 != 0 {
            return Err(format!("Registry error: {}", err.0));
        }
        let data = std::slice::from_raw_parts(name_wide.as_ptr() as *const u8, name_wide.len() * 2);
        let err2 = RegSetValueExW(hkey, windows::core::PCWSTR(val_name.as_ptr()), 0, REG_SZ, Some(data));
        let _ = RegCloseKey(hkey);
        if err2.0 != 0 {
            return Err(format!("Registry write error: {}", err2.0));
        }
        Ok(())
    }
}
fn clear_autologin_user() -> Result<(), String> {
    write_autologin_user("")
}
fn write_remember_password() {
    let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    if let Ok(key) =
        hkcu.open_subkey_with_flags("SOFTWARE\\Valve\\Steam", winreg::enums::KEY_SET_VALUE)
    {
        let _ = key.set_value("RememberPassword", &1u32);
    }
}
fn kill_steam_process() -> Result<(), String> {
    for proc in [
        "steam.exe",
        "steamwebhelper.exe",
        "steamservice.exe",
        "steamerrorreporter.exe",
        "streaming_client.exe",
    ] {
        let _ = Command::new("taskkill").args(["/F", "/IM", proc, "/T"]).output();
    }
    for _ in 0..30 {
        if !steam_is_running() {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    std::thread::sleep(Duration::from_millis(400));
    Ok(())
}
fn steam_is_running() -> bool {
    let Ok(out) = Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq steam.exe", "/NH"])
        .output()
    else {
        return false;
    };
    String::from_utf8_lossy(&out.stdout).to_lowercase().contains("steam.exe")
}
fn delete_steam_files_and_folder(config_dir: &Path, steam_base: &Path) -> Result<(), String> {
    for name in &["config.vdf", "loginusers.vdf"] {
        let p = config_dir.join(name);
        if p.exists() {
            fs::remove_file(&p).map_err(|e| io_msg(&format!("delete {}", name), &e))?;
        }
    }
    if steam_base.exists() {
        fs::remove_dir_all(steam_base).map_err(|e| io_msg("delete Steam folder", &e))?;
    }
    Ok(())
}
use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
fn is_elevated() -> bool {
    unsafe { IsUserAnAdmin().as_bool() }
}
fn relaunch_as_admin() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let exe_h = HSTRING::from(exe.as_os_str());
    let result = unsafe {
        ShellExecuteW(HWND::default(), w!("runas"), PCWSTR(exe_h.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL)
    };
    result.0 as isize > 32
}
fn io_msg(action: &str, e: &io::Error) -> String {
    if e.kind() == io::ErrorKind::PermissionDenied || e.raw_os_error() == Some(5) {
        "Permission denied. Please run NFA Loader as Administrator.".into()
    } else {
        format!("Couldn't {} ({}).", action, e.kind())
    }
}
