extern crate actix_web;
extern crate askama;
extern crate postgres;

use self::actix_web::{Form, HttpRequest, HttpResponse};
use self::askama::Template;
use super::super::state;
use super::redirect;

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

pub fn saved(req: HttpRequest<state::AppState>) -> HttpResponse {
    let msg = req.query().get("msg").unwrap_or("").to_string();
    let conn = req.state().db.get().expect("get db");
    let prep_stmt = conn.prepare(include_str!("../../sql/select_saved_posts.sql"))
        .expect("prepare get unread statement");
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
    let s = SavedTpl {
        posts: posts,
        urls: get_urls(&req),
        msg: msg,
    }.render()
        .unwrap();
    HttpResponse::Ok().content_type("text/html").body(s)
}

pub fn toggle_saved(req: HttpRequest<state::AppState>) -> HttpResponse {
    let to = format!("/{}", req.query().get("to").unwrap_or(""));
    let id = req.match_info().get("id").expect("get id");
    let id: i32 = match id.parse() {
        Ok(n) => n,
        Err(msg) => return redirect::to(&format!("{}?msg={}", to, msg)),
    };
    let conn = req.state().db.get().expect("get db");
    let prep_stmt = conn.prepare(include_str!("../../sql/toggle_saved.sql"))
        .expect("prepare get by id statement");
    let _ = prep_stmt.execute(&[&id]);
    redirect::to(&to)
}

#[derive(Deserialize)]
pub struct MarkAsReadParams {
    ids: String,
}

pub fn mark_all_as_read(
    (params, req): (Form<MarkAsReadParams>, HttpRequest<state::AppState>),
) -> HttpResponse {
    let conn = req.state().db.get().expect("get db");
    let prep_stmt = conn.prepare(include_str!("../../sql/mark_as_read.sql"))
        .expect("prepare mark all as read statement");
    for id_str in params.ids.split(",") {
        let id: i32 = match id_str.parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        let _ = prep_stmt.execute(&[&id]);
    }
    redirect::to("/")
}

fn get_urls(req: &HttpRequest<state::AppState>) -> Vec<Url> {
    let conn = req.state().db.get().expect("get db");
    let prep_stmt = conn.prepare(include_str!("../../sql/select_urls.sql"))
        .expect("prepare get urls statement");
    prep_stmt
        .query(&[])
        .unwrap()
        .iter()
        .map(|row| Url {
            id: row.get(0),
            link: row.get(1),
            title: row.get(2),
        })
        .collect()
}
