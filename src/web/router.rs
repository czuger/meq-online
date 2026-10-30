//! URL routes. Screen routes match the board states they display (see
//! [`crate::engine::route`]).

use std::path::Path;

use axum::Router;
use axum::routing::{get, post};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::app_state::AppState;
use crate::web::handlers::{
    boards, characters, choices, combats, defeat, events, explorations, heroes, logs, maps,
    messages, play, plot_cards, sauron, sauron_actions, sauron_mobs, sessions, shadow_cards,
};

/// Builds the application router.
///
/// # Arguments
/// * `state` - shared state.
/// * `static_dir` - folder served under `/static`.
pub fn router(state: AppState, static_dir: &Path) -> Router {
    Router::new()
        .merge(board_routes())
        .merge(hero_routes())
        .merge(exploration_routes())
        .merge(sauron_routes())
        .merge(plot_card_routes())
        .merge(sauron_action_routes())
        .merge(combat_routes())
        .route_service(
            "/favicon.ico",
            ServeFile::new(static_dir.join("favicon.ico")),
        )
        .route_service("/robots.txt", ServeFile::new(static_dir.join("robots.txt")))
        .nest_service("/static", ServeDir::new(static_dir))
        .layer(request_tracing())
        .with_state(state)
}

/// Logs every request and its response (status, latency) at debug level. Headers stay
/// out: they carry the session cookie.
fn request_tracing()
-> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
        .on_response(DefaultOnResponse::new().level(Level::DEBUG))
}

fn board_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(boards::index))
        .route("/welcome", get(boards::welcome))
        .route("/sessions/new", get(sessions::new))
        .route(
            "/sessions",
            post(sessions::destroy).delete(sessions::destroy),
        )
        .route("/auth/developer/callback", post(sessions::create))
        .route("/auth/failure", get(sessions::failure))
        .route("/boards", get(boards::index).post(boards::create))
        .route("/boards/new", get(boards::new))
        .route(
            "/boards/{board_id}/join",
            get(boards::join_new).post(boards::join),
        )
        .route(
            "/boards/{board_id}/inactive_actor",
            get(play::inactive_actor),
        )
        .route("/boards/{board_id}/play", get(play::play))
        .route("/boards/{board_id}/status", get(play::status))
        .route("/boards/{board_id}/story_screen", get(boards::story_screen))
        .route("/boards/{board_id}/logs", get(logs::show))
        .route("/boards/{board_id}/events", get(events::stream))
        .route(
            "/boards/{board_id}/choices/choice_screen",
            get(choices::choice_screen),
        )
        .route("/boards/{board_id}/choices/choose", post(choices::choose))
        .route("/maps/{hero_id}", get(maps::show))
        .route("/maps/{sauron_id}/edit", get(maps::edit))
}

fn hero_routes() -> Router<AppState> {
    let hero = |path: &str| format!("/heroes/{{hero_id}}/{path}");
    Router::new()
        .route("/heroes/{hero_id}", get(heroes::show))
        .route(&hero("rest_screen"), get(heroes::rest_screen))
        .route(&hero("rest_rest"), post(heroes::rest_rest))
        .route(&hero("rest_heal"), post(heroes::rest_heal))
        .route(&hero("rest_train"), post(heroes::rest_train))
        .route(&hero("rest_skip"), post(heroes::rest_skip))
        .route(
            &hero("after_rest_advance_story_marker_screen"),
            get(heroes::after_rest_advance_story_marker_screen),
        )
        .route(
            &hero("after_rest_advance_story_marker"),
            post(heroes::after_rest_advance_story_marker),
        )
        .route(&hero("movement_screen"), get(heroes::movement_screen))
        .route(&hero("move"), post(heroes::move_hero))
        .route(&hero("movement_finished"), post(heroes::movement_finished))
        .route(
            &hero("argalad_power_screen"),
            get(heroes::argalad_power_screen),
        )
        .route(&hero("defeat_screen"), get(defeat::defeat_screen))
        .route(&hero("defeat"), post(defeat::defeat))
        .route(&hero("draw_cards_screen"), get(heroes::draw_cards_screen))
        .route(&hero("draw_cards"), post(heroes::draw_cards))
        .route(
            &hero("draw_cards_finished"),
            post(heroes::draw_cards_finished),
        )
        .route(
            &hero("board_messages"),
            get(messages::hero_index).post(messages::hero_create),
        )
        .route(&hero("board_messages/new"), get(messages::hero_new))
}

fn exploration_routes() -> Router<AppState> {
    let update = post(explorations::update)
        .put(explorations::update)
        .patch(explorations::update);
    Router::new()
        .route(
            "/heroes/{hero_id}/exploration",
            get(explorations::show).merge(update),
        )
        .route(
            "/heroes/{hero_id}/exploration/next_movement",
            post(explorations::next_movement),
        )
        .route(
            "/heroes/{hero_id}/exploration/next_step",
            post(explorations::next_step),
        )
}

fn sauron_routes() -> Router<AppState> {
    Router::new()
        .route("/sauron/{sauron_id}", get(sauron::show))
        .route(
            "/sauron/{sauron_id}/setup_screen",
            get(sauron::setup_screen),
        )
        .route(
            "/sauron/{sauron_id}/setup_finished",
            post(sauron::setup_finished),
        )
        .route(
            "/sauron/{sauron_id}/haven_screen",
            get(defeat::haven_screen),
        )
        .route("/sauron/{sauron_id}/haven", post(defeat::haven))
        .route(
            "/sauron/{sauron_id}/board_messages",
            get(messages::sauron_index).post(messages::sauron_create),
        )
        .route(
            "/sauron/{sauron_id}/board_messages/new",
            get(messages::sauron_new),
        )
        .route("/characters/{sauron_id}/edit", get(characters::edit))
        .route(
            "/characters/{sauron_id}",
            post(characters::update).patch(characters::update),
        )
}

fn plot_card_routes() -> Router<AppState> {
    let plot = |path: &str| format!("/sauron/{{sauron_id}}/plot_cards/{path}");
    Router::new()
        .route(&plot("play_screen"), get(plot_cards::play_screen))
        .route(&plot("play"), post(plot_cards::play))
        .route(&plot("play_finished"), post(plot_cards::play_finished))
        .route(&plot("discard_screen"), get(plot_cards::discard_screen))
        .route(&plot("discard"), post(plot_cards::discard))
        .route(&plot("keep_screen"), get(plot_cards::keep_screen))
        .route(&plot("keep"), post(plot_cards::keep))
        .route(
            &plot("look_for_gollum_cards"),
            get(plot_cards::look_for_gollum_cards),
        )
        .route(&plot("get_gollum_card"), post(plot_cards::get_gollum_card))
}

fn sauron_action_routes() -> Router<AppState> {
    let pool = post(sauron_actions::update_shadow_pool).patch(sauron_actions::update_shadow_pool);
    let mob_update = post(sauron_mobs::update)
        .put(sauron_mobs::update)
        .patch(sauron_mobs::update);
    Router::new()
        .route(
            "/sauron/{sauron_id}/sauron_actions/edit",
            get(sauron_actions::edit),
        )
        .route(
            "/sauron/{sauron_id}/sauron_actions/start",
            post(sauron_actions::start),
        )
        .route(
            "/sauron/{sauron_id}/sauron_actions/place_influence_screen",
            get(sauron_actions::place_influence_screen),
        )
        .route(
            "/sauron/{sauron_id}/sauron_actions/finish_action",
            post(sauron_actions::finish_action),
        )
        .route(
            "/sauron/{sauron_id}/sauron_actions/terminate",
            post(sauron_actions::terminate),
        )
        .route(
            "/sauron/{sauron_id}/sauron_actions/set_influence",
            post(sauron_actions::set_influence),
        )
        .route(
            "/sauron/{sauron_id}/shadow_cards/play",
            post(shadow_cards::play),
        )
        .route("/shadow_pools/{sauron_id}/update_from_map", pool)
        .route(
            "/sauron/{sauron_id}/sauron_mobs/command_screen",
            get(sauron_mobs::index),
        )
        .route(
            "/sauron/{sauron_id}/sauron_mobs",
            get(sauron_mobs::index).post(sauron_mobs::create),
        )
        .route("/sauron/{sauron_id}/sauron_mobs/new", get(sauron_mobs::new))
        .route("/sauron/{sauron_id}/sauron_mobs/{mob_id}", mob_update)
        .route(
            "/sauron/{sauron_id}/sauron_mobs/{mob_id}/edit",
            get(sauron_mobs::edit),
        )
        .route(
            "/sauron/{sauron_id}/sauron_mobs/{mob_id}/heal",
            post(sauron_mobs::heal),
        )
}

fn combat_routes() -> Router<AppState> {
    let combat = |path: &str| format!("/boards/{{board_id}}/combats/{path}");
    Router::new()
        .route("/boards/{board_id}/combats", get(combats::show))
        .route(
            &combat("combat_setup_screen"),
            get(combats::combat_setup_screen),
        )
        .route(&combat("combat_setup"), post(combats::combat_setup))
        .route(
            &combat("cards_loss_screen"),
            get(combats::cards_loss_screen),
        )
        .route(&combat("cards_loss"), post(combats::cards_loss))
        .route(
            &combat("declare_exhaustion_hero"),
            post(combats::declare_exhaustion_hero),
        )
        .route(
            &combat("declare_exhaustion_mob"),
            post(combats::declare_exhaustion_mob),
        )
        .route(
            &combat("{side}/play_combat_card_screen"),
            get(combats::play_combat_card_screen),
        )
        .route(
            &combat("play_combat_card_hero"),
            post(combats::play_combat_card_hero),
        )
        .route(
            &combat("play_combat_card_mob"),
            post(combats::play_combat_card_mob),
        )
        .route(&combat("terminate"), post(combats::terminate))
}
