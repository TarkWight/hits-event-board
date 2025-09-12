use std::sync::Arc;

use anyhow::Result;
use dotenvy::dotenv;

use teloxide::{
    prelude::*,
    requests::Requester,
    dispatching::dialogue::InMemStorage,
    dispatching::UpdateFilterExt,
    types::{Update, Message, CallbackQuery},
    utils::command::BotCommands,
};

mod app;
mod api;
mod dto;
mod util;
mod conversation;

use conversation::{Command, State};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();
    let app = Arc::new(app::App::from_env());
    let storage = InMemStorage::<State>::new();

    bot.set_my_commands(Command::bot_commands()).await?;

    let schema = dptree::entry()
        // ------- сообщения -------
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .filter_command::<Command>()
                .endpoint(conversation::handle_command)
        )
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .endpoint(conversation::handle_message)
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, InMemStorage<State>, State>()
                .endpoint(conversation::handle_callback)
        );

    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![storage, app])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}