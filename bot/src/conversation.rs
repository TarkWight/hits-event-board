use std::sync::Arc;

use teloxide::Bot;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Dialogue;
use teloxide::requests::Requester;
use teloxide::types::{
    CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup,
    Message, ReplyMarkup,
};
use teloxide::utils::command::BotCommands;

use crate::{api, app::App, dto, util};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды бота")]
pub enum Command {
    #[command(description = "Показать меню")]
    Start,
    #[command(description = "Помощь")]
    Help,
    #[command(description = "Сбросить диалог")]
    Reset,
}

#[derive(Clone, Debug)]
pub enum State {
    Menu,

    LoginEmail,
    LoginPassword { email: String },

    RegRole,

    RegStudentEmail,
    RegStudentPassword { email: String },

    RegManagerEmail,
    RegManagerPassword { email: String },
    RegManagerChooseCompany {
        email: String,
        password: String,
        companies: Vec<dto::Company>,
    },
}

impl Default for State {
    fn default() -> Self {
        State::Menu
    }
}

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;

/* ====================== Keyboards ====================== */

fn start_keyboard() -> ReplyMarkup {
    let kb = KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("Войти")],
        vec![KeyboardButton::new("Зарегистрироваться")],
    ])
        .resize_keyboard();
    ReplyMarkup::Keyboard(kb)
}

fn reg_role_keyboard() -> ReplyMarkup {
    let kb = KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("Студент")],
        vec![KeyboardButton::new("Менеджер")],
        vec![KeyboardButton::new("Назад")],
    ])
        .resize_keyboard();
    ReplyMarkup::Keyboard(kb)
}

fn companies_inline(companies: &[dto::Company]) -> InlineKeyboardMarkup {
    let rows: Vec<Vec<InlineKeyboardButton>> = companies
        .chunks(2)
        .map(|chunk| {
            chunk
                .iter()
                .map(|c| InlineKeyboardButton::callback(c.name.clone(), format!("pick_company:{}", c.id)))
                .collect()
        })
        .collect();

    InlineKeyboardMarkup::new(rows)
}

/* ====================== Commands ====================== */

pub async fn handle_command(bot: Bot, msg: Message, cmd: Command, d: MyDialogue) -> anyhow::Result<()> {
    let chat_id = msg.chat.id;

    match cmd {
        Command::Start => {
            d.update(State::Menu).await?;
            bot.send_message(chat_id, "Выберите действие:")
                .reply_markup(start_keyboard())
                .await?;
        }
        Command::Help => {
            bot.send_message(chat_id, Command::descriptions().to_string()).await?;
        }
        Command::Reset => {
            d.update(State::Menu).await?;
            bot.send_message(chat_id, "Диалог сброшен.")
                .reply_markup(start_keyboard())
                .await?;
        }
    }
    Ok(())
}

/* ====================== Messages ====================== */

pub async fn handle_message(bot: Bot, msg: Message, d: MyDialogue, app: Arc<App>) -> anyhow::Result<()> {
    let Some(text) = msg.text().map(|s| s.trim().to_string()) else {
        return Ok(());
    };
    let chat_id = msg.chat.id;
    let tg_id = util::user_id_from_msg(&msg); // i64

    let state_opt = d.get().await?;
    let Some(state) = state_opt else {
        d.update(State::Menu).await?;
        bot.send_message(chat_id, "Выберите действие:")
            .reply_markup(start_keyboard())
            .await?;
        return Ok(());
    };

    match state {
        /* ----- MENU ----- */
        State::Menu => match text.as_str() {
            "Войти" => {
                bot.send_message(chat_id, "Введите email:")
                    .reply_markup(ReplyMarkup::kb_remove())
                    .await?;
                d.update(State::LoginEmail).await?;
            }
            "Зарегистрироваться" => {
                bot.send_message(chat_id, "Кем вы будете?")
                    .reply_markup(reg_role_keyboard())
                    .await?;
                d.update(State::RegRole).await?;
            }
            _ => {
                bot.send_message(chat_id, "Выберите действие с клавиатуры.")
                    .reply_markup(start_keyboard())
                    .await?;
            }
        },

        /* ----- LOGIN ----- */
        State::LoginEmail => {
            bot.send_message(chat_id, "Введите пароль:").await?;
            d.update(State::LoginPassword { email: text }).await?;
        }
        State::LoginPassword { email } => {
            bot.send_message(chat_id, "Выполняю вход...").await?;
            match api::login_and_link(&app, &email, &text, tg_id).await {
                Ok(_tokens) => {
                    bot.send_message(chat_id, "Успешный вход и привязка Telegram!").await?;
                    d.update(State::Menu).await?;
                    bot.send_message(chat_id, "Главное меню:")
                        .reply_markup(start_keyboard())
                        .await?;
                }
                Err(e) => {
                    bot.send_message(chat_id, format!("Ошибка входа: {e}")).await?;
                    bot.send_message(chat_id, "Попробуйте снова. Введите email:").await?;
                    d.update(State::LoginEmail).await?;
                }
            }
        }

        /* ----- REG ROLE ----- */
        State::RegRole => match text.as_str() {
            "Студент" => {
                bot.send_message(chat_id, "Введите email для регистрации студента:")
                    .reply_markup(ReplyMarkup::kb_remove())
                    .await?;
                d.update(State::RegStudentEmail).await?;
            }
            "Менеджер" => {
                bot.send_message(chat_id, "Введите email для регистрации менеджера:")
                    .reply_markup(ReplyMarkup::kb_remove())
                    .await?;
                d.update(State::RegManagerEmail).await?;
            }
            "Назад" => {
                d.update(State::Menu).await?;
                bot.send_message(chat_id, "Главное меню:")
                    .reply_markup(start_keyboard())
                    .await?;
            }
            _ => {
                bot.send_message(chat_id, "Пожалуйста, выберите роль с клавиатуры.").await?;
            }
        },

        /* ----- REG STUDENT ----- */
        State::RegStudentEmail => {
            bot.send_message(chat_id, "Придумайте пароль:").await?;
            d.update(State::RegStudentPassword { email: text }).await?;
        }
        State::RegStudentPassword { email } => {
            bot.send_message(chat_id, "Регистрирую...").await?;
            match api::register_student_and_link(&app, &email, &text, tg_id).await {
                Ok(_tokens) => {
                    bot.send_message(chat_id, "Студент зарегистрирован и Telegram привязан!").await?;
                    d.update(State::Menu).await?;
                    bot.send_message(chat_id, "Главное меню:")
                        .reply_markup(start_keyboard())
                        .await?;
                }
                Err(e) => {
                    bot.send_message(chat_id, format!("Ошибка регистрации: {e}")).await?;
                    bot.send_message(chat_id, "Введите email ещё раз:").await?;
                    d.update(State::RegStudentEmail).await?;
                }
            }
        }

        /* ----- REG MANAGER ----- */
        State::RegManagerEmail => {
            bot.send_message(chat_id, "Придумайте пароль:").await?;
            d.update(State::RegManagerPassword { email: text }).await?;
        }
        State::RegManagerPassword { email } => {
            bot.send_message(chat_id, "Получаю список компаний...").await?;
            match api::list_companies(&app).await {
                Ok(companies) if !companies.is_empty() => {
                    let kb = companies_inline(&companies);
                    bot.send_message(chat_id, "Выберите компанию:")
                        .reply_markup(kb)
                        .await?;
                    d.update(State::RegManagerChooseCompany {
                        email,
                        password: text,
                        companies,
                    })
                        .await?;
                }
                Ok(_) => {
                    bot.send_message(chat_id, "Нет компаний для выбора. Обратитесь в деканат.")
                        .await?;
                    d.update(State::Menu).await?;
                    bot.send_message(chat_id, "Главное меню:")
                        .reply_markup(start_keyboard())
                        .await?;
                }
                Err(e) => {
                    bot.send_message(chat_id, format!("Не удалось получить компании: {e}")).await?;
                    d.update(State::Menu).await?;
                    bot.send_message(chat_id, "Главное меню:")
                        .reply_markup(start_keyboard())
                        .await?;
                }
            }
        }
        State::RegManagerChooseCompany { .. } => {
            bot.send_message(chat_id, "Выберите компанию нажатием на кнопку.").await?;
        }
    }

    Ok(())
}

/* ====================== Callback buttons ====================== */

pub async fn handle_callback(bot: Bot, q: CallbackQuery, d: MyDialogue, app: Arc<App>) -> anyhow::Result<()> {
    let Some(data) = q.data.as_deref() else { return Ok(()); };
    // Достаём обычное сообщение из MaybeInaccessibleMessage
    let Some(m) = q.message.as_ref().and_then(|m| m.regular_message()) else { return Ok(()); };

    let chat_id = m.chat.id;
    let msg_id = m.id;
    let tg_id = q.from.id.0 as i64;

    if let Some(State::RegManagerChooseCompany { email, password, companies }) = d.get().await? {
        if let Some(sel) = data.strip_prefix("pick_company:") {
            if let Some(c) = companies.iter().find(|c| c.id.to_string() == sel) {
                bot.answer_callback_query(q.id.clone()).await.ok();
                bot.edit_message_text(chat_id, msg_id, format!("Вы выбрали: {}", c.name)).await.ok();

                bot.send_message(chat_id, "Регистрирую менеджера...").await?;
                match api::register_manager_and_link(&app, &email, &password, c.id, tg_id).await {
                    Ok(_tokens) => {
                        d.update(State::Menu).await?;
                        bot.send_message(chat_id, "✅ Менеджер зарегистрирован и Telegram привязан! Заявка отправлена на одобрение.")
                            .await?;
                        bot.send_message(chat_id, "Главное меню:")
                            .reply_markup(start_keyboard())
                            .await?;
                    }
                    Err(e) => {
                        bot.send_message(chat_id, format!("❌ Ошибка регистрации: {e}")).await?;
                    }
                }
            }
        }
    }

    Ok(())
}