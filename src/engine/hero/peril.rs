//! Perils suffered when entering dangerous locations.

use rand::RngExt;
use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;

/// Faces of the die rolled to escape a peril; only the highest face escapes.
const PERIL_DIE: u32 = 5;

/// What happened when a hero entered a location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerilOutcome {
    /// The location is safe.
    Safe,
    /// The location is perilous but the hero was lucky.
    Lucky,
    /// The hero suffered the peril.
    Suffered,
}

impl Game {
    /// Rolls the peril die and resolves the peril of the hero's location.
    ///
    /// # Errors
    /// Fails when the hero or its location is unknown or the corruption deck is empty.
    pub fn suffer_peril(
        &mut self,
        hero_id: i64,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<PerilOutcome, EngineError> {
        let roll = rng.random_range(1..=PERIL_DIE);
        self.suffer_peril_with_roll(hero_id, roll, data, rng)
    }

    /// Resolves the peril of the hero's location with a given die roll.
    ///
    /// A location is perilous when printed so or when its influence exceeds the
    /// hero's wisdom. On a failed roll the hero gains a corruption card and, if its
    /// wisdom doesn't exceed the number of surrounding locations holding influence,
    /// also loses 1 favor.
    ///
    /// # Errors
    /// Fails when the hero or its location is unknown or the corruption deck is empty.
    pub fn suffer_peril_with_roll(
        &mut self,
        hero_id: i64,
        roll: u32,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<PerilOutcome, EngineError> {
        let hero = self.hero(hero_id)?;
        let location = hero.location.clone();
        let influence = self.board.influence.get(&location).copied().unwrap_or(0);
        if !data.locations.get(&location)?.perilous && influence <= hero.wisdom {
            return Ok(PerilOutcome::Safe);
        }
        let location_name = data.locations.name(&location).to_owned();
        if roll == PERIL_DIE {
            self.log(
                ActorRef::Hero(hero_id),
                "peril.pass_trough",
                &[("location", json!(location_name))],
            );
            return Ok(PerilOutcome::Lucky);
        }

        let connected = data.paths.connected(&location);
        let surrounding = self
            .board
            .influence
            .iter()
            .filter(|(loc, count)| connected.contains(&loc.as_str()) && **count >= 0)
            .count();
        let card = self.gain_random_corruption(hero_id, data, rng)?;
        let card_name = data.corruption_card(card)?.name.to_lowercase();

        let hero = self.hero_mut(hero_id)?;
        let action = if usize::try_from(hero.wisdom).unwrap_or(0) <= surrounding {
            hero.favor_loss(1);
            "peril.shadow_background_memories"
        } else {
            "peril.encounter_orcs_couts"
        };
        self.log(
            ActorRef::Hero(hero_id),
            action,
            &[
                ("location", json!(location_name)),
                ("corruption", json!(card_name)),
            ],
        );
        Ok(PerilOutcome::Suffered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::test_support::{data, game, rng};
    use crate::game_data::corruption_card::Flaw;
    use crate::model::game::Game;

    // Port of corruption_test.rb: the hero stands in a perilous location.
    fn setup() -> Game {
        let mut g = game();
        g.heroes[0].location = "the_ruins_of_angmar".to_owned();
        g
    }

    fn peril(g: &mut Game) {
        g.suffer_peril_with_roll(1, 1, data(), &mut rng()).unwrap();
    }

    #[test]
    fn on_a_roll_of_5_nothing_happens() {
        let mut g = setup();
        g.board.decks.corruption.draw = vec![8];
        let outcome = g.suffer_peril_with_roll(1, 5, data(), &mut rng()).unwrap();
        assert_eq!(outcome, PerilOutcome::Lucky);
        assert!(g.heroes[0].corruption_cards.is_empty());
    }

    #[test]
    fn modifications_cards_acquisition() {
        for (card, stat) in [
            (8, "wisdom"),
            (2, "agility"),
            (7, "fortitude"),
            (14, "strength"),
        ] {
            let mut g = setup();
            g.board.decks.corruption.draw = vec![card];
            let before = g.heroes[0].clone();
            peril(&mut g);
            let after = &g.heroes[0];
            let delta = match stat {
                "wisdom" => after.wisdom - before.wisdom,
                "agility" => after.agility - before.agility,
                "fortitude" => after.fortitude - before.fortitude,
                _ => after.strength - before.strength,
            };
            assert_eq!(delta, -1, "{stat}");
            assert_eq!(after.corruption_cards, vec![card]);
        }
    }

    #[test]
    fn flaws_cards() {
        for (card, flaw) in [
            (5, Flaw::Cowardly),
            (9, Flaw::Dispairing),
            (11, Flaw::Isolated),
        ] {
            let mut g = setup();
            g.board.decks.corruption.draw = vec![card];
            peril(&mut g);
            assert!(g.heroes[0].has_flaw(flaw, data()), "{flaw:?}");
        }
    }

    #[test]
    fn careless() {
        let mut g = setup();
        g.board.decks.corruption.draw = vec![10];
        g.heroes[0].favor = 1;
        peril(&mut g);
        assert!(g.heroes[0].corruption_cards.is_empty());
        assert_eq!(g.heroes[0].favor, 0);

        g.board.decks.corruption.draw = vec![10];
        g.board.decks.corruption.discard = vec![4, 5, 6];
        g.heroes[0].favor = 6;
        peril(&mut g);
        assert!(g.heroes[0].corruption_cards.is_empty());
        assert_eq!(g.heroes[0].favor, 4);
        assert_eq!(g.board.decks.corruption.draw.len(), 4);
    }

    #[test]
    fn dispairing() {
        let mut g = setup();
        g.board.decks.corruption.draw = vec![9];
        g.heroes[0].favor = 1;
        peril(&mut g);
        assert_eq!(g.heroes[0].favor, 1);

        let mut g = setup();
        g.board.decks.corruption.draw = vec![9];
        g.heroes[0].favor = 20;
        peril(&mut g);
        assert_eq!(g.heroes[0].favor, 3);
    }
}
