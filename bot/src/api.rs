use std::sync::Arc;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::json;

use crate::{app::App, dto};

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Option<ErrorBody>,
}
#[derive(Debug, Deserialize)]
struct ErrorBody {
    code: Option<String>,
    message: Option<String>,
}

async fn parse_error(resp: reqwest::Response) -> String {
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    if let Ok(err) = serde_json::from_str::<ErrorResponse>(&text) {
        if let Some(body) = err.error {
            if let Some(msg) = body.message {
                return msg;
            }
        }
    }
    format!("HTTP {status}: {text}")
}

pub async fn login_and_link(
    app: &Arc<App>,
    email: &str,
    password: &str,
    telegram_user_id: i64,
) -> Result<dto::Tokens> {
    // login
    let url = format!("{}/api/v1/auth/login", app.base_url);
    let resp = app.http
        .post(&url)
        .json(&json!({ "email": email, "password": password }))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!(parse_error(resp).await));
    }
    let lo: dto::LoginOut = resp.json().await?;

    link_telegram(app, &lo.tokens.access_token, telegram_user_id).await?;

    Ok(lo.tokens)
}

pub async fn register_student_and_link(
    app: &Arc<App>,
    email: &str,
    password: &str,
    telegram_user_id: i64,
) -> Result<dto::Tokens> {
    let url = format!("{}/api/v1/auth/register/student", app.base_url);
    let resp = app.http
        .post(&url)
        .json(&json!({
            "name": email,
            "email": email,
            "password": password
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!(parse_error(resp).await));
    }
    let out: dto::RegisterOut = resp.json().await?;

    link_telegram(app, &out.tokens.access_token, telegram_user_id).await?;

    Ok(out.tokens)
}

pub async fn register_manager_and_link(
    app: &Arc<App>,
    email: &str,
    password: &str,
    company_id: uuid::Uuid,
    telegram_user_id: i64,
) -> Result<dto::Tokens> {
    let url = format!("{}/api/v1/auth/register/manager", app.base_url);
    let resp = app.http
        .post(&url)
        .json(&json!({
            "name": email,
            "email": email,
            "password": password,
            "company_id": company_id
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!(parse_error(resp).await));
    }
    let out: dto::RegisterOut = resp.json().await?;

    link_telegram(app, &out.tokens.access_token, telegram_user_id).await?;

    Ok(out.tokens)
}

pub async fn list_companies(app: &Arc<App>) -> Result<Vec<dto::Company>> {
    let url = format!("{}/api/v1/companies?page=1&limit=1000", app.base_url);
    let resp = app.http.get(&url).send().await?;

    if !resp.status().is_success() {
        return Err(anyhow!(parse_error(resp).await));
    }
    let list: Vec<dto::Company> = resp.json().await?;
    Ok(list)
}

async fn link_telegram(app: &Arc<App>, access_token: &str, telegram_user_id: i64) -> Result<()> {
    let url = format!("{}/api/v1/telegram/link", app.base_url);
    let resp = app.http
        .post(&url)
        .bearer_auth(access_token)
        .json(&json!({ "telegram_user_id": telegram_user_id }))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!(parse_error(resp).await));
    }
    Ok(())
}