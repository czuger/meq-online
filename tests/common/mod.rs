//! In-process test server: router on an in-memory database, requests without sockets.

// Test support: failing fast with unwrap is the point here.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use meq_online::app_state::AppState;
use meq_online::board_versions::BoardVersions;
use meq_online::db::{game_repo, pool};
use meq_online::game_data::GameData;
use meq_online::model::game::Game;
use meq_online::web::router::router;
use tower::ServiceExt;

pub mod driver;

pub struct TestApp {
    pub router: Router,
    pub state: AppState,
}

pub struct Answer {
    pub status: StatusCode,
    pub location: Option<String>,
    pub set_cookie: Vec<String>,
    pub content_type: Option<String>,
    pub body: String,
}

impl TestApp {
    pub async fn new() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let state = AppState {
            db: pool::connect("sqlite::memory:").await.unwrap(),
            data: Arc::new(GameData::load(&root.join("data")).unwrap()),
            versions: BoardVersions::new(),
        };
        Self {
            router: router(state.clone(), &root.join("static")),
            state,
        }
    }

    pub async fn request(
        &self,
        method: Method,
        path: &str,
        cookie: Option<&str>,
        form: Option<&str>,
    ) -> Answer {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(cookie) = cookie {
            builder = builder.header(header::COOKIE, cookie);
        }
        let body = match form {
            Some(form) => {
                builder = builder.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
                Body::from(form.to_owned())
            }
            None => Body::empty(),
        };
        let response = self
            .router
            .clone()
            .oneshot(builder.body(body).unwrap())
            .await
            .unwrap();
        let header = |name| {
            response
                .headers()
                .get(name)
                .map(|v: &header::HeaderValue| v.to_str().unwrap().to_owned())
        };
        let location = header(header::LOCATION);
        let content_type = header(header::CONTENT_TYPE);
        let set_cookie = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_owned())
            .collect();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        Answer {
            status,
            location,
            set_cookie,
            content_type,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        }
    }

    pub async fn get(&self, path: &str, cookie: &str) -> Answer {
        self.request(Method::GET, path, Some(cookie), None).await
    }

    pub async fn post(&self, path: &str, cookie: &str, form: &str) -> Answer {
        self.request(Method::POST, path, Some(cookie), Some(form))
            .await
    }

    pub async fn patch(&self, path: &str, cookie: &str, form: &str) -> Answer {
        self.request(Method::PATCH, path, Some(cookie), Some(form))
            .await
    }

    /// Logs in with the developer provider and returns the cookie header to send.
    pub async fn login(&self, name: &str) -> String {
        let form = format!("name={name}&email={name}%40example.org");
        let answer = self
            .request(Method::POST, "/auth/developer/callback", None, Some(&form))
            .await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER);
        let cookie = answer.set_cookie.first().expect("session cookie");
        cookie.split(';').next().unwrap().to_owned()
    }

    /// Creates a board as the logged-in user (who plays Sauron), with Beravor, Eleanor
    /// and Thalin on computer seats, and returns its game.
    pub async fn create_board(&self, cookie: &str) -> Game {
        let form = "heroes=beravor&heroes=eleanor&heroes=thalin&seats=computer";
        let answer = self.post("/boards", cookie, form).await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER);
        let id: i64 = sqlx::query_scalar("SELECT MAX(id) FROM boards")
            .fetch_one(&self.state.db)
            .await
            .unwrap();
        self.game(id).await
    }

    pub async fn game(&self, board_id: i64) -> Game {
        let mut conn = self.state.db.acquire().await.unwrap();
        game_repo::load(&mut conn, board_id).await.unwrap()
    }

    pub async fn save(&self, game: &mut Game) {
        let mut conn = self.state.db.acquire().await.unwrap();
        game_repo::save(&mut conn, game).await.unwrap();
    }
}
