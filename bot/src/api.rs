use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde_json::json;
use time::{Duration, OffsetDateTime};
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::{app::App, dto};

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max { format!("{}…", &s[..max]) } else { s.to_string() }
}

fn extract_err_message(body_text: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(body_text) {
        Ok(v) => v.get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .or_else(|| v.get("message").and_then(|m| m.as_str()))
            .unwrap_or(body_text)
            .to_string(),
        Err(_) => body_text.to_string(),
    }
}

pub async fn login_and_link(app: &Arc<App>, email: &str, password: &str, telegram_user_id: i64)
    -> Result<dto::Tokens> {
    let url = format!("{}/api/v1/auth/login", app.base_url);
    let body = json!({
        "email": email,
        "password": password,
        "telegram_user_id": telegram_user_id,
    });

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

    Ok(lo.tokens)
}

pub async fn register_student_and_link(app: &Arc<App>, email: &str, password: &str, telegram_user_id: i64)
    -> Result<dto::Tokens> {
    let url = format!("{}/api/v1/auth/register/student", app.base_url);
    let body = json!({
        "name": email,
        "email": email,
        "password": password,
        "telegram_user_id": telegram_user_id,
    });

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

    let out: dto::RegisterOut = serde_json::from_str(&text)
        .map_err(|e| anyhow!("decode register student: {e}"))?;

    Ok(out.tokens)
}

pub async fn register_manager_and_link(app: &Arc<App>, email: &str, password: &str, company_id: Uuid, telegram_user_id: i64)
    -> Result<dto::Tokens> {
    let url = format!("{}/api/v1/auth/register/manager", app.base_url);
    let body = json!({
        "name": email,
        "email": email,
        "password": password,
        "company_id": company_id,
        "telegram_user_id": telegram_user_id,
    });

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

    let out: dto::RegisterOut = serde_json::from_str(&text)
        .map_err(|e| anyhow!("decode register manager: {e}"))?;

    Ok(out.tokens)
}

pub async fn list_companies(app: &Arc<App>) -> Result<Vec<dto::Company>> {
    let url = format!("{}/api/v1/companies?page=1&limit=1000", app.base_url);

    println!("[bot][api] -> GET {url}");
    let resp = app.http.get(&url).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    println!("[bot][api] <- status={}", status);
    println!("[bot][api] response: {}", truncate(&text, 500));

    if !status.is_success() {
        let msg = extract_err_message(&text);
        return Err(anyhow!("HTTP {}: {}", status, msg));
    }

    let list: Vec<dto::Company> = serde_json::from_str(&text)
        .map_err(|e| anyhow!("decode companies: {e}"))?;

    Ok(list)
}

/* ---- /me ---- */
pub async fn me(app: &Arc<App>, access_token: &str) -> Result<dto::MeOut> {
    let url = format!("{}/api/v1/me", app.base_url);
    println!("[bot][api] -> GET {url}");
    let resp = app.http.get(&url).bearer_auth(access_token).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    println!("[bot][api] <- status={status}");
    println!("[bot][api] response: {}", truncate(&text, 500));

    if !status.is_success() {
        let msg = extract_err_message(&text);
        return Err(anyhow::anyhow!("HTTP {}: {}", status, msg));
    }
    let me: dto::MeOut = serde_json::from_str(&text)
        .map_err(|e| anyhow::anyhow!("decode /me: {e}"))?;
    Ok(me)
}

pub async fn student_list_available_events(
    app: &Arc<App>,
    access_token: &str,
) -> Result<Vec<dto::EventShort>> {
    let now = OffsetDateTime::now_utc();
    let to  = now + Duration::days(90);

    let from_s = now.format(&Rfc3339)?;
    let to_s   = to.format(&Rfc3339)?;

    let url = format!(
        "{}/api/v1/events?published=true&from={}&to={}",
        app.base_url,
        urlencoding::encode(&from_s),
        urlencoding::encode(&to_s),
    );

    println!("[bot][api] -> GET {url}");
    let resp = app.http.get(&url).bearer_auth(access_token).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    println!("[bot][api] <- status={status}");
    println!("[bot][api] response: {}", truncate(&text, 500));

    if !status.is_success() {
        let msg = extract_err_message(&text);
        anyhow::bail!("HTTP {}: {}", status, msg);
    }

    let list: Vec<dto::EventShort> =
        serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("decode events: {e}"))?;
    Ok(list)
}

pub async fn student_register_event(app: &Arc<App>, token: &str, event_id: Uuid) -> Result<()> {
    let url = format!("{}/api/v1/events/{event_id}/register", app.base_url);
    println!("[bot][api] -> POST {url}");
    let resp = app.http.post(&url).bearer_auth(token).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    println!("[bot][api] <- status={status} {}", truncate(&text, 300));
    if !status.is_success() {
        return Err(anyhow!("HTTP {}: {}", status, extract_err_message(&text)));
    }
    Ok(())
}
pub async fn student_unregister_event(app: &Arc<App>, token: &str, event_id: Uuid) -> Result<()> {
    let url = format!("{}/api/v1/events/{event_id}/register", app.base_url);
    println!("[bot][api] -> DELETE {url}");
    let resp = app.http.delete(&url).bearer_auth(token).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    println!("[bot][api] <- status={status} {}", truncate(&text, 300));
    if !status.is_success() {
        return Err(anyhow!("HTTP {}: {}", status, extract_err_message(&text)));
    }
    Ok(())
}

/* ---- manager: список менеджеров компании ---- */
pub async fn manager_list_company_managers(app: &Arc<App>, token: &str, company_id: Uuid) -> Result<serde_json::Value> {
    let url = format!("{}/api/v1/companies/{company_id}/managers", app.base_url);
    println!("[bot][api] -> GET {url}");
    let resp = app.http.get(&url).bearer_auth(token).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    println!("[bot][api] <- status={status}");
    if !status.is_success() {
        return Err(anyhow!("HTTP {}: {}", status, extract_err_message(&text)));
    }
    Ok(serde_json::from_str(&text)?)
}

pub async fn manager_set_manager_status(app: &Arc<App>, token: &str, company_id: Uuid, user_id: Uuid, status: &str) -> Result<()> {
    let url = format!("{}/api/v1/companies/{company_id}/managers/{user_id}/status/{status}", app.base_url);
    println!("[bot][api] -> POST {url}");
    let resp = app.http.post(&url).bearer_auth(token).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    println!("[bot][api] <- status={status} {}", truncate(&text, 300));
    if !status.is_success() {
        return Err(anyhow!("HTTP {}: {}", status, extract_err_message(&text)));
    }
    Ok(())
}

pub async fn manager_list_event_students(app: &Arc<App>, token: &str, event_id: Uuid) -> Result<serde_json::Value> {
    let url = format!("{}/api/v1/events/{event_id}/registrations", app.base_url);
    println!("[bot][api] -> GET {url}");
    let resp = app.http.get(&url).bearer_auth(token).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    println!("[bot][api] <- status={status}");
    if !status.is_success() {
        return Err(anyhow!("HTTP {}: {}", status, extract_err_message(&text)));
    }
    Ok(serde_json::from_str(&text)?)
}


pub async fn manager_create_event(
    app: &Arc<App>,
    access_token: &str,
    company_id: Uuid,
    title: &str,
    short_desc: &str,
    starts_at: &str,
    ends_at: &str,
    signup_deadline: &str,
    location: &str,
    capacity: Option<i32>,
    is_published: bool,
) -> Result<serde_json::Value> {
    let url = format!("{}/api/v1/events", app.base_url);
    let mut body = json!({
        "company_id": company_id,
        "title": title,
        "short_desc": short_desc,
        "starts_at": starts_at,
        "ends_at": ends_at,
        "signup_deadline": signup_deadline,
        "location": location,
        "is_published": is_published,
    });
    if let Some(c) = capacity {
        body.as_object_mut().unwrap().insert("capacity".into(), json!(c));
    }

    println!("[bot][api] -> POST {url}");
    println!("[bot][api] body: {}", truncate(&body.to_string(), 500));

    let resp = app.http.post(&url).bearer_auth(access_token).json(&body).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    println!("[bot][api] <- status={status}");
    println!("[bot][api] response: {}", truncate(&text, 500));

    if !status.is_success() {
        let msg = extract_err_message(&text);
        anyhow::bail!("HTTP {}: {}", status, msg);
    }
    Ok(serde_json::from_str(&text).unwrap_or(json!({ "ok": true })))
}
pub async fn manager_update_event_title(
    app: &Arc<App>,
    token: &str,
    event_id: Uuid,
    new_title: &str,
) -> Result<()> {
    let url = format!("{}/api/v1/events/{event_id}", app.base_url);
    let body = json!({ "title": new_title });
    println!("[bot][api] -> PATCH {url} {}", body);
    let resp = app.http.patch(&url).bearer_auth(token).json(&body).send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    println!("[bot][api] <- status={status} {}", truncate(&text, 300));
    if !status.is_success() {
        return Err(anyhow!("HTTP {}: {}", status, extract_err_message(&text)));
    }
    Ok(())
}
/* опционально: оставляем на будущее; сейчас не используется */
// async fn link_telegram(app: &Arc<App>, access_token: &str, telegram_user_id: i64) -> Result<()> {
//     let url = format!("{}/api/v1/telegram/link", app.base_url);
//     let body = json!({ "telegramUserId": telegram_user_id });
//
//     println!("[bot][api] -> POST {url} (auth)");
//     println!("[bot][api] body: {}", body);
//
//     let resp = app.http.post(&url).bearer_auth(access_token).json(&body).send().await?;
//     let status = resp.status();
//     let text = resp.text().await.unwrap_or_default();
//
//     println!("[bot][api] <- status={}", status);
//     println!("[bot][api] response: {}", truncate(&text, 500));
//
//     if !status.is_success() {
//         let msg = extract_err_message(&text);
//         return Err(anyhow!("HTTP {}: {}", status, msg));
//     }
//     Ok(())
// }