//! Sauron placing influence on locations and in the Shadow Pool (§ 5.7, § 5.8).
//!
//! Every placement is paid from the board's [`InfluenceBudget`], filled by a Place
//! Influence action, the starting plot or an undefeated monster or minion. Tokens placed
//! from the current budget may be taken back while it lasts.

use serde_json::json;

use crate::engine::influence_paths::InfluencePaths;
use crate::game_data::GameData;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;
use crate::model::sauron_action::InfluenceBudget;

/// Shadow Pool capacity per story stage.
const SHADOW_POOL_PER_STAGE: i32 = 4;

/// Why influence couldn't be placed or taken back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfluenceRefusal {
    /// The location isn't linked to a stronghold.
    NoConnection,
    /// The requested value exceeds what the strongholds support.
    MaxReached,
    /// Influence is never placed in Havens.
    Haven,
    /// No influence token is left to place.
    NoToken,
    /// Only tokens placed from the current budget may be taken back.
    NotPlacedNow,
    /// The Shadow Pool is full, or this budget can't put more tokens there.
    ShadowPoolFull,
}

/// Tokens of `budget` that may still go to the Shadow Pool.
pub fn shadow_pool_left(budget: &InfluenceBudget) -> i32 {
    (budget.shadow_pool_max - budget.shadow_pool_placed)
        .min(budget.tokens)
        .max(0)
}

impl Game {
    /// Gives Sauron influence tokens to place, `shadow_pool_max` of which may go to the
    /// Shadow Pool.
    #[tracing::instrument(level = "debug", skip_all, fields(tokens, shadow_pool_max))]
    pub fn grant_influence(&mut self, tokens: i32, shadow_pool_max: i32) {
        let budget = &mut self.board.influence_budget;
        budget.tokens += tokens;
        budget.shadow_pool_max += shadow_pool_max;
    }

    /// Drops the tokens left to place; placed tokens can't be taken back anymore.
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn close_influence_budget(&mut self) {
        self.board.influence_budget = InfluenceBudget::default();
    }

    /// Sauron sets the influence of a location: raising it spends tokens of the budget,
    /// lowering it takes back tokens placed from it. A Shadow Stronghold holds at most
    /// its red number, another location at most the strength of the strongest
    /// stronghold it is connected to, and Havens hold none (§ 5.8). Unknown locations
    /// are ignored.
    ///
    /// # Returns
    /// Why the change was refused, if it was.
    #[tracing::instrument(level = "debug", skip_all, fields(%location, value))]
    pub fn set_influence(
        &mut self,
        location: &str,
        value: i32,
        data: &GameData,
    ) -> Option<InfluenceRefusal> {
        let def = data.locations.get(location).ok()?;
        let current = self.board.influence.get(location).copied().unwrap_or(0);
        let refusal = if value > current {
            if def.haven {
                return Some(InfluenceRefusal::Haven);
            }
            let max = def.max_influence.unwrap_or_else(|| {
                InfluencePaths::new(&self.board.influence, &data.paths).max_for_new_token(location)
            });
            self.place_influence(location, value - current, max)
        } else {
            self.take_back_influence(location, current - value)
        };
        if refusal.is_none() && value != current {
            let params = [("location", json!(def.name)), ("val", json!(value))];
            self.log(ActorRef::Sauron, "place_influence.success", &params);
        }
        refusal
    }

    #[tracing::instrument(level = "debug", skip_all, fields(%location, count, max))]
    fn place_influence(
        &mut self,
        location: &str,
        count: i32,
        max: i32,
    ) -> Option<InfluenceRefusal> {
        let current = self.board.influence.get(location).copied().unwrap_or(0);
        if max == 0 {
            return Some(InfluenceRefusal::NoConnection);
        }
        if current + count > max {
            return Some(InfluenceRefusal::MaxReached);
        }
        let budget = &mut self.board.influence_budget;
        if budget.tokens < count {
            return Some(InfluenceRefusal::NoToken);
        }
        budget.tokens -= count;
        *budget.placed.entry(location.to_owned()).or_default() += count;
        self.board
            .influence
            .insert(location.to_owned(), current + count);
        None
    }

    #[tracing::instrument(level = "debug", skip_all, fields(%location, count))]
    fn take_back_influence(&mut self, location: &str, count: i32) -> Option<InfluenceRefusal> {
        let budget = &mut self.board.influence_budget;
        let placed = budget.placed.get(location).copied().unwrap_or(0);
        if placed < count {
            return Some(InfluenceRefusal::NotPlacedNow);
        }
        budget.tokens += count;
        budget.placed.insert(location.to_owned(), placed - count);
        let current = self.board.influence.get(location).copied().unwrap_or(0);
        if current - count > 0 {
            self.board
                .influence
                .insert(location.to_owned(), current - count);
        } else {
            self.board.influence.remove(location);
        }
        None
    }

    /// Adds influence to a location outside of a budget (card effects); Havens never
    /// hold influence (§ 5.8).
    #[tracing::instrument(level = "debug", skip_all, fields(%location, count))]
    pub fn add_influence(&mut self, location: &str, count: i32, data: &GameData) {
        if data.locations.get(location).is_ok_and(|l| l.haven) {
            return;
        }
        *self.board.influence.entry(location.to_owned()).or_default() += count;
    }

    /// Adds influence to the Shadow Pool outside of a budget (card effects), up to its
    /// capacity of 4 × the current stage (§ 5.8).
    #[tracing::instrument(level = "debug", skip_all, fields(count))]
    pub fn add_to_shadow_pool(&mut self, count: i32) {
        let capacity = self.board.story.stage() * SHADOW_POOL_PER_STAGE;
        let old = self.board.shadow_pool;
        self.board.shadow_pool = (old + count).min(capacity).max(old);
        let params = [
            ("old_value", json!(old)),
            ("new_value", json!(self.board.shadow_pool)),
        ];
        self.log(ActorRef::Sauron, "change_shadow_pool", &params);
    }

    /// Removes up to `count` influence from a location.
    #[tracing::instrument(level = "debug", skip_all, fields(%location, count))]
    pub fn remove_influence(&mut self, location: &str, count: i32) {
        let left = self.board.influence.get(location).copied().unwrap_or(0) - count;
        if left > 0 {
            self.board.influence.insert(location.to_owned(), left);
        } else {
            self.board.influence.remove(location);
        }
    }

    /// Excess influence is removed down to what the connected strongholds support, a
    /// location cut from them keeping one token (§ 5.8).
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn trim_unsupported_influence(&mut self, data: &GameData) {
        let locations: Vec<String> = self.board.influence.keys().cloned().collect();
        for location in locations {
            tracing::trace!(%location, influence = ?self.board.influence.get(&location), "checking influence limit");
            let max = data
                .locations
                .get(&location)
                .ok()
                .and_then(|l| l.max_influence)
                .unwrap_or_else(|| {
                    InfluencePaths::new(&self.board.influence, &data.paths)
                        .max_for_existing_token(&location)
                });
            let count = self.board.influence.get(&location).copied().unwrap_or(0);
            if max < count {
                self.board.influence.insert(location.clone(), max);
                let params = [
                    ("location", json!(data.locations.name(&location))),
                    ("count", json!(count - max)),
                ];
                self.log(ActorRef::System, "rally.auto_remove", &params);
            }
        }
    }

    /// Sauron puts one token of the budget in the Shadow Pool (`add`), or takes back
    /// one he put there from it. The pool holds at most 4 × the current stage (§ 5.8).
    ///
    /// # Errors
    /// The [`InfluenceRefusal`] when the change isn't allowed.
    #[tracing::instrument(level = "debug", skip_all, fields(add), err(level = "debug", Debug))]
    pub fn change_shadow_pool(&mut self, add: bool) -> Result<i32, InfluenceRefusal> {
        let old = self.board.shadow_pool;
        let capacity = self.board.story.stage() * SHADOW_POOL_PER_STAGE;
        let budget = &mut self.board.influence_budget;
        if add {
            if shadow_pool_left(budget) == 0 || old >= capacity {
                return Err(InfluenceRefusal::ShadowPoolFull);
            }
            budget.tokens -= 1;
            budget.shadow_pool_placed += 1;
            self.board.shadow_pool += 1;
        } else {
            if budget.shadow_pool_placed == 0 {
                return Err(InfluenceRefusal::NotPlacedNow);
            }
            budget.tokens += 1;
            budget.shadow_pool_placed -= 1;
            self.board.shadow_pool -= 1;
        }
        let params = [
            ("old_value", json!(old)),
            ("new_value", json!(self.board.shadow_pool)),
        ];
        self.log(ActorRef::Sauron, "change_shadow_pool", &params);
        Ok(self.board.shadow_pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::test_support::{data, game};

    // § 5.8: never in Havens; a Shadow Stronghold holds at most its red number.
    #[test]
    fn influence_limits() {
        let mut g = game();
        g.grant_influence(20, 0);
        assert_eq!(
            g.set_influence("rivendell", 1, data()),
            Some(InfluenceRefusal::Haven)
        );
        assert_eq!(g.set_influence("barad_dur", 6, data()), None);
        assert_eq!(
            g.set_influence("barad_dur", 7, data()),
            Some(InfluenceRefusal::MaxReached)
        );
        assert_eq!(
            g.set_influence("dol_guldur", 4, data()),
            Some(InfluenceRefusal::MaxReached)
        );
    }

    // § 5.7 Place Influence: X tokens, at most two in the Shadow Pool.
    #[test]
    fn placement_spends_the_budget() {
        let mut g = game();
        let before = g.board.influence.get("barad_dur").copied().unwrap_or(0);
        assert_eq!(
            g.set_influence("barad_dur", before + 1, data()),
            Some(InfluenceRefusal::NoToken)
        );
        g.grant_influence(3, 2);
        assert_eq!(g.set_influence("barad_dur", before + 1, data()), None);
        let pool = g.board.shadow_pool;
        assert_eq!(g.change_shadow_pool(true), Ok(pool + 1));
        assert_eq!(g.change_shadow_pool(true), Ok(pool + 2));
        assert_eq!(g.board.influence_budget.tokens, 0);
        assert_eq!(
            g.change_shadow_pool(true),
            Err(InfluenceRefusal::ShadowPoolFull)
        );
    }

    #[test]
    fn only_tokens_of_the_budget_are_taken_back() {
        let mut g = game();
        let before = g.board.influence.get("barad_dur").copied().unwrap_or(0);
        g.grant_influence(1, 0);
        assert_eq!(
            g.set_influence("barad_dur", before - 1, data()),
            Some(InfluenceRefusal::NotPlacedNow)
        );
        g.set_influence("barad_dur", before + 1, data());
        assert_eq!(g.set_influence("barad_dur", before, data()), None);
        assert_eq!(g.board.influence_budget.tokens, 1);
        assert_eq!(
            g.change_shadow_pool(true),
            Err(InfluenceRefusal::ShadowPoolFull)
        );
    }
}
