// Game table: map zoom levels, drag to pan, Sauron's influence clicks, and following
// the game by polling the board version.
"use strict";

const MAP_WIDTH = 4890;
const MAP_HEIGHT = 3362;
// The three zoom levels, from the whole map to the map's natural size.
const ZOOM_LEVELS = ["fit", "half", "full"];
const HALF_SCALE = 0.5;
const FULL_SCALE = 1;
// Ctrl + wheel (or a trackpad pinch) changes one level at most this often.
const WHEEL_COOLDOWN_MS = 300;

// How often the page asks whether the board changed.
const POLL_MS = 2000;

let shiftPressed = false;
// Board version the page shows; changes made by the page itself (influence clicks) take
// the new version without reloading.
let boardVersion = "";
let ownChanges = 0;
// Raised when an own change starts and ends, to drop poll answers that may predate it.
let ownGeneration = 0;
let polling = false;
let dirty = false;
// The page is going away (a form was sent, a link followed): its own navigation wins.
let leaving = false;

// Remembered zoom and position of a board's map: the page reloads whenever the game
// changes. Only a convenience, so a browser refusing storage just starts on the whole map.
function readMapView(key) {
  try {
    return JSON.parse(window.localStorage.getItem(key)) || {};
  } catch (error) {
    return {};
  }
}

function writeMapView(key, view) {
  try {
    window.localStorage.setItem(key, JSON.stringify(view));
  } catch (error) {
    console.debug("map view not remembered", error);
  }
}

// Scales the map picture (drawn at its natural size) inside its scrolling viewport.
function setupMapView() {
  const scroll = document.querySelector("[data-map-scroll]");
  const sizer = document.querySelector("[data-map-sizer]");
  const map = document.getElementById("map");
  if (!scroll || !sizer || !map) return;
  const buttons = document.querySelectorAll("[data-map-zoom]");
  const storageKey = `meq.map.${map.dataset.boardId}`;

  // The whole map, in both directions: the viewport only grows up to its max-height.
  const fitScale = () => {
    const height = parseFloat(getComputedStyle(scroll).maxHeight) || Infinity;
    return Math.min(scroll.clientWidth / MAP_WIDTH, height / MAP_HEIGHT, FULL_SCALE);
  };
  const levelScale = (name) => {
    if (name === "full") return FULL_SCALE;
    return name === "half" ? Math.max(HALF_SCALE, fitScale()) : fitScale();
  };

  let level = "fit";
  let scale = 1;
  // Point of the map under `fx`, `fy` in the viewport.
  const mapPoint = (fx, fy) => [(scroll.scrollLeft + fx) / scale, (scroll.scrollTop + fy) / scale];
  const draw = (nextLevel, [mapX, mapY], fx, fy) => {
    level = nextLevel;
    scale = levelScale(level);
    map.style.transform = `scale(${scale})`;
    map.style.setProperty("--map-unscale", String(1 / scale));
    sizer.style.width = `${MAP_WIDTH * scale}px`;
    sizer.style.height = `${MAP_HEIGHT * scale}px`;
    scroll.scrollLeft = mapX * scale - fx;
    scroll.scrollTop = mapY * scale - fy;
    buttons.forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.mapZoom === level)));
  };
  // Keeps the point under the cursor (or the centre) in place while zooming.
  const zoom = (nextLevel, focusX, focusY) => {
    const fx = focusX ?? scroll.clientWidth / 2;
    const fy = focusY ?? scroll.clientHeight / 2;
    draw(nextLevel, mapPoint(fx, fy), fx, fy);
  };

  const saved = readMapView(storageKey);
  const start = ZOOM_LEVELS.includes(saved.level) ? saved.level : "fit";
  draw(start, [0, 0], 0, 0);
  if (Number.isFinite(saved.x) && Number.isFinite(saved.y)) {
    draw(start, [saved.x, saved.y], scroll.clientWidth / 2, scroll.clientHeight / 2);
  }
  window.addEventListener("pagehide", () => {
    const [x, y] = mapPoint(scroll.clientWidth / 2, scroll.clientHeight / 2);
    writeMapView(storageKey, { level, x, y });
  });

  buttons.forEach((button) => {
    button.addEventListener("click", () => zoom(button.dataset.mapZoom));
  });
  let lastWheel = -Infinity;
  scroll.addEventListener("wheel", (event) => {
    if (!event.ctrlKey && !event.metaKey && !event.altKey) return;
    event.preventDefault();
    if (event.timeStamp - lastWheel < WHEEL_COOLDOWN_MS) return;
    const next = ZOOM_LEVELS[ZOOM_LEVELS.indexOf(level) + (event.deltaY < 0 ? 1 : -1)];
    if (!next) return;
    lastWheel = event.timeStamp;
    const rect = scroll.getBoundingClientRect();
    zoom(next, event.clientX - rect.left, event.clientY - rect.top);
  }, { passive: false });
  window.addEventListener("resize", () => zoom(level));
  setupDrag(scroll);
}

// Dragging the map pans it; a drag does not count as a click on what is under it.
function setupDrag(scroll) {
  let start = null;
  let moved = false;
  scroll.addEventListener("pointerdown", (event) => {
    if (event.button !== 0 || event.target.closest("button")) return;
    start = { x: event.clientX, y: event.clientY, left: scroll.scrollLeft, top: scroll.scrollTop };
    moved = false;
  });
  window.addEventListener("pointermove", (event) => {
    if (!start) return;
    const dx = event.clientX - start.x;
    const dy = event.clientY - start.y;
    if (!moved && Math.hypot(dx, dy) < 5) return;
    moved = true;
    scroll.classList.add("dragging");
    scroll.scrollLeft = start.left - dx;
    scroll.scrollTop = start.top - dy;
  });
  window.addEventListener("pointerup", () => {
    start = null;
    scroll.classList.remove("dragging");
  });
  scroll.addEventListener("click", (event) => {
    if (moved) {
      event.stopPropagation();
      event.preventDefault();
      moved = false;
    }
  }, true);
}

function showInfluenceAlert(pawn, message) {
  const alert = document.getElementById("influence-alert");
  if (alert.style.display === "block") return;
  alert.textContent = message;
  alert.style.display = "block";
  // Anchored by its bottom centre, inside the part of the map on screen.
  const scroll = document.querySelector("[data-map-scroll]");
  const sizer = document.querySelector("[data-map-sizer]");
  const scale = sizer.offsetWidth / MAP_WIDTH;
  const left = scroll.scrollLeft - sizer.offsetLeft;
  const top = scroll.scrollTop - sizer.offsetTop;
  const clamp = (value, min, max) => Math.min(Math.max(value, min), max);
  alert.style.left = clamp(
    pawn.offsetLeft + pawn.offsetWidth / 2,
    (left + alert.offsetWidth / 2) / scale,
    (left + scroll.clientWidth - alert.offsetWidth / 2) / scale,
  ) + "px";
  alert.style.top = clamp(
    pawn.offsetTop,
    (top + alert.offsetHeight) / scale,
    (top + scroll.clientHeight) / scale,
  ) + "px";
  setTimeout(() => { alert.style.display = "none"; }, 2000);
}

// Click places one influence; shift-click takes back one just placed.
function setupInfluence(sauronId) {
  document.querySelectorAll(".influence-pawn-box").forEach((pawn) => {
    pawn.addEventListener("click", async () => {
      const value = pawn.querySelector(".influence-value");
      let val = parseInt(value.textContent, 10);
      if (shiftPressed) { if (val > 0) val -= 1; } else { val += 1; }
      const body = new URLSearchParams({ location: pawn.dataset.location, val: String(val) });
      const data = await ownChange(`/sauron/${sauronId}/sauron_actions/set_influence`, { method: "POST", body });
      if (!data) return;
      if (data.result) value.textContent = data.val;
      else showInfluenceAlert(pawn, data.message);
    });
  });
}

// Clicking an empty slot puts one influence in the shadow pool, a filled one takes it back.
function setupShadowPool(sauronId) {
  document.querySelectorAll(".influence-pool-value").forEach((slot) => {
    slot.addEventListener("click", async () => {
      const body = new URLSearchParams({ current_val: slot.textContent.trim() });
      const data = await ownChange(`/shadow_pools/${sauronId}/update_from_map`, { method: "PATCH", body });
      if (!data) return;
      if (data.message) showInfluenceAlert(slot, data.message);
      document.querySelectorAll(".influence-pool-value").forEach((s, i) => { s.textContent = i < data.pool ? "1" : "0"; });
    });
  });
}

function setupPlotZoom() {
  document.querySelectorAll(".plot-card-on-map").forEach((card) => {
    card.addEventListener("mouseenter", () => card.classList.add("zoomed-plot-card"));
    card.addEventListener("mouseleave", () => card.classList.remove("zoomed-plot-card"));
  });
}

// The player started filling the current screen: a change elsewhere must not wipe it.
function setupDirtyTracking() {
  const panel = document.querySelector(".table-panel");
  if (!panel) return;
  panel.addEventListener("change", () => { dirty = true; });
  panel.addEventListener("click", (event) => {
    if (event.target.closest(".selectable-card-selection-unique, .selectable-card-selection-multiple")) dirty = true;
  });
}

function showChangedBanner() {
  if (document.querySelector(".changed-banner")) return;
  const banner = document.createElement("div");
  banner.className = "changed-banner";
  banner.setAttribute("role", "status");
  banner.textContent = "La partie a changé. ";
  const button = document.createElement("button");
  button.type = "button";
  button.className = "btn btn-primary";
  button.textContent = "Actualiser";
  button.addEventListener("click", () => window.location.reload());
  banner.appendChild(button);
  document.body.appendChild(banner);
}

// Sends a change made by the page itself, then takes the board version it made, so the
// next poll doesn't reload the page. Returns the JSON answer, or null on failure.
async function ownChange(url, options) {
  ownChanges += 1;
  ownGeneration += 1;
  try {
    const response = await fetch(url, options);
    if (!response.ok) return null;
    return await response.json();
  } catch (error) {
    return null;
  } finally {
    ownChanges -= 1;
    ownGeneration += 1;
    if (ownChanges === 0) pollBoard(true);
  }
}

// Asks whether the board changed; when it did, goes to the screen the game now expects
// from this user, or shows the new state of the current one. `adopt` only takes the new
// version (after a change made by this page).
async function pollBoard(adopt = false) {
  if (leaving || (!adopt && (polling || document.hidden || ownChanges > 0))) return;
  const boardId = document.getElementById("map").dataset.boardId;
  const generation = ownGeneration;
  if (!adopt) polling = true;
  let status;
  try {
    const version = encodeURIComponent(boardVersion);
    const response = await fetch(`/boards/${boardId}/status?version=${version}`, { headers: { Accept: "application/json" } });
    if (!response.ok) return;
    status = await response.json();
  } catch (error) {
    return;
  } finally {
    if (!adopt) polling = false;
  }
  // An own change started or ended meanwhile: this answer may predate it.
  if (leaving || generation !== ownGeneration || status.version === boardVersion) return;
  boardVersion = status.version;
  if (adopt) return;
  if (status.url && status.url !== window.location.pathname) {
    window.location.assign(status.url);
  } else if (dirty) {
    showChangedBanner();
  } else {
    window.location.reload();
  }
}

function setupRefresh(version) {
  boardVersion = version;
  window.addEventListener("beforeunload", () => { leaving = true; });
  window.addEventListener("pageshow", () => { leaving = false; pollBoard(); });
  document.addEventListener("visibilitychange", () => { if (!document.hidden) pollBoard(); });
  const loop = async () => {
    await pollBoard();
    setTimeout(loop, POLL_MS);
  };
  setTimeout(loop, POLL_MS);
}

document.addEventListener("DOMContentLoaded", () => {
  const map = document.getElementById("map");
  if (!map) return;
  window.addEventListener("keydown", (e) => { shiftPressed = e.shiftKey; });
  window.addEventListener("keyup", (e) => { shiftPressed = e.shiftKey; });
  setupMapView();
  setupPlotZoom();
  setupDirtyTracking();
  setupRefresh(map.dataset.boardVersion);
  if (map.dataset.sauronId) {
    setupInfluence(map.dataset.sauronId);
    setupShadowPool(map.dataset.sauronId);
  }
});
