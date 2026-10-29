//! A game survives a save and a load unchanged.

use std::path::Path;

use meq_online::db::{game_repo, pool, user_repo};
use meq_online::engine::setup::new_game;
use meq_online::game_data::GameData;
use meq_online::model::played_card::PlayedCard;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[tokio::test]
async fn new_game_round_trip() {
    let data = GameData::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap();
    let pool = pool::connect("sqlite::memory:").await.unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let creator = user_repo::find_or_create(&mut conn, "dev", "me", "Moi", "me@x")
        .await
        .unwrap();
    let ai = user_repo::ai_users(&mut conn, 4).await.unwrap();
    let mut game = new_game(
        creator.id,
        [ai[0], ai[1], ai[2], ai[3]],
        &data,
        &mut StdRng::seed_from_u64(1),
    )
    .unwrap();
    game_repo::save(&mut conn, &mut game).await.unwrap();
    let board_id = game.board.id.unwrap();
    assert!(game.heroes.iter().all(|h| h.id.is_some()));
    assert!(game.new_logs.is_empty());

    let loaded = game_repo::load(&mut conn, board_id).await.unwrap();
    assert_eq!(loaded, game);

    // Changes to the mobs and a combat are saved too.
    let mut game = loaded;
    game.mobs.remove(0);
    game.board.current_hero_id = game.heroes[0].id;
    game.combat = Some(meq_online::model::combat::Combat {
        hero_id: game.heroes[0].id.unwrap(),
        mob_id: game.mobs[0].id.unwrap(),
        hero_cards: vec![PlayedCard::exhausted()],
        ..Default::default()
    });
    game_repo::save(&mut conn, &mut game).await.unwrap();
    assert_eq!(game_repo::load(&mut conn, board_id).await.unwrap(), game);
}
