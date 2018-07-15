extern crate actix_web;
extern crate failure;
extern crate postgres;
extern crate r2d2;

use self::actix_web::{error, http, HttpResponse};
use std::convert::From;
use std::num::ParseIntError;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "internal error")]
    InternalError,
    /*
    #[fail(display = "bad request")]
    BadClientData,
    #[fail(display = "timeout")]
    Timeout,
    */
}

impl error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match *self {
            Error::InternalError => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR),
            // Error::BadClientData => HttpResponse::new(http::StatusCode::BAD_REQUEST),
            // Error::Timeout => HttpResponse::new(http::StatusCode::GATEWAY_TIMEOUT),
        }
    }
}

impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Error {
        println!("r2d2 Error: {}", e);
        Error::InternalError
    }
}

impl From<postgres::Error> for Error {
    fn from(e: postgres::Error) -> Error {
        println!("postgres Error: {}", e);
        Error::InternalError
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Error {
        println!("postgres Error: {}", e);
        Error::InternalError
    }
}
