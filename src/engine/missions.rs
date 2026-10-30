//! Mission cards of both sides (§ 9): dealing and checking them.

use rand::RngExt;
use rand::rngs::StdRng;

use crate::engine::mobs::RINGWRAITHS;
use crate::engine::plots::PLOT_SLOTS;
use crate::engine::story::stage_of;
use crate::game_data::mob_def::MobKind;
use crate::game_data::plot::StoryType;
use crate::model::game::Game;
use crate::model::mission::{HEROES_MISSIONS, HeroesMission, SAURON_MISSIONS, SauronMission};

/// Location the Ringwraiths or Sauron's influence must reach for "To Find Them All".
const THE_SHIRE: &str = "the_shire";

/// Influence on the Shire fulfilling "To Find Them All".
const SHIRE_INFLUENCE: i32 = 6;

/// Most Corruption cards the heroes may hold in all for "Noble Blood".
const NOBLE_BLOOD_CORRUPTION: usize = 1;

/// Most minions in play for "Minas Morgul Kept at Bay".
const MOST_MINIONS: usize = 2;

/// Most monster tokens in play for "Against the Shadow".
const MOST_MONSTER_TOKENS: usize = 2;

/// Favor the heroes must hold in all for "Isildur's Secret".
const ISILDUR_FAVOR: i32 = 5;

impl Game {
    /// Deals the secret missions of both sides (§ 3).
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_objectives(&mut self, rng: &mut StdRng) {
        let card = |rng: &mut StdRng, count: usize| {
            u32::try_from(rng.random_range(0..count)).unwrap_or_default()
        };
        self.board.heroes_objective = Some(card(rng, HEROES_MISSIONS.len()));
        self.board.sauron_objective = Some(card(rng, SAURON_MISSIONS.len()));
        tracing::debug!(
            heroes = ?self.board.heroes_objective,
            sauron = ?self.board.sauron_objective,
            "missions drawn"
        );
    }

    /// Sauron's Mission card, once dealt.
    pub fn sauron_mission(&self) -> Option<SauronMission> {
        self.board
            .sauron_objective
            .and_then(SauronMission::from_card)
    }

    /// The heroes' Mission card, once dealt.
    pub fn heroes_mission(&self) -> Option<HeroesMission> {
        self.board
            .heroes_objective
            .and_then(HeroesMission::from_card)
    }

    /// Whether Sauron fulfils his Mission now; never without a Mission.
    pub fn sauron_mission_fulfilled(&self) -> bool {
        let stage_iii = |track| stage_of(self.board.story.sauron_marker(track)) == 3;
        match self.sauron_mission() {
            None => false,
            Some(SauronMission::HisDarkThrone) => self.board.plots.len() >= usize::from(PLOT_SLOTS),
            Some(SauronMission::WhereTheShadowsLie) => stage_iii(StoryType::Corruption),
            Some(SauronMission::InDarknessBindThem) => stage_iii(StoryType::Ring),
            Some(SauronMission::ToRuleThemAll) => stage_iii(StoryType::Conquest),
            Some(SauronMission::ToFindThemAll) => {
                let ringwraiths = self
                    .mobs
                    .iter()
                    .any(|m| m.code == RINGWRAITHS && m.location == THE_SHIRE);
                let influence = self.board.influence.get(THE_SHIRE).copied().unwrap_or(0);
                ringwraiths || influence >= SHIRE_INFLUENCE
            }
        }
    }

    /// Whether the heroes fulfil their Mission now; never without a Mission.
    ///
    /// "The Spear of the West" is never fulfilled: quests can't be completed yet.
    /// Blank monster tokens count as monster tokens in play.
    pub fn heroes_mission_fulfilled(&self) -> bool {
        let in_play = |kind| self.mobs.iter().filter(|m| m.kind == kind).count();
        match self.heroes_mission() {
            None | Some(HeroesMission::SpearOfTheWest) => false,
            Some(HeroesMission::NobleBlood) => {
                let held: usize = self.heroes.iter().map(|h| h.corruption_cards.len()).sum();
                held <= NOBLE_BLOOD_CORRUPTION
            }
            Some(HeroesMission::MinasMorgulKeptAtBay) => in_play(MobKind::Minion) <= MOST_MINIONS,
            Some(HeroesMission::AgainstTheShadow) => {
                in_play(MobKind::Monster) <= MOST_MONSTER_TOKENS
            }
            Some(HeroesMission::IsildursSecret) => {
                self.heroes.iter().map(|h| h.favor).sum::<i32>() >= ISILDUR_FAVOR
            }
        }
    }
}

/// Picture of a mission card.
pub fn objective_pic_path(side: &str, card: u32) -> String {
    format!("objectives/original/{side}/{side}_{card}.png")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::test_support::{data, game, mob};
    use crate::model::mob::NOTHING_CODE;

    fn with_missions(sauron: SauronMission, heroes: HeroesMission) -> Game {
        let mut g = game();
        g.board.sauron_objective = SAURON_MISSIONS
            .iter()
            .position(|m| *m == sauron)
            .and_then(|i| u32::try_from(i).ok());
        g.board.heroes_objective = HEROES_MISSIONS
            .iter()
            .position(|m| *m == heroes)
            .and_then(|i| u32::try_from(i).ok());
        g
    }

    // § 9: "His Dark Throne" needs the three Active Plot Track spaces filled.
    #[test]
    fn his_dark_throne_needs_three_plots() {
        let mut g = with_missions(SauronMission::HisDarkThrone, HeroesMission::NobleBlood);
        g.board.plots.insert(1, 0);
        g.board.plots.insert(2, 3);
        assert!(!g.sauron_mission_fulfilled());
        g.board.plots.insert(3, 4);
        assert!(g.sauron_mission_fulfilled());
    }

    // § 9: the marker missions need the marker in stage III (space 13 or more).
    #[test]
    fn marker_missions_need_stage_three() {
        let mut g = with_missions(SauronMission::ToRuleThemAll, HeroesMission::NobleBlood);
        g.board.story.conquest = 12;
        g.board.story.ring = 15;
        assert!(!g.sauron_mission_fulfilled());
        g.board.story.conquest = 13;
        assert!(g.sauron_mission_fulfilled());
    }

    // § 9: "To Find Them All": the Ringwraiths or 6 influence on the Shire.
    #[test]
    fn to_find_them_all() {
        let mut g = with_missions(SauronMission::ToFindThemAll, HeroesMission::NobleBlood);
        g.board.influence.insert(THE_SHIRE.to_owned(), 5);
        assert!(!g.sauron_mission_fulfilled());
        g.board.influence.insert(THE_SHIRE.to_owned(), 6);
        assert!(g.sauron_mission_fulfilled());
        g.board.influence.clear();
        g.create_mob(RINGWRAITHS, THE_SHIRE, None, data()).unwrap();
        assert!(g.sauron_mission_fulfilled());
    }

    // § 9: hero missions count what the heroes hold and what is in play.
    #[test]
    fn heroes_missions() {
        let mut g = with_missions(SauronMission::HisDarkThrone, HeroesMission::NobleBlood);
        g.heroes[0].corruption_cards = vec![0, 1];
        assert!(!g.heroes_mission_fulfilled());
        g.heroes[0].corruption_cards.pop();
        assert!(g.heroes_mission_fulfilled());

        let mut g = with_missions(
            SauronMission::HisDarkThrone,
            HeroesMission::AgainstTheShadow,
        );
        g.mobs = vec![mob("orc"), mob("black_serpent")];
        g.create_mob(NOTHING_CODE, "bree", None, data()).unwrap();
        assert!(g.heroes_mission_fulfilled());
        g.mobs.push(mob("snaga"));
        assert!(!g.heroes_mission_fulfilled());

        let mut g = with_missions(SauronMission::HisDarkThrone, HeroesMission::IsildursSecret);
        g.heroes[0].favor = 5;
        assert!(g.heroes_mission_fulfilled());

        let g = with_missions(SauronMission::HisDarkThrone, HeroesMission::SpearOfTheWest);
        assert!(!g.heroes_mission_fulfilled());
    }

    #[test]
    fn no_mission_is_never_fulfilled() {
        let g = game();
        assert!(!g.sauron_mission_fulfilled());
        assert!(!g.heroes_mission_fulfilled());
    }
}
