use teloxide::types::Message;

pub fn user_id_from_msg(msg: &Message) -> i64 {
    msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or_else(|| msg.chat.id.0)
}