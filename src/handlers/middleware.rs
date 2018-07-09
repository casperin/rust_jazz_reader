extern crate actix_web;

use self::actix_web::middleware::identity::RequestIdentity;
use self::actix_web::middleware::{Middleware, Started};
use self::actix_web::{http, HttpRequest, HttpResponse, Result};

pub struct MustBeLoggedIn {
    allowed: &'static [&'static str],
}

impl MustBeLoggedIn {
    pub fn new(allowed: &'static [&'static str]) -> MustBeLoggedIn {
        MustBeLoggedIn { allowed: allowed }
    }
}

impl<S> Middleware<S> for MustBeLoggedIn {
    fn start(&self, req: &mut HttpRequest<S>) -> Result<Started> {
        if let Some("logged-in") = req.identity() {
            return Ok(Started::Done);
        }

        let is_allowed_path = self.allowed.iter().any(|p| req.path().starts_with(p));
        if is_allowed_path {
            return Ok(Started::Done);
        }

        Ok(Started::Response(
            HttpResponse::Found()
                .header(http::header::LOCATION, "/login")
                .finish(),
        ))
    }
}
