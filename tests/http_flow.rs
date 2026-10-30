//! Ports of the Rails controller and integration tests, run against the full router.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use meq_online::engine::board_state::BoardState;
use meq_online::engine::deck::SauronDeck;
use meq_online::game_data::mob_def::MobKind;
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
        .get(&format!("/sauron/{sauron}/setup_finished"), &me)
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
        .get(&format!("/sauron/{sauron}/setup_finished"), &me)
        .await;
    assert_eq!(
        finished.location.as_deref(),
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
    app.get(&finish, &me).await;

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
    app.get(&finish, &me).await;

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
        .get(&format!("/sauron/{sauron}/sauron_actions/terminate"), &me)
        .await;
    assert_eq!(end.location.as_deref(), Some("/boards"));
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
        app.get(&format!("/heroes/{id}/draw_cards_finished"), &me)
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

    let healed = app.get(&format!("{base}/{serpent}/heal"), &me).await;
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
    assert_eq!(first.location.as_deref(), Some("/boards"));
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
        .get(&format!("/boards/{board}/combats/terminate"), &me)
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
        .get(&format!("/sauron/{sauron}/sauron_actions/terminate"), &me)
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
async fn events_stream_is_server_sent_events() {
    let app = TestApp::new().await;
    let me = app.login("legolas").await;
    let game = app.create_board(&me).await;
    let board = game.board.id.unwrap();
    // Shutting the hub down first lets the stream end, so the body can be read.
    let router = app.router.clone();
    let request = axum::http::Request::get(format!("/boards/{board}/events"))
        .header("cookie", &me)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(router, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
}

// § 5.6: an event asking Sauron to choose waits on the choice screen until answered.
#[tokio::test]
async fn event_choice_flow() {
    use rand::SeedableRng;

    let app = TestApp::new().await;
    let me = app.login("frodo").await;
    let mut game = app.create_board(&me).await;
    let board = game.board.id.unwrap();
    // "The Thieves of Tharbad": Sauron places a character at the Mouth of the Greyflood.
    game.board.state = BoardState::PlayScreenSauronPlotCards;
    game.board.decks.event.draw.retain(|c| *c != 2);
    game.board.decks.event.draw.insert(0, 2);
    let mut rng = rand::rngs::StdRng::seed_from_u64(1);
    game.event_step(&app.state.data, &mut rng).unwrap();
    app.save(&mut game).await;
    assert_eq!(game.board.state, BoardState::ChoiceScreenBoardChoices);

    let screen = app
        .get(&format!("/boards/{board}/choices/choice_screen"), &me)
        .await;
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
    assert_eq!(
        answer.location.as_deref(),
        Some(format!("/sauron/{}/sauron_actions/edit", game.sauron.id.unwrap()).as_str())
    );
    let game = app.game(board).await;
    let code = character.strip_prefix("character:").unwrap();
    assert_eq!(game.board.characters[code], "mouth_of_the_greyflood");
    assert!(game.board.pending_choice.is_none());
    assert_eq!(game.board.state, BoardState::EditSauronSauronActions);
}
