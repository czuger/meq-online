// Page behaviours shared by every screen: card selection, alerts and Sauron's action markers.
"use strict";

// Cards with the class selectable-card-selection-unique: one card at a time, stored
// in #selected_card; the #validate_unique button is enabled once a card is chosen.
function setupUniqueSelection() {
  const cards = document.querySelectorAll(".selectable-card-selection-unique");
  cards.forEach((card) => {
    card.addEventListener("click", () => {
      cards.forEach((c) => c.classList.remove("selected-card"));
      card.classList.add("selected-card");
      const input = document.getElementById("selected_card");
      if (input) input.value = card.dataset.cardId;
      const button = document.getElementById("validate_unique");
      if (button) button.disabled = false;
    });
  });
}

// Cards with the class selectable-card-selection-multiple: the chosen cards are stored
// comma-separated in #selected_cards; #validate_multiple needs at least one card.
function setupMultipleSelection() {
  const cards = document.querySelectorAll(".selectable-card-selection-multiple");
  cards.forEach((card) => {
    card.addEventListener("click", () => {
      card.classList.toggle("selected-card");
      const selected = Array.from(document.querySelectorAll(".selectable-card-selection-multiple.selected-card"));
      const input = document.getElementById("selected_cards");
      if (input) input.value = selected.map((c) => c.dataset.cardId).join(",");
      const button = document.getElementById("validate_multiple");
      if (button) button.disabled = selected.length === 0;
    });
  });
}

function setupAlerts() {
  document.querySelectorAll("[data-dismiss='alert']").forEach((button) => {
    button.addEventListener("click", () => button.closest(".alert").remove());
  });
}

// Sauron's action markers: at most 3 at once (a 4th one replaces the others).
function setupSauronActions() {
  const area = document.querySelector("[data-sauron-actions]");
  if (!area) return;
  const boxes = area.querySelectorAll(".sauron_action");
  boxes.forEach((box) => {
    box.addEventListener("change", async () => {
      const checked = Array.from(boxes).filter((b) => b.checked);
      if (checked.length >= 4) {
        boxes.forEach((b) => { b.checked = false; });
        box.checked = true;
      }
      const body = new URLSearchParams();
      Array.from(boxes).filter((b) => b.checked).forEach((b) => body.append("actions", b.id));
      const response = await fetch(area.dataset.sauronActions, { method: "PATCH", body });
      if (response.ok) document.getElementById("used_actions").textContent = await response.json();
    });
  });
}

document.addEventListener("DOMContentLoaded", () => {
  setupUniqueSelection();
  setupMultipleSelection();
  setupAlerts();
  setupSauronActions();
});
