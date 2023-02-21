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
    pub day: String,
}

pub async fn scrape_full_timetable(
    username: String,
    password: String,
    website: String,
) -> Result<Json<FullTimetable>, fantoccini::error::CmdError> {
    let arg =
        serde_json::json!({"args": ["--no-sandbox", /*"--headless",*/ "--disable-dev-shm-usage"]});
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

    c.wait()
        .for_element(Locator::Css(".colour-timetable"))
        .await?; // Actually very bad, very possible to timeout

    c.find(Locator::Css(".colour-timetable"))
        .await?
        .click()
        .await?;

    let table = c.find(Locator::Css(".timetable")).await?;

    let periods = table.find_all(Locator::Css(".timetable-dayperiod")).await?;
    let days_table = table.find_all(Locator::Css(".timetable-day")).await?;

    let amount_of_weeks = days_table.len() / 5; // Will break if using an irregular week, i.e. includes weekends
    let days_in_week = days_table.len() / amount_of_weeks;

    let weeks = split_vec(periods, amount_of_weeks);

    let mut full_timetable: Vec<Week> = Vec::new();

    for (week_counter, week) in weeks.iter().enumerate() {
        let mut days: Vec<Day> = Vec::new();
        for i in 0..days_in_week {
            let mut periods: Vec<Period> = Vec::new();

            for day in week.iter().skip(i).step_by(days_in_week) {
                let elems = day.find_all(Locator::Css("div")).await?;
                let day_text = day.text().await?;

                let mut css: VecDeque<String> = VecDeque::new();
                for elem in elems {
                    css.push_back(elem.css_value("border-left-color").await?);
                }
                let colour: String = match css.pop_front() {
                    Some(colour) => colour,
                    None => "".to_owned(),
                };

                let mut subject: String = match day_text.lines().next() {
                    Some(subject) => subject.to_owned(),
                    None => "".to_owned(),
                };
                let subject_short = match subject.split_whitespace().last() {
                    Some(subject) => subject.to_owned(),
                    None => "".to_owned(),
                };
                subject = subject.replace(&subject_short, "");

                let rest_of_text = match day_text.lines().last() {
                    Some(subject) => subject.to_owned(),
                    None => "".to_owned(),
                };

                let room = match rest_of_text.split_whitespace().nth(1) {
                    Some(room) => room.to_owned(),
                    None => "".to_owned(),
                };

                let mut teacher = match rest_of_text.split_whitespace().nth(3) {
                    Some(teacher) => teacher.to_owned(),
                    None => "".to_owned(),
                };
                let teacher_1 = match rest_of_text.split_whitespace().nth(4) {
                    Some(teacher) => teacher.to_owned(),
                    None => "".to_owned(),
                };
                teacher.push(' ');
                teacher.push_str(&teacher_1);

                let parent_element = day.find(Locator::XPath("./..")).await?;

                let period = parent_element
                    .find(Locator::Css(".timetable-period"))
                    .await?
                    .text()
                    .await?;

                periods.push(Period {
                    period,
                    subject,
                    subject_short,
                    room,
                    teacher,
                    colour,
                });
            }

            periods.pop();
            days.push(Day {
                periods,
                day: days_table[(week_counter * days_in_week) + 1].text().await?,
            });
        }

        full_timetable.push(Week { days });
    }

    Ok(Json(FullTimetable {
        weeks: full_timetable,
    }))
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
