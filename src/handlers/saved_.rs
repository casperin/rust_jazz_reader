extern crate actix_web;
extern crate askama;

use self::actix_web::{http, HttpRequest, HttpResponse};
use self::askama::Template;
use super::super::state;

struct Posts {
    id: i64,
    title: String,
    feed_title: String,
}

#[derive(Template)]
#[template(path = "saved.html")]
struct SavedTpl {
    posts: Vec<Posts>,
}

pub fn saved(req: HttpRequest<state::AppState>) -> HttpResponse {
    let stmt = "select id, title, feed_title from posts where saved='1' order by id desc";
    let conn = req.state().db.get().expect("get db");
    let prep_stmt = conn.prepare(stmt).expect("prepare get unread statement");
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
    let s = SavedTpl { posts: posts }.render().unwrap();
    HttpResponse::Ok().content_type("text/html").body(s)
}

pub fn toggle_saved(req: HttpRequest<state::AppState>) -> HttpResponse {
    let to = format!("/{}", req.query().get("to").unwrap_or(""));
    let id = req.match_info().get("id").expect("get id");
    let id: i64 = match id.parse() {
        Ok(n) => n,
        Err(e) => {
            println!("{:?}", e);
            return HttpResponse::Found()
                .header(http::header::LOCATION, to)
                .finish();
        }
    };
    let stmt = "update posts set saved = not saved where id = $1";
    let conn = req.state().db.get().expect("get db");
    let prep_stmt = conn.prepare(stmt).expect("prepare get by id statement");
    let _ = prep_stmt.query(&[&id]).unwrap();
    HttpResponse::Found()
        .header(http::header::LOCATION, to)
        .finish()
}
