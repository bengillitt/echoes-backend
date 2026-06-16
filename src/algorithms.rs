use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng}};

use super::structs::{PasswordPair, MessageWithScore};

// -----------------
// Hashing Functions
// -----------------

pub fn hash_password(password: String) -> Result<PasswordPair, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();

    let salt_bytes = Vec::from(salt.as_str().as_bytes());

    // salt.decode_b64(&mut saltBytes);

    return Ok(PasswordPair {
        hashed_password: password_hash.to_string(),
        salt: salt_bytes,
    });
}

pub fn hash_password_with_salt(password: String, salt_str: Vec<u8>) -> Result<PasswordPair, String> {
    let salt = SaltString::from_b64(str::from_utf8(&salt_str).unwrap()).unwrap();

    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();

    return Ok(PasswordPair {
        hashed_password: password_hash.to_string(),
        salt: salt_str,
    });
}

// -----------------
// Sorting Functions
// -----------------

pub fn sort_messages_by_similarity(messages: Vec<MessageWithScore>) -> Result<Vec<MessageWithScore>, String> {
    let mut sorted_messages = messages.clone();

    if sorted_messages.len() <= 1 {
        return Ok(sorted_messages);
     }

     let midpoint = sorted_messages.len() / 2;

     let left = sort_messages_by_similarity(sorted_messages[..midpoint].to_vec())?;
     let right = sort_messages_by_similarity(sorted_messages[midpoint..].to_vec())?;

     return Ok(merge_messages(left, right));
}

fn merge_messages(left: Vec<MessageWithScore>, right: Vec<MessageWithScore>) -> Vec<MessageWithScore> {
    let mut merged: Vec<MessageWithScore> = Vec::new();

    let mut left_index = 0;
    let mut right_index = 0;

    while left_index < left.len() && right_index < right.len() {
        if left[left_index].score > right[right_index].score {
            merged.push(left[left_index].clone());
            left_index += 1;
        } else {
            merged.push(right[right_index].clone());
            right_index += 1;
        }
    }

    while left_index < left.len() {
        merged.push(left[left_index].clone());
        left_index += 1;
    }

    while right_index < right.len() {
        merged.push(right[right_index].clone());
        right_index += 1;
     }

     return merged;
}