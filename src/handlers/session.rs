extern crate actix_web;
extern crate askama;
extern crate serde;

use self::actix_web::middleware::identity::RequestIdentity;
use self::actix_web::{http, Form, HttpRequest, HttpResponse};
use self::askama::Template;
use super::super::state;

#[derive(Template)]
#[template(path = "login.html")]
struct Tpl {
    error: String,
}

pub fn login(_req: HttpRequest<state::AppState>) -> HttpResponse {
    let s = Tpl {
        error: "".to_string(),
    }.render()
        .unwrap();
    HttpResponse::Ok().content_type("text/html").body(s)
}

pub fn logout(mut req: HttpRequest<state::AppState>) -> HttpResponse {
    req.forget();
    return HttpResponse::Found()
        .header(http::header::LOCATION, "/login")
        .finish();
}

#[derive(Deserialize)]
pub struct LoginParams {
    password: String,
}

pub fn perform_login(
    (params, mut req): (Form<LoginParams>, HttpRequest<state::AppState>),
) -> HttpResponse {
    let password = state::SETTINGS.get::<String>("password").unwrap();
    if params.password == password {
        req.remember("logged-in".to_owned());
        return HttpResponse::Found()
            .header(http::header::LOCATION, "/")
            .finish();
    };

    let s = Tpl {
        error: "Wrong password :(".to_string(),
    }.render()
        .unwrap();
    HttpResponse::Ok().content_type("text/html").body(s)
}
