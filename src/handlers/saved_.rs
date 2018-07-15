extern crate actix_web;
extern crate askama;
extern crate postgres;

use self::actix_web::{Form, HttpRequest, HttpResponse};
use self::askama::Template;
use super::super::state;
use super::error::Error;
use super::go;

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
struct SavedTpl {
    msg: String,
    posts: Vec<Posts>,
    urls: Vec<Url>,
}

pub fn saved(req: HttpRequest<state::AppState>) -> Result<HttpResponse, Error> {
    let msg = req.query().get("msg").unwrap_or("").to_string();
    let conn = req.state().db.get()?;
    let prep_stmt = conn.prepare(include_str!("../../sql/select_saved_posts.sql"))?;
    let posts: Vec<Posts> = prep_stmt
        .query(&[])
        .unwrap()
        .iter()
        .map(|row| Posts {
            id: row.get(0),
            title: row.get(1),
            feed_title: row.get(2),
        })
        .collect();
    go::render(&SavedTpl {
        posts: posts,
        urls: get_urls(&req)?,
        msg: msg,
    })
}

pub fn toggle_saved(req: HttpRequest<state::AppState>) -> Result<HttpResponse, Error> {
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
    (params, req): (Form<MarkAsReadParams>, HttpRequest<state::AppState>),
) -> Result<HttpResponse, Error> {
    let conn = req.state().db.get()?;
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

fn get_urls(req: &HttpRequest<state::AppState>) -> Result<Vec<Url>, Error> {
    let conn = req.state().db.get()?;
    let prep_stmt = conn.prepare(include_str!("../../sql/select_urls.sql"))?;
    Ok(prep_stmt
        .query(&[])
        .unwrap()
        .iter()
        .map(|row| Url {
            id: row.get(0),
            link: row.get(1),
            title: row.get(2),
        })
        .collect())
}
