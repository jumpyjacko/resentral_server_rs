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
) -> Result<Json<Announcements>, fantoccini::error::CmdError> {
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

    let mut announcements: Vec<Announcement> = Vec::new();

    let mut notice_wrap = c.find_all(Locator::Css(".notice-wrap")).await?;

    for announcement in notice_wrap {
        let header = announcement.find(Locator::Css(".notice-header")).await?.text().await?;
        let body = announcement.find(Locator::Css(".notice-content")).await?.text().await?;
        
        let title: String = match header.lines().next() {
            Some(title) => title.to_owned(),
            None => "".to_owned(),
        };

        let name = announcement.find(Locator::Css(".small-caps > strong")).await?.text().await?;

        announcements.push(Announcement {
            name,
            title,
            body,
        });
    }

    c.close().await?;

    Ok(Json(Announcements { announcements }))
}