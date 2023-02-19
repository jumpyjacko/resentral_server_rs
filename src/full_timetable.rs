use crate::daily_timetable::Period;
use fantoccini::wd::Capabilities;

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
    let arg =
        serde_json::json!({"args": ["--no-sandbox", "--headless", "--disable-dev-shm-usage"]});
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

    c.wait()
        .for_element(Locator::Css(".colour-timetable"))
        .await?; // Possibly bad

    c.find(Locator::Css(".colour-timetable"))
        .await?
        .click()
        .await?;

    let mut full_timetabe: Vec<Week> = Vec::new();

    let table = c.find(Locator::Css(".timetable")).await?;

    let periods = table.find_all(Locator::Css(".timetable-dayperiod")).await?;
    let days = table.find_all(Locator::Css(".timetable-day")).await?;

    let amount_of_weeks = days.len() / 5; // Will break if using an irregular week, i.e. includes weekends
    let days_in_week = days.len() / amount_of_weeks;

    let mut weeks = split_vec(periods, amount_of_weeks);

    for (idx, week) in weeks.iter().enumerate() {
        // for (idx1, day) in week.iter().enumerate() {
        
        // }
        println!();
        println!();
        for thing in week.iter().step_by(days_in_week) {
            println!("{:?} ", thing.text().await?);
        }
    }
    
    todo!();
    // Ok(Json(FullTimetable { weeks: week }))
}

// Thank you ChatGPT for existing
fn split_vec<T: Clone>(v: Vec<T>, n: usize) -> Vec<Vec<T>> {
    let len = v.len();
    let chunk_size = (len as f64 / n as f64).ceil() as usize;

    let mut result = Vec::new();
    let mut start = 0;

    for _ in 0..n {
        let end = (start + chunk_size).min(len);
        result.push(v[start..end].to_vec());
        start = end;
    }

    result
}
