extern crate actix_web;
extern crate askama;

use self::actix_web::{HttpRequest, HttpResponse};
use super::super::state;
use askama::Template;

#[derive(Template)]
#[template(path = "404.html")]
struct ErrorTpl<'a> {
    error: &'a String,
}

#[derive(Template)]
#[template(path = "read.html")]
struct ReadTpl<'a> {
    title: &'a String,
    feed_title: &'a String,
    link: &'a String,
    content: &'a String,
}

pub fn read(req: HttpRequest<state::AppState>) -> HttpResponse {
    let id = req.match_info().get("id").expect("get id");
    let id: i32 = match id.parse() {
        Ok(n) => n,
        Err(e) => {
            println!("{:?}", e);
            let s = ErrorTpl {
                error: &e.to_string(),
            }.render()
                .unwrap();
            return HttpResponse::Ok().content_type("text/html").body(s);
        }
    };
    let conn = req.state().db.get().expect("get db");
    let prep_stmt = conn.prepare(include_str!("../../sql/select_post.sql"))
        .expect("prepare get by id statement");
    let result = prep_stmt.query(&[&id]).unwrap();
    if result.is_empty() {
        let s = ErrorTpl {
            error: &"Could not find post".to_string(),
        }.render()
            .unwrap();

        return HttpResponse::Ok().content_type("text/html").body(s);
    };
    let row = result.get(0);
    let s = ReadTpl {
        title: &row.get(0),
        feed_title: &row.get(1),
        link: &row.get(2),
        content: &row.get(3),
    }.render()
        .expect("render readtpl");
    HttpResponse::Ok().content_type("text/html").body(s)
}
