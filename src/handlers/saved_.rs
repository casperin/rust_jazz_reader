extern crate actix_web;
extern crate askama;
extern crate postgres;

use self::actix_web::{Form, HttpRequest, HttpResponse, Query, State};
use self::askama::Template;
use super::super::state::AppState;
use super::error::Error;
use super::go;
use std::collections::HashMap;

struct Posts {
    id: i32,
    title: String,
    feed_title: String,
}

struct Url {
    id: i32,
    link: String,
    title: String,
}

#[derive(Template)]
#[template(path = "saved.html")]
struct SavedTpl<'a> {
    msg: &'a str,
    posts: Vec<Posts>,
    urls: Vec<Url>,
}

pub fn saved(
    (params, state): (Query<HashMap<String, String>>, State<AppState>),
) -> Result<HttpResponse, Error> {
    let conn = state.db.get()?;

    // Posts
    let posts = conn.prepare(include_str!("../../sql/select_saved_posts.sql"))?
        .query(&[])
        .unwrap()
        .iter()
        .map(|row| Posts {
            id: row.get(0),
            title: row.get(1),
            feed_title: row.get(2),
        })
        .collect();

    // Urls
    let urls = conn.prepare(include_str!("../../sql/select_urls.sql"))?
        .query(&[])
        .unwrap()
        .iter()
        .map(|row| Url {
            id: row.get(0),
            link: row.get(1),
            title: row.get(2),
        })
        .collect();

    go::render(&SavedTpl {
        posts: posts,
        urls: urls,
        msg: params.get("msg").unwrap_or(&String::from("")),
    })
}

pub fn toggle_saved(req: HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    let to = format!("/{}", req.query().get("to").unwrap_or(""));
    let id: i32 = req.match_info().get("id").expect("get id").parse()?;
    let conn = req.state().db.get()?;
    let prep_stmt = conn.prepare(include_str!("../../sql/toggle_saved.sql"))?;
    let _ = prep_stmt.execute(&[&id]);
    go::to(&to)
}

#[derive(Deserialize)]
pub struct MarkAsReadParams {
    ids: String,
}

pub fn mark_all_as_read(
    (params, state): (Form<MarkAsReadParams>, State<AppState>),
) -> Result<HttpResponse, Error> {
    let conn = state.db.get()?;
    let prep_stmt = conn.prepare(include_str!("../../sql/mark_as_read.sql"))?;
    for id_str in params.ids.split(",") {
        let id: i32 = match id_str.parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        let _ = prep_stmt.execute(&[&id]);
    }
    go::to("/")
}
