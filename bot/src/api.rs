use std::sync::Arc;
use anyhow::{Result, anyhow};
use reqwest::StatusCode;
use serde_json::json;

use crate::{app::App, dto};

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max { format!("{}…", &s[..max]) } else { s.to_string() }
}

fn extract_err_message(body_text: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(body_text) {
        Ok(v) => {
            v.get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .or_else(|| v.get("message").and_then(|m| m.as_str()))
                .unwrap_or(body_text)
                .to_string()
        }
        Err(_) => body_text.to_string(),
    }
}

pub async fn login_and_link(
    app: &Arc<App>,
    email: &str,
    password: &str,
    telegram_user_id: i64,
) -> Result<dto::Tokens> {
    let url = format!("{}/api/v1/auth/login", app.base_url);
    let body = json!({ "email": email, "password": password });

    println!("[bot][api] -> POST {url}");
    println!("[bot][api] body: {}", truncate(&body.to_string(), 500));

    let resp = app.http.post(&url).json(&body).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    println!("[bot][api] <- status={}", status);
    println!("[bot][api] response: {}", truncate(&text, 500));

    if !status.is_success() {
        let msg = extract_err_message(&text);
        return Err(anyhow!("HTTP {}: {}", status, msg));
    }

    let lo: dto::LoginOut = serde_json::from_str(&text)
        .map_err(|e| anyhow!("decode login: {e}"))?;

    println!("[bot][api] decoded login ok, user={}", lo.user.email);

    link_telegram(app, &lo.tokens.access_token, telegram_user_id).await?;
    println!("[bot][api] telegram linked for tg_id={telegram_user_id}");

    Ok(lo.tokens)
}

pub async fn register_student_and_link(
    app: &Arc<App>,
    email: &str,
    password: &str,
    telegram_user_id: i64
) -> Result<dto::Tokens> {
    let url = format!("{}/api/v1/auth/register/student", app.base_url);
    let body = json!({ "name": email, "email": email, "password": password });

    log::info!("[bot][api] -> POST {} body={}", url, truncate(&body.to_string(), 500));
    let resp = app.http.post(&url).json(&body).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    log::info!("[bot][api] <- {} {}", status, truncate(&text, 500));

    if !status.is_success() {
        let msg = extract_err_message(&text);
        return Err(anyhow!("HTTP {}: {}", status, msg));
    }
    let out: dto::RegisterOut = serde_json::from_str(&text)
        .map_err(|e| anyhow!("decode register student: {e}"))?;

    link_telegram(app, &out.tokens.access_token, telegram_user_id).await?;
    Ok(out.tokens)
}

pub async fn register_manager_and_link(
    app: &Arc<App>,
    email: &str,
    password: &str,
    company_id: uuid::Uuid,
    telegram_user_id: i64
) -> Result<dto::Tokens> {
    let url = format!("{}/api/v1/auth/register/manager", app.base_url);
    let body = json!({ "name": email, "email": email, "password": password, "companyId": company_id });

    log::info!("[bot][api] -> POST {} body={}", url, truncate(&body.to_string(), 500));
    let resp = app.http.post(&url).json(&body).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    log::info!("[bot][api] <- {} {}", status, truncate(&text, 500));

    if !status.is_success() {
        let msg = extract_err_message(&text);
        return Err(anyhow!("HTTP {}: {}", status, msg));
    }
    let out: dto::RegisterOut = serde_json::from_str(&text)
        .map_err(|e| anyhow!("decode register manager: {e}"))?;

    link_telegram(app, &out.tokens.access_token, telegram_user_id).await?;
    Ok(out.tokens)
}

pub async fn list_companies(app: &Arc<App>) -> Result<Vec<dto::Company>> {
    let url = format!("{}/api/v1/companies?page=1&limit=1000", app.base_url);
    log::info!("[bot][api] -> GET {}", url);

    let resp = app.http.get(&url).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    log::info!("[bot][api] <- {} {}", status, truncate(&text, 500));

    if !status.is_success() {
        let msg = extract_err_message(&text);
        return Err(anyhow!("HTTP {}: {}", status, msg));
    }

    let list: Vec<dto::Company> = serde_json::from_str(&text)
        .map_err(|e| anyhow!("decode companies: {e}"))?;

    Ok(list)
}

async fn link_telegram(app: &Arc<App>, access_token: &str, telegram_user_id: i64) -> Result<()> {
    let url = format!("{}/api/v1/telegram/link", app.base_url);
    let body = json!({ "telegramUserId": telegram_user_id });

    log::info!("[bot][api] -> POST {} (auth) body={}", url, body);
    let resp = app.http.post(&url).bearer_auth(access_token).json(&body).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    log::info!("[bot][api] <- {} {}", status, truncate(&text, 500));

    if !status.is_success() {
        let msg = extract_err_message(&text);
        return Err(anyhow!("HTTP {}: {}", status, msg));
    }
    Ok(())
}