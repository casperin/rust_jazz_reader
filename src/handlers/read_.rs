extern crate actix_web;
extern crate askama;

use self::actix_web::{HttpResponse, Path, State};
use super::super::state::AppState;
use super::error::Error;
use super::go;
use askama::Template;

#[derive(Template)]
#[template(path = "read.html")]
struct ReadTpl<'a> {
    title: &'a String,
    feed_title: &'a String,
    link: &'a String,
    content: &'a String,
}

#[derive(Deserialize)]
pub struct ReadParams {
    id: i32,
}

pub fn read((params, state): (Path<ReadParams>, State<AppState>)) -> Result<HttpResponse, Error> {
    let conn = state.db.get()?;
    let result = conn.prepare(include_str!("../../sql/select_post.sql"))?
        .query(&[&params.id])
        .unwrap();

    if result.is_empty() {
        return Err(Error::InternalError);
    };

    let row = result.get(0);

    go::render(&ReadTpl {
        title: &row.get(0),
        feed_title: &row.get(1),
        link: &row.get(2),
        content: &row.get(3),
    })
}
