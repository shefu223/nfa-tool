const TAURI = window.__TAURI__;

const ICON = {
  settings: '<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>',
  minus: '<path d="M5 12h14"/>',
  x: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  plus: '<path d="M5 12h14"/><path d="M12 5v14"/>',
  search: '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>',
  refresh: '<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/>',
  stop: '<rect x="6" y="6" width="12" height="12" rx="2"/>',
  eye: '<path d="M2.06 12.35a1 1 0 0 1 0-.7 10.75 10.75 0 0 1 19.88 0 1 1 0 0 1 0 .7 10.75 10.75 0 0 1-19.88 0"/><circle cx="12" cy="12" r="3"/>',
  more: '<circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>',
  user: '<circle cx="12" cy="8" r="5"/><path d="M20 21a8 8 0 0 0-16 0"/>',
  logout: '<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><path d="m16 17 5-5-5-5"/><path d="M21 12H9"/>',
  eraser: '<path d="m7 21-4.3-4.3c-1-1-1-2.5 0-3.4l9.6-9.6c1-1 2.5-1 3.4 0l5.6 5.6c1 1 1 2.5 0 3.4L13 21"/><path d="M22 21H7"/><path d="m5 11 9 9"/>',
  trash: '<path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><path d="M10 11v6"/><path d="M14 11v6"/>',
  pencil: '<path d="M21.17 6.81a1 1 0 0 0-3.98-3.98L3.84 16.17a2 2 0 0 0-.5.83l-1.32 4.35a.5.5 0 0 0 .62.62l4.35-1.32a2 2 0 0 0 .83-.5z"/><path d="m15 5 4 4"/>',
  check: '<path d="M20 6 9 17l-5-5"/>',
  alert: '<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/>',
  clock: '<circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/>',
  timer: '<path d="M10 2h4"/><path d="M12 14l3-3"/><circle cx="12" cy="14" r="8"/>',
  shield: '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="m9 12 2 2 4-4"/>',
  shieldx: '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="m14.5 9.5-5 5"/><path d="m9.5 9.5 5 5"/>',
  infinity: '<path d="M12 12c-2-2.67-4-4-6-4a4 4 0 1 0 0 8c2 0 4-1.33 6-4Zm0 0c2 2.67 4 4 6 4a4 4 0 0 0 0-8c-2 0-4 1.33-6 4Z"/>',
  copy: '<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>',
  box: '<path d="M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73z"/><path d="M12 22V12"/><path d="m3.3 7 7.7 4.73a2 2 0 0 0 2 0L20.7 7"/>',
  key: '<path d="M2.59 18.59A2 2 0 0 0 2 20v2h4v-2h2v-2h2l1.41-1.41a6.5 6.5 0 1 0-4-4Z"/><circle cx="16.5" cy="7.5" r=".5" fill="currentColor"/>',
  up: '<circle cx="12" cy="12" r="10"/><path d="m16 12-4-4-4 4"/><path d="M12 16V8"/>',
  trophy: '<path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6"/><path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18"/><path d="M4 22h16"/><path d="M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22"/><path d="M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22"/><path d="M18 2H6v7a6 6 0 0 0 12 0V2Z"/>',
};
function svg(name, size = 16, sw = 2) {
  return `<svg class="i" viewBox="0 0 24 24" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="${sw}" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${ICON[name] || ""}</svg>`;
}
function hydrateIcons(root) {
  for (const el of root.querySelectorAll("[data-icon]")) {
    el.innerHTML = svg(el.dataset.icon, Number(el.dataset.size) || 16, Number(el.dataset.stroke) || 2);
  }
}

function mockInvoke(cmd, args) {
  const now = Math.floor(Date.now() / 1000);
  if (!mockInvoke.db) {
    const n = Number(new URLSearchParams(location.search).get("n")) || 6;
    const names = ["shefu_main", "1kazqmii", "supreme_ak", "nova_grind", "faceit_lvl9", "awp_god", "silent_mid", "prime_ready", "eco_king", "clutch_or_kick", "rush_b_now", "smoke_crim", "entry_frag", "lurk_mode", "igl_vibes", "a_very_long_account_name_here"];
    const accounts = [], snaps = {};
    for (let i = 0; i < n; i++) {
      const sid = String(76561198000000000n + BigInt(i * 7919 + 1234567));
      const kind = i % 7;
      accounts.push({
        username: names[i % names.length] + (i >= names.length ? "_" + i : ""), steamid: sid, added_at: now - i * 3600,
        no_warranty: kind === 2, warranty_expiry: kind === 2 ? null : kind === 3 ? now - 200 : now + 3600 + i * 977,
        token_expiry: kind === 4 ? now - 100 : now + 5e6,
      });
      if (kind === 5) continue;
      snaps[sid] = {
        steamid: sid, token_state: kind === 4 ? "invalid" : kind === 6 ? "unknown" : "valid",
        persona: i % 3 === 0 ? null : names[(i + 3) % names.length],
        avatar: i % 4 === 0 ? null : "assets/brand/logo-badge.png",
        profile_state: [0, 1, 6][i % 3], private: i % 9 === 4,
        premier_rating: kind === 4 || kind === 1 ? null : 1200 + ((i * 4337) % 33000),
        wingman_rank: kind === 1 ? null : 1 + (i % 18),
        level: i % 2 ? 10 + i * 3 : null, vac: i % 2 ? i % 11 === 1 : null,
        cooldown_expires: i % 8 === 3 ? now + 7200 : null,
        deep_checked_at: now - 60, public_checked_at: now - 60,
      };
    }
    mockInvoke.db = { accounts, snaps, active: accounts[0] ? accounts[0].username : null, hasKey: false };
  }
  const db = mockInvoke.db;
  const find = (id) => db.accounts.find((a) => a.steamid === id);
  return new Promise((resolve, reject) => {
    setTimeout(() => {
      switch (cmd) {
        case "bootstrap": return resolve({ accounts: db.accounts, snapshots: Object.values(db.snaps), active_user: db.active, elevated: true, has_api_key: db.hasKey });
        case "active_user": return resolve(db.active);
        case "warranty": { const a = find(args.steamid); return resolve(a ? { expiry: a.warranty_expiry, no_warranty: a.no_warranty } : null); }
        case "add_account": {
          const a = { username: args.line.split("----")[0] || "new_acct", steamid: "7656119" + Math.floor(Math.random() * 1e10), added_at: now, no_warranty: false, warranty_expiry: now + 10800, token_expiry: now + 5e6 };
          db.accounts.push(a); return resolve(a);
        }
        case "remove_account": db.accounts = db.accounts.filter((a) => a.steamid !== args.steamid); return resolve();
        case "rename_account": { const a = find(args.steamid); if (a) a.username = args.name; return resolve(); }
        case "clear_all": db.accounts = []; return resolve();
        case "login": { const a = find(args.steamid); db.active = a ? a.username : db.active; return resolve(`Logged in as ${db.active}. Steam is opening.`); }
        case "logout": db.active = null; return resolve("Logged out. Steam will ask which account to use.");
        case "clear_steam": return resolve("Steam data cleared.");
        case "view_token": { const a = find(args.steamid); return resolve(a ? `${a.username}----eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9.eyJpc3MiOiJzdGVhbSIsInN1YiI6Ijc2NTYxMTk4MDAwMDAwMDAwIn0.preview_only_signature` : ""); }
        case "set_api_key": db.hasKey = !!(args.key && args.key.trim()); return resolve(db.hasKey);
        case "public_check": { const s = db.snaps[args.steamid] || { steamid: args.steamid }; s.public_checked_at = now; db.snaps[args.steamid] = s; return resolve(s); }
        case "deep_check": {
          const s = db.snaps[args.steamid] || { steamid: args.steamid };
          s.token_state = Math.random() > 0.15 ? "valid" : "invalid";
          if (s.token_state === "valid") { s.premier_rating = Math.floor(3000 + Math.random() * 30000); s.wingman_rank = 1 + Math.floor(Math.random() * 18); }
          s.deep_checked_at = now; db.snaps[args.steamid] = s; return resolve(s);
        }
        case "inventory_value": return Math.random() > 0.3 ? resolve(Math.round(Math.random() * 40000) / 100) : reject("Can't check the inventory right now. It may be private, or try again later.");
        case "version_info": return resolve({ current: "1", latest: "1", url: "https://github.com/shefu223/nfa-tool/releases/latest", update_available: false });
        default: return resolve();
      }
    }, cmd === "deep_check" ? 700 : cmd === "public_check" ? 250 : cmd === "inventory_value" ? 1100 : 90);
  });
}

const invoke = TAURI ? TAURI.core.invoke : mockInvoke;
const appWindow = TAURI ? TAURI.window.getCurrentWindow() : { minimize() {}, close() {} };
const $ = (id) => document.getElementById(id);

let accounts = [];
let snaps = {};
let activeUser = null;
let elevated = true;
let hasApiKey = false;
let query = "";
const deepStatus = {};
const cards = new Map();
const warrantyTried = new Set();
let run = null;
let publicLeft = 0;
let paused = false;
let firstRender = true;

const PUBLIC_TTL = 30 * 60;
const DEEP_TTL = 6 * 3600;
const DEEP_AUTO_CAP = 10;
const DEEP_FAIL_LIMIT = 3;
const DEEP_GAP_MS = 1200;
const PUBLIC_WORKERS = 2;
const PUBLIC_GAP_MS = 300;

const PREMIER_BOUNDS = [5000, 10000, 15000, 20000, 25000, 30000];
const TIER_COLOR = ["#c3cbd9", "#8fd0ff", "#6f96ff", "#be86ff", "#ee7bf6", "#ff6464", "#ffd257"];
const WINGMAN_NAMES = {
  1: "Silver I", 2: "Silver II", 3: "Silver III", 4: "Silver IV", 5: "Silver Elite", 6: "Silver Elite Master",
  7: "Gold Nova I", 8: "Gold Nova II", 9: "Gold Nova III", 10: "Gold Nova Master",
  11: "Master Guardian I", 12: "Master Guardian II", 13: "Master Guardian Elite", 14: "Distinguished Master Guardian",
  15: "Legendary Eagle", 16: "Legendary Eagle Master", 17: "Supreme Master First Class", 18: "The Global Elite",
};

const nowSec = () => Math.floor(Date.now() / 1000);
const pad2 = (n) => String(n).padStart(2, "0");
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const snapOf = (id) => snaps[id] || {};
const findAcc = (id) => accounts.find((a) => a.steamid === id);
const isActive = (a) => !!activeUser && activeUser.toLowerCase() === a.username.toLowerCase();
const displayName = (a) => snapOf(a.steamid).persona || a.username;
const premierTier = (r) => PREMIER_BOUNDS.reduce((n, b) => n + (r >= b ? 1 : 0), 0);
function esc(s) { return String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c])); }
function fmtCountdown(secs) {
  const d = Math.floor(secs / 86400);
  const h = Math.floor((secs % 86400) / 3600), m = Math.floor((secs % 3600) / 60), s = secs % 60;
  return d > 0 ? `${d}d ${pad2(h)}:${pad2(m)}:${pad2(s)}` : `${pad2(h)}:${pad2(m)}:${pad2(s)}`;
}
function fmtLeft(secs) {
  if (secs < 3600) return `${Math.max(1, Math.round(secs / 60))}m`;
  if (secs < 86400) return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m`;
  return `${Math.floor(secs / 86400)}d ${Math.floor((secs % 86400) / 3600)}h`;
}
function setText(el, text) { if (el.textContent !== text) el.textContent = text; }
function setHTML(el, html) { if (el._html !== html) { el.innerHTML = html; el._html = html; } }

let toastTimer;
function toast(msg, kind = "info") {
  const t = $("toast");
  const ic = kind === "ok" ? "check" : kind === "err" ? "alert" : "clock";
  t.innerHTML = `<span class="ti">${svg(ic, 15)}</span><span>${esc(msg)}</span>`;
  t.className = `toast show ${kind}`;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => { t.className = `toast ${kind}`; }, kind === "err" ? 5200 : 3400);
}

function tokenState(a) {
  const s = snapOf(a.steamid);
  if (a.token_expiry && nowSec() >= a.token_expiry) return "invalid";
  const d = deepStatus[a.steamid];
  if (d === "checking") return "checking";
  if (s.token_state === "valid") return "valid";
  if (s.token_state === "invalid") return "invalid";
  if (d === "pending") return "queued";
  if (s.token_state === "unknown") return "unknown";
  return "unchecked";
}
const TOKEN_LABEL = { valid: "Token valid", invalid: "Token invalid", checking: "Checking", queued: "In queue", unknown: "Can't check", unchecked: "Not checked" };

function premierHTML(r) {
  const tier = premierTier(r);
  const head = r >= 1000 ? Math.floor(r / 1000) : r;
  const tail = r >= 1000 ? "," + String(r % 1000).padStart(3, "0") : "";
  return `<div class="premier" title="Premier rating ${r.toLocaleString()}"><img src="assets/premier/${tier}.png" alt="" /><span class="rating" style="color:${TIER_COLOR[tier]}">${head}<small style="font-size:11px">${tail}</small></span></div>`;
}
function wingmanHTML(w) {
  return `<div class="wingman" title="Wingman: ${esc(WINGMAN_NAMES[w] || "")}"><img src="assets/wingman/${w}.png" alt="" /></div>`;
}
function rankHTML(a) {
  const s = snapOf(a.steamid);
  const st = tokenState(a);
  const hasP = s.premier_rating > 0, hasW = s.wingman_rank >= 1 && s.wingman_rank <= 18;
  if (hasP || hasW) return (hasP ? premierHTML(s.premier_rating) : "") + (hasW ? wingmanHTML(s.wingman_rank) : "");
  if (st === "checking") return `<div class="skel a"></div><div class="skel b"></div>`;
  if (st === "invalid") return `<div class="rank-empty bad">${svg("alert", 14)}Rank unavailable, token is not working</div>`;
  if (st === "queued") return `<div class="rank-empty">${svg("clock", 14)}Waiting to check</div>`;
  if (s.deep_checked_at) return `<div class="rank-empty">${svg("trophy", 14)}No CS2 rank yet</div>`;
  return `<div class="rank-empty">${svg("trophy", 14)}Rank not checked yet</div>`;
}
function chipsHTML(a) {
  const s = snapOf(a.steamid);
  const out = [];
  if (s.vac === true) out.push(`<span class="chip bad" title="This account has a VAC ban">${svg("shieldx", 12)}VAC ban</span>`);
  if (s.cooldown_expires && s.cooldown_expires > nowSec()) out.push(`<span class="chip warn" title="Competitive cooldown is active">${svg("timer", 12)}Cooldown</span>`);
  if (s.premier_rating > 0) out.push(`<span class="chip prime" title="Prime (has a Premier rating)"><img src="assets/prime.png" alt="" />Prime</span>`);
  if (s.vac === false) out.push(`<span class="chip good" title="No VAC or game bans">${svg("shield", 12)}Clean</span>`);
  if (s.level != null) out.push(`<span class="chip lvl" title="Steam level">Lvl ${s.level}</span>`);
  return out.join("");
}
function warrantyView(a) {
  if (a.no_warranty) return { cls: "", icon: "infinity", text: "No warranty", title: "This account has no warranty" };
  if (a.warranty_expiry == null) {
    return warrantyTried.has(a.steamid)
      ? { cls: "", icon: "clock", text: "No warranty info", title: "Warranty could not be loaded" }
      : { cls: "", icon: "clock", text: "Loading warranty", title: "" };
  }
  const left = a.warranty_expiry - nowSec();
  if (left > 0) return { cls: "active", icon: "clock", text: fmtCountdown(left), title: "Warranty time left" };
  return { cls: "expired", icon: "alert", text: "Warranty ended", title: "The warranty for this account has ended" };
}

function createCard(a) {
  const el = $("card-tpl").content.firstElementChild.cloneNode(true);
  el.dataset.steamid = a.steamid;
  const r = {
    img: el.querySelector(".avatar-img"),
    dot: el.querySelector(".state-dot"),
    name: el.querySelector(".card-name"),
    pill: el.querySelector(".active-pill"),
    sid: el.querySelector(".card-sid"),
    rank: el.querySelector(".rank-panel"),
    badge: el.querySelector(".token-badge"),
    blabel: el.querySelector(".tlabel"),
    chips: el.querySelector(".chips"),
    warranty: el.querySelector(".warranty"),
    login: el.querySelector(".btn-login"),
  };
  r.img.addEventListener("load", () => r.img.classList.add("loaded"));
  r.img.addEventListener("error", () => { r.img.classList.remove("loaded"); r.img.removeAttribute("src"); });
  const c = { el, r };
  cards.set(a.steamid, c);
  paintCard(a);
  return c;
}

function paintCard(a) {
  const c = cards.get(a.steamid);
  if (!c) return;
  const { el, r } = c;
  const s = snapOf(a.steamid);
  const active = isActive(a);

  el.classList.toggle("active", active);
  r.pill.classList.toggle("hidden", !active);
  setText(r.name, displayName(a));
  r.name.title = displayName(a) !== a.username ? `${displayName(a)} (${a.username})` : a.username;
  setText(r.sid, a.steamid);

  if (s.avatar) { if (r.img.getAttribute("src") !== s.avatar) { r.img.classList.remove("loaded"); r.img.src = s.avatar; } }
  else if (r.img.hasAttribute("src")) { r.img.classList.remove("loaded"); r.img.removeAttribute("src"); }

  const dotCls = s.private ? "private" : s.profile_state === 6 ? "ingame" : s.profile_state === 1 ? "online" : "";
  r.dot.className = "state-dot" + (dotCls ? " " + dotCls : "");
  r.dot.title = s.private ? "Private profile" : s.profile_state === 6 ? (s.in_game || "In game") : s.profile_state === 1 ? "Online" : "Offline";

  const st = tokenState(a);
  r.badge.className = `token-badge ${st}`;
  setText(r.blabel, TOKEN_LABEL[st]);

  setHTML(r.rank, rankHTML(a));
  setHTML(r.chips, chipsHTML(a));
  paintWarranty(a, c);

  const label = active ? "Re-login" : "Log In";
  setText(r.login, label);
  r.login.className = `btn-login${active ? " relogin" : ""}`;
}

function paintWarranty(a, c = cards.get(a.steamid)) {
  if (!c) return;
  const w = warrantyView(a);
  const el = c.r.warranty;
  const sig = w.cls + "|" + w.icon;
  if (el._sig !== sig) {
    el._sig = sig;
    el.className = "warranty" + (w.cls ? " " + w.cls : "");
    el.innerHTML = `${svg(w.icon, 13)}<span class="wt"></span>`;
  }
  setText(el.lastChild, w.text);
  el.title = w.title;
}

const repaint = (id) => { const a = findAcc(id); if (a) paintCard(a); };

function matches(a) {
  if (!query) return true;
  const q = query.toLowerCase();
  const s = snapOf(a.steamid);
  return a.username.toLowerCase().includes(q) || a.steamid.includes(q) || (s.persona || "").toLowerCase().includes(q);
}

function render() {
  const grid = $("account-grid");
  const visible = accounts.filter(matches);
  const ids = new Set(accounts.map((a) => a.steamid));

  for (const [id, c] of cards) if (!ids.has(id)) { c.el.remove(); cards.delete(id); }

  let i = 0;
  let prev = null;
  for (const a of visible) {
    let c = cards.get(a.steamid);
    if (!c) {
      c = createCard(a);
      if (firstRender && i < 18) { c.el.classList.add("enter"); c.el.style.animationDelay = `${i * 22}ms`; }
    }
    const want = prev ? prev.nextSibling : grid.firstChild;
    if (c.el !== want) grid.insertBefore(c.el, want);
    c.el.classList.remove("hidden");
    prev = c.el;
    i++;
  }
  const shown = new Set(visible.map((a) => a.steamid));
  for (const [id, c] of cards) if (!shown.has(id)) c.el.classList.add("hidden");
  firstRender = false;

  const n = accounts.length;
  setText($("acct-count"), query ? `${visible.length} of ${n}` : `${n} account${n === 1 ? "" : "s"}`);
  $("empty-state").classList.toggle("hidden", n > 0);
  $("no-results").classList.toggle("hidden", !(n > 0 && visible.length === 0));
  setText($("no-results-sub"), `Nothing matches "${query}". Try a name or SteamID.`);
  $("admin-banner").classList.toggle("hidden", elevated);
}

function paintAll() { for (const a of accounts) paintCard(a); }

function tick() {
  for (const a of accounts) {
    if (a.no_warranty || a.warranty_expiry == null) continue;
    const c = cards.get(a.steamid);
    if (c && !c.el.classList.contains("hidden")) paintWarranty(a, c);
  }
}

function setStatus() {
  const el = $("check-status");
  const btn = $("refresh-all");
  const running = !!run;
  btn.classList.toggle("running", running);
  btn.title = running ? "Stop checking" : "Check every account again";
  const ico = btn.querySelector(".tbtn-icon");
  const want = running ? "stop" : "refresh";
  if (ico.dataset.icon !== want) { ico.dataset.icon = want; ico.innerHTML = svg(want, 14); }
  setText(btn.querySelector(".tbtn-label"), running ? "Stop" : "Refresh all");

  el.classList.remove("warn");
  if (running) {
    el.innerHTML = `<span class="spin"></span><span>Checking accounts ${Math.min(run.done + 1, run.total)} of ${run.total}</span>`;
  } else if (paused) {
    el.classList.add("warn");
    el.innerHTML = `${svg("alert", 13)}<span>Checks paused, Steam looks busy. Try Refresh all in a few minutes.</span>`;
  } else if (publicLeft > 0) {
    el.innerHTML = `<span class="spin"></span><span>Loading profiles</span>`;
  } else if (accounts.length) {
    let bad = 0, unchecked = 0;
    for (const a of accounts) {
      const st = tokenState(a);
      if (st === "invalid") bad++;
      else if (st === "unchecked" || st === "unknown") unchecked++;
    }
    const parts = [];
    if (bad) parts.push(`<span class="bad">${bad} not working</span>`);
    if (unchecked) parts.push(`<span>${unchecked} not checked yet</span>`);
    el.innerHTML = parts.length
      ? parts.join(`<span>&middot;</span>`)
      : `<span>${svg("check", 13)}</span><span>All accounts checked</span>`;
    el.title = unchecked ? "Press Refresh all to check the rest" : "";
  } else {
    el.innerHTML = "";
  }
}

const stale = (ts, ttl) => !ts || nowSec() - ts > ttl;

async function publicCheck(id) {
  try {
    const s = await invoke("public_check", { steamid: id });
    if (s) { snaps[id] = { ...snaps[id], ...s }; repaint(id); }
  } catch {}
}

async function deepCheck(id, manual) {
  deepStatus[id] = "checking";
  repaint(id);
  try {
    const s = await invoke("deep_check", { steamid: id });
    snaps[id] = { ...snaps[id], ...s };
    return (s && s.token_state) || "unknown";
  } catch (e) {
    if (manual) toast(String(e), "err");
    return "error";
  } finally {
    delete deepStatus[id];
    repaint(id);
  }
}

async function runPublic(list) {
  publicLeft += list.length; setStatus();
  let i = 0;
  const worker = async () => {
    while (i < list.length) {
      const a = list[i++];
      await publicCheck(a.steamid);
      publicLeft--; setStatus();
      await sleep(PUBLIC_GAP_MS);
    }
  };
  await Promise.all(Array.from({ length: Math.min(PUBLIC_WORKERS, list.length) }, worker));
}

async function runChecks(force) {
  if (run) return;
  paused = false;

  for (const a of accounts) {
    if (a.token_expiry && nowSec() >= a.token_expiry) snaps[a.steamid] = { ...snaps[a.steamid], token_state: "invalid" };
  }

  runPublic(accounts.filter((a) => force || stale(snapOf(a.steamid).public_checked_at, PUBLIC_TTL)));

  let deep = accounts.filter((a) => {
    if (a.token_expiry && nowSec() >= a.token_expiry) return false;
    if (force) return true;
    const s = snapOf(a.steamid);
    if (!s.deep_checked_at || s.token_state === "unknown") return true;
    if (s.token_state === "invalid") return false;
    return stale(s.deep_checked_at, DEEP_TTL);
  });
  if (!force) deep = deep.slice(0, DEEP_AUTO_CAP);
  if (!deep.length) { setStatus(); return; }

  for (const a of deep) deepStatus[a.steamid] = "pending";
  paintAll();
  run = { total: deep.length, done: 0, stop: false };
  setStatus();

  let misses = 0;
  for (const a of deep) {
    if (run.stop || paused) break;
    if (!findAcc(a.steamid)) { run.done++; continue; }
    setStatus();
    const res = await deepCheck(a.steamid, false);
    run.done++;
    if (res === "valid" || res === "invalid") misses = 0;
    else if (++misses >= DEEP_FAIL_LIMIT) paused = true;
    if (run.done < run.total && !run.stop && !paused) await sleep(TAURI ? DEEP_GAP_MS : 150);
  }

  for (const a of deep) if (deepStatus[a.steamid] === "pending") delete deepStatus[a.steamid];
  run = null;
  paintAll();
  setStatus();
}

function onRefreshAll() {
  if (run) { run.stop = true; toast("Stopping after the current account.", "info"); return; }
  runChecks(true);
}

async function doLogin(id) {
  const c = cards.get(id);
  if (c) { c.r.login.classList.add("busy"); setText(c.r.login, "Opening..."); }
  try {
    const msg = await invoke("login", { steamid: id });
    activeUser = await invoke("active_user");
    paintAll();
    toast(msg, "ok");
  } catch (e) {
    toast(String(e), "err");
  } finally {
    if (c) c.r.login.classList.remove("busy");
    repaint(id);
  }
}

async function addAccount() {
  const input = $("add-input");
  const line = input.value.trim();
  if (!line) { toast("Paste an account first, like username----token.", "err"); input.focus(); return; }
  const btn = $("add-btn");
  btn.disabled = true;
  try {
    const a = await invoke("add_account", { line });
    const existing = findAcc(a.steamid);
    if (existing) Object.assign(existing, a); else accounts.push(a);
    input.value = "";
    if (query && !matches(a)) { query = ""; $("search").value = ""; $("search-clear").classList.add("hidden"); }
    render();
    repaint(a.steamid);
    const c = cards.get(a.steamid);
    if (c) {
      c.el.classList.remove("flash"); void c.el.offsetWidth; c.el.classList.add("flash");
      c.el.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
    loadWarranty(a);
    publicCheck(a.steamid);
    deepCheck(a.steamid, false).then(setStatus);
    toast(existing ? `Updated ${a.username}.` : `Added ${a.username}.`, "ok");
  } catch (e) {
    toast(String(e), "err");
  } finally {
    btn.disabled = false;
  }
}

function loadWarranty(a) {
  if (a.no_warranty || a.warranty_expiry != null) return;
  invoke("warranty", { steamid: a.steamid })
    .then((info) => { if (info) { a.warranty_expiry = info.expiry; a.no_warranty = info.no_warranty; } })
    .catch(() => {})
    .finally(() => { warrantyTried.add(a.steamid); paintWarranty(a); });
}

async function doLogout() {
  try { toast(await invoke("logout"), "ok"); activeUser = await invoke("active_user"); paintAll(); }
  catch (e) { toast(String(e), "err"); }
}

function confirmModal({ icon = "alert", neutral = false, title, text, yes, danger = true, onYes }) {
  openModal(`
    <div class="confirm-icon${neutral ? " neutral" : ""}">${svg(icon, 20)}</div>
    <div class="confirm-title">${title}</div>
    <div class="sub center">${text}</div>
    <div class="modal-actions">
      <button class="btn btn-ghost" data-close>Cancel</button>
      <button class="btn ${danger ? "btn-soft-red" : "btn-primary"}" id="cf-yes">${yes}</button>
    </div>`);
  $("cf-yes").onclick = async () => { closeModal(); await onYes(); };
}

function clearSteam() {
  confirmModal({
    icon: "eraser", title: "Clear Steam data?",
    text: "This signs every account out of Steam on this PC and closes Steam. Your saved accounts in the loader are not touched.",
    yes: "Clear Steam",
    onYes: async () => {
      try { toast(await invoke("clear_steam"), "ok"); activeUser = await invoke("active_user"); paintAll(); }
      catch (e) { toast(String(e), "err"); }
    },
  });
}

function clearAll() {
  if (!accounts.length) { toast("There are no accounts to remove.", "info"); return; }
  confirmModal({
    icon: "trash", title: `Remove all ${accounts.length} accounts?`,
    text: "Every account is taken off the list. This can't be undone, so keep a copy of your tokens if you still need them.",
    yes: "Remove all",
    onYes: async () => {
      try {
        await invoke("clear_all");
        accounts = []; snaps = {};
        if (run) run.stop = true;
        render(); setStatus();
        toast("All accounts removed.", "ok");
      } catch (e) { toast(String(e), "err"); }
    },
  });
}

let lastFocus = null;
function openModal(html) {
  const m = $("modal");
  if ($("modal-backdrop").classList.contains("hidden")) lastFocus = document.activeElement;
  m.innerHTML = html;
  hydrateIcons(m);
  for (const b of m.querySelectorAll("[data-close]")) b.onclick = closeModal;
  const back = $("modal-backdrop");
  back.classList.remove("hidden");
  requestAnimationFrame(() => back.classList.add("show"));
  const focusable = m.querySelector("input") || m.querySelector("[data-close]");
  if (focusable) setTimeout(() => focusable.focus({ preventScroll: true }), 30);
}
function closeModal() {
  const b = $("modal-backdrop");
  if (b.classList.contains("hidden")) return;
  b.classList.remove("show");
  setTimeout(() => { b.classList.add("hidden"); $("modal").innerHTML = ""; }, 200);
  if (lastFocus && lastFocus.focus) lastFocus.focus();
}
const modalHead = (title) => `<div class="modal-head"><h2>${title}</h2><button class="modal-close" data-close aria-label="Close">${svg("x", 16)}</button></div>`;

function openDetail(id) {
  const a = findAcc(id);
  if (!a) return;
  const s = snapOf(id);
  const st = tokenState(a);
  const dotCls = s.private ? "private" : s.profile_state === 6 ? "ingame" : s.profile_state === 1 ? "online" : "";
  const cd = s.cooldown_expires && s.cooldown_expires > nowSec();
  const stat = (label, value, cls = "") => `<div class="stat"><div class="sl">${label}</div><div class="sv ${cls}">${value}</div></div>`;
  const needKey = `<span title="Add a Steam API key in Settings">Needs API key</span>`;
  openModal(`
    ${modalHead("Account")}
    <div class="detail-head">
      <div class="avatar">
        ${s.avatar ? `<img class="avatar-img loaded" src="${esc(s.avatar)}" alt="" />` : `<span class="avatar-fallback">${svg("user", 24)}</span>`}
        <span class="state-dot ${dotCls}"></span>
      </div>
      <div style="min-width:0;flex:1">
        <div class="detail-name">${esc(displayName(a))}</div>
        <div class="detail-sid">${a.steamid}${displayName(a) !== a.username ? ` &middot; ${esc(a.username)}` : ""}</div>
        <span class="token-badge ${st}"><i class="tdot"></i><span>${TOKEN_LABEL[st]}</span></span>
      </div>
    </div>
    <div class="rank-panel detail-rank">${rankHTML(a)}</div>
    <div class="stat-grid">
      ${stat("Premier rating", s.premier_rating > 0 ? s.premier_rating.toLocaleString() + (s.premier_wins > 0 ? `<span class="sx">${s.premier_wins} wins</span>` : "") : "None", s.premier_rating > 0 ? "" : "muted")}
      ${stat("Wingman", s.wingman_rank ? esc(WINGMAN_NAMES[s.wingman_rank] || "None") : "None", s.wingman_rank ? "" : "muted")}
      ${stat("Cooldown", cd ? (s.cooldown_expires >= 2e9 ? "Permanent" : `Active<span class="sx">${fmtLeft(s.cooldown_expires - nowSec())} left</span>`) : "None", cd ? "warn" : "muted")}
      ${stat("VAC", s.vac === true ? "Banned" : s.vac === false ? "Clean" : hasApiKey ? "Not checked" : needKey, s.vac === true ? "bad" : s.vac === false ? "good" : "muted")}
      ${stat("Steam level", s.level != null ? s.level : hasApiKey ? "Not checked" : needKey, s.level != null ? "" : "muted")}
      ${stat("Inventory", `<span id="inv-val">${s.inv_value != null ? "$" + s.inv_value.toFixed(2) : "Not checked"}</span>`, s.inv_value != null ? "good" : "muted")}
    </div>
    <div class="detail-actions">
      <button class="btn btn-primary span2" id="d-login">${isActive(a) ? "Re-login" : "Log In"}</button>
      <button class="btn btn-ghost" id="d-recheck">${svg("refresh", 14)}Check again</button>
      <button class="btn btn-ghost" id="d-inv">${svg("box", 14)}<span>Check inventory</span></button>
      <button class="btn btn-ghost" id="d-token">${svg("eye", 14)}View token</button>
      <button class="btn btn-ghost" id="d-rename">${svg("pencil", 14)}Rename</button>
    </div>
    <button class="detail-remove" id="d-remove">Remove this account</button>
  `);
  $("d-login").onclick = () => { closeModal(); doLogin(id); };
  $("d-recheck").onclick = async (e) => {
    const b = e.currentTarget; b.disabled = true; b.innerHTML = `<span class="spin"></span>Checking...`;
    publicCheck(id);
    await deepCheck(id, true);
    setStatus();
    if (!$("modal-backdrop").classList.contains("hidden")) openDetail(id);
  };
  $("d-inv").onclick = async (e) => {
    const b = e.currentTarget; b.disabled = true; b.innerHTML = `<span class="spin"></span>Checking...`;
    try {
      const v = await invoke("inventory_value", { steamid: id });
      snaps[id] = { ...snaps[id], inv_value: v };
      const el = $("inv-val");
      if (el) { el.textContent = "$" + v.toFixed(2); el.parentElement.className = "sv good"; }
    } catch (err) { toast(String(err), "err"); }
    finally { b.disabled = false; b.innerHTML = `${svg("box", 14)}<span>Check inventory</span>`; }
  };
  $("d-token").onclick = () => showToken(id);
  $("d-rename").onclick = () => openRename(a);
  $("d-remove").onclick = () => openRemove(a);
}

async function showToken(id) {
  const a = findAcc(id);
  openModal(`
    ${modalHead("Your token")}
    <div class="sub">The full line you pasted for <b>${esc(a ? a.username : "")}</b>. Keep it private, anyone who has it can log in to the account.</div>
    <div class="token-box" id="token-box">Loading...</div>
    <div class="modal-actions">
      <button class="btn btn-ghost" data-close>Close</button>
      <button class="btn btn-primary" id="tok-copy">${svg("copy", 14)}Copy token</button>
    </div>`);
  let full = "";
  try {
    full = await invoke("view_token", { steamid: id });
    const i = full.indexOf("----");
    $("token-box").innerHTML = i >= 0
      ? `<span class="tu">${esc(full.slice(0, i))}</span><span class="ts">----</span><span class="tt">${esc(full.slice(i + 4))}</span>`
      : esc(full);
  } catch { $("token-box").textContent = "Could not read the token."; }
  $("tok-copy").onclick = async (e) => {
    const b = e.currentTarget;
    try {
      await navigator.clipboard.writeText(full);
      b.innerHTML = `${svg("check", 14)}Copied`;
      setTimeout(() => { if (b.isConnected) b.innerHTML = `${svg("copy", 14)}Copy token`; }, 1600);
    } catch {
      const range = document.createRange(); range.selectNodeContents($("token-box"));
      const sel = getSelection(); sel.removeAllRanges(); sel.addRange(range);
      toast("Selected. Press Ctrl+C to copy.", "info");
    }
  };
}

function openMenu(id) {
  const a = findAcc(id);
  if (!a) return;
  const row = (rid, icon, t, sub, danger) => `<button class="menu-row${danger ? " danger" : ""}" id="${rid}"><span class="ico">${svg(icon, 16)}</span><span><div class="t">${t}</div><div class="s">${sub}</div></span></button>`;
  openModal(`
    ${modalHead(esc(displayName(a)))}
    <div class="menu">
      ${row("m-check", "refresh", "Check again", "Refresh token status and rank")}
      ${row("m-token", "eye", "View token", "Show and copy the full line you pasted")}
      ${row("m-rename", "pencil", "Rename", "Change the name shown on the card")}
      ${row("m-remove", "trash", "Remove", "Take this account off the list", true)}
    </div>`);
  $("m-check").onclick = () => { closeModal(); publicCheck(id); deepCheck(id, true).then(setStatus); };
  $("m-token").onclick = () => showToken(id);
  $("m-rename").onclick = () => openRename(a);
  $("m-remove").onclick = () => openRemove(a);
}

function openRename(a) {
  openModal(`
    ${modalHead("Rename account")}
    <div class="sub">This is the account's login name, so only change it if it was typed wrong. Current name: <b>${esc(a.username)}</b></div>
    <label class="set-field"><input id="rn-input" type="text" spellcheck="false" maxlength="64" /></label>
    <div class="modal-actions">
      <button class="btn btn-ghost" data-close>Cancel</button>
      <button class="btn btn-primary" id="rn-save">Save name</button>
    </div>`);
  const inp = $("rn-input");
  inp.value = a.username;
  setTimeout(() => inp.select(), 40);
  inp.addEventListener("keydown", (e) => { if (e.key === "Enter") $("rn-save").click(); });
  $("rn-save").onclick = async () => {
    const name = inp.value.trim();
    if (!name) { inp.focus(); return; }
    try { await invoke("rename_account", { steamid: a.steamid, name }); a.username = name; repaint(a.steamid); closeModal(); toast("Name saved.", "ok"); }
    catch (e) { toast(String(e), "err"); }
  };
}

function openRemove(a) {
  confirmModal({
    icon: "trash", title: "Remove this account?",
    text: `<b>${esc(a.username)}</b> will be taken off the list. You can add it back any time with the same token.`,
    yes: "Remove",
    onYes: async () => {
      try {
        await invoke("remove_account", { steamid: a.steamid });
        accounts = accounts.filter((x) => x.steamid !== a.steamid);
        delete snaps[a.steamid];
        render(); setStatus();
        toast(`Removed ${a.username}.`, "ok");
      } catch (e) { toast(String(e), "err"); }
    },
  });
}

function openSettings() {
  openModal(`
    ${modalHead("Settings")}
    <div class="modal-label" style="margin-top:0">Steam Web API key (optional)</div>
    <label class="set-field"><input id="api-input" class="mono" type="text" spellcheck="false" autocomplete="off"
      placeholder="${hasApiKey ? "Key saved. Paste a new one to replace it." : "Paste your key here"}" /></label>
    <div class="key-state ${hasApiKey ? "on" : "off"}">${svg(hasApiKey ? "check" : "key", 13)}${hasApiKey ? "A key is saved on this PC" : "No key saved"}</div>
    <div class="set-hint">Only needed to show VAC status and Steam level. Everything else works without it. Get a free key at <a id="api-link">steamcommunity.com/dev/apikey</a>. It never leaves this PC.</div>
    <div class="modal-actions">
      ${hasApiKey ? `<button class="btn btn-ghost" id="set-clear">Remove key</button>` : `<button class="btn btn-ghost" data-close>Close</button>`}
      <button class="btn btn-primary" id="set-save">Save</button>
    </div>`);
  $("api-link").onclick = () => invoke("open_url", { url: "https://steamcommunity.com/dev/apikey" }).catch(() => {});
  const save = async (key) => {
    try {
      hasApiKey = await invoke("set_api_key", { key });
      closeModal();
      toast(hasApiKey ? "API key saved." : "API key removed.", "ok");
      if (hasApiKey) runPublic(accounts.slice());
    } catch (e) { toast(String(e), "err"); }
  };
  $("set-save").onclick = () => {
    const key = $("api-input").value.trim();
    if (!key) { if (hasApiKey) { closeModal(); return; } toast("Paste a key first, or press Close.", "err"); return; }
    save(key);
  };
  const clr = $("set-clear");
  if (clr) clr.onclick = () => save("");
}

async function checkVersion() {
  try {
    const info = await invoke("version_info");
    if (!info || !info.update_available) return;
    confirmModal({
      icon: "up", neutral: true, danger: false, title: "Update available",
      text: `You have v${esc(info.current)}. The newest is v${esc(info.latest)}. Download it from GitHub and replace this file.`,
      yes: "Get the update",
      onYes: () => invoke("open_url", { url: info.url || "https://github.com/shefu223/nfa-tool/releases/latest" }).catch(() => {}),
    });
  } catch {}
}

function bind() {
  $("min-btn").onclick = () => appWindow.minimize();
  $("close-btn").onclick = () => appWindow.close();
  $("settings-btn").onclick = openSettings;
  $("add-btn").onclick = addAccount;
  $("add-input").addEventListener("keydown", (e) => { if (e.key === "Enter") addAccount(); });
  $("refresh-all").onclick = onRefreshAll;
  $("logout-btn").onclick = doLogout;
  $("clear-steam-btn").onclick = clearSteam;
  $("clear-all-btn").onclick = clearAll;
  $("admin-restart").onclick = () => invoke("restart_admin").catch(() => {});

  const grid = $("account-grid");
  grid.addEventListener("click", (e) => {
    const card = e.target.closest(".card");
    if (!card) return;
    const id = card.dataset.steamid;
    const act = e.target.closest("[data-action]");
    if (act) {
      e.stopPropagation();
      if (act.dataset.action === "login") doLogin(id);
      else if (act.dataset.action === "token") showToken(id);
      else if (act.dataset.action === "menu") openMenu(id);
      return;
    }
    openDetail(id);
  });
  grid.addEventListener("keydown", (e) => {
    if (e.key !== "Enter" || !e.target.classList.contains("card")) return;
    openDetail(e.target.dataset.steamid);
  });

  const search = $("search");
  let t;
  search.addEventListener("input", () => {
    $("search-clear").classList.toggle("hidden", !search.value);
    clearTimeout(t);
    t = setTimeout(() => { query = search.value.trim(); render(); }, 90);
  });
  search.addEventListener("keydown", (e) => { if (e.key === "Escape" && search.value) { e.stopPropagation(); $("search-clear").click(); } });
  $("search-clear").onclick = (e) => { e.preventDefault(); search.value = ""; query = ""; $("search-clear").classList.add("hidden"); render(); search.focus(); };

  const wrap = $("grid-wrap");
  wrap.addEventListener("scroll", () => { document.querySelector(".content").classList.toggle("scrolled", wrap.scrollTop > 2); }, { passive: true });

  $("modal-backdrop").addEventListener("mousedown", (e) => { if (e.target.id === "modal-backdrop") closeModal(); });
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") closeModal();
    if ((e.ctrlKey && e.key.toLowerCase() === "f") || (e.key === "/" && document.activeElement.tagName !== "INPUT")) {
      e.preventDefault(); search.focus(); search.select();
    }
  });

  if (TAURI && appWindow.onResized) {
    const sync = async () => { try { document.documentElement.classList.toggle("maximized", await appWindow.isMaximized()); } catch {} };
    appWindow.onResized(sync);
    sync();
  }
}

async function init() {
  hydrateIcons(document);
  hydrateIcons($("card-tpl").content);
  bind();
  try {
    const b = await invoke("bootstrap");
    accounts = b.accounts || [];
    activeUser = b.active_user;
    elevated = b.elevated;
    hasApiKey = b.has_api_key;
    for (const s of b.snapshots || []) snaps[s.steamid] = s;
  } catch {
    toast("Could not load your accounts. Close the loader and open it again.", "err");
  }
  render();
  setStatus();
  for (const a of accounts) loadWarranty(a);
  runChecks(false);
  checkVersion();
  setInterval(tick, 1000);
}

init();
