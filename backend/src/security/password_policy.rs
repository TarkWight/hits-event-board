use regex::Regex;

pub fn validate(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Пароль должен быть не менее 8 символов.");
    }
    let has_upper = Regex::new(r"[A-Z]").unwrap();
    let has_lower = Regex::new(r"[a-z]").unwrap();
    let has_digit = Regex::new(r"[0-9]").unwrap();
    let has_spec  = Regex::new(r"[!@#$%^&*(),.?{}|<>]").unwrap();

    if !has_upper.is_match(password) { return Err("Пароль должен содержать хотя бы одну заглавную букву."); }
    if !has_lower.is_match(password) { return Err("Пароль должен содержать хотя бы одну строчную букву."); }
    if !has_digit.is_match(password) { return Err("Пароль должен содержать хотя бы одну цифру."); }
    if !has_spec.is_match(password)  { return Err("Пароль должен содержать хотя бы один специальный символ."); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ok()  { assert!(validate("Abcd1234!").is_ok()); }
    #[test] fn bad() { assert!(validate("abc").is_err()); }
}
