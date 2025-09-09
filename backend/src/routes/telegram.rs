use axum::{Router, routing::post, extract::{State}, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{state::AppState, error::ApiResult};
use crate::auth::extractor::AuthUser;
use crate::auth::extractor::Role;
use crate::error::ApiError;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/telegram/link-code", post(create_code))
        .route("/api/v1/telegram/consume",   post(consume))
        .with_state(state)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkCodeOut {
    code: String,
    ttl_minutes: i64,
}

async fn create_code(
    State(st): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<LinkCodeOut>> {
    if user.role != Role::Student {
        return Err(ApiError::Forbidden);
    }

    let code = st.telegram.create_link_code_for_user(user.user_id).await?;
    Ok(Json(LinkCodeOut {
        code,
        ttl_minutes: st.config.telegram_code_ttl,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConsumeIn {
    code: String,
    telegram_user_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConsumeOut {
    user_id: Uuid,
}

async fn consume(State(st): State<AppState>, Json(body): Json<ConsumeIn>) -> ApiResult<Json<ConsumeOut>> {
    let uid = st.telegram.consume_link_code(&body.code, body.telegram_user_id).await?;
    Ok(Json(ConsumeOut { user_id: uid }))
}