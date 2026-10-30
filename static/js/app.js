// Page behaviours shared by every screen: card selection, agility preview, French card texts and alerts.
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

// Hovering a card picture shows its French text from /static/data/cards_fr.json,
// loaded on the first hover. Combat cards are found by their code, whatever the deck.
const CARD_PICTURE = /^\/static\/images\/(.+)$/;
const COMBAT_PICTURE = /^(?:hero_cards\/actions|monsters)\/[a-z_]+\/([a-z_]+)\.jpg$/;
let cardTexts = null;

function loadCardTexts() {
  if (!cardTexts) {
    cardTexts = fetch("/static/data/cards_fr.json")
      .then((response) => (response.ok ? response.json() : { combat: {}, cards: {} }))
      .catch(() => ({ combat: {}, cards: {} }));
  }
  return cardTexts;
}

function cardText(texts, src) {
  const picture = (src || "").match(CARD_PICTURE);
  if (!picture) return null;
  const path = decodeURIComponent(picture[1]);
  if (texts.cards[path]) return texts.cards[path];
  const combat = path.match(COMBAT_PICTURE);
  return combat ? texts.combat[combat[1]] || null : null;
}

function fillCardTip(tip, card) {
  tip.replaceChildren();
  const title = document.createElement("strong");
  title.textContent = card.name;
  tip.append(title);
  if (card.en && card.en !== card.name) {
    const english = document.createElement("span");
    english.className = "card-tip-en";
    english.textContent = card.en;
    tip.append(english);
  }
  if (card.kind) {
    const kind = document.createElement("span");
    kind.className = "card-tip-kind";
    kind.textContent = card.kind;
    tip.append(kind);
  }
  const text = document.createElement("p");
  text.textContent = card.text;
  tip.append(text);
}

// Beside the picture, on the side with room, kept inside the window.
function placeCardTip(tip, picture) {
  const rect = picture.getBoundingClientRect();
  const margin = 10;
  let left = rect.right + margin;
  if (left + tip.offsetWidth > window.innerWidth - margin) left = rect.left - tip.offsetWidth - margin;
  const top = Math.min(rect.top, window.innerHeight - tip.offsetHeight - margin);
  tip.style.left = `${Math.max(margin, left)}px`;
  tip.style.top = `${Math.max(margin, top)}px`;
}

function setupCardTexts() {
  const tip = document.createElement("div");
  tip.className = "card-tip";
  tip.hidden = true;
  document.body.append(tip);
  let hovered = null;
  document.addEventListener("mouseover", async (event) => {
    const picture = event.target.closest("img");
    if (!picture || picture === hovered || !CARD_PICTURE.test(picture.getAttribute("src") || "")) return;
    hovered = picture;
    const card = cardText(await loadCardTexts(), picture.getAttribute("src"));
    if (hovered !== picture || !card) return;
    fillCardTip(tip, card);
    tip.hidden = false;
    placeCardTip(tip, picture);
  });
  document.addEventListener("mouseout", (event) => {
    if (event.target !== hovered) return;
    hovered = null;
    tip.hidden = true;
  });
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
  setupCardTexts();
  setupAlerts();
});
