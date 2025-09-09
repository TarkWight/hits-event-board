use axum::Router;
use tower_http::{trace::TraceLayer, cors::CorsLayer, set_header::SetResponseHeaderLayer};
use http::header;

use crate::routes;
use crate::state::AppState;
use crate::middleware::{request_id::RequestIdLayer, json_errors::JsonErrorLayer};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(routes::health::router())
        .merge(routes::auth::router(state.clone()))
        .merge(routes::me::router(state.clone()))
        .merge(routes::oauth::router(state.clone()))
        // .merge(routes::users::router(state.clone())) // включим, когда добавим UserService в AppState
        .merge(routes::companies::router(state.clone()))
        .merge(routes::events::router(state.clone()))
        .merge(routes::telegram::router(state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(RequestIdLayer::new())
        .layer(JsonErrorLayer::new())
        .layer(CorsLayer::permissive())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::SERVER,
            header::HeaderValue::from_static("tsu-event-scheduler"),
        ))
}