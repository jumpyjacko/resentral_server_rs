use fantoccini::wd::Capabilities;

use crate::*;

#[derive(Serialize)]
pub struct Period {
    pub period: String,
    pub subject: String,
    pub subject_short: String,
    pub room: String,
    pub teacher: String,
    pub colour: String,
}

#[derive(Serialize)]
pub struct DailyTimetable {
    pub periods: Vec<Period>,
}

pub async fn scrape_daily_timetable(
    username: String,
    password: String,
    website: String,
) -> Result<Json<DailyTimetable>, fantoccini::error::CmdError> {
    let arg =
        serde_json::json!({"args": ["--no-sandbox", "--headless", "--disable-dev-shm-usage"]});
    let mut cap = Capabilities::new();
    cap.insert("goog:chromeOptions".to_string(), arg);
    let c = ClientBuilder::native()
        .capabilities(cap)
        .connect("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver");

    c.goto(&website).await?;

    let f = c.form(Locator::Css("#login-form")).await?;
    f.set_by_name("username", &username).await?;
    f.set_by_name("password", &password).await?;
    f.submit().await?;

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

        timetable.push(Period {
            period: period_name,
            subject,
            subject_short,
            room,
            teacher,
            colour,
        });
    }

    c.close().await?;
    timetable.pop();

    Ok(Json(DailyTimetable { periods: timetable }))
}
