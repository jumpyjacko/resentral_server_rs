use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use fantoccini::{ClientBuilder, Locator};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        .route("/daily_timetable", post(daily_timetable));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

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
        .expect("Couldn't scrape");

    (StatusCode::OK, timetable)
}

async fn scrape_daily_timetable(
    username: String,
    password: String,
) -> Result<Json<DailyTimetable>, fantoccini::error::CmdError> {
    let mut caps = serde_json::map::Map::new();
    let opts = serde_json::json!({ "args": ["--headless"] });
    caps.insert("moz:firefoxOptions".to_string(), opts.clone());
    let c = ClientBuilder::native()
        .capabilities(caps)
        .connect("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver");

    c.goto("https://theforest-h.sentral.com.au/portal/login")
        .await?;

    let f = c.form(Locator::Css("#login-form")).await?;
    f.set_by_name("username", &username).await?;
    f.set_by_name("password", &password).await?;
    f.submit().await?;

    // TODO: All the scraping lol
    let table = c.find(Locator::Css(".timetable")).await?;
    let periods = table.find_all(Locator::Css(".timetable-dayperiod")).await?;

    for p in periods {
        let pt = p.text().await?;
        println!("{}", pt);
    }

    c.close().await?;

    Ok(Json(DailyTimetable {
        periods: vec![Period {
            period: "test".to_owned(),
            subject: "test".to_owned(),
            room: "test".to_owned(),
            teacher: "test".to_owned(),
            colour: "#test".to_owned(),
        }],
    }))
}

#[derive(Serialize)]
struct Period {
    period: String,
    subject: String,
    room: String,
    teacher: String,
    colour: String,
}

#[derive(Serialize)]
struct DailyTimetable {
    periods: Vec<Period>,
}

#[derive(Deserialize)]
struct User {
    username: String,
    password: String,
}
