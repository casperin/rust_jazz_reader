extern crate actix_web;
extern crate askama;
extern crate serde;

use self::actix_web::{Form, HttpRequest, HttpResponse};
use self::askama::Template;
use super::super::rss;
use super::super::state;
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

pub fn feeds(req: HttpRequest<state::AppState>) -> Result<HttpResponse, Error> {
    go::render(&FeedsTpl {
        feeds: get_feeds(&req)?,
        error: String::new(),
    })
}

struct FeedPost {
    id: i32,
    title: String,
}

#[derive(Template)]
#[template(path = "feed.html")]
struct FeedTpl {
    id: i32,
    title: String,
    url: String,
    posts: Vec<FeedPost>,
}

pub fn feed(req: HttpRequest<state::AppState>) -> Result<HttpResponse, Error> {
    let id: i32 = req.match_info().get("id").expect("get id").parse()?;
    let conn = req.state().db.get()?;

    // Get feed
    let prep_feed_stmt = conn.prepare(include_str!("../../sql/select_feed.sql"))?;
    let result = prep_feed_stmt.query(&[&id]).unwrap();
    if result.is_empty() {
        return show_msg(&req, String::from("Could not find that feed."));
    }
    let row = result.get(0);
    let feed_title = row.get(0);
    let feed_url = row.get(1);

    // Get posts
    let prep_posts_stmt = conn.prepare(include_str!("../../sql/select_feed_posts.sql"))?;
    let posts: Vec<FeedPost> = prep_posts_stmt
        .query(&[&id])
        .unwrap()
        .iter()
        .map(|row| FeedPost {
            id: row.get(0),
            title: row.get(1),
        })
        .collect();

    go::render(&FeedTpl {
        id: id,
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
    (params, req): (Form<PreviewParams>, HttpRequest<state::AppState>),
) -> Result<HttpResponse, Error> {
    match rss::feed::fetch(&params.url) {
        Ok(feed) => go::render(&PreviewFeedsTpl { feed: feed }),
        Err(err) => go::render(&FeedsTpl {
            feeds: get_feeds(&req)?,
            error: err,
        }),
    }
}

pub fn add_feed(
    (params, req): (Form<PreviewParams>, HttpRequest<state::AppState>),
) -> Result<HttpResponse, Error> {
    let feed_result = rss::feed::fetch(&params.url);

    if let Err(err) = feed_result {
        return show_msg(&req, err);
    }

    let feed = feed_result.unwrap();
    let conn = req.state().db.get()?;
    let insert_feed_stmt = conn.prepare(include_str!("../../sql/insert_feed.sql"))?;

    let feed_id: i32 = match insert_feed_stmt.query(&[&feed.url, &feed.title]) {
        Err(err) => {
            println!("Error: {}", err);
            return show_msg(&req, format!("Could not insert {}", feed.title));
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
    (params, req): (Form<UnsubscribeParams>, HttpRequest<state::AppState>),
) -> Result<HttpResponse, Error> {
    let conn = req.state().db.get()?;

    // Delete posts
    let prep_post_stmt = conn.prepare(include_str!("../../sql/delete_feed_posts.sql"))?;
    prep_post_stmt.execute(&[&params.id]).unwrap();

    // Delete feed
    let prep_feed_stmt = conn.prepare(include_str!("../../sql/delete_feed.sql"))?;
    prep_feed_stmt.execute(&[&params.id]).unwrap();

    show_msg(&req, String::from("Feed is gone. It will miss you."))
}

fn get_feeds(req: &HttpRequest<state::AppState>) -> Result<Vec<Feed>, Error> {
    let conn = req.state().db.get()?;
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

fn show_msg(req: &HttpRequest<state::AppState>, err: String) -> Result<HttpResponse, Error> {
    go::render(&FeedsTpl {
        feeds: get_feeds(req)?,
        error: err,
    })
}
