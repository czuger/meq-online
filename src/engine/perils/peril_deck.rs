//! Drawing Peril cards and Sauron's pick (§ 6.5).

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::choices::{target, target_option};
use crate::engine::engine_error::EngineError;
use crate::engine::threats::Threat;
use crate::game_data::GameData;
use crate::game_data::corruption_card::Flaw;
use crate::game_data::peril::PerilCode;
use crate::model::actor_ref::ActorRef;
use crate::model::choice::{ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::game::Game;

/// Peril cards Sauron draws.
const PERIL_DRAW: usize = 3;

/// Option code of Sauron's pick among the drawn Peril cards.
pub const PICK_PERIL: &str = "pick_peril";

impl Game {
    /// Sauron draws three Peril cards (four against a reckless hero) and picks one
    /// affecting the hero's location for him to resolve; when none does, all are
    /// discarded without effect (§ 6.5).
    ///
    /// # Errors
    /// Lookup and transition errors.
    pub fn draw_perils(
        &mut self,
        hero_id: i64,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Threat, EngineError> {
        let hero = self.hero(hero_id)?;
        let reckless = hero.has_flaw(Flaw::Reckless, data);
        let location_code = hero.location.clone();
        let location = data.locations.get(&location_code)?;
        let count = PERIL_DRAW + usize::from(reckless);
        let cards = self.draw_peril_cards(count, data, rng);
        let matching: Vec<String> = cards
            .iter()
            .filter(|c| {
                data.peril(**c)
                    .is_ok_and(|p| p.affects(&location_code, location))
            })
            .map(|c| target("peril", c))
            .collect();
        let params = [
            ("location", json!(location.name)),
            ("count", json!(cards.len())),
        ];
        self.log(ActorRef::Sauron, "peril.draw", &params);
        if matching.is_empty() {
            self.board.decks.peril.discard.extend(cards);
            self.log(ActorRef::Sauron, "peril.none_matching", &[]);
            return Ok(Threat::None);
        }
        self.ask(PendingChoice {
            chooser: Chooser::Sauron,
            source: ChoiceSource::PerilDraw { hero_id, cards },
            options: vec![target_option(PICK_PERIL, matching, 1, 1)],
            resume: Resume::Exploration,
        })?;
        Ok(Threat::Choice)
    }

    /// Draws Peril cards, rebuilding the deck from its discard pile (or from every card
    /// when both are empty, for games created before the Peril deck existed).
    fn draw_peril_cards(
        &mut self,
        count: usize,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Vec<PerilCode> {
        let pile = &mut self.board.decks.peril;
        if pile.draw.len() < count {
            pile.draw.append(&mut pile.discard);
            pile.draw.shuffle(rng);
        }
        if pile.draw.is_empty() {
            pile.draw = data.perils.keys().copied().collect();
            pile.draw.shuffle(rng);
        }
        let count = count.min(pile.draw.len());
        pile.draw.drain(..count).collect()
    }

    /// Sauron picked a Peril card: the drawn cards are discarded and the hero resolves
    /// the picked one.
    pub(crate) fn apply_peril_draw(
        &mut self,
        hero_id: i64,
        cards: &[PerilCode],
        targets: &[String],
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let card = cards
            .iter()
            .copied()
            .find(|c| targets.contains(&target("peril", c)))
            .ok_or_else(|| EngineError::Refused("choose a peril card".to_owned()))?;
        self.board.decks.peril.discard.extend_from_slice(cards);
        let params = [("peril", json!(data.peril(card)?.name_fr))];
        self.log(ActorRef::Hero(hero_id), "peril.resolve", &params);
        self.start_peril(hero_id, card, data, rng)
    }

    /// "Orc Scouts": the card and the discard pile are shuffled back into the deck.
    pub(crate) fn shuffle_perils_back(&mut self, rng: &mut StdRng) {
        let pile = &mut self.board.decks.peril;
        pile.draw.append(&mut pile.discard);
        pile.draw.shuffle(rng);
    }
}
