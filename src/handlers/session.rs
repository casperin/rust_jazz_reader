extern crate actix_web;
extern crate askama;
extern crate serde;

use self::actix_web::middleware::identity::RequestIdentity;
use self::actix_web::{Form, HttpRequest, HttpResponse};
use self::askama::Template;
use super::super::state;
use super::go;

#[derive(Template)]
#[template(path = "login.html")]
struct Tpl {
    error: String,
}

pub fn login(_req: HttpRequest<state::AppState>) -> HttpResponse {
    go::render(&Tpl {
        error: "".to_string(),
    })
}

pub fn logout(mut req: HttpRequest<state::AppState>) -> HttpResponse {
    req.forget();
    go::to("/login")
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
        return go::to("/");
    };

    go::render(&Tpl {
        error: "Wrong password :(".to_string(),
    })
}
