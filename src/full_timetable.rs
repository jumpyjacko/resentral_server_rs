use fantoccini::wd::Capabilities;
use crate::daily_timetable::Period;

use crate::*;

#[derive(Serialize)]
pub struct FullTimetable {
    pub weeks: Vec<Week>,
}

#[derive(Serialize)]
pub struct Week {
    pub days: Vec<Day>,
}

#[derive(Serialize)]
pub struct Day {
    pub periods: Vec<Period>,
    pub day: u32,
}

pub async fn scrape_full_timetable(
    username: String,
    password: String,
) -> Result<Json<FullTimetable>, fantoccini::error::CmdError> {
    

    let arg = serde_json::json!({"args": ["--no-sandbox", /*"--headless",*/ "--disable-dev-shm-usage"]});
    let mut cap = Capabilities::new();
    cap.insert("goog:chromeOptions".to_string(), arg);
    let c = ClientBuilder::native()
        .capabilities(cap)
        .connect("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver");

    c.goto("https://theforest-h.sentral.com.au/portal/login")
        .await?;

    let f = c.form(Locator::Css("#login-form")).await?;
    f.set_by_name("username", &username).await?;
    f.set_by_name("password", &password).await?;
    f.submit().await?;

    c.wait().for_element(Locator::Css(".colour-timetable")).await?; // Possibly bad

    c.find(Locator::Css(".colour-timetable")).await?.click().await?;
    
    todo!();
    // Ok(Json(FullTimetable { weeks: week }))
}