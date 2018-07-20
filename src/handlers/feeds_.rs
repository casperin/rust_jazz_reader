extern crate actix_web;
extern crate askama;
extern crate serde;

use self::actix_web::{Form, HttpResponse, Path, State};
use self::askama::Template;
use super::super::rss;
use super::super::state::AppState;
use super::error::Error;
use super::go;

struct Feed {
    id: i32,
    title: String,
}

#[derive(Template)]
#[template(path = "feeds.html")]
struct FeedsTpl {
    feeds: Vec<Feed>,
    error: String,
}

pub fn feeds(state: State<AppState>) -> Result<HttpResponse, Error> {
    go::render(&FeedsTpl {
        feeds: get_feeds(&state)?,
        error: String::new(),
    })
}

struct FeedPost {
    id: i32,
    title: String,
    saved: bool,
}

#[derive(Template)]
#[template(path = "feed.html")]
struct FeedTpl {
    id: i32,
    title: String,
    url: String,
    posts: Vec<FeedPost>,
}

#[derive(Deserialize)]
pub struct FeedParams {
    id: i32,
}

pub fn feed((params, state): (Path<FeedParams>, State<AppState>)) -> Result<HttpResponse, Error> {
    let conn = state.db.get()?;

    // Get feed
    let prep_feed_stmt = conn.prepare(include_str!("../../sql/select_feed.sql"))?;
    let result = prep_feed_stmt.query(&[&params.id]).unwrap();
    if result.is_empty() {
        return show_msg(&state, String::from("Could not find that feed."));
    }
    let row = result.get(0);
    let feed_title = row.get(0);
    let feed_url = row.get(1);

    // Get posts
    let prep_posts_stmt = conn.prepare(include_str!("../../sql/select_feed_posts.sql"))?;
    let posts: Vec<FeedPost> = prep_posts_stmt
        .query(&[&params.id])
        .unwrap()
        .iter()
        .map(|row| FeedPost {
            id: row.get(0),
            title: row.get(1),
            saved: row.get(2),
        })
        .collect();

    go::render(&FeedTpl {
        id: params.id,
        title: feed_title,
        url: feed_url,
        posts: posts,
    })
}

#[derive(Template)]
#[template(path = "feed_preview.html")]
struct PreviewFeedsTpl {
    feed: rss::feed::Feed,
}

#[derive(Deserialize)]
pub struct PreviewParams {
    url: String,
}

pub fn preview_feed(
    (params, state): (Form<PreviewParams>, State<AppState>),
) -> Result<HttpResponse, Error> {
    match rss::feed::fetch(&params.url) {
        Ok(feed) => go::render(&PreviewFeedsTpl { feed: feed }),
        Err(err) => go::render(&FeedsTpl {
            feeds: get_feeds(&state)?,
            error: err,
        }),
    }
}

pub fn add_feed(
    (params, state): (Form<PreviewParams>, State<AppState>),
) -> Result<HttpResponse, Error> {
    let feed_result = rss::feed::fetch(&params.url);
    if let Err(err) = feed_result {
        return show_msg(&state, err);
    }
    let feed = feed_result.unwrap();
    let conn = state.db.get()?;
    let insert_feed_stmt = conn.prepare(include_str!("../../sql/insert_feed.sql"))?;

    let feed_id: i32 = match insert_feed_stmt.query(&[&feed.url, &feed.title]) {
        Err(err) => {
            println!("Error: {}", err);
            return show_msg(&state, format!("Could not insert {}", feed.title));
        }
        Ok(rows) => rows.iter()
            .nth(0)
            .expect("Unwrap first row of inserting feed")
            .get(0),
    };

    for post in feed.posts.iter() {
        let r = conn.prepare(include_str!("../../sql/insert_post.sql"))?
            .execute(&[
                &post.guid,
                &feed_id,
                &feed.title,
                &post.title,
                &post.link,
                &post.author,
                &post.content,
            ]);
        if let Err(err) = r {
            println!(
                "Error inserting post {} from {}: {}",
                post.title, feed.title, err
            );
        }
    }

    go::to("/feeds")
}

#[derive(Deserialize)]
pub struct UnsubscribeParams {
    id: i32,
}

pub fn unsubscribe_feed(
    (params, state): (Form<UnsubscribeParams>, State<AppState>),
) -> Result<HttpResponse, Error> {
    let conn = state.db.get()?;

    // Delete posts
    let prep_post_stmt = conn.prepare(include_str!("../../sql/delete_feed_posts.sql"))?;
    prep_post_stmt.execute(&[&params.id]).unwrap();

    // Delete feed
    let prep_feed_stmt = conn.prepare(include_str!("../../sql/delete_feed.sql"))?;
    prep_feed_stmt.execute(&[&params.id]).unwrap();

    show_msg(&state, String::from("Feed is gone. It will miss you."))
}

fn get_feeds(state: &AppState) -> Result<Vec<Feed>, Error> {
    let conn = state.db.get()?;
    let prep_stmt = conn.prepare(include_str!("../../sql/select_feeds.sql"))?;
    Ok(prep_stmt
        .query(&[])
        .unwrap()
        .iter()
        .map(|row| Feed {
            id: row.get(0),
            title: row.get(1),
        })
        .collect())
}

fn show_msg(state: &AppState, err: String) -> Result<HttpResponse, Error> {
    go::render(&FeedsTpl {
        feeds: get_feeds(state)?,
        error: err,
    })
}
