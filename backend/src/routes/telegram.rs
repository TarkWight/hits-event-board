use axum::{Router, routing::post, extract::State, Json};
use serde::Deserialize;

use crate::{state::AppState, error::ApiResult};

#[derive(Deserialize)]
struct LinkIn {
    user_id: uuid::Uuid,
    telegram_user_id: i64,
}

#[derive(Deserialize)]
struct UnlinkIn {
    user_id: uuid::Uuid,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/telegram/link", post(link))
        .route("/api/v1/telegram/unlink", post(unlink))
        .with_state(state)
}

async fn link(State(st): State<AppState>, Json(body): Json<LinkIn>) -> ApiResult<()> {
    st.telegram.link_student(body.user_id, body.telegram_user_id).await
}

async fn unlink(State(st): State<AppState>, Json(body): Json<UnlinkIn>) -> ApiResult<()> {
    st.telegram.unlink_student(body.user_id).await
}