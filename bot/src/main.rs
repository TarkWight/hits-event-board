use std::sync::Arc;
use anyhow::Result;
use teloxide::{prelude::*, utils::command::BotCommands};
use reqwest::Client;

#[derive(Clone)]
struct App {
    http: Client,
    ping_url: String,
}

impl App {
    fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let ping_url = std::env::var("BACKEND_PING_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080/health".into());
        Self {
            http: Client::new(),
            ping_url,
        }
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
enum Command {
    Help,
    Test,
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

async fn handle(
    bot: Bot,
    msg: Message,
    cmd: Command,
    app: Arc<App>,
) -> Result<(), teloxide::RequestError> {
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
    }
    Ok(())
}