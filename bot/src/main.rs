use std::sync::Arc;
use anyhow::Result;
use dotenvy::dotenv;
use teloxide::{
    prelude::*,
    requests::Requester,
    dispatching::dialogue::InMemStorage,
    dispatching::UpdateFilterExt,
    types::{Message, CallbackQuery},
    utils::command::BotCommands,
};
use teloxide::types::Update;
mod app;
mod api;
mod dto;
mod util;
mod conversation;

use conversation::{Command, State};

type HandlerResult = anyhow::Result<()>;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();
    let app = Arc::new(app::App::from_env());
    let storage = InMemStorage::<State>::new();

    if let Err(e) = bot.set_my_commands(Command::bot_commands()).send().await {
        log::warn!("[bot] set_my_commands failed: {e}");
    }

    let schema = dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .filter_command::<Command>()
                .endpoint(conversation::handle_command),
        )
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .endpoint(conversation::handle_message),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, InMemStorage<State>, State>()
                .endpoint(conversation::handle_callback),
        );

    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![storage, app])
        .enable_ctrlc_handler()
        .default_handler(|upd: Arc<Update>| async move {
            log::warn!("[bot] unhandled update: {:?}", upd);
        })
        .build()
        .dispatch()
        .await;

    log::info!("[bot] shutdown");
    Ok(())
}