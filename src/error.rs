use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    LoginFailed,
    IoError(String),
    AuthFailNoAuthTokenCookie,
    AuthFailTonkenWrongFormat,
    AuthFailCtxNotInResultExt,
    TicketDeleteFailIdNotFound { id: u64 },
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {:?}", "INTO_RES", self);

        match self {
            Error::LoginFailed => (StatusCode::UNAUTHORIZED, "Login failed").into_response(),
            Error::TicketDeleteFailIdNotFound { id } => (
                StatusCode::NOT_FOUND,
                format!("Ticket with id {} not found", id),
            )
                .into_response(),
            Error::IoError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("IO error: {}", err),
            )
                .into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}

impl std::error::Error for Error {}
