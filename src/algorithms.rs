use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::{Salt, SaltString, rand_core::OsRng}};

use super::structs::{PasswordPair};

pub fn hash_password(password: String) -> Result<PasswordPair, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();

    let mut saltBytes = Vec::new();

    salt.decode_b64(&mut saltBytes);

    return Ok(PasswordPair {
        hashed_password: password_hash.to_string(),
        salt: saltBytes,
    });
}

pub fn hash_password_with_salt(password: String, saltStr: Vec<u8>) -> Result<PasswordPair, String> {
    let salt = SaltString::from_b64(str::from_utf8(&saltStr).unwrap()).unwrap();

    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();

    return Ok(PasswordPair {
        hashed_password: password_hash.to_string(),
        salt: saltStr,
    });
}