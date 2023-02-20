use crate::*;

#[derive(Serialize)]
pub struct Announcement {
    pub name: String,
    // pub date: String,
    // pub time: String,
    pub title: String,
    pub body: String,
}

#[derive(Serialize)]
pub struct Announcements {
    pub announcements: Vec<Announcement>,
}

pub async fn scrape_announcements(
    username: String,
    password: String,
    website: String,
) -> Result<Json<Announcements>, fantoccini::error::CmdError> {
    let arg =
        serde_json::json!({"args": ["--no-sandbox", "--headless", "--disable-dev-shm-usage"]});
    let mut cap = fantoccini::wd::Capabilities::new();
    cap.insert("goog:chromeOptions".to_string(), arg);
    let c = ClientBuilder::native()
        .capabilities(cap)
        .connect("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver");

    c.goto(&website)
        .await?;

    let f = c.form(Locator::Css("#login-form")).await?;
    f.set_by_name("username", &username).await?;
    f.set_by_name("password", &password).await?;
    f.submit().await?;

    let mut announcements: Vec<Announcement> = Vec::new();

    let notice_wrap = c.find_all(Locator::Css(".notice-wrap")).await?;

    for announcement in notice_wrap {
        let header = announcement
            .find(Locator::Css(".notice-header"))
            .await?
            .text()
            .await?;
        let body = announcement
            .find(Locator::Css(".notice-content"))
            .await?
            .text()
            .await?;

        let title: String = match header.lines().next() {
            Some(title) => title.to_owned(),
            None => "".to_owned(),
        };

        let name = announcement
            .find(Locator::Css(".small-caps > strong"))
            .await?
            .text()
            .await?;

        announcements.push(Announcement { name, title, body });
    }

    c.close().await?;

    Ok(Json(Announcements { announcements }))
}
