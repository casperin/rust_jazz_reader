extern crate actix_web;
extern crate askama;
extern crate serde;

use self::actix_web::{Form, HttpRequest, HttpResponse};
use self::askama::Template;
use super::super::rss;
use super::super::state;
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

pub fn feeds(req: HttpRequest<state::AppState>) -> HttpResponse {
    go::render(&FeedsTpl {
        feeds: get_feeds(&req),
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

pub fn feed(req: HttpRequest<state::AppState>) -> HttpResponse {
    let id = req.match_info().get("id").expect("get id");
    let id: i32 = match id.parse() {
        Ok(id) => id,
        Err(err) => return show_error(&req, err.to_string()),
    };
    let conn = req.state().db.get().expect("get db");

    // Get feed
    let prep_feed_stmt = conn.prepare(include_str!("../../sql/select_feed.sql"))
        .expect("prepare get feed by id statement");
    let result = prep_feed_stmt.query(&[&id]).unwrap();
    if result.is_empty() {
        return show_error(&req, String::from("Could not find that feed."));
    }
    let row = result.get(0);
    let feed_title = row.get(0);
    let feed_url = row.get(1);

    // Get posts
    let prep_posts_stmt = conn.prepare(include_str!("../../sql/select_feed_posts.sql"))
        .expect("prepare get posts by id statement");
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
) -> HttpResponse {
    match rss::feed::fetch(&params.url) {
        Ok(feed) => go::render(&PreviewFeedsTpl { feed: feed }),
        Err(err) => go::render(&FeedsTpl {
            feeds: get_feeds(&req),
            error: err,
        }),
    }
}

pub fn add_feed(
    (params, req): (Form<PreviewParams>, HttpRequest<state::AppState>),
) -> HttpResponse {
    let feed_result = rss::feed::fetch(&params.url);

    if let Err(err) = feed_result {
        return show_error(&req, err);
    }

    let feed = feed_result.unwrap();
    let conn = req.state().db.get().expect("get db");
    let insert_feed_stmt = conn.prepare(include_str!("../../sql/insert_feed.sql"))
        .expect("prepare insert feed sql");

    let feed_id: i32 = match insert_feed_stmt.query(&[&feed.url, &feed.title]) {
        Err(err) => {
            println!("Error: {}", err);
            return show_error(&req, format!("Could not insert {}", feed.title));
        }
        Ok(rows) => rows.iter()
            .nth(0)
            .expect("Unwrap first row of inserting feed")
            .get(0),
    };

    for post in feed.posts.iter() {
        let r = conn.prepare(include_str!("../../sql/insert_post.sql"))
            .expect("prepare insert post sql")
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
) -> HttpResponse {
    let conn = req.state().db.get().expect("get db");

    // Delete posts
    let prep_post_stmt = conn.prepare(include_str!("../../sql/delete_feed_posts.sql"))
        .expect("prepare delete posts by feed_id statement");
    prep_post_stmt.execute(&[&params.id]).unwrap();

    // Delete feed
    let prep_feed_stmt = conn.prepare(include_str!("../../sql/delete_feed.sql"))
        .expect("prepare delete feed by id statement");
    prep_feed_stmt.execute(&[&params.id]).unwrap();

    show_error(&req, String::from("Feed is gone. It will miss you."))
}

fn get_feeds(req: &HttpRequest<state::AppState>) -> Vec<Feed> {
    let conn = req.state().db.get().expect("get db");
    let prep_stmt = conn.prepare(include_str!("../../sql/select_feeds.sql"))
        .expect("prepare get unread statement");
    prep_stmt
        .query(&[])
        .unwrap()
        .iter()
        .map(|row| Feed {
            id: row.get(0),
            title: row.get(1),
        })
        .collect()
}

fn show_error(req: &HttpRequest<state::AppState>, err: String) -> HttpResponse {
    go::render(&FeedsTpl {
        feeds: get_feeds(req),
        error: err,
    })
}
