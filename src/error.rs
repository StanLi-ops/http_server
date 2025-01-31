use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
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
    fn into_response(self) -> Response<Body> {
        println!("->> {:<12} - {:?}", "INTO_RES", self);

        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        response.extensions_mut().insert(self);

        response
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        match self {
            Error::LoginFailed => (StatusCode::FORBIDDEN, ClientError::LoginFail),
            Error::AuthFailNoAuthTokenCookie => (StatusCode::FORBIDDEN, ClientError::NotAuth),
            Error::AuthFailTonkenWrongFormat => (StatusCode::FORBIDDEN, ClientError::NotAuth),
            Error::AuthFailCtxNotInResultExt => (StatusCode::FORBIDDEN, ClientError::NotAuth),
            Error::TicketDeleteFailIdNotFound { id: _ } => {
                (StatusCode::BAD_REQUEST, ClientError::InvalidParams)
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::ServiceError),
        }
    }
}

#[derive(Debug, strum_macros::AsRefStr)]
pub enum ClientError {
    LoginFail,
    NotAuth,
    InvalidParams,
    ServiceError,
}
