extern crate config;
extern crate lazy_static;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

pub struct AppState {
    pub db: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>,
}

lazy_static! {
    pub static ref SETTINGS: config::Config = {
        let mut settings = config::Config::default();
        settings.merge(config::File::with_name("config")).unwrap();
        let _ = settings.set_default("port", "8080");
        let _ = settings.set_default("db_user", "postgres");
        let _ = settings.set_default("db_pass", "postgres");
        settings
    };
}
