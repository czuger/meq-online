// Game table: map fit and zoom, drag to pan, Sauron's influence clicks, and following
// the game by polling the board version.
"use strict";

const MAP_WIDTH = 2934;
const MAP_HEIGHT = 2017;
const ZOOM_STEP = 1.25;
const MAX_SCALE = 1;

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

// Scales the map picture (drawn at its natural size) inside its scrolling viewport.
function setupMapView() {
  const scroll = document.querySelector("[data-map-scroll]");
  const sizer = document.querySelector("[data-map-sizer]");
  const map = document.getElementById("map");
  if (!scroll || !sizer || !map) return;
  const fitScale = () => scroll.clientWidth / MAP_WIDTH;
  let scale = fitScale();

  const apply = (next, focusX, focusY) => {
    const previous = scale;
    scale = Math.min(Math.max(next, fitScale()), MAX_SCALE);
    // Keeps the point under the cursor (or the centre) in place while zooming.
    const fx = focusX ?? scroll.clientWidth / 2;
    const fy = focusY ?? scroll.clientHeight / 2;
    const mapX = (scroll.scrollLeft + fx) / previous;
    const mapY = (scroll.scrollTop + fy) / previous;
    map.style.transform = `scale(${scale})`;
    sizer.style.width = `${MAP_WIDTH * scale}px`;
    sizer.style.height = `${MAP_HEIGHT * scale}px`;
    scroll.scrollLeft = mapX * scale - fx;
    scroll.scrollTop = mapY * scale - fy;
  };
  apply(scale);

  document.querySelectorAll("[data-map-zoom]").forEach((button) => {
    button.addEventListener("click", () => {
      const kind = button.dataset.mapZoom;
      if (kind === "in") apply(scale * ZOOM_STEP);
      else if (kind === "out") apply(scale / ZOOM_STEP);
      else apply(fitScale());
    });
  });
  scroll.addEventListener("wheel", (event) => {
    if (!event.ctrlKey && !event.metaKey && !event.altKey) return;
    event.preventDefault();
    const rect = scroll.getBoundingClientRect();
    const factor = event.deltaY < 0 ? ZOOM_STEP : 1 / ZOOM_STEP;
    apply(scale * factor, event.clientX - rect.left, event.clientY - rect.top);
  }, { passive: false });
  window.addEventListener("resize", () => apply(Math.max(scale, fitScale())));
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
  alert.style.top = (pawn.offsetTop - 80) + "px";
  alert.style.left = (pawn.offsetLeft - 180) + "px";
  alert.style.display = "block";
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
