// Map page: Sauron sets influence and the shadow pool; every viewer reloads when the board changes.
"use strict";

let shiftPressed = false;

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
      const response = await fetch(`/sauron/${sauronId}/sauron_actions/set_influence`, { method: "POST", body });
      if (!response.ok) return;
      const data = await response.json();
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
      const response = await fetch(`/shadow_pools/${sauronId}/update_from_map`, { method: "PATCH", body });
      if (!response.ok) return;
      const data = await response.json();
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

function setupRefresh(boardId) {
  const events = new EventSource(`/boards/${boardId}/events`);
  events.addEventListener("refresh", () => window.location.reload());
}

document.addEventListener("DOMContentLoaded", () => {
  const map = document.getElementById("map");
  if (!map) return;
  window.addEventListener("keydown", (e) => { shiftPressed = e.shiftKey; });
  window.addEventListener("keyup", (e) => { shiftPressed = e.shiftKey; });
  setupPlotZoom();
  setupRefresh(map.dataset.boardId);
  if (map.dataset.sauronId) {
    setupInfluence(map.dataset.sauronId);
    setupShadowPool(map.dataset.sauronId);
  }
});
