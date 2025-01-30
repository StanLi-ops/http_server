use crate::{ctx::Ctx, error::Error, model::ModelController, web::AUTH_TOKEN, Result};

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response},
    middleware::Next,
};
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};

pub async fn mw_require_auth(
    ctx: Result<Ctx>,
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>> {
    let user_id = ctx?.user_id();

    println!("->> {:<12} - mw_require_auth - {:?}", "MIDDLEWARE", user_id);

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(
    // _mc: State<ModelController>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    let request_ctx = match auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)
    {
        Ok((user_id, _exp, _sign)) => Ok(Ctx::new(user_id)),
        Err(e) => Err(e),
    };

    if request_ctx.is_err() && !matches!(request_ctx, Err(Error::AuthFailTonkenWrongFormat)) {
        cookies.remove(Cookie::build(AUTH_TOKEN).into());
    }

    req.extensions_mut().insert(request_ctx);


    Ok(next.run(req).await)
}

fn parse_token(token: String) -> Result<(u64, String, String)> {
    let (_whole, user_id, exp, sign) = regex_captures!(r#"^user-(\d+)\.(.+)\.(.+)"#, &token)
        .ok_or(Error::AuthFailTonkenWrongFormat)?;

    let user_id = user_id
        .parse()
        .map_err(|_| Error::AuthFailTonkenWrongFormat)?;

    Ok((user_id, exp.to_string(), sign.to_string()))
}
