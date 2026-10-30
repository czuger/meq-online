//! Plays a game through HTTP only: each step reads the board state and takes the
//! simplest legal action for it, as the user playing Sauron and the computer seats.

use std::collections::BTreeSet;

use axum::http::StatusCode;
use meq_online::engine::ai::waiting_hero::waiting_computer_hero;
use meq_online::engine::board_state::BoardState;
use meq_online::engine::choices::PASS;
use meq_online::engine::deck::SauronDeck;
use meq_online::engine::defeat::{DefeatLoss, closest_havens, defeat_losses};
use meq_online::engine::route::{RouteActor, route_for};
use meq_online::game_data::plot::StoryType;
use meq_online::model::game::Game;
use meq_online::model::hero::Item;
use meq_online::model::sauron_action::SauronAction;

use crate::common::TestApp;

/// Steps after which a round is considered stuck.
const MAX_STEPS: usize = 400;

/// A game played over HTTP by one user.
pub struct Driver<'a> {
    pub app: &'a TestApp,
    pub cookie: String,
    pub board: i64,
    pub sauron: i64,
    /// `(hero id, hero turn)` of every hero turn seen.
    pub hero_turns: BTreeSet<(i64, i32)>,
    /// `(hero id, hero turn)` of the hero turns where the hero moved.
    pub moves: BTreeSet<(i64, i32)>,
    /// The AI plays the heroes: the driver only plays Sauron.
    pub heroes_by_ai: bool,
}

impl<'a> Driver<'a> {
    pub fn new(app: &'a TestApp, cookie: &str, game: &Game) -> Self {
        Self {
            app,
            cookie: cookie.to_owned(),
            board: game.board.id.unwrap(),
            sauron: game.sauron.id.unwrap(),
            hero_turns: BTreeSet::new(),
            moves: BTreeSet::new(),
            heroes_by_ai: false,
        }
    }

    /// Plays until `done` holds for the game, failing when the game gets stuck.
    pub async fn play_until(&mut self, done: impl Fn(&Game) -> bool) -> Game {
        for _ in 0..MAX_STEPS {
            let game = self.app.game(self.board).await;
            if done(&game) {
                return game;
            }
            self.step(&game).await;
        }
        panic!(
            "the game is stuck: {:?}",
            self.app.game(self.board).await.board.state
        );
    }

    /// Takes one action for the current board state.
    async fn step(&mut self, game: &Game) {
        let state = game.board.state;
        if self.heroes_by_ai {
            let heroes: Vec<i64> = game.heroes.iter().filter_map(|h| h.id).collect();
            let waiting = waiting_computer_hero(game, &heroes);
            assert_eq!(waiting, None, "the AI left hero {waiting:?} in {state:?}");
        }
        // Every screen of the state renders for the player it waits on.
        let drawing = game
            .heroes
            .iter()
            .find(|h| h.active && state == BoardState::HeroDrawCardsScreen);
        let actor = match (drawing, game.board.current_hero_id) {
            (Some(hero), _) => RouteActor::Hero(hero.id.unwrap()),
            (None, Some(hero)) if !game.sauron.active => RouteActor::Hero(hero),
            _ => RouteActor::Sauron(self.sauron),
        };
        let screen = route_for(state, self.board, actor);
        if !screen.ends_with("/play") {
            let page = self.app.get(&screen, &self.cookie).await;
            assert_eq!(page.status, StatusCode::OK, "{screen}: {}", page.body);
        }
        self.note_hero_turn(game);
        match state {
            BoardState::SauronSetupScreen => self.setup(game).await,
            BoardState::EditSauronSauronActions => self.sauron_action(game).await,
            BoardState::PlaceInfluenceScreenSauronActions | BoardState::CommandScreenSauronMobs => {
                self.sauron_post("sauron_actions/finish_action", "").await;
            }
            BoardState::KeepScreenSauronPlotCards => self.keep_plots(game).await,
            BoardState::PlayScreenSauronPlotCards => {
                self.sauron_post("plot_cards/play_finished", "").await
            }
            BoardState::HeroDrawCardsScreen => self.heroes_draw(game).await,
            BoardState::ChoiceScreenBoardChoices => self.choose(game).await,
            BoardState::Finished => panic!("the game ended"),
            _ => self.hero_step(game).await,
        }
    }

    /// States of the current hero's turn.
    async fn hero_step(&mut self, game: &Game) {
        let hero = game.board.current_hero_id.unwrap();
        match game.board.state {
            BoardState::HeroRestScreen => self.hero_post(hero, "rest_skip", "").await,
            BoardState::AfterRestAdvanceStoryMarker => {
                let marker = marker_code(game.board.story.lowest_sauron_markers()[0]);
                let form = format!("marker={marker}");
                self.hero_post(hero, "after_rest_advance_story_marker", &form)
                    .await;
            }
            BoardState::HeroMovementScreen => self.travel(game, hero).await,
            BoardState::HeroExploration => self.hero_post(hero, "exploration/next_step", "").await,
            BoardState::HeroDefeatScreen => self.defeat(game, hero).await,
            BoardState::SauronHavenScreen => {
                let location = &game.hero(hero).unwrap().location;
                let haven = closest_havens(location, &self.app.state.data)[0];
                self.sauron_post("haven", &format!("haven={haven}")).await;
            }
            BoardState::CombatSetupScreenBoardCombats => {
                self.board_post("combats/combat_setup", "draw=0").await;
            }
            BoardState::PlayCombatCardScreenBoardCombats => self.combat_round(game).await,
            BoardState::CardsLossScreenBoardCombats => self.cards_loss(game).await,
            other => panic!("no step for {other:?}"),
        }
    }

    /// The hero moves once to an adjacent location, paying with a single card, then
    /// ends his Travel step.
    async fn travel(&mut self, game: &Game, hero_id: i64) {
        let hero = game.hero(hero_id).unwrap();
        let turn = (hero_id, hero.turn);
        if self.moves.contains(&turn) {
            return self.hero_post(hero_id, "movement_finished", "").await;
        }
        let data = &self.app.state.data;
        let adjacent: Vec<&str> = data
            .paths
            .distances_from(&hero.location)
            .into_iter()
            .filter(|(_, distance)| *distance == 1)
            .map(|(code, _)| code)
            .collect();
        let url = format!("/heroes/{hero_id}/move");
        let screen = self
            .app
            .get(&format!("/heroes/{hero_id}/movement_screen"), &self.cookie)
            .await;
        // Every destination can also be clicked on the table's map.
        for destination in &adjacent {
            let target = format!(r#"form="move-form" name="button" value="{destination}""#);
            assert!(screen.body.contains(&target), "{destination}");
        }
        for destination in adjacent {
            for card in &hero.hand {
                let form = format!("button={destination}&selected_cards={card}");
                self.app.post(&url, &self.cookie, &form).await;
                let after = self.app.game(self.board).await;
                if after.hero(hero_id).unwrap().location != hero.location {
                    self.moves.insert(turn);
                    return;
                }
            }
        }
        // No single card opens a path: the hero stays.
        self.moves.insert(turn);
    }

    /// Sauron keeps the starting plot cards, places the starting influence, then ends
    /// the setup.
    async fn setup(&mut self, game: &Game) {
        if !game.sauron.drawn_plot.is_empty() {
            return self.keep_plots(game).await;
        }
        if game.board.influence_budget.tokens > 0 {
            return self.place_one_influence(game).await;
        }
        self.sauron_post("setup_finished", "").await;
    }

    /// Places one influence token on the first location that takes it.
    async fn place_one_influence(&mut self, game: &Game) {
        let path = format!("/sauron/{}/sauron_actions/set_influence", self.sauron);
        for (code, _) in self.app.state.data.locations.iter() {
            let count = game
                .board
                .influence
                .get(code.as_str())
                .copied()
                .unwrap_or(0);
            let form = format!("location={code}&val={}", count + 1);
            let answer = self.app.post(&path, &self.cookie, &form).await;
            if answer.body.contains("\"result\":true") {
                return;
            }
        }
        panic!("no location takes influence");
    }

    /// Sauron takes his next action, Command or Place influence first, or ends his turn.
    async fn sauron_action(&mut self, game: &Game) {
        if game.sauron_actions_left() == 0 {
            return self.sauron_post("sauron_actions/terminate", "").await;
        }
        let action = [
            SauronAction::Command,
            SauronAction::PlaceInfluence,
            SauronAction::DrawCards,
        ]
        .into_iter()
        .find(|a| game.next_action_space(*a).is_some())
        .unwrap();
        let code = match action {
            SauronAction::Command => "command",
            SauronAction::PlaceInfluence => "place_influence",
            SauronAction::DrawCards => "draw_cards",
        };
        self.sauron_post("sauron_actions/start", &format!("action={code}"))
            .await;
    }

    async fn keep_plots(&mut self, game: &Game) {
        let keep = game
            .cards_to_keep(SauronDeck::Plot, &self.app.state.data)
            .clamp(1, game.sauron.drawn_plot.len());
        let kept: Vec<String> = game.sauron.drawn_plot[..keep]
            .iter()
            .map(u32::to_string)
            .collect();
        let form = format!("selected_cards={}", kept.join(","));
        self.sauron_post("plot_cards/keep", &form).await;
    }

    /// Each active hero draws its cards and finishes its Hero Draw step.
    async fn heroes_draw(&mut self, game: &Game) {
        let hero = game.heroes.iter().find(|h| h.active).unwrap();
        let id = hero.id.unwrap();
        let form = format!("nb_cards={}", hero.fortitude.max(0));
        let drawn = self
            .app
            .post(&format!("/heroes/{id}/draw_cards"), &self.cookie, &form)
            .await;
        assert_eq!(drawn.status, StatusCode::SEE_OTHER, "{}", drawn.body);
        self.hero_post(id, "draw_cards_finished", "").await;
    }

    /// Answers the pending choice: passes when allowed, otherwise picks the first
    /// option with as few targets as it takes.
    async fn choose(&mut self, game: &Game) {
        let choice = game.board.pending_choice.as_ref().unwrap();
        let option = choice
            .options
            .iter()
            .find(|o| o.code == PASS)
            .unwrap_or(&choice.options[0]);
        let mut form = format!("option={}", encode(&option.code));
        for target in option.targets.iter().take(option.min) {
            let value = format!("{}|{target}", option.code);
            form.push_str(&format!("&targets={}", encode(&value)));
        }
        self.board_post("choices/choose", &form).await;
    }

    async fn defeat(&mut self, game: &Game, hero: i64) {
        let mut form = String::new();
        if let Some(marker) = game.defeat_marker_choices().first() {
            form.push_str(&format!("marker={}&", marker_code(*marker)));
        }
        let losses = defeat_losses(game.hero(hero).unwrap());
        if losses.len() > 1 {
            let loss = match losses[0] {
                DefeatLoss::Favor => "favor",
                DefeatLoss::Item(Item::Horse) => "horse",
                DefeatLoss::Item(Item::Boat) => "boat",
            };
            form.push_str(&format!("loss={loss}"));
        }
        let answer = self
            .app
            .post(&format!("/heroes/{hero}/defeat"), &self.cookie, &form)
            .await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
    }

    /// The first combat move the rules accept: ending the combat, a card from the
    /// hero, then from the mob, then a declared exhaustion.
    async fn combat_round(&mut self, game: &Game) {
        let combat = game.combat.as_ref().unwrap();
        let hero_card = game.hero(combat.hero_id).unwrap().hand.first().copied();
        let mob_card = game
            .mobs
            .iter()
            .find(|m| m.id == Some(combat.mob_id))
            .and_then(|m| m.hand.first().copied());
        let mut moves = vec![("combats/terminate".to_owned(), String::new())];
        let hero_card = hero_card.filter(|_| !self.heroes_by_ai);
        if let Some(card) = hero_card {
            let form = format!("selected_card={card}");
            moves.push(("combats/play_combat_card_hero".to_owned(), form));
        }
        if let Some(card) = mob_card {
            let form = format!("selected_card={card}");
            moves.push(("combats/play_combat_card_mob".to_owned(), form));
        }
        let sides: &[&str] = if self.heroes_by_ai {
            &["mob"]
        } else {
            &["hero", "mob"]
        };
        for side in sides {
            moves.push((format!("combats/declare_exhaustion_{side}"), String::new()));
        }
        for (path, form) in moves {
            let url = format!("/boards/{}/{path}", self.board);
            let answer = self.app.post(&url, &self.cookie, &form).await;
            if answer.status == StatusCode::SEE_OTHER {
                return;
            }
        }
        panic!("no combat move accepted");
    }

    /// Loses as few hand cards as the combat accepts.
    async fn cards_loss(&mut self, game: &Game) {
        let hero = game.combat.as_ref().unwrap().hero_id;
        let hand = game.hero(hero).unwrap().hand.clone();
        let url = format!("/boards/{}/combats/cards_loss", self.board);
        for count in 0..=hand.len() {
            let cards: Vec<String> = hand[..count].iter().map(u32::to_string).collect();
            let form = format!("selected_cards={}", cards.join(","));
            let answer = self.app.post(&url, &self.cookie, &form).await;
            if answer.status == StatusCode::SEE_OTHER {
                return;
            }
        }
        panic!("no card loss accepted");
    }

    /// Notes the hero turn in progress.
    fn note_hero_turn(&mut self, game: &Game) {
        let hero_turn = matches!(
            game.board.state,
            BoardState::HeroRestScreen
                | BoardState::HeroMovementScreen
                | BoardState::HeroExploration
                | BoardState::CombatSetupScreenBoardCombats
                | BoardState::HeroDefeatScreen
        );
        if let (true, Some(id)) = (hero_turn, game.board.current_hero_id) {
            self.hero_turns.insert((id, game.hero(id).unwrap().turn));
        }
    }

    async fn sauron_post(&self, path: &str, form: &str) {
        let url = format!("/sauron/{}/{path}", self.sauron);
        let answer = self.app.post(&url, &self.cookie, form).await;
        assert_eq!(
            answer.status,
            StatusCode::SEE_OTHER,
            "{url}: {}",
            answer.body
        );
    }

    async fn hero_post(&self, hero: i64, path: &str, form: &str) {
        let url = format!("/heroes/{hero}/{path}");
        let answer = self.app.post(&url, &self.cookie, form).await;
        assert_eq!(
            answer.status,
            StatusCode::SEE_OTHER,
            "{url}: {}",
            answer.body
        );
    }

    async fn board_post(&self, path: &str, form: &str) {
        let url = format!("/boards/{}/{path}", self.board);
        let answer = self.app.post(&url, &self.cookie, form).await;
        assert_eq!(
            answer.status,
            StatusCode::SEE_OTHER,
            "{url} {form}: {}",
            answer.body
        );
    }
}

fn marker_code(marker: StoryType) -> &'static str {
    match marker {
        StoryType::Ring => "ring",
        StoryType::Conquest => "conquest",
        StoryType::Corruption => "corruption",
    }
}

/// Percent-encodes a form value.
fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}
