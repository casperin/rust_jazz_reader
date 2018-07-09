extern crate rss;

pub struct Feed {
    pub title: String,
    pub url: String,
    pub posts: Vec<Post>,
}

pub struct Post {
    pub guid: String,
    pub title: String,
    pub link: String,
    pub author: String,
    pub content: String,
}

pub fn fetch(url: &String) -> Result<Feed, String> {
    let chan = match rss::Channel::from_url(&url) {
        Ok(chan) => chan,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    Ok(Feed {
        title: chan.title().to_string(),
        url: url.to_string(),
        posts: chan.items().iter().map(|item| to_post(url, item)).collect(),
    })
}

fn to_post(feed_url: &String, item: &rss::Item) -> Post {
    let title = item.title()
        .map(|s| s.to_string())
        .unwrap_or(String::from("[No title]"));
    let link = item.link()
        .map(|s| s.to_string())
        .unwrap_or(feed_url.to_string());
    Post {
        guid: item.guid()
            .map(|guid| guid.value().to_string())
            .unwrap_or(format!("{}-{}", title, link)),
        title: title,
        link: link,
        author: item.author()
            .map(|s| s.to_string())
            .unwrap_or(String::new()),
        content: item.content()
            .or(item.description())
            .map(|s| s.to_string())
            .unwrap_or(String::new()),
    }
}
