extern crate actix_web;
extern crate reqwest;
extern crate serde;

use self::actix_web::{Form, HttpResponse, Path, State};
use super::super::state::AppState;
use super::error::Error;
use super::go;

#[derive(Deserialize)]
pub struct SaveUrlParams {
    url: String,
}

pub fn save_url(
    (params, state): (Form<SaveUrlParams>, State<AppState>),
) -> Result<HttpResponse, Error> {
    let body = match get_body(&params.url) {
        Ok(body) => body,
        Err(msg) => return go::to(&format!("/saved?msg={}", msg)),
    };
    let title = find_title(&body).unwrap_or(&params.url);
    let conn = state.db.get().expect("get db");
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

#[derive(Deserialize)]
pub struct ForgetUrlParams {
    id: i32,
}

pub fn forget_url(
    (params, state): (Path<ForgetUrlParams>, State<AppState>),
) -> Result<HttpResponse, Error> {
    let conn = state.db.get()?;
    let prep_stmt = conn.prepare(include_str!("../../sql/forget_url.sql"))?;
    let _ = prep_stmt.execute(&[&params.id]);
    go::to("/saved")
}
