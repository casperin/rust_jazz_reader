extern crate actix_web;
extern crate askama;

use super::error::Error;
use askama::Template;

use self::actix_web::{http, HttpResponse, Result};

pub fn to(path: &str) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, path)
        .finish())
}

pub fn render(tpl: &Template) -> Result<HttpResponse, Error> {
    let s = tpl.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
