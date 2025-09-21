use mailchecker::is_valid;

use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use rand::RngCore;

use crate::model::model::{CreateUserRequest, User};

pub fn validate_user(user: &User) -> Result<(), String> {
    if !is_valid(&user.email) {
        return Err("Invalid email address".to_string());
    }

    if !strong_password(&user.password) {
        return Err("Password is not strong enough".to_string());
    }

    if user.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    Ok(())
}

pub fn validate_user_registration(user: &CreateUserRequest) -> Result<(), String> {
    if !is_valid(&user.email) {
        return Err("Invalid email address".to_string());
    }

    if !strong_password(&user.password) {
        return Err("Password is not strong enough".to_string());
    }

    if user.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if user.name.trim().len() > 100 {
        return Err("Name is too long".to_string());
    }

    Ok(())
}

pub fn strong_password(password: &str) -> bool {
    let has_min_length = password.len() >= 8;
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special_char = password.chars().any(|c| !c.is_alphanumeric());

    has_min_length && has_uppercase && has_lowercase && has_digit && has_special_char
}

pub fn generate_base64_string() -> String {
    let target_len = 96;
    let byte_len = (target_len * 3) / 4;

    let mut buf = vec![0u8; byte_len];
    rand::rng().fill_bytes(&mut buf);

    let encoded = STANDARD_NO_PAD.encode(&buf);
    encoded[..target_len].to_string()
}
