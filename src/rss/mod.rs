extern crate env_logger;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rss;
extern crate tokio_core;

use std::{thread, time};

pub mod feed;

pub fn start_sync(
    frequency_min: u64,
    db_pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>,
) {
    thread::sleep(time::Duration::from_secs(120)); // wait 2 min before starting

    let frequency = time::Duration::from_secs(frequency_min * 60);
    let conn = db_pool.get().expect("get db");
    let prep_stmt = conn
        .prepare(include_str!("../../sql/select_feeds_for_sync.sql"))
        .expect("prepare get unread statement");

    loop {
        let rows = prep_stmt.query(&[]).unwrap();

        for row in rows.iter() {
            let feed_id: i32 = row.get(0);
            let url: String = row.get(1);
            let feed = match feed::fetch(&url) {
                Ok(chan) => chan,
                Err(err) => {
                    info!("Feed error, {}: {}", url, err);
                    break;
                }
            };

            for post in feed.posts.iter() {
                let r = conn
                    .prepare(include_str!("../../sql/insert_post.sql"))
                    .expect("prepare insert post sql")
                    .execute(&[
                        &post.guid,
                        &feed_id,
                        &feed.title,
                        &post.title,
                        &post.link,
                        &post.author,
                        &post.content,
                    ]);

                if let Err(err) = r {
                    error!(
                        "Error inserting post {} from {}: {}",
                        post.title, feed.title, err
                    );
                }
            }
        }

        thread::sleep(frequency);
    }
}
