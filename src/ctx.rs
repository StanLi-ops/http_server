use crate::{error::Error, Result};

use axum::{extract::FromRequestParts, http::request::Parts};

#[derive(Debug, Clone)]
pub struct Ctx {
    user_id: u64,
}

impl Ctx {
    pub fn new(user_id: u64) -> Self {
        Self { user_id }
    }

    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("->> {:<12} - CTX", "EXTRACTOR");

        let request_ctx = parts
            .extensions
            .get::<Result<Ctx>>()
            .ok_or(Error::AuthFailCtxNotInResultExt)?;
        request_ctx.clone()
    }
}
