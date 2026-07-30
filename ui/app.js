const TAURI = window.__TAURI__;

function mockInvoke(cmd, args) {
  const now = Math.floor(Date.now() / 1000);
  const db = mockInvoke.db || (mockInvoke.db = {
    accounts: [
      { username: "shefu_main", steamid: "76561198012345678", added_at: now - 7200, no_warranty: false, warranty_expiry: now + 9294 },
      { username: "1kazqmii", steamid: "76561199572012286", added_at: now - 18000, no_warranty: false, warranty_expiry: now - 200 },
      { username: "shefu_perm", steamid: "76561198011112222", added_at: now - 36000, no_warranty: true, warranty_expiry: null },
    ],
    active: "shefu_main",
  });
  const find = (id) => db.accounts.find((a) => a.steamid === id);
  return new Promise((resolve, reject) => {
    setTimeout(() => {
      switch (cmd) {
        case "bootstrap": return resolve({ accounts: db.accounts, active_user: db.active, elevated: true });
        case "active_user": return resolve(db.active);
        case "warranty": { const a = find(args.steamid); return resolve(a ? { expiry: a.warranty_expiry, no_warranty: a.no_warranty } : null); }
        case "add_account": {
          const a = { username: args.line.split("----")[0] || "new_acct", steamid: "7656119" + Math.floor(Math.random()*1e10), added_at: now, no_warranty: false, warranty_expiry: now + 10800 };
          db.accounts.push(a); return resolve(a);
        }
        case "remove_account": db.accounts = db.accounts.filter((a) => a.steamid !== args.steamid); return resolve();
        case "rename_account": { const a = find(args.steamid); if (a) a.username = args.name; return resolve(); }
        case "clear_all": db.accounts = []; return resolve();
        case "login": { const a = find(args.steamid); db.active = a ? a.username : db.active; return resolve(`Logged in as '${db.active}'. Steam is starting.`); }
        case "logout": db.active = null; return resolve("Logged out.");
        case "clear_steam": return resolve("Steam data cleared.");
        case "check": return Math.random() > 0.2
          ? resolve({ prime: true, vac_clean: true, cooldown: false, level: 21, inv_value: "$184.50", medals: 3, premier_rating: 18432 })
          : reject("Check failed, token may be dead.");
        case "version_info": return resolve({ current: "1", latest: "1", url: "https://github.com/shefu223/nfa-tool/releases/latest", update_available: false });
        case "open_url": return resolve();
        default: return resolve();
      }
    }, cmd === "check" ? 1400 : cmd === "warranty" ? 500 : 150);
  });
}

const invoke = TAURI ? TAURI.core.invoke : mockInvoke;
const appWindow = TAURI ? TAURI.window.getCurrentWindow() : { minimize() {}, close() {} };

const $ = (id) => document.getElementById(id);
let accounts = [];
let activeUser = null;
let elevated = true;

const fmt2 = (n) => String(n).padStart(2, "0");
function fmtDate(secs) {
  const d = new Date(secs * 1000);
  return `${fmt2(d.getDate())}.${fmt2(d.getMonth() + 1)}.${d.getFullYear()}  ${fmt2(d.getHours())}:${fmt2(d.getMinutes())}:${fmt2(d.getSeconds())}`;
}
function fmtCountdown(secs) {
  return `${fmt2(Math.floor(secs / 3600))}:${fmt2(Math.floor((secs % 3600) / 60))}:${fmt2(secs % 60)}`;
}
const isActive = (a) => activeUser && activeUser.toLowerCase() === a.username.toLowerCase();

let toastTimer;
function toast(msg, kind) {
  const t = $("toast");
  t.textContent = msg;
  t.className = `toast show ${kind}`;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => { t.className = "toast"; }, 3600);
}

function warrantyState(a) {
  const now = Math.floor(Date.now() / 1000);
  if (a.no_warranty) return { kind: "no", accent: "rgba(255,255,255,0.14)" };
  if (a.warranty_expiry == null) return { kind: "checking", accent: "var(--dim)" };
  const remaining = a.warranty_expiry - now;
  if (remaining > 0) return { kind: "active", accent: "var(--green)", remaining };
  return { kind: "expired", accent: "var(--red)" };
}

function warrantyHTML(a) {
  const s = warrantyState(a);
  if (s.kind === "no") return `<span class="badge badge-dim">&#x221E; No warranty</span>`;
  if (s.kind === "checking") return `<span class="badge badge-dim">&#x2026; checking</span>`;
  if (s.kind === "active") return `<span class="badge badge-green"><span class="dot"></span>${fmtCountdown(s.remaining)}</span>`;
  return `<span class="badge badge-red">&#x26A0; Warranty expired</span>`;
}

function render() {
  $("acct-count").textContent = `${accounts.length} account${accounts.length === 1 ? "" : "s"}`;
  $("admin-banner").classList.toggle("hidden", elevated);
  const list = $("account-list");
  const empty = $("empty-state");
  empty.classList.toggle("hidden", accounts.length > 0);
  list.innerHTML = "";
  for (const a of accounts) list.appendChild(buildCard(a));
}

function buildCard(a) {
  const tpl = $("card-tpl").content.firstElementChild.cloneNode(true);
  tpl.dataset.steamid = a.steamid;
  tpl.querySelector(".card-name").textContent = a.username;
  tpl.querySelector(".card-steamid").textContent = a.steamid;
  tpl.querySelector(".added-date").textContent = fmtDate(a.added_at);

  const login = tpl.querySelector(".btn-login");
  const check = tpl.querySelector(".btn-check");
  login.addEventListener("click", (e) => { e.stopPropagation(); doLogin(a.steamid); });
  check.addEventListener("click", (e) => { e.stopPropagation(); doCheck(a.steamid); });

  let pressTimer;
  const openMenu = () => openSheet(a.steamid);
  tpl.addEventListener("contextmenu", (e) => { e.preventDefault(); openMenu(); });
  tpl.addEventListener("mousedown", (e) => {
    if (e.button !== 0) return;
    pressTimer = setTimeout(openMenu, 450);
  });
  const cancel = () => clearTimeout(pressTimer);
  tpl.addEventListener("mouseup", cancel);
  tpl.addEventListener("mouseleave", cancel);

  paintCard(tpl, a);
  return tpl;
}

function paintCard(card, a) {
  const active = isActive(a);
  card.classList.toggle("active", active);
  card.querySelector(".active-pill").classList.toggle("hidden", !active);
  const login = card.querySelector(".btn-login");
  login.textContent = active ? "Re-login" : "Log In";
  login.className = `btn btn-login ${active ? "relogin" : "login"}`;
  const s = warrantyState(a);
  card.querySelector(".accent").style.background = active ? "var(--red)" : s.accent;
  card.querySelector(".warranty").innerHTML = warrantyHTML(a);
}

function tick() {
  for (const card of document.querySelectorAll(".card")) {
    const a = accounts.find((x) => x.steamid === card.dataset.steamid);
    if (a) paintCard(card, a);
  }
}

async function loadWarranties() {
  for (const a of accounts) {
    if (a.no_warranty || a.warranty_expiry != null) continue;
    invoke("warranty", { steamid: a.steamid }).then((info) => {
      if (!info) return;
      a.warranty_expiry = info.expiry;
      a.no_warranty = info.no_warranty;
      tick();
    }).catch(() => {});
  }
}

async function refreshActive() {
  try { activeUser = await invoke("active_user"); tick(); } catch {}
}

async function doLogin(steamid) {
  try {
    const msg = await invoke("login", { steamid });
    await refreshActive();
    render();
    toast(msg, "ok");
  } catch (e) { toast(String(e), "err"); }
}

async function addAccount() {
  const input = $("add-input");
  const line = input.value.trim();
  if (!line) { toast("Paste a line first (username----token).", "err"); return; }
  try {
    const a = await invoke("add_account", { line });
    const existing = accounts.find((x) => x.steamid === a.steamid);
    if (existing) Object.assign(existing, a); else accounts.push(a);
    input.value = "";
    render();
    loadWarranties();
    toast(`${existing ? "Updated" : "Added"} '${a.username}'.`, "ok");
  } catch (e) { toast(String(e), "err"); }
}

let confirmSteam = false, confirmAll = false, cSteamT, cAllT;
function resetConfirms() {
  confirmSteam = confirmAll = false;
  $("clear-steam-btn").textContent = "Clear Steam";
  $("clear-steam-btn").className = "btn btn-ghost";
  $("clear-all-btn").textContent = "Clear All Accounts";
  $("clear-all-btn").className = "btn btn-ghost btn-wide";
}

async function clearSteam() {
  if (!confirmSteam) {
    confirmSteam = true;
    const b = $("clear-steam-btn");
    b.textContent = "Tap again";
    b.className = "btn btn-danger";
    clearTimeout(cSteamT); cSteamT = setTimeout(resetConfirms, 2600);
    return;
  }
  resetConfirms();
  try { toast(await invoke("clear_steam"), "ok"); await refreshActive(); render(); }
  catch (e) { toast(String(e), "err"); }
}

async function clearAll() {
  if (!confirmAll) {
    confirmAll = true;
    const b = $("clear-all-btn");
    b.textContent = "Tap again to remove all accounts";
    b.className = "btn btn-danger btn-wide";
    clearTimeout(cAllT); cAllT = setTimeout(resetConfirms, 2600);
    return;
  }
  resetConfirms();
  try { await invoke("clear_all"); accounts = []; render(); toast("All accounts removed.", "ok"); }
  catch (e) { toast(String(e), "err"); }
}

async function doLogout() {
  try { toast(await invoke("logout"), "ok"); await refreshActive(); render(); }
  catch (e) { toast(String(e), "err"); }
}

function openSheet(steamid) {
  const a = accounts.find((x) => x.steamid === steamid);
  if (!a) return;
  sheetMenu(a);
}

function showSheet(html) {
  const sheet = $("sheet");
  sheet.innerHTML = `<div class="sheet-handle"></div>${html}`;
  $("sheet-backdrop").classList.remove("hidden");
  requestAnimationFrame(() => $("sheet-backdrop").classList.add("show"));
}
function closeSheet() {
  const b = $("sheet-backdrop");
  b.classList.remove("show");
  setTimeout(() => b.classList.add("hidden"), 300);
}

function sheetMenu(a) {
  showSheet(`
    <div class="sheet-head">
      <div class="avatar"><svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="8" r="4"/><path d="M4 21c0-4 4-6 8-6s8 2 8 6"/></svg></div>
      <div><div class="nm">${esc(a.username)}</div><div class="sid">${a.steamid}</div></div>
    </div>
    <div class="sheet-sep"></div>
    <div class="sheet-row" id="row-rename">
      <div class="ico" style="background:rgba(228,56,47,0.14);color:var(--red-soft)">&#x270F;</div>
      <div class="rt"><div class="t">Rename Account</div><div class="s">Change the display name</div></div>
      <div class="chev">&#x203A;</div>
    </div>
    <div class="sheet-row danger" id="row-remove">
      <div class="ico" style="background:rgba(237,66,69,0.14);color:var(--err)">&#x1F5D1;</div>
      <div class="rt"><div class="t">Remove Account</div><div class="s">Delete from the list</div></div>
      <div class="chev">&#x203A;</div>
    </div>
    <div style="height:12px"></div>
    <div class="sheet-actions"><button class="btn btn-ghost" id="row-cancel">Cancel</button></div>
  `);
  $("row-rename").onclick = () => sheetRename(a);
  $("row-remove").onclick = () => sheetRemove(a);
  $("row-cancel").onclick = closeSheet;
}

function sheetRename(a) {
  showSheet(`
    <h2>Rename Account</h2>
    <div class="sub">Current: <b>${esc(a.username)}</b></div>
    <div class="sheet-field"><input id="rename-input" type="text" spellcheck="false" /></div>
    <div class="sheet-actions">
      <button class="btn btn-ghost" id="rn-back" style="flex:0.5">Back</button>
      <button class="btn btn-primary" id="rn-save">Save Name</button>
    </div>
  `);
  const inp = $("rename-input");
  inp.value = a.username;
  inp.focus(); inp.select();
  inp.addEventListener("keydown", (e) => { if (e.key === "Enter") $("rn-save").click(); });
  $("rn-back").onclick = () => sheetMenu(a);
  $("rn-save").onclick = async () => {
    const name = inp.value.trim();
    if (!name) return;
    try {
      await invoke("rename_account", { steamid: a.steamid, name });
      a.username = name;
      render(); closeSheet(); toast("Account renamed.", "ok");
    } catch (e) { toast(String(e), "err"); }
  };
}

function sheetRemove(a) {
  showSheet(`
    <div class="center-ico">&#x1F5D1;</div>
    <h2 style="text-align:center">Remove Account?</h2>
    <div class="sub" style="text-align:center">'<b>${esc(a.username)}</b>' will be removed from the list.</div>
    <div class="sheet-actions">
      <button class="btn btn-ghost" id="rm-cancel">Cancel</button>
      <button class="btn btn-danger" id="rm-confirm">Remove</button>
    </div>
  `);
  $("rm-cancel").onclick = () => sheetMenu(a);
  $("rm-confirm").onclick = async () => {
    try {
      await invoke("remove_account", { steamid: a.steamid });
      const card = document.querySelector(`.card[data-steamid="${a.steamid}"]`);
      accounts = accounts.filter((x) => x.steamid !== a.steamid);
      closeSheet();
      if (card) { card.classList.add("removing"); setTimeout(render, 250); } else render();
      toast(`Removed '${a.username}'.`, "ok");
    } catch (e) { toast(String(e), "err"); }
  };
}

function showCheck(html) {
  $("check-popup").innerHTML = `
    <div class="popup-head"><h2>Account Info</h2><button class="popup-close" id="check-close">&#x2715;</button></div>${html}`;
  $("check-backdrop").classList.remove("hidden");
  requestAnimationFrame(() => $("check-backdrop").classList.add("show"));
  $("check-close").onclick = closeCheck;
}
function closeCheck() {
  const b = $("check-backdrop");
  b.classList.remove("show");
  setTimeout(() => b.classList.add("hidden"), 250);
}

async function doCheck(steamid) {
  showCheck(`<div class="check-state"><div class="spinner"></div><div class="t" style="color:var(--red-soft)">Checking account&#x2026;</div><div class="s">Fetching live data from Steam</div></div>`);
  try {
    const d = await invoke("check", { steamid });
    renderCheck(d);
  } catch (e) {
    showCheck(`<div class="check-state"><div class="t" style="color:var(--err)">Check failed</div><div class="s">${esc(String(e))}</div></div>`);
  }
}

function renderCheck(d) {
  const on = (c, v) => v ? `style="color:${c};background:${c}1f;border-color:${c}4d"` : "";
  const prime = d.prime === true, vac = d.vac_clean === true, cd = d.cooldown === true;
  let badges = `<span class="cbadge" style="color:var(--green);background:rgba(70,196,106,0.1);border-color:rgba(70,196,106,0.28)">&#x2713; DONE</span>`;
  badges += `<span class="cbadge" ${on("var(--red-soft)", prime)}>&#x2605; PRIME</span>`;
  badges += `<span class="cbadge" ${on("var(--green)", vac)}>&#x2713; VAC CLEAN</span>`;
  if (d.vac_clean === false) badges += `<span class="cbadge" style="color:var(--err);background:rgba(240,69,61,0.12);border-color:rgba(240,69,61,0.32)">&#x2717; VAC BAN</span>`;
  if (cd) badges += `<span class="cbadge" style="color:var(--red-soft);background:rgba(228,56,47,0.1);border-color:var(--border-red)">&#x23F1; COOLDOWN</span>`;
  if (d.level != null) badges += `<span class="cbadge" style="color:var(--dim)">LV.${d.level}</span>`;

  const stat = (l, v, c) => `<div class="stat"><div class="sl">${l}</div><div class="sv" style="color:${c}">${v}</div></div>`;
  const medals = d.medals ? d.medals : "None";
  const premier = d.premier_rating ? d.premier_rating.toLocaleString() : "No Rank";
  const stats = `<div class="stats">
    ${stat("&#x1F3C5; Medals", medals, d.medals ? "var(--red-soft)" : "var(--dim)")}
    ${stat("&#x1F4E6; Inventory", d.inv_value || "N/A", "var(--green)")}
    ${stat("&#x2B50; Prime", prime ? "Active" : "No", prime ? "var(--red-soft)" : "var(--dim)")}
    ${stat("&#x1F6E1; VAC", vac ? "Clean" : "Banned", vac ? "var(--green)" : "var(--err)")}
    ${stat("&#x1F3AE; Level", d.level != null ? "Level " + d.level : "N/A", "var(--text)")}
    ${stat("&#x1F3C6; Premier", premier, d.premier_rating ? "var(--red-soft)" : "var(--faint)")}
  </div>`;
  showCheck(`<div class="badges">${badges}</div><div class="popup-sep"></div>${stats}`);
}

function showUpdate(info) {
  const link = info.url || "https://github.com/shefu223";
  $("update-popup").innerHTML = `
    <div class="center-ico">&#x2B06;</div>
    <h2 style="text-align:center">Update available</h2>
    <div class="sub" style="text-align:center">You're on <b>v${esc(info.current)}</b>. The latest release is <b>v${esc(info.latest)}</b>.<br/>Grab the newest build from GitHub.</div>
    <div class="sheet-actions">
      <button class="btn btn-ghost" id="upd-later" style="flex:0.6">Later</button>
      <button class="btn btn-primary" id="upd-get">Get latest version</button>
    </div>`;
  $("update-backdrop").classList.remove("hidden");
  requestAnimationFrame(() => $("update-backdrop").classList.add("show"));
  $("upd-later").onclick = closeUpdate;
  $("upd-get").onclick = () => { invoke("open_url", { url: link }).catch(() => {}); closeUpdate(); };
}
function closeUpdate() {
  const b = $("update-backdrop");
  b.classList.remove("show");
  setTimeout(() => b.classList.add("hidden"), 250);
}
async function checkVersion() {
  try {
    const info = await invoke("version_info");
    if (info && info.update_available) showUpdate(info);
  } catch {}
}

function esc(s) {
  return String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
}

function bind() {
  $("min-btn").onclick = () => appWindow.minimize();
  $("close-btn").onclick = () => appWindow.close();
  $("add-btn").onclick = addAccount;
  $("add-input").addEventListener("keydown", (e) => { if (e.key === "Enter") addAccount(); });
  $("logout-btn").onclick = doLogout;
  $("clear-steam-btn").onclick = clearSteam;
  $("clear-all-btn").onclick = clearAll;
  $("admin-restart").onclick = () => invoke("restart_admin").catch(() => {});
  $("sheet-backdrop").addEventListener("click", (e) => { if (e.target.id === "sheet-backdrop") closeSheet(); });
  $("check-backdrop").addEventListener("click", (e) => { if (e.target.id === "check-backdrop") closeCheck(); });
  $("update-backdrop").addEventListener("click", (e) => { if (e.target.id === "update-backdrop") closeUpdate(); });
  document.addEventListener("keydown", (e) => { if (e.key === "Escape") { closeSheet(); closeCheck(); closeUpdate(); } });
}

async function init() {
  bind();
  try {
    const b = await invoke("bootstrap");
    accounts = b.accounts || [];
    activeUser = b.active_user;
    elevated = b.elevated;
  } catch (e) { toast("Failed to load: " + e, "err"); }
  render();
  loadWarranties();
  checkVersion();
  setInterval(tick, 1000);
}

init();
