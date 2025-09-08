use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;
use core::num::NonZeroU32;
use pbkdf2::pbkdf2_hmac;
use rand::{rng, RngCore};
use sha2::Sha256;

const SALT_SIZE: usize = 16;
const HASH_SIZE: usize = 20;
const ITERATIONS: u32 = 10_000;

pub fn hash_password(plain: &str) -> String {
    let mut salt = [0u8; SALT_SIZE];
    rng().fill_bytes(&mut salt);

    let mut derived = [0u8; HASH_SIZE];
    let iters = NonZeroU32::new(ITERATIONS).expect("ITERATIONS > 0");
    pbkdf2_hmac::<Sha256>(
        plain.as_bytes(),
        &salt,
        u32::from(iters),
        &mut derived,
    );

    let mut out = [0u8; SALT_SIZE + HASH_SIZE];
    out[..SALT_SIZE].copy_from_slice(&salt);
    out[SALT_SIZE..].copy_from_slice(&derived);

    b64.encode(out)
}

pub fn verify_password(plain: &str, stored_b64: &str) -> bool {
    let decoded = match b64.decode(stored_b64) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if decoded.len() != SALT_SIZE + HASH_SIZE {
        return false;
    }
    let (salt, expected) = decoded.split_at(SALT_SIZE);

    let mut actual = [0u8; HASH_SIZE];
    let iters = NonZeroU32::new(ITERATIONS).expect("ITERATIONS > 0");
    pbkdf2_hmac::<Sha256>(plain.as_bytes(), salt, u32::from(iters), &mut actual);

    ct_eq(expected, &actual)
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) { diff |= x ^ y; }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let h = hash_password("Abcd1234!");
        assert!(verify_password("Abcd1234!", &h));
        assert!(!verify_password("wrong", &h));
    }
}
