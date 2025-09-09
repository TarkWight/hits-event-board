use anyhow::Result;
use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use rand_core::OsRng;

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);


    let phc = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e))?
        .to_string();

    Ok(phc)
}

pub fn verify_password(password: &str, phc_hash: &str) -> bool {
    let parsed = match PasswordHash::new(phc_hash) {
        Ok(p) => p,
        Err(_) => return false,
    };
    Pbkdf2.verify_password(password.as_bytes(), &parsed).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let phc = hash_password("Abcd1234!").expect("hash");
        assert!(verify_password("Abcd1234!", &phc));
        assert!(!verify_password("wrong", &phc));
    }
}