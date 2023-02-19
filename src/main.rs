use crate::announcements::scrape_announcements;
use crate::daily_timetable::scrape_daily_timetable;
use crate::full_timetable::scrape_full_timetable;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use fantoccini::{ClientBuilder, Locator};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, net::SocketAddr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod announcements;
mod daily_timetable;
mod full_timetable;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "resentral_server=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/daily_timetable", post(daily_timetable))
        .route("/announcements", post(announcements))
        .route("/full_timetable", post(full_timetable));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::debug!("listening on port {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start up server");
}

async fn root() -> &'static str {
    "Nothing at root, try other routes (maybe i'll put a route index here)"
}

async fn daily_timetable(Json(payload): Json<User>) -> impl IntoResponse {
    let response = User {
        username: payload.username,
        password: payload.password,
    };

    let timetable = scrape_daily_timetable(response.username, response.password)
        .await
        .expect("Couldn't scrape daily timetable");

    (StatusCode::OK, timetable)
}

async fn announcements(Json(payload): Json<User>) -> impl IntoResponse {
    let response = User {
        username: payload.username,
        password: payload.password,
    };

    let announcements = scrape_announcements(response.username, response.password)
        .await
        .expect("Couldn't scrape announcements");

    (StatusCode::OK, announcements)
}

async fn full_timetable(Json(payload): Json<User>) -> impl IntoResponse {
    let response = User {
        username: payload.username,
        password: payload.password,
    };

    let announcements = scrape_full_timetable(response.username, response.password)
        .await
        .expect("Couldn't scrape full timetable");

    (StatusCode::OK, announcements)
}

#[derive(Deserialize)]
struct User {
    username: String,
    password: String,
}
