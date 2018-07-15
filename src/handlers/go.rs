extern crate actix_web;
extern crate askama;

use askama::Template;

use self::actix_web::{http, HttpResponse};

pub fn to(path: &str) -> HttpResponse {
    HttpResponse::Found()
        .header(http::header::LOCATION, path)
        .finish()
}

pub fn render(tpl: &Template) -> HttpResponse {
    let s = tpl.render().unwrap();
    HttpResponse::Ok().content_type("text/html").body(s)
}
