use axum::{Router, routing::get, extract::{State, Query}, response::Redirect};
use serde::Deserialize;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new().route("/api/v1/oauth/google/callback", get(google_callback)).with_state(state)
}

#[derive(Deserialize)]
struct GoogleCb { code: String, state: String }

async fn google_callback(_st: State<AppState>, _q: Query<GoogleCb>) -> Redirect { Redirect::temporary("/connected") }
