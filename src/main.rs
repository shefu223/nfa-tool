#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::State;
use winreg::RegKey;

const WARRANTY_API_URL: &str = "https://shefu223.shop/api/warranty";
const CHECK_API_URL: &str = "https://shefu223.shop/token-checker/api/check";
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
}

impl From<&Account> for AccountView {
    fn from(a: &Account) -> Self {
        AccountView {
            username: a.username.clone(),
            steamid: a.steamid.clone(),
            added_at: a.added_at,
            no_warranty: a.no_warranty,
            warranty_expiry: a.warranty_expiry,
        }
    }
}

#[derive(Serialize)]
struct Bootstrap {
    accounts: Vec<AccountView>,
    active_user: Option<String>,
    elevated: bool,
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

#[derive(Deserialize, Serialize, Clone, Default)]
struct CheckResponse {
    prime: Option<bool>,
    vac_clean: Option<bool>,
    cooldown: Option<bool>,
    level: Option<u32>,
    inv_value: Option<String>,
    medals: Option<u32>,
    premier_rating: Option<u32>,
}

struct AppData {
    accounts: Mutex<Vec<Account>>,
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[tauri::command]
fn bootstrap(state: State<AppData>) -> Bootstrap {
    let accounts = state.accounts.lock().unwrap().iter().map(AccountView::from).collect();
    Bootstrap { accounts, active_user: read_autologin_user(), elevated: is_elevated() }
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
async fn check(steamid: String, state: State<'_, AppData>) -> Result<CheckResponse, String> {
    let cred = {
        let accounts = state.accounts.lock().unwrap();
        accounts.iter().find(|a| a.steamid == steamid).map(|a| format!("{}----{}", a.username, a.token))
    }
    .ok_or("Account not found.")?;
    let data = tauri::async_runtime::spawn_blocking(move || fetch_check_data(&cred))
        .await
        .map_err(|e| e.to_string())?;
    data.ok_or_else(|| "Check failed, token may be dead or the service is unavailable.".to_string())
}

#[tauri::command]
async fn version_info() -> Option<VersionInfo> {
    tauri::async_runtime::spawn_blocking(fetch_version_info).await.ok().flatten()
}

#[tauri::command]
fn open_url(url: String) {
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
        .manage(AppData { accounts: Mutex::new(load_accounts()) })
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
            check,
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

fn fetch_check_data(token_line: &str) -> Option<CheckResponse> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(45))
        .build()
        .ok()?;
    let resp = client
        .post(CHECK_API_URL)
        .json(&serde_json::json!({ "token": token_line }))
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json::<CheckResponse>().ok()
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

fn accounts_path() -> PathBuf {
    let base = std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."));
    base.join("shefu223-nfa").join("accounts.json")
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
    write_autologin_user(&acc.username)?;
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
            for inner in lines.by_ref() {
                if inner.contains("\"AccountName\"") {
                    result.push_str(&format!("\t\t\t\"AccountName\"\t\t\"{}\"\n", username));
                } else if inner.contains("\"PersonaName\"") {
                    result.push_str(&format!("\t\t\t\"PersonaName\"\t\t\"{}\"\n", username));
                } else if inner.contains("\"MostRecent\"") {
                    result.push_str("\t\t\t\"MostRecent\"\t\t\"1\"\n");
                } else if inner.contains("\"Timestamp\"") {
                    result.push_str(&format!("\t\t\t\"Timestamp\"\t\t\"{}\"\n", current_timestamp()));
                } else {
                    result.push_str(inner);
                    result.push('\n');
                }
                if inner.trim() == "}" {
                    break;
                }
            }
        }
    }
    Ok(result)
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
fn kill_steam_process() -> Result<(), String> {
    let output = Command::new("taskkill").args(["/F", "/IM", "steam.exe", "/T"]).output();
    let killed = matches!(output, Ok(o) if o.status.success());
    let _ = Command::new("taskkill").args(["/F", "/IM", "steamwebhelper.exe", "/T"]).output();
    if killed {
        std::thread::sleep(Duration::from_millis(1200));
    }
    Ok(())
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
