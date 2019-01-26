extern crate actix;
extern crate actix_web;
extern crate askama;
extern crate env_logger;
extern crate jazz_reader;
extern crate log;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{http, server, App};
use std::thread;

use r2d2_postgres::{PostgresConnectionManager, TlsMode};

use jazz_reader::handlers;
use jazz_reader::rss;
use jazz_reader::state::{AppState, SETTINGS};

fn main() {
    env_logger::init();
    let port: String = SETTINGS.get("port").unwrap();
    let sys = actix::System::new("template-askama");
    let conn_string = format!(
        "postgres://{}:{}@localhost/{}",
        SETTINGS.get::<String>("db_user").unwrap(),
        SETTINGS.get::<String>("db_pass").unwrap(),
        SETTINGS.get::<String>("db_name").unwrap()
    );
    let manager = PostgresConnectionManager::new(conn_string, TlsMode::None).unwrap();
    let pool = r2d2::Pool::new(manager).unwrap();

    // start rss sync loop
    let sync_rss_every_x_minute: u64 = SETTINGS.get("sync_rss_every_x_minute").unwrap();
    let rss_db_pool = pool.clone();
    thread::spawn(move || rss::start_sync(sync_rss_every_x_minute, rss_db_pool));

    // start http server
    server::new(move || {
        App::with_state(AppState { db: pool.clone() })
            .middleware(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-cookie")
                    .secure(false),
            ))
            .middleware(handlers::MustBeLoggedIn::new(&["/login", "/read"]))
            .resource("/login", |r| {
                r.method(http::Method::GET).with(handlers::login);
                r.method(http::Method::POST).with(handlers::perform_login)
            })
            .resource("/logout", |r| {
                r.method(http::Method::GET).with(handlers::logout)
            })
            .resource("/read/{id}", |r| r.with(handlers::read))
            .resource("/toggle-saved/{id}", |r| r.with(handlers::toggle_saved))
            .resource("/saved", |r| r.with(handlers::saved))
            .resource("/feeds", |r| r.with(handlers::feeds))
            .resource("/feeds/{id}", |r| r.with(handlers::feed))
            .resource("/preview-feed", |r| {
                r.method(http::Method::POST).with(handlers::preview_feed)
            })
            .resource("/add-feed", |r| {
                r.method(http::Method::POST).with(handlers::add_feed)
            })
            .resource("/unsubscribe", |r| {
                r.method(http::Method::POST)
                    .with(handlers::unsubscribe_feed)
            })
            .resource("/mark-all-as-read", |r| {
                r.method(http::Method::POST)
                    .with(handlers::mark_all_as_read)
            })
            .resource("/save-url", |r| {
                r.method(http::Method::POST).with(handlers::save_url)
            })
            .resource("/forget-url/{id}", |r| r.with(handlers::forget_url))
            .resource("/", |r| r.with(handlers::index))
    })
    .bind(format!("0.0.0.0:{}", port))
    .unwrap()
    .start();

    let _ = sys.run();
}
