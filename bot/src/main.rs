use std::sync::Arc;
use anyhow::Result;
use teloxide::{prelude::*, utils::command::BotCommands};
use reqwest::Client;

#[derive(Clone)]
struct App {
    http: Client,
    base_url: String,
    ping_url: String,
}

impl App {
    fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let base_url = std::env::var("BACKEND_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".into());
        let ping_url = std::env::var("BACKEND_PING_URL")
            .unwrap_or_else(|_| format!("{base_url}/health"));
        Self {
            http: Client::new(),
            base_url,
            ping_url,
        }
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды:")]
enum Command {
    Help,
    Test,
    Link(String),
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let bot = Bot::from_env();
    let app = Arc::new(App::from_env());

    Command::repl(bot, move |bot, msg, cmd| {
        let app = app.clone();
        async move { handle(bot, msg, cmd, app).await }
    })
        .await;

    Ok(())
}

async fn handle(bot: Bot, msg: Message, cmd: Command, app: Arc<App>) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Test => {
            let text = match app.http.get(&app.ping_url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    format!("Backend responded:\nStatus: {status}\nBody: {body}")
                }
                Err(err) => format!("Backend request failed: {err}"),
            };
            bot.send_message(msg.chat.id, text).await?;
        }
        Command::Link(code) => {
            let tg_id: i64 = msg.from()
                .map(|u| u.id.0 as i64)
                .unwrap_or_else(|| msg.chat.id.0);

            #[derive(serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            struct ConsumeIn<'a> {
                code: &'a str,
                telegram_user_id: i64,
            }

            let url = format!("{}/api/v1/telegram/consume", app.base_url);
            let payload = ConsumeIn { code: &code, telegram_user_id: tg_id };

            let reply = match app.http.post(&url).json(&payload).send().await {
                Ok(resp) if resp.status().is_success() => {
                    "✅ Аккаунт привязан! Спасибо. Теперь функционал разблокирован."
                        .to_string()
                }
                Ok(resp) => {
                    let status = resp.status();
                    let err = resp.text().await.unwrap_or_default();
                    format!("❌ Привязка не удалась. Status: {status}\n{err}")
                }
                Err(err) => format!("❌ Запрос к backend не удался: {err}"),
            };

            bot.send_message(msg.chat.id, reply).await?;
        }
    }
    Ok(())
}