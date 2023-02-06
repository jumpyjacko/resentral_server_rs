use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use fantoccini::{ClientBuilder, Locator};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, collections::VecDeque};
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

    // TODO: Need to find some Regex
    let mut timetable: Vec<Period> = Vec::new();

    let table = c.find(Locator::Css(".timetable")).await?;
    let periods = table.find_all(Locator::Css(".timetable-dayperiod")).await?;
    let mut period_names = table.find_all(Locator::Css(".timetable-period")).await?;

    for period in periods.iter().zip(period_names.iter_mut()) {
        let period_text = period.0.text().await?;
        let period_name = period.1.text().await?;
        let elems = period.0.find_all(Locator::Css("div")).await?;

        let mut subject: String = match period_text.lines().next() {
            Some(subject) => subject.to_owned(),
            None => "".to_owned(),
        };
        let subject_short = match subject.split_whitespace().last() {
            Some(subject) => subject.to_owned(),
            None => "".to_owned(),
        };
        subject = subject.replace(&subject_short, "");

        let rest_of_period_text = match period_text.lines().last() {
            Some(subject) => subject.to_owned(),
            None => "".to_owned(),
        };

        let room = match rest_of_period_text.split_whitespace().nth(1) {
            Some(room) => room.to_owned(),
            None => "".to_owned(),
        };

        let mut teacher = match rest_of_period_text.split_whitespace().nth(3) {
            Some(teacher) => teacher.to_owned(),
            None => "".to_owned(),
        };
        let teacher_1 = match rest_of_period_text.split_whitespace().nth(4) {
            Some(teacher) => teacher.to_owned(),
            None => "".to_owned(),
        };
        teacher.push(' ');
        teacher.push_str(&teacher_1);

        let mut css: VecDeque<String> = VecDeque::new();
        for elem in elems {
            css.push_back(elem.css_value("border-left-color").await?);
        }
        let colour: String = match css.pop_front() {
            Some(colour) => colour,
            None => "".to_owned(),
        };

        timetable.push(Period { period: period_name, subject, subject_short, room, teacher, colour });
    }

    c.close().await?;

    Ok(Json(DailyTimetable {
        periods: timetable
    }))
}

#[derive(Serialize)]
struct Period {
    period: String,
    subject: String,
    subject_short: String,
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
