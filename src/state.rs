extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

pub struct AppState {
    pub db: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>,
}
