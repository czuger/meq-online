//! Plot cards played by Sauron.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::choices::target_option;
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::mob_def::MobKind;
use crate::game_data::plot::PlotRequirement;
use crate::model::actor_ref::ActorRef;
use crate::model::board::PlotSlot;
use crate::model::choice::{ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::game::Game;

/// The two Gollum plots exclude each other.
pub const GOLLUM_CARDS: [u32; 2] = [5, 12];

/// Plot that lets Sauron search for a Gollum card.
const SMEAGOL_ESCAPES: u32 = 13;

/// Plot that also satisfies the Edoras requirement.
const SARUMAN_FALLS: u32 = 6;

/// Option code of Sauron picking a plot's location.
pub const PLOT_LOCATION: &str = "plot_location";

/// Number of plot slots on the board.
pub const PLOT_SLOTS: PlotSlot = 3;

/// What happens after a plot is played.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotPlayed {
    /// Sauron continues with his actions.
    Finished,
    /// Sauron must pick a Gollum card first.
    LookForGollum,
    /// Sauron picks the plot's location on the choice screen.
    ChooseLocation,
}

/// Why Sauron can't play a plot card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotBlock {
    /// The shadow pool holds too little influence.
    Influence {
        /// Influence the card needs.
        required: i32,
        /// Influence in the shadow pool.
        pool: i32,
    },
    /// The condition printed on the card isn't met.
    Requirement(PlotRequirement),
    /// No location can receive the plot.
    NoLocation,
}

impl Game {
    /// Tells whether Sauron may play `card` now.
    ///
    /// # Errors
    /// [`EngineError::GameData`] when the plot is unknown.
    pub fn plot_playable(&self, card: u32, data: &GameData) -> Result<bool, EngineError> {
        Ok(self.plot_block(card, data)?.is_none())
    }

    /// Why Sauron can't play `card` now, if so.
    ///
    /// # Errors
    /// [`EngineError::GameData`] when the plot is unknown.
    pub fn plot_block(&self, card: u32, data: &GameData) -> Result<Option<PlotBlock>, EngineError> {
        let plot = data.plot(card)?;
        if self.board.shadow_pool < plot.required_influence {
            return Ok(Some(PlotBlock::Influence {
                required: plot.required_influence,
                pool: self.board.shadow_pool,
            }));
        }
        if let Some(requirement) = plot.require.filter(|r| !self.requirement_met(*r)) {
            return Ok(Some(PlotBlock::Requirement(requirement)));
        }
        let located = plot
            .choose_location
            .is_none_or(|rule| !self.plot_location_targets(rule, data).is_empty());
        Ok((!located).then_some(PlotBlock::NoLocation))
    }

    fn requirement_met(&self, requirement: PlotRequirement) -> bool {
        let influence_sum = |locations: &[&str]| -> i32 {
            locations
                .iter()
                .map(|l| self.board.influence.get(*l).copied().unwrap_or(0))
                .sum()
        };
        let mob_at = |kind: Option<MobKind>, locations: &[&str]| {
            self.mobs.iter().any(|m| {
                kind.is_none_or(|k| m.kind == k) && locations.contains(&m.location.as_str())
            })
        };
        match requirement {
            PlotRequirement::MinionInErebor => mob_at(Some(MobKind::Minion), &["erebor"]),
            PlotRequirement::MonsterInOsgiliath => mob_at(Some(MobKind::Monster), &["osgiliath"]),
            PlotRequirement::InfluenceNearEdoras => {
                influence_sum(&[
                    "edoras",
                    "plains_of_rhoan",
                    "the_nidale",
                    "dunharrow",
                    "helms_deep",
                ]) >= 6
                    || self.board.plots.values().any(|c| *c == SARUMAN_FALLS)
            }
            PlotRequirement::InfluenceInIsengard => influence_sum(&["isengard"]) >= 3,
            PlotRequirement::InfluenceNearMountGundabad => {
                influence_sum(&[
                    "mount_gundabad",
                    "the_ruins_of_angmar",
                    "the_northern_waste",
                    "the_high_pass",
                ]) >= 8
            }
            PlotRequirement::RequireMobNearWoddenLand => mob_at(
                None,
                &["the_forest_trail", "the_forest_road", "lake_esgaroth"],
            ),
        }
    }

    /// Slots without a plot.
    pub fn free_plot_slots(&self) -> Vec<PlotSlot> {
        (1..=PLOT_SLOTS)
            .filter(|s| !self.board.plots.contains_key(s))
            .collect()
    }

    /// Sauron plays a plot card from his hand into a free slot. A card that doesn't
    /// print its affected location first asks Sauron to pick one.
    ///
    /// Requirements aren't checked here: the play screen only offers playable cards.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when the card isn't in hand,
    /// [`EngineError::Refused`] when the slot is invalid or used.
    #[tracing::instrument(level = "debug", skip_all, fields(card), err(level = "debug"))]
    pub fn play_plot(
        &mut self,
        card: u32,
        slot: PlotSlot,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<PlotPlayed, EngineError> {
        self.check_plot_play(card, slot)?;
        if let Some(rule) = data.plot(card)?.choose_location {
            let options = vec![target_option(
                PLOT_LOCATION,
                self.plot_location_targets(rule, data),
                1,
                1,
            )];
            self.ask(PendingChoice {
                chooser: Chooser::Sauron,
                source: ChoiceSource::PlotLocation { card, slot },
                options,
                resume: Resume::EventStep,
            })?;
            return Ok(PlotPlayed::ChooseLocation);
        }
        self.take_plot_from_hand(card, slot)?;
        self.activate_plot(card, slot);

        if card == SMEAGOL_ESCAPES {
            // The original engine never saved this transition, so the board stays on
            // the play screen until a Gollum card is picked.
            return Ok(PlotPlayed::LookForGollum);
        }
        self.plot_play_finished(data, rng)?;
        Ok(PlotPlayed::Finished)
    }

    fn check_plot_play(&self, card: u32, slot: PlotSlot) -> Result<(), EngineError> {
        if !self.sauron.plot_hand.contains(&card) {
            return Err(EngineError::CardNotAvailable {
                card,
                pile: "plot hand",
            });
        }
        if !(1..=PLOT_SLOTS).contains(&slot) || self.board.plots.contains_key(&slot) {
            return Err(EngineError::Refused(format!(
                "plot slot {slot} is not available"
            )));
        }
        Ok(())
    }

    /// Takes a plot card Sauron plays out of his hand.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when the card isn't in hand,
    /// [`EngineError::Refused`] when the slot is invalid or used.
    #[tracing::instrument(level = "debug", skip_all, fields(card), err(level = "debug"))]
    pub(crate) fn take_plot_from_hand(
        &mut self,
        card: u32,
        slot: PlotSlot,
    ) -> Result<(), EngineError> {
        self.check_plot_play(card, slot)?;
        self.sauron.plot_hand.retain(|c| *c != card);
        Ok(())
    }

    /// Puts a plot card on the Active Plot Track.
    #[tracing::instrument(level = "debug", skip_all, fields(card))]
    pub fn activate_plot(&mut self, card: u32, slot: PlotSlot) {
        self.board.plots.insert(slot, card);
        self.log(
            ActorRef::Sauron,
            "plot_cards.play",
            &[("plot_card", json!(card))],
        );
        self.remove_other_gollum_card(card);
    }

    /// Ends Sauron's Plot step: the Event step follows.
    ///
    /// # Errors
    /// Fails when the transition isn't allowed or the event deck can't be drawn.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn plot_play_finished(
        &mut self,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        self.event_step(data, rng)
    }

    /// Sauron ends his Plot step without playing a card.
    ///
    /// # Errors
    /// See [`Game::plot_play_finished`].
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn plot_step_passed(
        &mut self,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        self.log(ActorRef::Sauron, "plot_cards.pass", &[]);
        self.plot_play_finished(data, rng)
    }

    /// Gollum cards still in the plot deck or discard.
    pub fn gollum_cards_available(&self) -> Vec<u32> {
        let plot = &self.board.decks.plot;
        let in_pile = |pile: &[u32]| {
            GOLLUM_CARDS
                .into_iter()
                .filter(|c| pile.contains(c))
                .collect::<Vec<_>>()
        };
        let mut found = in_pile(&plot.draw);
        found.extend(in_pile(&plot.discard));
        found
    }

    /// Sauron takes a Gollum card; the discard is shuffled back into the plot deck.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when `card` isn't a Gollum card.
    #[tracing::instrument(level = "debug", skip_all, fields(card), err(level = "debug"))]
    pub fn take_gollum_card(
        &mut self,
        card: u32,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        if !GOLLUM_CARDS.contains(&card) {
            return Err(EngineError::CardNotAvailable {
                card,
                pile: "gollum cards",
            });
        }
        self.sauron.plot_hand.push(card);
        let plot = &mut self.board.decks.plot;
        plot.draw.append(&mut plot.discard);
        plot.draw.retain(|c| *c != card);
        plot.draw.shuffle(rng);
        tracing::trace!(pile = "plot.draw", order = ?plot.draw, "shuffled");
        self.plot_play_finished(data, rng)
    }

    /// Sauron removes the plot of a slot; the card goes to the discard pile (§ 5.4).
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn discard_plot(&mut self, slot: PlotSlot) {
        if let Some(card) = self.board.plots.remove(&slot) {
            self.forget_plot_location(slot);
            self.board.decks.plot.discard.push(card);
            self.log(
                ActorRef::Sauron,
                "plot_cards.discard",
                &[("plot_card", json!(card))],
            );
        }
    }

    /// Plot step: Sauron discards one of his active plots instead of playing one (§ 5.4);
    /// the step ends.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when the slot is empty; transition and event errors.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn plot_step_discard(
        &mut self,
        slot: PlotSlot,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        if !self.board.plots.contains_key(&slot) {
            return Err(EngineError::Refused(format!("no plot in slot {slot}")));
        }
        self.discard_plot(slot);
        self.plot_play_finished(data, rng)
    }

    /// Playing one Gollum plot discards the other one if it is in play.
    #[tracing::instrument(level = "debug", skip_all, fields(played))]
    fn remove_other_gollum_card(&mut self, played: u32) {
        let Some(other) = GOLLUM_CARDS
            .into_iter()
            .find(|c| *c != played && GOLLUM_CARDS.contains(&played))
        else {
            return;
        };
        let slot = self
            .board
            .plots
            .iter()
            .find(|(_, c)| **c == other)
            .map(|(s, _)| *s);
        if let Some(slot) = slot {
            self.board.plots.remove(&slot);
            self.board.decks.plot.discard.push(other);
            self.log(
                ActorRef::System,
                "plot_cards.discard_gollum",
                &[("plot_card", json!(other))],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, rng};
    use crate::model::game::Game;

    // Ports of plot_card_controller_test.rb: shadow pool 10, every plot in hand.
    fn setup() -> Game {
        let mut g = game();
        g.board.state = BoardState::EditSauronSauronActions;
        g.board.shadow_pool = 10;
        g.sauron.plot_hand = data().plots.keys().copied().collect();
        g
    }

    /// Plots with a printed location that can't be played (those whose location
    /// Sauron picks are tested in `plot_locations`).
    fn unplayable(g: &Game) -> Vec<u32> {
        g.sauron
            .plot_hand
            .iter()
            .copied()
            .filter(|c| data().plot(*c).unwrap().choose_location.is_none())
            .filter(|c| !g.plot_playable(*c, data()).unwrap())
            .collect()
    }

    #[test]
    fn requirements_block_some_plots() {
        assert_eq!(unplayable(&setup()), vec![3, 4, 7, 9, 13, 16]);
    }

    #[test]
    fn dark_messenger_with_minion_in_erebor() {
        let mut g = setup();
        g.create_mob("mouth_of_sauron", "erebor", None, data())
            .unwrap();
        assert_eq!(unplayable(&g), vec![4, 7, 9, 13, 16]);
    }

    #[test]
    fn osgiliath_invaded_with_monster_in_osgiliath() {
        let mut g = setup();
        g.create_mob("orc", "osgiliath", None, data()).unwrap();
        assert_eq!(unplayable(&g), vec![3, 7, 9, 13, 16]);
    }

    #[test]
    fn wormtongue_with_saruman_falls_in_play() {
        let mut g = setup();
        g.board.plots.insert(1, 6);
        assert_eq!(unplayable(&g), vec![3, 4, 9, 13, 16]);
    }

    #[test]
    fn orcs_in_the_mountains_with_influence_near_gundabad() {
        let mut g = setup();
        g.board.influence.insert("mount_gundabad".into(), 3);
        g.board.influence.insert("the_ruins_of_angmar".into(), 3);
        g.board.influence.insert("the_high_pass".into(), 2);
        assert_eq!(unplayable(&g), vec![3, 4, 7, 13, 16]);
    }

    #[test]
    fn smeagol_escapes_with_mob_near_the_woodland_realm() {
        let mut g = setup();
        g.create_mob("mouth_of_sauron", "the_forest_trail", None, data())
            .unwrap();
        assert_eq!(unplayable(&g), vec![3, 4, 7, 9, 16]);
    }

    #[test]
    fn uruk_hai_with_influence_in_isengard() {
        let mut g = setup();
        g.board.influence.insert("isengard".into(), 3);
        assert_eq!(unplayable(&g), vec![3, 4, 7, 9, 13]);
    }

    #[test]
    fn plays_a_card() {
        let mut g = setup();
        g.board.state = BoardState::PlayScreenSauronPlotCards;
        g.sauron.plot_hand = vec![8];
        assert_eq!(
            g.play_plot(8, 2, data(), &mut rng()).unwrap(),
            PlotPlayed::Finished
        );
        assert_eq!(g.board.plots.get(&2), Some(&8));
        assert!(g.sauron.plot_hand.is_empty());
        assert_eq!(g.board.state, BoardState::EditSauronSauronActions);
    }

    #[test]
    fn smeagol_leads_to_gollum_choice() {
        let mut g = setup();
        g.board.state = BoardState::PlayScreenSauronPlotCards;
        g.sauron.plot_hand = vec![13];
        assert_eq!(
            g.play_plot(13, 2, data(), &mut rng()).unwrap(),
            PlotPlayed::LookForGollum
        );
        assert_eq!(g.board.plots.get(&2), Some(&13));
    }

    #[test]
    fn gollum_card_removes_the_other_gollum_card() {
        let mut g = setup();
        g.board.state = BoardState::PlayScreenSauronPlotCards;
        g.board.plots.insert(1, 5);
        g.sauron.plot_hand = vec![12];
        g.play_plot(12, 2, data(), &mut rng()).unwrap();
        assert_eq!(g.board.plots.get(&2), Some(&12));
        assert_eq!(g.board.plots.get(&1), None);
        assert!(g.board.decks.plot.discard.contains(&5));
    }

    #[test]
    fn takes_a_gollum_card() {
        let mut g = setup();
        g.board.state = BoardState::PlayScreenSauronPlotCards;
        g.take_gollum_card(12, data(), &mut rng()).unwrap();
        assert!(g.sauron.plot_hand.contains(&12));
        assert!(!g.board.decks.plot.draw.contains(&12));
        assert_eq!(g.board.state, BoardState::EditSauronSauronActions);
    }

    #[test]
    fn refuses_cards_not_in_hand_and_used_slots() {
        let mut g = setup();
        g.sauron.plot_hand = vec![];
        assert!(g.play_plot(8, 1, data(), &mut rng()).is_err());
        g.sauron.plot_hand = vec![8];
        g.board.plots.insert(1, 3);
        assert!(g.play_plot(8, 1, data(), &mut rng()).is_err());
    }

    // § 5.4: a discarded Plot card goes faceup to the discard pile.
    #[test]
    fn discards_a_slot() {
        let mut g = setup();
        g.discard_plot(1);
        g.board.plots.insert(1, 8);
        g.discard_plot(1);
        assert!(g.board.plots.is_empty());
        assert_eq!(g.board.decks.plot.discard, vec![8]);
        assert_ne!(g.board.decks.plot.draw.last(), Some(&8));
    }
}
