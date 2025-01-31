mod ctx;
mod error;
mod log;
mod model;
mod web;

use crate::ctx::Ctx;
use crate::error::{Error, Result};
use crate::log::log_request;
use crate::model::ModelController;

use axum::{
    extract::{Path, Query},
    http::{Method, Uri},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

const ADDRESS: &str = "127.0.0.1";
const PORT: u16 = 3001;

#[derive(serde::Deserialize, Debug, Clone)]
struct HelloParams {
    name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mc: ModelController = ModelController::new().await;

    let app_apis = web::routes_tickets::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let app = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", app_apis)
        .layer(middleware::map_response(main_response_mapper))
        // .layer(middleware::from_fn(web::mv_auth::mw_ctx_resolver))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static());

    let addr = format!("{}:{}", ADDRESS, PORT);
    println!("Listening on {:?}\n", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");

    let uuid = Uuid::new_v4();

    let service_error = res.extensions().get::<Error>();

    let client_status_error = service_error.map(|e| e.client_status_and_error());

    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error":{
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string()
                }
            });

            println!("   -->> client_error_body: {client_error_body}");

            (*status_code, Json(client_error_body)).into_response()
        });

    println!("   -->> server log line - {uuid} - Error: {service_error:?}");

    let client_error = client_status_error.unzip().1;

    let _ = log_request(uuid, uri, req_method, ctx, service_error, client_error).await;

    println!();
    
    error_response.unwrap_or(res)
}

fn routes_static() -> Router {
    Router::new().fallback_service(ServeDir::new("./"))
}

fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/{name}", get(handler_hello2))
}

async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.unwrap_or_else(|| "World".to_string());

    Html(format!("Hello <strong>{name}</strong>!"))
}

async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello2 - {name:?}", "HANDLER");

    Html(format!("Hello <strong>{name}</strong>!"))
}
