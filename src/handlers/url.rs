extern crate actix_web;
extern crate reqwest;
extern crate serde;

use self::actix_web::{Form, HttpRequest, HttpResponse};
use super::super::state;
use super::go;

#[derive(Deserialize)]
pub struct SaveUrlParams {
    url: String,
}

pub fn save_url(
    (params, req): (Form<SaveUrlParams>, HttpRequest<state::AppState>),
) -> HttpResponse {
    let body = match get_body(&params.url) {
        Ok(body) => body,
        Err(msg) => return go::to(&format!("/saved?msg={}", msg)),
    };
    let title = find_title(&body).unwrap_or(&params.url);
    let conn = req.state().db.get().expect("get db");
    let prep_stmt = conn.prepare(include_str!("../../sql/insert_url.sql"))
        .expect("prepare inserting url");
    let _ = prep_stmt.execute(&[&params.url, &title]);
    go::to("/saved")
}

// TODO: This abomination
fn get_body(url: &str) -> Result<String, &'static str> {
    let url = reqwest::Url::parse(url).map_err(|e| {
        println!("{}", e);
        "That seems like it was not a url"
    })?;
    reqwest::get(url)
        .map_err(|e| {
            println!("{}", e);
            "Could not make a get request"
        })?
        .error_for_status()
        .map_err(|e| {
            println!("{}", e);
            "Could not find that url"
        })?
        .text()
        .map_err(|e| {
            println!("{}", e);
            "Seems like there were no body"
        })
}

fn find_title(text: &str) -> Option<&str> {
    let start_bytes = text.find("<title>")? + 7;
    let mut result = &text[start_bytes..];
    if let Some(end) = result.find("</title>") {
        result = &text[start_bytes..start_bytes + end];
    }
    Some(result)
}

pub fn forget_url(req: HttpRequest<state::AppState>) -> HttpResponse {
    let id = req.match_info().get("id").expect("get id");
    let id: i32 = match id.parse() {
        Ok(n) => n,
        Err(msg) => return go::to(&format!("/saved?msg={}", msg)),
    };
    let conn = req.state().db.get().expect("get db");
    let prep_stmt = conn.prepare(include_str!("../../sql/forget_url.sql"))
        .expect("prepare delete url statement");
    let _ = prep_stmt.execute(&[&id]);
    go::to("/saved")
}
