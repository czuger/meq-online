//! Locations affected by active plots, printed on the card or picked by Sauron (§ 5.4).

use serde_json::json;

use crate::engine::choices::target;
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::plot::PlotLocationRule;
use crate::model::actor_ref::ActorRef;
use crate::model::board::PlotSlot;
use crate::model::game::Game;

/// Influence "A Broken Line of Kings" needs on the locations next to its Haven.
const BROKEN_LINE_INFLUENCE: i32 = 6;

/// Locations "Loss of an Heir" may affect.
const HEIR_LOCATIONS: [&str; 2] = ["edoras", "minas_tirith"];

impl Game {
    /// Location affected by the plot of `slot`: picked by Sauron, or printed on the card.
    pub fn plot_location(&self, slot: PlotSlot, data: &GameData) -> Option<String> {
        if let Some(location) = self.board.turn_effects.plot_locations.get(&slot) {
            return Some(location.clone());
        }
        let card = self.board.plots.get(&slot)?;
        data.plot(*card).ok()?.affect.clone()
    }

    /// The active plot affecting `location`, as `(slot, card)`.
    pub fn plot_at(&self, location: &str, data: &GameData) -> Option<(PlotSlot, u32)> {
        self.board
            .plots
            .iter()
            .find(|(slot, _)| self.plot_location(**slot, data).as_deref() == Some(location))
            .map(|(slot, card)| (*slot, *card))
    }

    /// Heroes may not rest on the Haven of "A Broken Line of Kings".
    pub fn rest_forbidden_by_plot(&self, location: &str, data: &GameData) -> bool {
        self.plot_at(location, data).is_some_and(|(_, card)| {
            data.plot(card)
                .is_ok_and(|p| p.choose_location == Some(PlotLocationRule::BrokenLineOfKings))
        })
    }

    /// Locations Sauron may pick for a plot following `rule`, as choice targets.
    pub fn plot_location_targets(&self, rule: PlotLocationRule, data: &GameData) -> Vec<String> {
        let free = |code: &str| self.plot_at(code, data).is_none();
        let character = |code: &str| self.board.characters.values().any(|l| l == code);
        let mob_at = |code: &str| self.mobs.iter().any(|m| m.location == code);
        data.locations
            .iter()
            .map(|(code, location)| (code.as_str(), location))
            .filter(|(code, location)| match rule {
                PlotLocationRule::BrokenLineOfKings => {
                    location.haven
                        && free(code)
                        && self.adjacent_influence(code, data) >= BROKEN_LINE_INFLUENCE
                }
                PlotLocationRule::LossOfAnHeir => {
                    HEIR_LOCATIONS.contains(code)
                        && (mob_at(code) || data.paths.connected(code).into_iter().any(mob_at))
                }
                PlotLocationRule::PromiseOfRings => character(code) && mob_at(code),
                PlotLocationRule::UrgentSummons => character(code) && free(code),
            })
            .map(|(code, _)| target("location", code))
            .collect()
    }

    /// Influence on the locations adjacent to `location`.
    fn adjacent_influence(&self, location: &str, data: &GameData) -> i32 {
        data.paths
            .connected(location)
            .into_iter()
            .map(|l| self.board.influence.get(l).copied().unwrap_or(0))
            .sum()
    }

    /// Sauron picked the location of the plot he plays from his hand: the plot becomes
    /// active there; "Urgent Summons" sends the character tokens there back.
    ///
    /// # Errors
    /// [`EngineError::Refused`] without a location; play errors.
    #[tracing::instrument(level = "debug", skip_all, fields(card, ?targets), err(level = "debug"))]
    pub(crate) fn apply_plot_location(
        &mut self,
        card: u32,
        slot: PlotSlot,
        targets: &[String],
        data: &GameData,
    ) -> Result<(), EngineError> {
        let location = targets
            .first()
            .and_then(|t| t.strip_prefix("location:"))
            .ok_or_else(|| EngineError::Refused("choose the plot's location".to_owned()))?
            .to_owned();
        self.take_plot_from_hand(card, slot)?;
        self.activate_plot(card, slot);
        let params = [("location", json!(data.locations.name(&location)))];
        self.log(ActorRef::Sauron, "plot_cards.location", &params);
        if data.plot(card)?.choose_location == Some(PlotLocationRule::UrgentSummons) {
            self.send_characters_back(&location, data);
        }
        self.board
            .turn_effects
            .plot_locations
            .insert(slot, location);
        Ok(())
    }

    /// "Urgent Summons": the character tokens of `location` leave the board.
    #[tracing::instrument(level = "debug", skip_all, fields(%location))]
    fn send_characters_back(&mut self, location: &str, data: &GameData) {
        let summoned: Vec<String> = self
            .board
            .characters
            .iter()
            .filter(|(_, at)| *at == location)
            .map(|(code, _)| code.clone())
            .collect();
        for code in summoned {
            tracing::trace!(%code, "removing summoned character");
            self.board.characters.remove(&code);
            let name = data
                .character(&code)
                .map_or(code.as_str(), |c| c.name.as_str());
            let params = [("name", json!(name))];
            self.log(ActorRef::Sauron, "character.remove", &params);
        }
    }

    /// An active plot leaves its slot: the location Sauron picked for it is forgotten.
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn forget_plot_location(&mut self, slot: PlotSlot) {
        self.board.turn_effects.plot_locations.remove(&slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::plots::{PLOT_LOCATION, PlotPlayed};
    use crate::engine::test_support::{data, game, mob, rng};
    use crate::model::choice::ChoiceSource;

    const BROKEN_LINE_OF_KINGS: u32 = 10;
    const LOSS_OF_AN_HEIR: u32 = 14;
    const URGENT_SUMMONS: u32 = 17;

    fn plot_step(card: u32) -> Game {
        let mut g = game();
        g.board.state = BoardState::PlayScreenSauronPlotCards;
        g.board.shadow_pool = 4;
        g.sauron.plot_hand = vec![card];
        g
    }

    // § 5.4 "Urgent Summons": Sauron picks a location with a character token; the
    // characters there leave the board.
    #[test]
    fn urgent_summons_sends_characters_back() {
        let mut g = plot_step(URGENT_SUMMONS);
        assert!(!g.plot_playable(URGENT_SUMMONS, data()).unwrap());
        g.board
            .characters
            .insert("aragorn".to_owned(), "bree".to_owned());
        assert!(g.plot_playable(URGENT_SUMMONS, data()).unwrap());
        let played = g.play_plot(URGENT_SUMMONS, 2, data(), &mut rng());
        assert_eq!(played.unwrap(), PlotPlayed::ChooseLocation);
        g.answer_choice(
            PLOT_LOCATION,
            &[target("location", "bree")],
            data(),
            &mut rng(),
        )
        .unwrap();
        assert_eq!(g.board.plots.get(&2), Some(&URGENT_SUMMONS));
        assert_eq!(g.plot_location(2, data()).as_deref(), Some("bree"));
        // The Event step follows and may place characters again.
        assert!(g.new_logs.iter().any(|l| l.action == "character.remove"));
        assert!(g.sauron.plot_hand.is_empty());
        let pending = g.board.pending_choice.as_ref().map(|c| &c.source);
        assert!(!matches!(pending, Some(ChoiceSource::PlotLocation { .. })));
        g.discard_plot(2);
        assert!(g.board.turn_effects.plot_locations.is_empty());
    }

    // "A Broken Line of Kings": a Haven with 6 influence around it, without another plot
    // marker; heroes may not rest there.
    #[test]
    fn broken_line_of_kings_forbids_rest() {
        let mut g = plot_step(BROKEN_LINE_OF_KINGS);
        let rule = PlotLocationRule::BrokenLineOfKings;
        assert!(
            !g.plot_location_targets(rule, data())
                .contains(&target("location", "rivendell"))
        );
        g.board.influence.insert("the_high_pass".to_owned(), 3);
        g.board.influence.insert("the_trollshaws".to_owned(), 3);
        assert_eq!(
            g.plot_location_targets(rule, data()),
            vec![target("location", "rivendell")]
        );
        g.board.plots.insert(3, BROKEN_LINE_OF_KINGS);
        g.board
            .turn_effects
            .plot_locations
            .insert(3, "rivendell".to_owned());
        assert!(g.rest_forbidden_by_plot("rivendell", data()));
        assert!(
            g.plot_location_targets(rule, data()).is_empty(),
            "one plot marker per location"
        );
    }

    // "Loss of an Heir": Edoras or Minas Tirith with a monster or minion within 1 space.
    #[test]
    fn loss_of_an_heir_needs_a_mob_nearby() {
        let g = plot_step(LOSS_OF_AN_HEIR);
        let rule = PlotLocationRule::LossOfAnHeir;
        assert!(g.plot_location_targets(rule, data()).is_empty());
        let mut g = g;
        let mut orc = mob("orc");
        orc.location = "helms_deep".to_owned();
        g.mobs.push(orc);
        assert_eq!(
            g.plot_location_targets(rule, data()),
            vec![target("location", "edoras")]
        );
    }
}
