extern crate askama;

#[macro_use]
extern crate lazy_static;
extern crate actix;
extern crate actix_web;
extern crate config;
extern crate jazz_reader;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

use actix_web::{http, server, App};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

use jazz_reader::handlers;
use jazz_reader::state;

lazy_static! {
    static ref SETTINGS: config::Config = {
        let mut settings = config::Config::default();
        settings.merge(config::File::with_name("config")).unwrap();
        let _ = settings.set_default("port", "8080");
        let _ = settings.set_default("db_user", "postgres");
        let _ = settings.set_default("db_pass", "postgres");
        settings
    };
}

fn main() {
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

    // start http server
    server::new(move || {
        App::with_state(state::AppState { db: pool.clone() })
            .resource("/toggle-saved/{id}", |r| {
                r.method(http::Method::GET).with(handlers::toggle_saved)
            })
            .resource("/read/{id}", |r| {
                r.method(http::Method::GET).with(handlers::read)
            })
            .resource("/saved", |r| {
                r.method(http::Method::GET).with(handlers::saved)
            })
            .resource("/", |r| r.method(http::Method::GET).with(handlers::index))
    }).bind(format!("0.0.0.0:{}", port))
        .unwrap()
        .start();

    println!("Server started on port {}", port);

    let _ = sys.run();
}
