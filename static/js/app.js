// Page behaviours shared by every screen: card selection, agility preview and alerts.
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
      if (button) {
        button.disabled = false;
        button.removeAttribute("title");
      }
      const hint = document.querySelector("[data-selection-hint]");
      if (hint) hint.textContent = "Carte choisie.";
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
      const count = document.querySelector("[data-selection-count]");
      if (count) count.textContent = selectionText(selected.length);
    });
  });
}

function selectionText(count) {
  if (count === 0) return "Aucune carte choisie.";
  if (count === 1) return "1 carte choisie.";
  return `${count} cartes choisies.`;
}

// Combat setup: shows what the agility split gives.
function setupAgilityPreview() {
  const input = document.querySelector("[data-agility]");
  const result = document.querySelector("[data-agility-result]");
  if (!input || !result) return;
  const update = () => {
    const agility = parseInt(input.dataset.agility, 10) || 0;
    const draw = Math.min(Math.max(parseInt(input.value, 10) || 0, 0), agility);
    result.textContent = `Piocher ${draw} · Force +${agility - draw}`;
  };
  input.addEventListener("input", update);
  update();
}

function setupAlerts() {
  document.querySelectorAll("[data-dismiss='alert']").forEach((button) => {
    button.addEventListener("click", () => button.closest(".flash").remove());
  });
}

document.addEventListener("DOMContentLoaded", () => {
  setupUniqueSelection();
  setupMultipleSelection();
  setupAgilityPreview();
  setupAlerts();
});
