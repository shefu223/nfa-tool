# NFA Loader

A lightweight Windows loader for switching between Steam accounts using login tokens. It is built for [shefu223.shop](https://shefu223.shop), but it will work with any valid Steam login token.

Paste a token, hit Add, and the loader writes the account straight into Steam's config so you can log in with one click. No passwords, no Steam Guard prompts, just the token.

## Features

- One click login and re-login between saved accounts
- Add accounts in the `username----token` format
- Built in token checker (Prime, VAC status, cooldown, level, inventory value, medals, Premier rating)
- Live warranty countdown for accounts bought from shefu223.shop
- Rename and remove accounts, clear Steam data, or wipe the whole list
- Automatic update notice when a newer version is out
- Runs elevated so it can write to Steam's files reliably

## Requirements

- Windows 10 or 11
- Steam installed, and signed into any account at least once (the loader needs Steam's config files to exist)
- Administrator rights (the loader will ask to elevate on launch)

## Getting started

1. Download the latest `nfa.exe` from the [Releases page](https://github.com/shefu223/nfa-tool/releases/latest).
2. Run it. Approve the Administrator prompt so it can update Steam's files.
3. Paste a line in the `username----token` format and press Add.
4. Press Log In on the account you want. Steam restarts and signs in on its own.

## The token format

Each account is one line:

```
username----token
```

The username is just the label shown in the list. The token is the Steam login (JWT) that actually signs you in. The loader reads the SteamID out of the token, so a bad or expired token gets rejected before it is ever saved.

## How the checker works

Press Check on any account and the loader sends that token to the shefu223.shop checker service. The service talks to Steam, pulls the account's live state, and sends back a summary that the loader shows in a popup:

- Prime status
- VAC status (clean or banned)
- Current cooldown, if any
- Account level
- Inventory value
- Medal count
- Premier rating

The check runs against live data at the moment you press the button, so it reflects the account as it is right now, not a cached snapshot. If a token is dead or the service is unreachable, the loader tells you instead of showing stale numbers.

## Warranty

Warranty is synced directly with shefu223.shop. When you add a token that was bought from the shop, the loader pulls its warranty from the shop's records and shows a live countdown on the card. When the countdown ends, the card marks the account as expired.

Tokens that did not come from shefu223.shop are not in the shop's records, so there is nothing to sync. Those accounts will simply show **No warranty available**. That is expected. The loader still logs you in and still checks the account the same way, you just do not get a warranty timer for tokens the shop did not sell.

## Updates

On launch the loader checks a small manifest hosted in this repo ([`latest.json`](latest.json)) and compares it against its own version. If a newer version is listed, you get a popup with a button that opens the Releases page so you can grab the newest build. Nothing is downloaded or installed automatically, it only points you to the link.

### Publishing a new version (maintainer notes)

1. Bump `APP_VERSION` in `src/main.rs` and the `version` fields in `Cargo.toml` and `tauri.conf.json`.
2. Build with `build.ps1` and attach the new `nfa.exe` to a GitHub Release.
3. Edit `latest.json` on the `main` branch and set `"version"` to the new number.

Everyone still running the old build will see the update popup the next time they open the loader.

## Building from source

You need the Rust toolchain and the Tauri prerequisites for Windows.

```powershell
./build.ps1
```

The script builds a release binary with a static CRT, scrubs local paths out of the executable, and copies the result to `nfa.exe` in the project root.

## Notes

This project is dedicated to shefu223.shop and its customers. It is not affiliated with or endorsed by Valve or Steam. Use it with accounts you own or are allowed to use.
