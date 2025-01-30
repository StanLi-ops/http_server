mod ctx;
mod error;
mod model;
mod web;

use axum::{
    extract::{Path, Query},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use error::Result;
use model::ModelController;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

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
        .route_layer(middleware::from_fn(web::mv_auth::mw_require_auth));

    let app = Router::new()
        .merge(route_hello())
        .merge(web::routes_login::routes())
        .nest("/api", app_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn(web::mv_auth::mw_ctx_resolver))
        // .layer(middleware::from_fn_with_state(
        //     mc.clone(),
        //     web::mv_auth::mw_ctx_resolver,
        // ))
        .layer(CookieManagerLayer::new())
        .fallback_service(route_static());

    let addr = format!("{}:{}", ADDRESS, PORT);
    println!("Listening on {:?}\n", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn main_response_mapper(res: Response) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");

    println!();

    res
}

fn route_static() -> Router {
    Router::new().fallback_service(ServeDir::new("./"))
}

fn route_hello() -> Router {
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
