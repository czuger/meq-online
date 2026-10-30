//! Decisions taken while resolving a Shadow card.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::choices::{PASS, available_option, parse_target, target, target_option};
use crate::engine::engine_error::EngineError;
use crate::engine::hero::damage::picked_hand_cards;
use crate::engine::mobs::RINGWRAITHS;
use crate::engine::shadows::effects::{
    DARK_PROMISES_DAMAGE, MORDOR, MORDOR_INFLUENCE, MORGUL_DAMAGE, MORGUL_DAMAGE_PER_CORRUPTION,
    ShadowAsk, option,
};
use crate::game_data::GameData;
use crate::game_data::shadow::{ShadowCode, ShadowEffect};
use crate::model::actor_ref::ActorRef;
use crate::model::choice::{ChoiceOption, Chooser, Resume};
use crate::model::combat::CombatOrigin;
use crate::model::game::Game;

/// A decision within a Shadow card, as answered.
pub(crate) struct ShadowAnswer<'a> {
    /// The Shadow card.
    pub card: ShadowCode,
    /// The hero the card is played against.
    pub hero_id: Option<i64>,
    /// Heroes still concerned.
    pub heroes: &'a [i64],
    /// Targets picked before.
    pub picked: &'a [String],
    /// Option picked.
    pub option: &'a str,
    /// Targets picked.
    pub targets: &'a [String],
    /// Where play goes on once the card is resolved.
    pub resume: Resume,
}

impl ShadowAnswer<'_> {
    fn hero(&self) -> Result<i64, EngineError> {
        self.hero_id
            .ok_or_else(|| EngineError::Refused("this card needs a hero".to_owned()))
    }

    fn first_target(&self) -> Result<(&str, Vec<&str>), EngineError> {
        self.targets
            .first()
            .map(|t| parse_target(t))
            .ok_or_else(|| EngineError::Refused("choose a target".to_owned()))
    }
}

impl Game {
    /// Applies a decision of a Shadow card.
    ///
    /// # Returns
    /// Where play goes on when no other decision is pending.
    pub(crate) fn apply_shadow_choice(
        &mut self,
        answer: &ShadowAnswer,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let resume = answer.resume;
        match answer.option {
            PASS => {}
            option::FIGHT => self.shadow_fight(answer, data, rng)?,
            option::CORRUPTION => {
                return self.shadow_corruption(answer.hero()?, resume, data, rng);
            }
            option::DAMAGE => {
                let hero_id = answer.hero()?;
                return self.shadow_damage(hero_id, DARK_PROMISES_DAMAGE, resume, data, rng);
            }
            option::DAMAGE_PER_CORRUPTION => {
                let hero_id = answer.hero()?;
                let damage = MORGUL_DAMAGE_PER_CORRUPTION * self.corruption_count(hero_id)?;
                return self.shadow_damage(hero_id, damage, resume, data, rng);
            }
            option::DAMAGE_AND_CORRUPTION => {
                let hero_id = answer.hero()?;
                let resume = self.shadow_corruption(hero_id, resume, data, rng)?;
                if resume == Resume::EndTurn {
                    return Ok(resume);
                }
                return self.shadow_damage(hero_id, MORGUL_DAMAGE, resume, data, rng);
            }
            option::DISCARD_CARDS => return self.storms_discard(answer, rng),
            _ => return self.apply_action_shadow_choice(answer, data, rng),
        }
        Ok(resume)
    }

    /// Decisions of the cards played during the Action step.
    fn apply_action_shadow_choice(
        &mut self,
        answer: &ShadowAnswer,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        match answer.option {
            option::MOVE_HERO | option::CHOOSE_HERO | option::INFLUENCE_ON_HERO => {
                self.apply_hero_pick(answer, data, rng)?;
            }
            option::MORDOR_INFLUENCE => self.mordor_influence(data),
            option::PLACE_MONSTERS => {
                for location in answer
                    .targets
                    .iter()
                    .filter_map(|t| t.strip_prefix("location:"))
                {
                    self.place_new_monster(location, data, rng)?;
                }
            }
            option::PLOT_TOP | option::PLOT_BOTTOM => {
                if answer.option == option::PLOT_BOTTOM {
                    let pile = &mut self.board.decks.plot;
                    if !pile.draw.is_empty() {
                        pile.draw.rotate_left(1);
                    }
                }
                self.sarumans_lies_marker();
            }
            option::TAKE_CARD => self.take_discarded_card(answer)?,
            option::TAKE_PLOT | option::PLAY_PLOT => self.ithil_stone_plot(answer, rng)?,
            option::PICK_MINION => self.ask_minion_location(answer)?,
            option::PLACE_MINION => self.place_terrible_minion(answer, data)?,
            option::CHOOSE_HEROES => self.ask_redistribution(answer)?,
            option::REDISTRIBUTE => self.redistribute_corruption(answer, data)?,
            option::CORRUPT_HERO => {
                let (_, values) = answer.first_target()?;
                let hero_id = parse_id(values.first())?;
                return self.shadow_corruption(hero_id, answer.resume, data, rng);
            }
            other => return Err(EngineError::Refused(format!("unknown option {other}"))),
        }
        Ok(answer.resume)
    }

    /// The hero fights the monster his Shadow card brought (it replaces his Ambush
    /// step); "Flocks of Crebain" takes his agility.
    fn shadow_fight(
        &mut self,
        answer: &ShadowAnswer,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let hero_id = answer.hero()?;
        let code = answer
            .picked
            .first()
            .and_then(|t| t.strip_prefix("monster_code:"))
            .ok_or_else(|| EngineError::Refused("no monster to fight".to_owned()))?;
        let location = self.hero(hero_id)?.location.clone();
        let mob_id = self
            .mobs
            .iter()
            .rev()
            .find(|m| m.code == code && m.location == location && m.pool.is_none())
            .and_then(|m| m.id)
            .ok_or(EngineError::Unpersisted("mob"))?;
        let origin = CombatOrigin::Shadow { card: answer.card };
        self.attack_hero(hero_id, mob_id, origin, data, rng)?;
        if data.shadow(answer.card)?.effect == ShadowEffect::FlocksOfCrebain {
            self.combat_modifiers(|m| m.no_agility = true);
        }
        Ok(())
    }

    /// "Storms of Mordor": the hero discards the picked cards, then a random card per
    /// Corruption card; the next heroes follow.
    fn storms_discard(
        &mut self,
        answer: &ShadowAnswer,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let hero_id = answer.hero()?;
        let hero = self.hero_mut(hero_id)?;
        let cards = picked_hand_cards(hero_id, hero, answer.targets)?;
        hero.hand_to_rest(&cards)?;
        let count = hero.corruption_cards.len();
        self.discard_random(hero_id, count, rng)?;
        self.storms_of_mordor(answer.card, answer.heroes.to_vec(), answer.resume, rng)
    }

    /// "An Evil Fog" and "Do Not Tempt Me!": the hero picked, moved or not, then the
    /// card's loss per Corruption card.
    fn apply_hero_pick(
        &mut self,
        answer: &ShadowAnswer,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let (_, values) = answer.first_target()?;
        let hero_id = parse_id(values.first())?;
        if let (option::MOVE_HERO, Some(to)) = (answer.option, values.get(1)) {
            self.hero_mut(hero_id)?.location = (*to).to_owned();
            let params = [("location", json!(data.locations.name(to)))];
            self.log(ActorRef::Hero(hero_id), "shadow.moved", &params);
        }
        if answer.option == option::INFLUENCE_ON_HERO {
            let location = self.hero(hero_id)?.location.clone();
            self.add_influence(&location, 1, data);
        }
        let count = self.hero(hero_id)?.corruption_cards.len();
        if data.shadow(answer.card)?.effect == ShadowEffect::DoNotTemptMe {
            self.hero_mut(hero_id)?
                .favor_loss(i32::try_from(count).unwrap_or(i32::MAX));
        } else {
            self.discard_random(hero_id, count, rng)?;
        }
        Ok(())
    }

    /// "A New Power Is Rising": 2 influence on each location of Mordor, excess removed
    /// as usual (§ 5.8).
    fn mordor_influence(&mut self, data: &GameData) {
        let mordor: Vec<String> = data
            .locations
            .iter()
            .filter(|(_, l)| l.region_code == MORDOR)
            .map(|(code, _)| code.clone())
            .collect();
        for location in &mordor {
            self.add_influence(location, MORDOR_INFLUENCE, data);
        }
        self.trim_unsupported_influence(data);
        let params = [("count", json!(MORDOR_INFLUENCE))];
        self.log(ActorRef::Sauron, "shadow.mordor_influence", &params);
    }

    /// "Lord of the Rings": the picked discarded card goes to Sauron's hand, shown to
    /// the heroes; then the yellow marker may advance.
    fn take_discarded_card(&mut self, answer: &ShadowAnswer) -> Result<(), EngineError> {
        let (kind, values) = answer.first_target()?;
        let card: u32 = parse_id(values.first())?
            .try_into()
            .map_err(|_| EngineError::Refused("bad card".to_owned()))?;
        let (pile, hand, param) = match kind {
            "plot_card" => (
                &mut self.board.decks.plot.discard,
                &mut self.sauron.plot_hand,
                "plot_card",
            ),
            _ => (
                &mut self.board.decks.shadow.discard,
                &mut self.sauron.shadow_hand,
                "shadow_card",
            ),
        };
        let position =
            pile.iter()
                .position(|c| *c == card)
                .ok_or(EngineError::CardNotAvailable {
                    card,
                    pile: "discard pile",
                })?;
        pile.remove(position);
        hand.push(card);
        self.log(
            ActorRef::Sauron,
            "shadow.take_card",
            &[(param, json!(card))],
        );
        self.dark_messenger_marker();
        Ok(())
    }

    /// "The Ithil-stone": the plot card leaves its pile for Sauron's hand or the Active
    /// Plot Track; the deck is shuffled after the search.
    fn ithil_stone_plot(
        &mut self,
        answer: &ShadowAnswer,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let (_, values) = answer.first_target()?;
        let card: u32 = parse_id(values.first())?
            .try_into()
            .map_err(|_| EngineError::Refused("bad card".to_owned()))?;
        let pile = &mut self.board.decks.plot;
        pile.draw.retain(|c| *c != card);
        pile.discard.retain(|c| *c != card);
        pile.draw.shuffle(rng);
        match (answer.option, self.free_plot_slots().first().copied()) {
            (option::PLAY_PLOT, Some(slot)) => self.activate_plot(card, slot),
            _ => {
                self.sauron.plot_hand.push(card);
                self.log(
                    ActorRef::Sauron,
                    "shadow.take_card",
                    &[("plot_card", json!(card))],
                );
            }
        }
        Ok(())
    }

    /// "They Are Terrible": Sauron picks where the minion goes.
    fn ask_minion_location(&mut self, answer: &ShadowAnswer) -> Result<(), EngineError> {
        let picked = answer
            .targets
            .first()
            .cloned()
            .ok_or_else(|| EngineError::Refused("choose a minion".to_owned()))?;
        let locations = self
            .board
            .influence
            .iter()
            .filter(|(_, count)| **count > 0)
            .map(|(code, _)| target("location", code))
            .collect();
        let Some(place) = available_option(option::PLACE_MINION, locations, 1, 1) else {
            return Ok(());
        };
        self.ask_shadow(
            ShadowAsk {
                card: answer.card,
                hero_id: None,
                chooser: Chooser::Sauron,
                options: vec![place],
                heroes: vec![],
                picked: vec![picked],
            },
            answer.resume,
        )
    }

    /// "They Are Terrible": the minion, healed (or back from defeat at full life),
    /// goes to the picked location.
    fn place_terrible_minion(
        &mut self,
        answer: &ShadowAnswer,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let (_, values) = answer.first_target()?;
        let location = values.first().copied().unwrap_or_default().to_owned();
        let (kind, minion) = answer
            .picked
            .first()
            .map(|t| parse_target(t))
            .ok_or_else(|| EngineError::Refused("no minion picked".to_owned()))?;
        let minion = minion.first().copied().unwrap_or_default();
        let name = if kind == "defeated_minion" {
            self.create_mob(minion, &location, None, data)?;
            if minion == RINGWRAITHS {
                self.board.ringwraiths_return = false;
            }
            data.mob(minion)?.name.clone()
        } else {
            let mob = self.mob_mut(parse_id(Some(&minion))?)?;
            mob.life = mob.max_life;
            mob.location.clone_from(&location);
            mob.name.clone()
        };
        let params = [
            ("name", json!(name)),
            ("location", json!(data.locations.name(&location))),
        ];
        self.log(ActorRef::Sauron, "shadow.minion", &params);
        Ok(())
    }

    /// "The Lidless Eye": Sauron splits the two heroes' Corruption cards.
    fn ask_redistribution(&mut self, answer: &ShadowAnswer) -> Result<(), EngineError> {
        let mut heroes: Vec<i64> = answer
            .targets
            .iter()
            .map(|t| parse_id(parse_target(t).1.first()))
            .collect::<Result<_, _>>()?;
        heroes.sort_unstable();
        let mut cards = Vec::new();
        for hero_id in &heroes {
            let hero = self.hero(*hero_id)?;
            cards.extend(
                hero.corruption_cards
                    .iter()
                    .map(|c| target("corruption", c)),
            );
        }
        let max = cards.len();
        let picked = heroes.iter().map(|id| target("hero", id)).collect();
        let options = if cards.is_empty() {
            self.lidless_eye_give_options()
        } else {
            vec![target_option(option::REDISTRIBUTE, cards, 0, max)]
        };
        self.ask_shadow(
            ShadowAsk {
                card: answer.card,
                hero_id: None,
                chooser: Chooser::Sauron,
                options,
                heroes,
                picked,
            },
            answer.resume,
        )
    }

    /// The picked Corruption cards go to the first hero, the others to the second;
    /// then Sauron picks the hero receiving a new one.
    fn redistribute_corruption(
        &mut self,
        answer: &ShadowAnswer,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let [first, second] = answer.heroes else {
            return Err(EngineError::Refused("two heroes are needed".to_owned()));
        };
        for (from, to) in [(*first, *second), (*second, *first)] {
            let cards = self.hero(from)?.corruption_cards.clone();
            for card in cards {
                let to_first = answer.targets.contains(&target("corruption", card));
                if to_first == (to == *first) {
                    self.move_corruption_card(from, to, card, data)?;
                }
            }
        }
        self.log(ActorRef::Sauron, "shadow.redistribute", &[]);
        let options = self.lidless_eye_give_options();
        self.ask_shadow(
            ShadowAsk {
                card: answer.card,
                hero_id: None,
                chooser: Chooser::Sauron,
                options,
                heroes: vec![],
                picked: vec![],
            },
            answer.resume,
        )
    }

    /// "The Lidless Eye": any hero may receive the Corruption card.
    fn lidless_eye_give_options(&self) -> Vec<ChoiceOption> {
        available_option(option::CORRUPT_HERO, self.hero_targets(), 1, 1)
            .into_iter()
            .collect()
    }
}

/// Number in a target value.
fn parse_id(value: Option<&&str>) -> Result<i64, EngineError> {
    value
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| EngineError::Refused("bad target".to_owned()))
}
