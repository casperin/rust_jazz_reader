extern crate actix_web;
extern crate askama;

use self::actix_web::{HttpRequest, HttpResponse};
use self::askama::Template;
use super::super::state;

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

pub fn index(req: HttpRequest<state::AppState>) -> HttpResponse {
    let stmt = "select id, title, feed_title, saved from posts where read=false order by id desc";
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
            saved: row.get(3),
        })
        .collect();
    let ids: Vec<String> = posts.iter().map(|p| p.id.to_string()).collect();
    let s = IndexTpl {
        posts: posts,
        ids: ids.join(","),
    }.render()
        .unwrap();
    HttpResponse::Ok().content_type("text/html").body(s)
}
