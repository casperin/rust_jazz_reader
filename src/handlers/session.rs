extern crate actix_web;
extern crate askama;
extern crate serde;

use self::actix_web::middleware::identity::RequestIdentity;
use self::actix_web::{Form, HttpRequest, HttpResponse};
use self::askama::Template;
use super::super::state::{AppState, SETTINGS};
use super::error::Error;
use super::go;

#[derive(Template)]
#[template(path = "login.html")]
struct Tpl {
    error: String,
}

pub fn login(_req: HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    go::render(&Tpl {
        error: "".to_string(),
    })
}

pub fn logout(mut req: HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    req.forget();
    go::to("/login")
}

#[derive(Deserialize)]
pub struct LoginParams {
    password: String,
}

pub fn perform_login(
    (params, mut req): (Form<LoginParams>, HttpRequest<AppState>),
) -> Result<HttpResponse, Error> {
    let password = SETTINGS.get::<String>("password").unwrap();
    if params.password == password {
        req.remember("logged-in".to_owned());
        return go::to("/");
    };

    go::render(&Tpl {
        error: "Wrong password :(".to_string(),
    })
}
