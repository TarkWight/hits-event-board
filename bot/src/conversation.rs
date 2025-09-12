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
use uuid::Uuid;

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
    /* общий пролог */
    Menu,

    /* логин */
    LoginEmail,
    LoginPassword { email: String },

    /* регистрация */
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

    StudentMenu { token: String },
    ManagerMenu { token: String, company_id: Option<Uuid> },

    ManagerNewEventTitle   { token: String, company_id: Uuid },
    ManagerNewEventShort   { token: String, company_id: Uuid, title: String },
    ManagerNewEventStarts  { token: String, company_id: Uuid, title: String, short_desc: String },
    ManagerNewEventEnds    { token: String, company_id: Uuid, title: String, short_desc: String, starts_at: String },
    ManagerNewEventDeadline{ token: String, company_id: Uuid, title: String, short_desc: String, starts_at: String, ends_at: String },
    ManagerNewEventLocation{ token: String, company_id: Uuid, title: String, short_desc: String, starts_at: String, ends_at: String, signup_deadline: String },
    ManagerNewEventCapacity{ token: String, company_id: Uuid, title: String, short_desc: String, starts_at: String, ends_at: String, signup_deadline: String, location: String },
    ManagerNewEventPublish { token: String, company_id: Uuid, title: String, short_desc: String, starts_at: String, ends_at: String, signup_deadline: String, location: String, capacity: Option<i32> },
}

impl Default for State {
    fn default() -> Self { State::Menu }
}

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;

/* ===== Keyboards ===== */

fn start_keyboard() -> ReplyMarkup {
    let kb = KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("Войти")],
        vec![KeyboardButton::new("Зарегистрироваться")],
    ]).resize_keyboard();
    ReplyMarkup::Keyboard(kb)
}

fn reg_role_keyboard() -> ReplyMarkup {
    let kb = KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("Студент")],
        vec![KeyboardButton::new("Менеджер")],
        vec![KeyboardButton::new("Назад")],
    ]).resize_keyboard();
    ReplyMarkup::Keyboard(kb)
}

fn student_keyboard() -> ReplyMarkup {
    let kb = KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("Доступные ивенты")],
        vec![KeyboardButton::new("Выйти")],
    ]).resize_keyboard();
    ReplyMarkup::Keyboard(kb)
}

fn manager_keyboard() -> ReplyMarkup {
    let kb = KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("Добавить ивент")],
        vec![KeyboardButton::new("Менеджеры компании")],
        vec![KeyboardButton::new("Студенты ивента")],
        vec![KeyboardButton::new("Выйти")],
    ]).resize_keyboard();
    ReplyMarkup::Keyboard(kb)
}

fn companies_inline(companies: &[dto::Company]) -> InlineKeyboardMarkup {
    let rows: Vec<Vec<InlineKeyboardButton>> = companies
        .chunks(2)
        .map(|chunk| {
            chunk.iter().map(|c| {
                InlineKeyboardButton::callback(
                    c.name.clone(),
                    format!("pick_company:{}", c.id),
                )
            }).collect()
        })
        .collect();
    InlineKeyboardMarkup::new(rows)
}

fn events_inline(events: &[dto::EventShort]) -> InlineKeyboardMarkup {
    // по кнопке — подписаться или отписаться
    let rows = events.iter().map(|e| {
        let text = format!("{} • {}", e.title, e.starts_at);
        // Для простоты: всегда показываем «записаться». Можешь расширить на основе is_registered
        vec![InlineKeyboardButton::callback(text, format!("evt_reg:{}", e.id))]
    }).collect::<Vec<_>>();
    InlineKeyboardMarkup::new(rows)
}

/* ===== Commands ===== */

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

/* ===== Messages ===== */

pub async fn handle_message(bot: Bot, msg: Message, d: MyDialogue, app: Arc<App>) -> anyhow::Result<()> {
    let Some(text) = msg.text().map(|s| s.trim().to_string()) else { return Ok(()) };
    let chat_id = msg.chat.id;
    let tg_id = util::user_id_from_msg(&msg);

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
                Ok(tokens) => {

                    match api::me(&app, &tokens.access_token).await {
                        Ok(me) if me.role == "student" => {
                            d.update(State::StudentMenu { token: tokens.access_token }).await?;
                            bot.send_message(chat_id, "Готово! Меню студента:")
                                .reply_markup(student_keyboard())
                                .await?;
                        }
                        Ok(me) if me.role == "manager" => {
                            d.update(State::ManagerMenu { token: tokens.access_token, company_id: me.company_id }).await?;
                            bot.send_message(chat_id, "Готово! Меню менеджера:")
                                .reply_markup(manager_keyboard())
                                .await?;
                        }
                        Ok(_) => {
                            d.update(State::Menu).await?;
                            bot.send_message(chat_id, "Вход выполнен. (Роль не студент/менеджер)")
                                .reply_markup(start_keyboard())
                                .await?;
                        }
                        Err(e) => {
                            d.update(State::Menu).await?;
                            bot.send_message(chat_id, format!("Не удалось получить профиль: {e}"))
                                .reply_markup(start_keyboard())
                                .await?;
                        }
                    }
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
                Ok(tokens) => {
                    // сразу в меню студента
                    d.update(State::StudentMenu { token: tokens.access_token }).await?;
                    bot.send_message(chat_id, "Студент зарегистрирован! Меню студента:")
                        .reply_markup(student_keyboard())
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
                    d.update(State::RegManagerChooseCompany { email, password: text, companies }).await?;
                }
                Ok(_) => {
                    bot.send_message(chat_id, "Нет компаний для выбора. Обратитесь в деканат.").await?;
                    d.update(State::Menu).await?;
                    bot.send_message(chat_id, "Главное меню:").reply_markup(start_keyboard()).await?;
                }
                Err(e) => {
                    bot.send_message(chat_id, format!("Не удалось получить компании: {e}")).await?;
                    d.update(State::Menu).await?;
                    bot.send_message(chat_id, "Главное меню:").reply_markup(start_keyboard()).await?;
                }
            }
        }
        State::RegManagerChooseCompany { .. } => {
            bot.send_message(chat_id, "Выберите компанию нажатием на кнопку.").await?;
        }

        /* ----- STUDENT MENU ----- */
        State::StudentMenu { token } => match text.as_str() {
            "Доступные ивенты" => {
                match api::student_list_available_events(&app, &token).await {
                    Ok(list) if !list.is_empty() => {
                        let kb = events_inline(&list);
                        bot.send_message(chat_id, "Опубликованные ивенты (нажми, чтобы записаться):")
                            .reply_markup(kb).await?;
                    }
                    Ok(_) => {
                        bot.send_message(chat_id, "Нет доступных ивентов.").await?;
                    }
                    Err(e) => {
                        bot.send_message(chat_id, format!("Не удалось получить ивенты: {e}")).await?;
                    }
                }
            }
            "Выйти" => {
                d.update(State::Menu).await?;
                bot.send_message(chat_id, "Главное меню:")
                    .reply_markup(start_keyboard())
                    .await?;
            }
            _ => {
                bot.send_message(chat_id, "Выберите пункт меню.")
                    .reply_markup(student_keyboard())
                    .await?;
            }
        }

        /* ----- MANAGER MENU ----- */
        State::ManagerMenu { token, company_id } => match text.as_str() {
            "Добавить ивент" => {
                if let Some(cid) = company_id {
                    d.update(State::ManagerNewEventTitle { token: token.clone(), company_id: cid }).await?;
                    bot.send_message(chat_id, "Название ивента?").await?;
                } else {
                    bot.send_message(chat_id, "Не определена компания менеджера.").await?;
                }
            }
            "Менеджеры компании" => {
                if let Some(cid) = company_id {
                    match api::manager_list_company_managers(&app, &token, cid).await {
                        Ok(val) => {
                            let mut lines = vec!["Менеджеры:".to_string()];
                            if let Some(arr) = val.as_array() {
                                for m in arr {
                                    let name = m.get("name").and_then(|x| x.as_str()).unwrap_or("-");
                                    let email = m.get("email").and_then(|x| x.as_str()).unwrap_or("-");
                                    let uid = m.get("user_id").and_then(|x| x.as_str()).unwrap_or("");
                                    let st = m.get("status").and_then(|x| x.as_str()).unwrap_or("-");
                                    lines.push(format!("{name} <{email}> — {st} (uid={uid})"));
                                }
                            }
                            lines.push("Для подтверждения/отклонения напиши: \"апрув <uid>\" или \"реджект <uid>\"".into());
                            bot.send_message(chat_id, lines.join("\n")).await?;
                        }
                        Err(e) => {
                            bot.send_message(chat_id, format!("Ошибка: {e}")).await?;
                        }
                    }
                } else {
                    bot.send_message(chat_id, "Не определена компания менеджера.").await?;
                }
            }
            "Студенты ивента" => {
                bot.send_message(chat_id, "Отправь команду: \"студенты <event_id>\"").await?;
            }
            "Выйти" => {
                d.update(State::Menu).await?;
                bot.send_message(chat_id, "Главное меню:")
                    .reply_markup(start_keyboard())
                    .await?;
            }
            other => {
                if other.starts_with("апрув ") {
                    if let (Some(cid), Some(uid)) = (company_id, other.split_whitespace().nth(1)) {
                        if let Ok(uid) = Uuid::parse_str(uid) {
                            let State::ManagerMenu{ token, .. } = d.get().await?.unwrap() else { return Ok(()) };
                            match api::manager_set_manager_status(&app, &token, cid, uid, "confirmed").await {
                                Ok(_) => { bot.send_message(chat_id, "Подтверждено.").await?; }
                                Err(e) => { bot.send_message(chat_id, format!("Ошибка: {e}")).await?; }
                            }
                        }
                    }
                } else if other.starts_with("реджект ") {
                    if let (Some(cid), Some(uid)) = (company_id, other.split_whitespace().nth(1)) {
                        if let Ok(uid) = Uuid::parse_str(uid) {
                            let State::ManagerMenu{ token, .. } = d.get().await?.unwrap() else { return Ok(()) };
                            match api::manager_set_manager_status(&app, &token, cid, uid, "rejected").await {
                                Ok(_) => { bot.send_message(chat_id, "Отклонено.").await?; }
                                Err(e) => { bot.send_message(chat_id, format!("Ошибка: {e}")).await?; }
                            }
                        }
                    }
                } else if other.starts_with("студенты ") {
                    if let Some(eid) = other.split_whitespace().nth(1) {
                        if let Ok(eid) = Uuid::parse_str(eid) {
                            let State::ManagerMenu{ token, .. } = d.get().await?.unwrap() else { return Ok(()) };
                            match api::manager_list_event_students(&app, &token, eid).await {
                                Ok(val) => {
                                    let mut lines = vec![format!("Записавшиеся на {eid}:")];
                                    if let Some(arr) = val.as_array() {
                                        for r in arr {
                                            let sid = r.get("student_id").and_then(|x| x.as_str()).unwrap_or("-");
                                            let when = r.get("registered_at").and_then(|x| x.as_str()).unwrap_or("-");
                                            lines.push(format!("{sid} — {when}"));
                                        }
                                    }
                                    bot.send_message(chat_id, lines.join("\n")).await?;
                                }
                                Err(e) => {
                                    bot.send_message(chat_id, format!("Ошибка: {e}")).await?;
                                }
                            }
                        }
                    }
                } else {
                    bot.send_message(chat_id, "Выберите пункт меню.")
                        .reply_markup(manager_keyboard())
                        .await?;
                }
            }
        }

        /* ----- менеджер: создание ивента ----- */
        State::ManagerNewEventTitle { token, company_id } => {
            let title = text;
            d.update(State::ManagerNewEventShort { token, company_id, title }).await?;
            bot.send_message(chat_id, "Короткое описание (short_desc)?").await?;
        }

        State::ManagerNewEventShort { token, company_id, title } => {
            let short_desc = text;
            d.update(State::ManagerNewEventStarts { token, company_id, title, short_desc }).await?;
            bot.send_message(chat_id, "Когда начнётся? ISO, напр. 2025-12-30T18:00:00Z").await?;
        }

        State::ManagerNewEventStarts { token, company_id, title, short_desc } => {
            let starts_at = text;
            d.update(State::ManagerNewEventEnds { token, company_id, title, short_desc, starts_at }).await?;
            bot.send_message(chat_id, "Когда закончится? ISO, напр. 2025-12-31T20:00:00Z").await?;
        }

        State::ManagerNewEventEnds { token, company_id, title, short_desc, starts_at } => {
            let ends_at = text;
            d.update(State::ManagerNewEventDeadline { token, company_id, title, short_desc, starts_at, ends_at }).await?;
            bot.send_message(chat_id, "Дедлайн записи (signup_deadline) — ISO, напр. 2025-12-29T23:59:59Z").await?;
        }

        State::ManagerNewEventDeadline { token, company_id, title, short_desc, starts_at, ends_at } => {
            let signup_deadline = text;
            d.update(State::ManagerNewEventLocation { token, company_id, title, short_desc, starts_at, ends_at, signup_deadline }).await?;
            bot.send_message(chat_id, "Локация (location)?").await?;
        }

        State::ManagerNewEventLocation { token, company_id, title, short_desc, starts_at, ends_at, signup_deadline } => {
            let location = text;
            d.update(State::ManagerNewEventCapacity { token, company_id, title, short_desc, starts_at, ends_at, signup_deadline, location }).await?;
            bot.send_message(chat_id, "Вместимость (capacity). Пусто или ∞ — без лимита.").await?;
        }

        State::ManagerNewEventCapacity { token, company_id, title, short_desc, starts_at, ends_at, signup_deadline, location } => {
            let capacity = {
                let t = text.trim();
                if t.is_empty() || t == "∞" { None }
                else {
                    match t.parse::<i32>() {
                        Ok(n) if n > 0 => Some(n),
                        _ => {
                            bot.send_message(chat_id, "Введите положительное число, либо оставьте пусто/∞.").await?;
                            return Ok(());
                        }
                    }
                }
            };
            d.update(State::ManagerNewEventPublish {
                token, company_id, title, short_desc, starts_at, ends_at, signup_deadline, location, capacity
            }).await?;
            bot.send_message(chat_id, "Публиковать сразу? (да/нет)").await?;
        }

        State::ManagerNewEventPublish { token, company_id, title, short_desc, starts_at, ends_at, signup_deadline, location, capacity } => {
            let publish = matches!(text.to_lowercase().as_str(), "да" | "yes" | "y" | "true" | "1");
            match api::manager_create_event(
                &app, &token, company_id,
                &title, &short_desc,
                &starts_at, &ends_at, &signup_deadline,
                &location, capacity, publish
            ).await {
                Ok(_) => {
                    d.update(State::ManagerMenu { token, company_id: Some(company_id) }).await?;
                    bot.send_message(chat_id, if publish { "Ивент создан и опубликован." } else { "Ивент создан (черновик)." })
                        .reply_markup(manager_keyboard())
                        .await?;
                }
                Err(e) => {
                    bot.send_message(chat_id, format!("Ошибка создания: {e}")).await?;
                    d.update(State::ManagerMenu { token, company_id: Some(company_id) }).await?;
                }
            }
        }
    }
    Ok(())
}

pub async fn handle_callback(bot: Bot, q: CallbackQuery, d: MyDialogue, app: Arc<App>) -> anyhow::Result<()> {
    let Some(data) = q.data.as_deref() else { return Ok(()) };
    let Some(m) = q.message.as_ref().and_then(|m| m.regular_message()) else { return Ok(()) };

    let chat_id = m.chat.id;
    let msg_id  = m.id;

    if let Some(event_id) = data.strip_prefix("evt_reg:") {
        if let Ok(eid) = Uuid::parse_str(event_id) {
            if let Some(State::StudentMenu { token }) = d.get().await? {
                bot.answer_callback_query(q.id.clone()).await.ok();
                match api::student_register_event(&app, &token, eid).await {
                    Ok(_) => {
                        bot.edit_message_text(chat_id, msg_id, "Готово: записан ✅").await.ok();
                    }
                    Err(e) => {
                        bot.edit_message_text(chat_id, msg_id, format!("Ошибка записи: {e}")).await.ok();
                    }
                }
            }
        }
        return Ok(());
    }

    if let Some(event_id) = data.strip_prefix("evt_unreg:") {
        if let Ok(eid) = Uuid::parse_str(event_id) {
            if let Some(State::StudentMenu { token }) = d.get().await? {
                bot.answer_callback_query(q.id.clone()).await.ok();
                match api::student_unregister_event(&app, &token, eid).await {
                    Ok(_) => {
                        bot.edit_message_text(chat_id, msg_id, "Готово: снят с записи ✅").await.ok();
                    }
                    Err(e) => {
                        bot.edit_message_text(chat_id, msg_id, format!("Ошибка: {e}")).await.ok();
                    }
                }
            }
        }
        return Ok(());
    }

    if let Some(cid) = data.strip_prefix("pick_company:") {
        if let Ok(company_id) = Uuid::parse_str(cid) {
            if let Some(State::RegManagerChooseCompany { email, password, companies: _ }) = d.get().await? {
                bot.answer_callback_query(q.id.clone()).await.ok();
                bot.edit_message_text(chat_id, msg_id, "Компания выбрана. Регистрирую менеджера...").await.ok();

                match api::register_manager_and_link(&app, &email, &password, company_id, q.from.id.0 as i64).await {
                    Ok(tokens) => {
                        // после регистрации — в меню менеджера
                        d.update(State::ManagerMenu { token: tokens.access_token, company_id: Some(company_id) }).await?;
                        bot.send_message(chat_id, "Менеджер зарегистрирован! Заявка отправлена на одобрение.")
                            .await?;
                        bot.send_message(chat_id, "Меню менеджера:")
                            .reply_markup(manager_keyboard())
                            .await?;
                    }
                    Err(e) => {
                        bot.send_message(chat_id, format!("Ошибка регистрации: {e}")).await?;
                    }
                }
            }
        }
        return Ok(());
    }

    Ok(())
}