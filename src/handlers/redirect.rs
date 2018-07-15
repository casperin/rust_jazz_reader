extern crate actix_web;

use self::actix_web::{http, HttpResponse};

pub fn to(path: &str) -> HttpResponse {
    HttpResponse::Found()
        .header(http::header::LOCATION, path)
        .finish()
}
