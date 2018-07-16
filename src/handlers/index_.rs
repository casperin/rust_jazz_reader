extern crate actix_web;
extern crate askama;
extern crate r2d2;

use self::actix_web::{HttpResponse, Result, State};
use self::askama::Template;
use super::super::state::AppState;
use super::error::Error;
use super::go;

struct Posts {
    id: i32,
    title: String,
    feed_title: String,
    saved: bool,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTpl {
    ids: String,
    posts: Vec<Posts>,
}

pub fn index(state: State<AppState>) -> Result<HttpResponse, Error> {
    let db = state.db.get()?;
    let posts: Vec<Posts> = db.prepare(include_str!("../../sql/select_unread_posts.sql"))?
        .query(&[])
        .unwrap()
        .iter()
        .map(|row| Posts {
            id: row.get(0),
            title: row.get(1),
            feed_title: row.get(2),
            saved: row.get(3),
        })
        .collect();
    let ids: Vec<String> = posts.iter().map(|p| p.id.to_string()).collect();
    go::render(&IndexTpl {
        posts: posts,
        ids: ids.join(","),
    })
}
