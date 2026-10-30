// Page behaviours shared by every screen: card selection and alerts.
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

document.addEventListener("DOMContentLoaded", () => {
  setupUniqueSelection();
  setupMultipleSelection();
  setupAlerts();
});
