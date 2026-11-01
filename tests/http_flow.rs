//! Ports of the Rails controller and integration tests, run against the full router.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use common::driver::Driver;
use meq_online::engine::board_state::BoardState;
use meq_online::engine::deck::SauronDeck;
use meq_online::game_data::mob_def::MobKind;
use meq_online::model::game::Game;
use meq_online::model::sauron_action::{ActionInProgress, SauronAction};

#[tokio::test]
async fn anonymous_users_are_sent_to_welcome() {
    let app = TestApp::new().await;
    let answer = app
        .request(axum::http::Method::GET, "/boards", None, None)
        .await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER);
    assert_eq!(answer.location.as_deref(), Some("/welcome"));
    let welcome = app
        .request(axum::http::Method::GET, "/welcome", None, None)
        .await;
    assert_eq!(welcome.status, StatusCode::OK);
    assert!(welcome.body.contains("Se connecter"));
}

#[tokio::test]
async fn board_creation_and_list() {
    let app = TestApp::new().await;
    let me = app.login("frodo").await;
    let game = app.create_board(&me).await;
    assert_eq!(game.heroes.len(), 3);
    assert_eq!(game.board.state, BoardState::SauronSetupScreen);

    let list = app.get("/boards", &me).await;
    assert_eq!(list.status, StatusCode::OK);
    let sauron_id = game.sauron.id.unwrap();
    assert!(
        list.body
            .contains(&format!("/sauron/{sauron_id}/setup_screen")),
        "{}",
        list.body
    );
    assert!(
        app.get("/boards?all=true", &me)
            .await
            .body
            .contains("Sauron (frodo)")
    );
    assert_eq!(app.get("/boards/new", &me).await.status, StatusCode::OK);

    let board = game.board.id.unwrap();
    assert_eq!(
        app.get(&format!("/boards/{board}/story_screen"), &me)
            .await
            .status,
        StatusCode::OK
    );
    let logs = app.get(&format!("/boards/{board}/logs"), &me).await;
    assert!(logs.body.contains("intrigue de départ."), "{}", logs.body);
}

#[tokio::test]
async fn play_page_follows_the_turn() {
    let app = TestApp::new().await;
    let owner = app.login("elrond").await;
    app.post("/boards", &owner, "heroes=argalad&seats=open")
        .await;
    let player = app.login("legolas").await;
    let joined = app.post("/boards/1/join", &player, "hero=argalad").await;
    assert_eq!(joined.location.as_deref(), Some("/boards/1/play"));
    let sauron = app.game(1).await.sauron.id.unwrap();

    // Sauron sets up: his play page opens his screen, the hero's waits.
    let setup = format!("/sauron/{sauron}/setup_screen");
    assert_eq!(
        app.get("/boards/1/play", &owner).await.location.as_deref(),
        Some(setup.as_str())
    );
    let status = app.get("/boards/1/status", &owner).await;
    assert!(
        status.body.contains(&format!("\"url\":\"{setup}\"")),
        "{}",
        status.body
    );
    let waiting = app.get("/boards/1/play", &player).await;
    assert_eq!(waiting.status, StatusCode::OK);
    assert!(
        waiting.body.contains("Ce n'est pas votre tour") && waiting.body.contains("Sauron"),
        "{}",
        waiting.body
    );
    let status = app.get("/boards/1/status", &player).await;
    assert!(status.body.ends_with("\"url\":null}"), "{}", status.body);
    // The waiting page is framed by the hero's table.
    assert!(
        waiting.body.contains("Déroulement des tours"),
        "{}",
        waiting.body
    );
}

#[tokio::test]
async fn players_join_open_seats() {
    let app = TestApp::new().await;
    let owner = app.login("elrond").await;
    let created = app
        .post("/boards", &owner, "heroes=argalad&heroes=eometh&seats=open")
        .await;
    assert_eq!(created.status, StatusCode::SEE_OTHER);
    let game = app.game(1).await;
    assert_eq!(game.board.state, BoardState::WaitingForPlayers);

    let player = app.login("legolas").await;
    let page = app.get("/boards/1/join", &player).await;
    assert!(page.body.contains("Argalad"), "{}", page.body);
    app.post("/boards/1/join", &player, "hero=argalad").await;
    assert_eq!(
        app.post("/boards/1/join", &player, "hero=argalad")
            .await
            .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let other = app.login("eowyn").await;
    app.post("/boards/1/join", &other, "hero=eometh").await;
    let game = app.game(1).await;
    assert_eq!(game.board.state, BoardState::SauronSetupScreen);
    assert!(game.board.waiting_heroes.is_empty());
    let hero = game.heroes.iter().find(|h| h.code == "argalad").unwrap();
    assert_eq!(
        app.get(&format!("/heroes/{}", hero.id.unwrap()), &player)
            .await
            .status,
        StatusCode::SEE_OTHER
    );
}

#[tokio::test]
async fn non_members_are_refused() {
    let app = TestApp::new().await;
    let owner = app.login("sam").await;
    let game = app.create_board(&owner).await;
    let stranger = app.login("gollum").await;
    let sauron = game.sauron.id.unwrap();
    assert_eq!(
        app.get(&format!("/sauron/{sauron}/setup_screen"), &stranger)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    let board = game.board.id.unwrap();
    assert_eq!(
        app.get(&format!("/boards/{board}/logs"), &stranger)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn sauron_setup_and_turn() {
    let app = TestApp::new().await;
    let me = app.login("sauron").await;
    let game = app.create_board(&me).await;
    let (board, sauron) = (game.board.id.unwrap(), game.sauron.id.unwrap());

    assert_eq!(
        app.get(&format!("/sauron/{sauron}/setup_screen"), &me)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        app.get(&format!("/maps/{sauron}/edit"), &me).await.status,
        StatusCode::OK
    );

    // The starting plot's cards: keep the number it says.
    let game = app.game(board).await;
    let drawn = game.sauron.drawn_plot.clone();
    let to_keep = game.cards_to_keep(SauronDeck::Plot, &app.state.data);
    assert!(to_keep > 0 && drawn.len() > to_keep);
    let kept: Vec<String> = drawn[..to_keep].iter().map(u32::to_string).collect();
    let keep = app
        .post(
            &format!("/sauron/{sauron}/plot_cards/keep"),
            &me,
            &format!("selected_cards={}", kept.join(",")),
        )
        .await;
    assert_eq!(
        keep.location.as_deref(),
        Some(format!("/sauron/{sauron}/setup_screen").as_str())
    );
    assert_eq!(app.game(board).await.sauron.plot_hand, drawn[..to_keep]);

    // The starting influence must be placed before finishing.
    let refused = app
        .post(&format!("/sauron/{sauron}/setup_finished"), &me, "")
        .await;
    assert_eq!(refused.status, StatusCode::UNPROCESSABLE_ENTITY);
    let barad_dur = game.board.influence.get("barad_dur").copied().unwrap_or(0);
    let influence = app
        .post(
            &format!("/sauron/{sauron}/sauron_actions/set_influence"),
            &me,
            &format!("location=barad_dur&val={}", barad_dur + 1),
        )
        .await;
    assert!(
        influence.body.contains("\"result\":true"),
        "{}",
        influence.body
    );
    let mut game = app.game(board).await;
    game.board.influence_budget.tokens = 0;
    // An event that needs no choice, so the Action step follows.
    game.board.decks.event.draw.retain(|c| *c != 13);
    game.board.decks.event.draw.insert(0, 13);
    app.save(&mut game).await;

    let finished = app
        .post(&format!("/sauron/{sauron}/setup_finished"), &me, "")
        .await;
    let play = format!("/boards/{board}/play");
    assert_eq!(finished.location.as_deref(), Some(play.as_str()));
    assert_eq!(
        app.get(&play, &me).await.location.as_deref(),
        Some(format!("/sauron/{sauron}/sauron_actions/edit").as_str())
    );
    let game = app.game(board).await;
    assert_eq!(game.board.state, BoardState::EditSauronSauronActions);
    assert!(game.sauron.active);
    assert!(game.board.last_event.is_some());
    assert_eq!(
        app.get(&format!("/sauron/{sauron}/sauron_actions/edit"), &me)
            .await
            .status,
        StatusCode::OK
    );

    // Three heroes: three actions.
    let start = |action: &'static str| {
        let path = format!("/sauron/{sauron}/sauron_actions/start");
        let me = me.clone();
        let app = &app;
        async move { app.post(&path, &me, &format!("action={action}")).await }
    };
    let influence = start("place_influence").await;
    assert_eq!(
        influence.location.as_deref(),
        Some(format!("/sauron/{sauron}/sauron_actions/place_influence_screen").as_str())
    );
    assert_eq!(
        app.get(influence.location.as_deref().unwrap(), &me)
            .await
            .status,
        StatusCode::OK
    );
    let haven = app
        .post(
            &format!("/sauron/{sauron}/sauron_actions/set_influence"),
            &me,
            "location=rivendell&val=1",
        )
        .await;
    assert!(haven.body.contains("refuge"), "{}", haven.body);
    let pool = app
        .patch(
            &format!("/shadow_pools/{sauron}/update_from_map"),
            &me,
            "current_val=0",
        )
        .await;
    assert!(pool.body.contains("\"message\":null"), "{}", pool.body);
    let finish = format!("/sauron/{sauron}/sauron_actions/finish_action");
    app.post(&finish, &me, "").await;

    let command = start("command").await;
    assert_eq!(
        command.location.as_deref(),
        Some(format!("/sauron/{sauron}/sauron_mobs/command_screen").as_str())
    );
    assert_eq!(
        app.get(command.location.as_deref().unwrap(), &me)
            .await
            .status,
        StatusCode::OK
    );
    app.post(&finish, &me, "").await;

    let draw = start("draw_cards").await;
    assert_eq!(
        draw.location.as_deref(),
        Some(format!("/sauron/{sauron}/plot_cards/keep_screen").as_str())
    );
    let drawn = app.game(board).await.sauron.drawn_plot;
    app.post(
        &format!("/sauron/{sauron}/plot_cards/keep"),
        &me,
        &format!("selected_cards={}", drawn[0]),
    )
    .await;
    assert_eq!(app.game(board).await.board.sauron_actions_count, 3);

    let end = app
        .post(
            &format!("/sauron/{sauron}/sauron_actions/terminate"),
            &me,
            "",
        )
        .await;
    assert_eq!(
        end.location.as_deref(),
        Some(format!("/boards/{board}/play").as_str())
    );
    let game = app.game(board).await;
    assert_eq!(game.board.state, BoardState::HeroDrawCardsScreen);
    assert!(game.heroes.iter().all(|h| h.active));

    // Heroes are computer seats played by the board members: their links show up.
    let list = app.get("/boards", &me).await;
    let first = game.heroes[0].id.unwrap();
    assert!(
        list.body
            .contains(&format!("/heroes/{first}/draw_cards_screen")),
        "{}",
        list.body
    );
}

#[tokio::test]
async fn heroes_draw_then_first_hero_plays() {
    let app = TestApp::new().await;
    let me = app.login("aragorn").await;
    let mut game = app.create_board(&me).await;
    let board = game.board.id.unwrap();
    game.board.state = BoardState::HeroDrawCardsScreen;
    game.set_heroes_active(true);
    app.save(&mut game).await;

    for hero in &game.heroes {
        let id = hero.id.unwrap();
        assert_eq!(
            app.get(&format!("/heroes/{id}/draw_cards_screen"), &me)
                .await
                .status,
            StatusCode::OK
        );
        let drawn = app
            .post(&format!("/heroes/{id}/draw_cards"), &me, "nb_cards=5")
            .await;
        assert_eq!(drawn.status, StatusCode::SEE_OTHER);
        app.post(&format!("/heroes/{id}/draw_cards_finished"), &me, "")
            .await;
    }
    let game = app.game(board).await;
    let first = game.heroes.iter().min_by_key(|h| h.playing_order).unwrap();
    assert_eq!(game.board.current_hero_id, first.id);
    assert!(first.active && first.hand.len() == 5);
    assert!(matches!(
        game.board.state,
        BoardState::HeroRestScreen
            | BoardState::HeroMovementScreen
            | BoardState::CombatSetupScreenBoardCombats
    ));
    let first = first.id.unwrap();
    assert_eq!(
        app.get(&format!("/heroes/{first}"), &me).await.status,
        StatusCode::OK
    );
    assert_eq!(
        app.get(&format!("/maps/{first}"), &me).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn sauron_mobs() {
    let app = TestApp::new().await;
    let me = app.login("saruman").await;
    let mut game = app.create_board(&me).await;
    let (board, sauron) = (game.board.id.unwrap(), game.sauron.id.unwrap());
    let serpent = game
        .mobs
        .iter_mut()
        .find(|m| m.code == "black_serpent")
        .unwrap();
    serpent.life = 9;
    // A Command action with three commands is in progress.
    game.board.state = BoardState::CommandScreenSauronMobs;
    game.board.current_action = Some(ActionInProgress {
        action: SauronAction::Command,
        value: 3,
        commands_left: 3,
        monster_placed: false,
    });
    app.save(&mut game).await;
    let id_of = |code: &str| {
        game.mobs
            .iter()
            .find(|m| m.code == code)
            .unwrap()
            .id
            .unwrap()
    };
    let (serpent, mouth) = (id_of("black_serpent"), id_of("mouth_of_sauron"));

    let base = format!("/sauron/{sauron}/sauron_mobs");
    for page in [
        base.clone(),
        format!("{base}/command_screen"),
        format!("{base}/new"),
        format!("{base}/{mouth}/edit"),
    ] {
        assert_eq!(app.get(&page, &me).await.status, StatusCode::OK, "{page}");
    }
    // Monster placement and mob moves can be clicked on the table's map.
    let placing = app.get(&format!("{base}/new"), &me).await;
    assert!(
        placing
            .body
            .contains(r#"form="place-mob-form" name="location" value="#),
        "{}",
        placing.body
    );
    let moving = app.get(&format!("{base}/{mouth}/edit"), &me).await;
    assert!(
        moving
            .body
            .contains(r#"form="move-mob-form" name="button" value="#),
        "{}",
        moving.body
    );

    let healed = app.post(&format!("{base}/{serpent}/heal"), &me, "").await;
    assert_eq!(healed.location.as_deref(), Some(base.as_str()));
    let refused = app
        .patch(&format!("{base}/{serpent}"), &me, "button=pelargir")
        .await;
    assert_eq!(refused.status, StatusCode::UNPROCESSABLE_ENTITY);
    let created = app.post(&base, &me, "location=pelargir").await;
    assert_eq!(created.location.as_deref(), Some(base.as_str()));
    let moved = app
        .patch(&format!("{base}/{mouth}"), &me, "button=rhosgobel")
        .await;
    assert_eq!(moved.location.as_deref(), Some(base.as_str()));

    let game = app.game(board).await;
    let serpent = game
        .mobs
        .iter()
        .find(|m| m.code == "black_serpent")
        .unwrap();
    assert_eq!(
        (serpent.life, serpent.location.as_str()),
        (11, "near_harad")
    );
    let mouth = game
        .mobs
        .iter()
        .find(|m| m.code == "mouth_of_sauron")
        .unwrap();
    assert_eq!(mouth.location, "rhosgobel");
    assert_eq!(
        game.mobs
            .iter()
            .filter(|m| m.kind == MobKind::Monster)
            .count(),
        1
    );
    assert_eq!(game.board.current_action.unwrap().commands_left, 0);
}

#[tokio::test]
async fn combat_flow() {
    let app = TestApp::new().await;
    let me = app.login("boromir").await;
    let mut game = app.create_board(&me).await;
    let board = game.board.id.unwrap();
    let hero = game.heroes[0].id.unwrap();

    // The current hero meets an orc while moving.
    game.board.current_hero_id = Some(hero);
    game.board.state = BoardState::HeroMovementScreen;
    game.heroes[0].hand = vec![game.heroes[0].life_pool[0]];
    // A plain location: no Haven consent, no Peril to choose.
    game.heroes[0].location = "bree".to_owned();
    let location = game.heroes[0].location.clone();
    game.create_mob("orc", &location, None, &app.state.data)
        .unwrap();
    app.save(&mut game).await;
    let mut rng: rand::rngs::StdRng = rand::make_rng();
    let encounter = game
        .face_location_threats(hero, false, &app.state.data, &mut rng)
        .unwrap();
    assert_eq!(encounter, meq_online::engine::threats::Threat::Combat);
    game.board.state = BoardState::CombatSetupScreenBoardCombats;
    app.save(&mut game).await;

    assert_eq!(
        app.get(&format!("/boards/{board}/combats/combat_setup_screen"), &me)
            .await
            .status,
        StatusCode::OK
    );
    let setup = app
        .post(
            &format!("/boards/{board}/combats/combat_setup"),
            &me,
            "draw=0",
        )
        .await;
    assert_eq!(
        setup.location.as_deref(),
        Some(format!("/boards/{board}/combats/hero/play_combat_card_screen").as_str())
    );
    for side in ["hero", "sauron"] {
        let screen = app
            .get(
                &format!("/boards/{board}/combats/{side}/play_combat_card_screen"),
                &me,
            )
            .await;
        assert_eq!(screen.status, StatusCode::OK, "{side}");
    }

    let game = app.game(board).await;
    let (hero_card, mob_card) = (game.heroes[0].hand[0], game.mobs.last().unwrap().hand[0]);
    let first = app
        .post(
            &format!("/boards/{board}/combats/play_combat_card_hero"),
            &me,
            &format!("selected_card={hero_card}"),
        )
        .await;
    assert_eq!(
        first.location.as_deref(),
        Some(format!("/boards/{board}/play").as_str())
    );
    let second = app
        .post(
            &format!("/boards/{board}/combats/play_combat_card_mob"),
            &me,
            &format!("selected_card={mob_card}"),
        )
        .await;
    assert_eq!(second.status, StatusCode::SEE_OTHER);

    // The hero played its only card and is exhausted: Sauron plays alone until the mob
    // is exhausted too.
    for _ in 0..10 {
        assert_eq!(
            app.get(&format!("/boards/{board}/combats"), &me)
                .await
                .status,
            StatusCode::OK
        );
        let game = app.game(board).await;
        let combat = game.combat.as_ref().unwrap();
        if game.board.state != BoardState::PlayCombatCardScreenBoardCombats || combat.mob_exhausted
        {
            break;
        }
        assert!(!game.heroes[0].active);
        let card = game.mobs.last().unwrap().hand[0];
        app.post(
            &format!("/boards/{board}/combats/play_combat_card_mob"),
            &me,
            &format!("selected_card={card}"),
        )
        .await;
    }
    let game = app.game(board).await;
    if game.board.state == BoardState::CardsLossScreenBoardCombats {
        return;
    }
    let end = app
        .post(&format!("/boards/{board}/combats/terminate"), &me, "")
        .await;
    assert_eq!(end.status, StatusCode::SEE_OTHER, "{}", end.body);
    assert!(app.game(board).await.combat.is_none());
}

#[tokio::test]
async fn refused_actions_are_reported() {
    let app = TestApp::new().await;
    let me = app.login("pippin").await;
    let game = app.create_board(&me).await;
    let sauron = game.sauron.id.unwrap();
    // Not Sauron's actions step yet: the transition is refused, nothing is saved.
    let answer = app
        .post(
            &format!("/sauron/{sauron}/sauron_actions/terminate"),
            &me,
            "",
        )
        .await;
    assert_eq!(answer.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        app.game(game.board.id.unwrap()).await.board.state,
        BoardState::SauronSetupScreen
    );
}

#[tokio::test]
async fn messages_between_players() {
    let app = TestApp::new().await;
    let me = app.login("merry").await;
    let game = app.create_board(&me).await;
    let (sauron, hero) = (game.sauron.id.unwrap(), game.heroes[0].id.unwrap());
    assert_eq!(
        app.get(&format!("/sauron/{sauron}/board_messages/new"), &me)
            .await
            .status,
        StatusCode::OK
    );
    let sent = app
        .post(
            &format!("/sauron/{sauron}/board_messages"),
            &me,
            &format!("receiver=hero-{hero}&text=Bonjour"),
        )
        .await;
    assert_eq!(sent.status, StatusCode::SEE_OTHER);
    let inbox = app
        .get(&format!("/heroes/{hero}/board_messages"), &me)
        .await;
    assert!(
        inbox.body.contains("Bonjour") && inbox.body.contains("De : Sauron"),
        "{}",
        inbox.body
    );
}

#[tokio::test]
async fn flash_message_shows_once() {
    let app = TestApp::new().await;
    let me = app.login("bilbo").await;
    let refused = app.post("/boards", &me, "").await;
    assert_eq!(refused.location.as_deref(), Some("/boards/new"));
    let created = app.post("/boards", &me, "heroes=argalad").await;
    let flash = created
        .set_cookie
        .iter()
        .find(|c| c.starts_with("meq_flash="))
        .unwrap();
    let cookies = format!("{me}; {}", flash.split(';').next().unwrap());
    let page = app.get("/boards", &cookies).await;
    assert!(page.body.contains("La partie a été créée."));
    assert!(page.set_cookie.iter().any(|c| c.starts_with("meq_flash=;")));
}

#[tokio::test]
async fn status_reports_the_board_version() {
    let app = TestApp::new().await;
    let me = app.login("legolas").await;
    let game = app.create_board(&me).await;
    let board = game.board.id.unwrap();
    let status = |query: String| {
        let app = &app;
        let me = &me;
        async move {
            let answer = app.get(&format!("/boards/{board}/status{query}"), me).await;
            assert_eq!(answer.status, StatusCode::OK);
            serde_json::from_str::<serde_json::Value>(&answer.body).unwrap()
        }
    };

    let first = status(String::new()).await;
    let version = first["version"].as_str().unwrap().to_owned();
    let same = status(format!("?version={version}")).await;
    assert_eq!(same["version"], version.as_str());
    assert!(same["url"].is_null(), "{same}");

    app.state.versions.board_changed(board);
    let changed = status(format!("?version={version}")).await;
    assert_ne!(changed["version"], version.as_str());
}

// § 5.6: an event asking Sauron to choose waits on the choice screen until answered;
// the actions screen sends Sauron there instead of offering tracks the board refuses.
#[tokio::test]
async fn event_choice_flow() {
    let app = TestApp::new().await;
    let me = app.login("frodo").await;
    let mut game = app.create_board(&me).await;
    let board = game.board.id.unwrap();
    let sauron = game.sauron.id.unwrap();
    // "The Thieves of Tharbad": Sauron places a character at the Mouth of the Greyflood.
    game.board.state = BoardState::PlayScreenSauronPlotCards;
    game.board.decks.event.draw.retain(|c| *c != 2);
    game.board.decks.event.draw.insert(0, 2);
    app.save(&mut game).await;

    let play = format!("/boards/{board}/play");
    let finished = app
        .post(
            &format!("/sauron/{sauron}/plot_cards/play_finished"),
            &me,
            "",
        )
        .await;
    assert_eq!(finished.location.as_deref(), Some(play.as_str()));
    let game = app.game(board).await;
    assert_eq!(game.board.state, BoardState::ChoiceScreenBoardChoices);
    let choice_screen = format!("/boards/{board}/choices/choice_screen");
    assert_eq!(
        app.get(&play, &me).await.location.as_deref(),
        Some(choice_screen.as_str())
    );
    assert_eq!(
        app.get(&format!("/sauron/{sauron}/sauron_actions/edit"), &me)
            .await
            .location
            .as_deref(),
        Some(play.as_str())
    );

    let screen = app.get(&choice_screen, &me).await;
    assert_eq!(screen.status, StatusCode::OK);
    assert!(
        screen.body.contains("value=\"place\"") && screen.body.contains("Appliquer"),
        "{}",
        screen.body
    );
    assert!(screen.body.contains("events/I/2.png"), "{}", screen.body);

    let choose = format!("/boards/{board}/choices/choose");
    let character = game.board.pending_choice.as_ref().unwrap().options[0].targets[0].clone();
    let unknown = app.post(&choose, &me, "option=pass").await;
    assert_eq!(unknown.status, StatusCode::UNPROCESSABLE_ENTITY);
    let answer = app
        .post(
            &choose,
            &me,
            &format!("option=place&targets=place%7C{character}"),
        )
        .await;
    assert_eq!(answer.location.as_deref(), Some(play.as_str()));
    assert_eq!(
        app.get(&play, &me).await.location.as_deref(),
        Some(format!("/sauron/{sauron}/sauron_actions/edit").as_str())
    );
    let game = app.game(board).await;
    let code = character.strip_prefix("character:").unwrap();
    assert_eq!(game.board.characters[code], "mouth_of_the_greyflood");
    assert!(game.board.pending_choice.is_none());
    assert_eq!(game.board.state, BoardState::EditSauronSauronActions);
}

#[tokio::test]
async fn finale_flow() {
    let app = TestApp::new().await;
    let me = app.login("aragorn").await;
    let mut game = app.create_board(&me).await;
    let board = game.board.id.unwrap();
    let hero = game.heroes[0].id.unwrap();

    // The last hero ends the round; the Story step brings the heroes to the Finale
    // space. No mission is fulfilled: "His Dark Throne" and "The Spear of the West".
    game.board.state = BoardState::HeroExploration;
    game.board.current_hero_id = Some(hero);
    for other in game.heroes.iter_mut().skip(1) {
        other.turn_finished = true;
    }
    game.board.story.heroes = 16;
    game.board.plots.clear();
    game.board.sauron_objective = Some(0);
    game.board.heroes_objective = Some(0);
    app.save(&mut game).await;

    let end = app
        .post(&format!("/heroes/{hero}/exploration/next_step"), &me, "")
        .await;
    assert_eq!(
        end.location.as_deref(),
        Some(format!("/boards/{board}/play").as_str())
    );
    let game = app.game(board).await;
    assert_eq!(game.board.state, BoardState::ChoiceScreenBoardChoices);
    let screen = app
        .get(&format!("/boards/{board}/choices/choice_screen"), &me)
        .await;
    assert!(
        screen.body.contains("Choisir le champion"),
        "{}",
        screen.body
    );
    let story = app.get(&format!("/boards/{board}/story_screen"), &me).await;
    // Sauron's user sees his own Mission only while the game goes on.
    assert!(story.body.contains("sauron_0.png"), "{}", story.body);
    assert!(!story.body.contains("heroes_0.png"), "{}", story.body);

    let answer = app
        .post(
            &format!("/boards/{board}/choices/choose"),
            &me,
            &format!("option=champion&targets=champion%7Chero%3A{hero}"),
        )
        .await;
    let play = format!("/boards/{board}/play");
    assert_eq!(answer.location.as_deref(), Some(play.as_str()));
    assert_eq!(
        app.get(&play, &me).await.location.as_deref(),
        Some(format!("/boards/{board}/combats/combat_setup_screen").as_str())
    );
    let mut game = app.game(board).await;
    assert_eq!(game.combat.as_ref().unwrap().hero_id, hero);

    // The champion defeats the Ringwraiths.
    game.board.state = BoardState::PlayCombatCardScreenBoardCombats;
    let mob = game.combat.as_ref().unwrap().mob_id;
    game.mobs
        .iter_mut()
        .filter(|m| m.id == Some(mob))
        .for_each(|m| m.life = 0);
    app.save(&mut game).await;
    let end = app
        .post(&format!("/boards/{board}/combats/terminate"), &me, "")
        .await;
    assert_eq!(
        end.location.as_deref(),
        Some(format!("/boards/{board}/play").as_str())
    );
    let story = app.get(&format!("/boards/{board}/story_screen"), &me).await;
    assert!(story.body.contains("Victoire des héros"), "{}", story.body);
    assert!(story.body.contains("heroes_0.png"), "{}", story.body);
}

/// Creates a board with `heroes` on computer seats and the optional rules in `options`,
/// then plays Sauron's setup and first turn, every hero turn and Sauron's second turn
/// through HTTP only.
async fn full_round<'a>(app: &'a TestApp, heroes: &[&str], options: &str) -> (Driver<'a>, Game) {
    let me = app.login("sauron").await;
    let seats: Vec<String> = heroes.iter().map(|h| format!("heroes={h}")).collect();
    let form = format!("{}&seats=computer{options}", seats.join("&"));
    assert_eq!(
        app.post("/boards", &me, &form).await.status,
        StatusCode::SEE_OTHER
    );
    let game = app.game(1).await;
    let mut driver = Driver::new(app, &me, &game);

    // § 4: Sauron's first turn has no Story nor Plot step; the heroes draw next.
    let game = driver
        .play_until(|g| g.board.state == BoardState::HeroDrawCardsScreen)
        .await;
    assert_eq!(game.board.turn, 1);
    assert!(game.board.sauron_objective.is_some() && game.board.heroes_objective.is_some());
    assert!(game.board.last_event.is_some());
    assert!(game.heroes.iter().all(|h| h.active));

    // § 5.3: the heroes' turns end the round; Sauron's second turn advances the story.
    let game = driver
        .play_until(|g| g.board.turn == 2 && g.board.state == BoardState::HeroDrawCardsScreen)
        .await;
    assert!(game.board.story.heroes >= 2, "{:?}", game.board.story);
    assert!(game.board.winner.is_none());
    (driver, game)
}

/// Creates a board with `heroes` played by the AI; the driver only plays Sauron, through
/// his setup, first turn and second turn.
async fn computer_round(app: &TestApp, heroes: &[&str]) -> Game {
    let me = app.login("sauron").await;
    let seats: Vec<String> = heroes.iter().map(|h| format!("heroes={h}")).collect();
    let form = format!("{}&seats=ai", seats.join("&"));
    assert_eq!(
        app.post("/boards", &me, &form).await.status,
        StatusCode::SEE_OTHER
    );
    let game = app.game(1).await;
    assert!(!game.board.options.computer_heroes_by_hand);
    let mut driver = Driver::new(app, &me, &game);
    driver.heroes_by_ai = true;
    let game = driver
        .play_until(|g| g.board.turn == 2 && g.board.state == BoardState::EditSauronSauronActions)
        .await;
    assert!(game.board.story.heroes >= 2, "{:?}", game.board.story);
    assert!(driver.hero_turns.is_empty(), "the driver played a hero");
    game
}

// The table tells what each step of the turn did and which ones couldn't be played.
#[tokio::test]
async fn the_table_tells_each_step_of_the_round() {
    let app = TestApp::new().await;
    let game = computer_round(&app, &["eleanor", "thalin"]).await;
    let me = app.login("sauron").await;
    let sauron = game.sauron.id.unwrap();
    let page = app
        .get(&format!("/sauron/{sauron}/sauron_actions/edit"), &me)
        .await;
    assert_eq!(page.status, StatusCode::OK);
    for expected in [
        "Déroulement des tours",
        "Résultat des dernières étapes",
        "Tour de Sauron",
        "Tour d&#39;Eleanor",
        "Tour de Thàlin",
        "Pioche des héros",
        // Sauron's first turn starts at the Event step.
        "Pas de cette étape au premier tour",
        // Every hero's turn tells how it ended.
        "Fin du tour.",
        // The event card is shown to everyone.
        "/static/images/events/",
    ] {
        assert!(page.body.contains(expected), "{expected}: {}", page.body);
    }
    let log = app.get("/boards/1/logs", &me).await;
    assert_eq!(log.status, StatusCode::OK);
    assert!(!log.body.contains("step.begin"), "raw key: {}", log.body);
    assert!(!log.body.contains("turn.hero"), "raw key: {}", log.body);
}

// The table draws the full size map, with three zoom levels and nothing else.
#[tokio::test]
async fn the_map_uses_the_full_picture_and_three_zoom_levels() {
    let app = TestApp::new().await;
    let me = app.login("sauron").await;
    let game = app.create_board(&me).await;
    let sauron = game.sauron.id.unwrap();
    let page = app
        .get(&format!("/sauron/{sauron}/setup_screen"), &me)
        .await;
    assert!(
        page.body.contains("/static/images/map.jpg"),
        "{}",
        page.body
    );
    assert!(!page.body.contains("map_half"), "{}", page.body);
    assert_eq!(page.body.matches("data-map-zoom=").count(), 3);
    for level in ["full", "half", "fit"] {
        let button = format!(r#"data-map-zoom="{level}""#);
        assert!(page.body.contains(&button), "{level}");
    }
}

// A browser must not run an old script against a new page after an update.
#[tokio::test]
async fn browsers_check_scripts_and_styles_before_reusing_them() {
    let app = TestApp::new().await;
    for path in [
        "/static/js/game.js",
        "/static/js/app.js",
        "/static/css/app.css",
        "/static/data/cards_fr.json",
    ] {
        let answer = app.request(axum::http::Method::GET, path, None, None).await;
        assert_eq!(answer.status, StatusCode::OK, "{path}");
        assert_eq!(answer.cache_control.as_deref(), Some("no-cache"), "{path}");
    }
    // Pictures are many and rarely change: every page reload would check them all.
    let map = app
        .request(
            axum::http::Method::GET,
            "/static/images/map.jpg",
            None,
            None,
        )
        .await;
    assert_eq!(map.status, StatusCode::OK);
    assert_eq!(map.cache_control, None);
}

// What a player can't do is shown disabled, with the reason.
#[tokio::test]
async fn screens_tell_why_an_action_is_not_possible() {
    let app = TestApp::new().await;
    let me = app.login("sauron").await;
    let mut game = app.create_board(&me).await;
    // Not Beravor, who may heal anywhere.
    let seat = game
        .heroes
        .iter()
        .position(|h| h.code == "eleanor")
        .unwrap();
    let hero = game.heroes[seat].id.unwrap();
    game.board.current_hero_id = Some(hero);
    game.heroes[seat].active = true;
    game.heroes[seat].location = "barad_dur".to_owned();
    game.heroes[seat].hand.clear();
    game.board.state = BoardState::HeroMovementScreen;
    app.save(&mut game).await;

    let movement = app
        .get(&format!("/heroes/{hero}/movement_screen"), &me)
        .await;
    assert!(
        movement.body.contains("Aucun chemin n&#39;est praticable"),
        "{}",
        movement.body
    );
    assert!(
        movement.body.contains("vous en avez 0."),
        "{}",
        movement.body
    );
    assert!(
        !movement.body.contains(r#"form="move-form" name="button""#),
        "a path nobody can pay for stays clickable on the map"
    );

    game.heroes[seat].location = "the_shire".to_owned();
    game.board.state = BoardState::HeroRestScreen;
    game.heroes[seat].corruption_cards = vec![1];
    game.heroes[seat].favor = 0;
    app.save(&mut game).await;
    let rest = app.get(&format!("/heroes/{hero}/rest_screen"), &me).await;
    assert!(
        rest.body.contains("Vous n&#39;êtes pas dans un refuge."),
        "{}",
        rest.body
    );
    assert!(rest.body.contains("pour la défausser"), "{}", rest.body);

    let sauron = game.sauron.id.unwrap();
    game.sauron.plot_hand.clear();
    game.board.state = BoardState::PlayScreenSauronPlotCards;
    app.save(&mut game).await;
    let plots = app
        .get(&format!("/sauron/{sauron}/plot_cards/play_screen"), &me)
        .await;
    assert!(
        plots.body.contains("aucune carte intrigue en main"),
        "{}",
        plots.body
    );
}

// A hero sees the steps of the others, but never Sauron's hidden cards.
#[tokio::test]
async fn the_table_keeps_hidden_cards_secret() {
    let app = TestApp::new().await;
    let (driver, game) = full_round(&app, &["eleanor", "thalin"], "").await;
    let hero = game.heroes[0].id.unwrap();
    let page = app
        .get(&format!("/heroes/{hero}/draw_cards_screen"), &driver.cookie)
        .await;
    let log = page
        .body
        .split("Déroulement des tours")
        .nth(1)
        .expect("turn log");
    let log = log.split("<script").next().unwrap_or(log);
    assert!(!log.contains("/sauron/shadows/"), "{log}");
    for card in game.sauron.plot_hand.iter().chain(&game.sauron.drawn_plot) {
        let picture = format!("/sauron/plots/{card}.png");
        assert!(
            !log.split("tray").next().unwrap_or(log).contains(&picture),
            "{log}"
        );
    }
}

// The AI plays one computer hero through its two turns of the round.
#[tokio::test]
async fn computer_plays_one_hero() {
    let app = TestApp::new().await;
    computer_round(&app, &["beravor"]).await;
}

// The AI plays three computer heroes through the round.
#[tokio::test]
async fn computer_plays_three_heroes() {
    let app = TestApp::new().await;
    computer_round(&app, &["eleanor", "thalin", "argalad"]).await;
}

// § 4: one hero plays two turns per Sauron turn.
#[tokio::test]
async fn full_round_with_one_hero() {
    let app = TestApp::new().await;
    let (driver, _) = full_round(&app, &["beravor"], "").await;
    let turns: Vec<i32> = driver.hero_turns.iter().map(|(_, turn)| *turn).collect();
    assert_eq!(turns, [1, 2]);
}

// § 4: two heroes, one turn each.
#[tokio::test]
async fn full_round_with_two_heroes() {
    let app = TestApp::new().await;
    let (driver, game) = full_round(&app, &["eleanor", "thalin"], "").await;
    let played: Vec<i64> = driver.hero_turns.iter().map(|(id, _)| *id).collect();
    let mut seated: Vec<i64> = game.heroes.iter().map(|h| h.id.unwrap()).collect();
    seated.sort_unstable();
    assert_eq!(played, seated);
}

// § 4, § 10.4: three heroes and both optional rules, shown on the story screen.
#[tokio::test]
async fn full_round_with_three_heroes_and_optional_rules() {
    let options = "&advanced_dominance=1&high_stakes=1";
    let app = TestApp::new().await;
    let (driver, game) = full_round(&app, &["beravor", "eometh", "argalad"], options).await;
    assert_eq!(driver.hero_turns.len(), 3);
    assert!(game.board.options.advanced_dominance && game.board.options.high_stakes);
    let story = driver
        .app
        .get(
            &format!("/boards/{}/story_screen", driver.board),
            &driver.cookie,
        )
        .await;
    assert!(
        story.body.contains("Domination avancée") && story.body.contains("Enjeux élevés"),
        "{}",
        story.body
    );
}

// § 10.4: High stakes stage III, the heroes dominant: a hero defeated in combat sends
// Sauron to the choice screen to pick the marker that advances.
#[tokio::test]
async fn high_stakes_defeat_flow() {
    use meq_online::model::board::StoryMarkers;

    let app = TestApp::new().await;
    let me = app.login("gandalf").await;
    let mut game = app.create_board(&me).await;
    let board = game.board.id.unwrap();
    let hero = game.heroes[0].id.unwrap();
    game.board.options.high_stakes = true;
    game.board.story = StoryMarkers {
        heroes: 14,
        ring: 13,
        conquest: 0,
        corruption: 1,
    };
    game.board.current_hero_id = Some(hero);
    game.board.state = BoardState::HeroMovementScreen;
    game.heroes[0].location = "bree".to_owned();
    game.create_mob("orc", "bree", None, &app.state.data)
        .unwrap();
    // The orc gets its id.
    app.save(&mut game).await;
    let mut rng: rand::rngs::StdRng = rand::make_rng();
    game.face_location_threats(hero, false, &app.state.data, &mut rng)
        .unwrap();
    // No Shadow card to play at the start of the combat.
    game.sauron.shadow_hand.clear();
    game.board.state = BoardState::CombatSetupScreenBoardCombats;
    app.save(&mut game).await;
    app.post(
        &format!("/boards/{board}/combats/combat_setup"),
        &me,
        "draw=0",
    )
    .await;
    // The hero has no card left: the combat is lost.
    let mut game = app.game(board).await;
    game.heroes[0].hand.clear();
    game.heroes[0].life_pool.clear();
    app.save(&mut game).await;

    let end = app
        .post(&format!("/boards/{board}/combats/terminate"), &me, "")
        .await;
    let screen = format!("/boards/{board}/choices/choice_screen");
    assert_eq!(
        end.location.as_deref(),
        Some(screen.as_str()),
        "{}",
        end.body
    );
    let page = app.get(&screen, &me).await;
    assert!(
        page.body.contains("Enjeux élevés") && page.body.contains("Marqueur de la conquête"),
        "{}",
        page.body
    );
    let answer = app
        .post(
            &format!("/boards/{board}/choices/choose"),
            &me,
            "option=advance_marker&targets=advance_marker%7Cmarker%3Aring",
        )
        .await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
    let game = app.game(board).await;
    assert_eq!((game.board.story.ring, game.board.story.conquest), (14, 0));
    assert_eq!(game.heroes[0].location, "fornost");
}
