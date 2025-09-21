use std::env;

use anyhow::Result;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use tracing::info;
use uuid::Uuid;

use crate::helpers::validation::generate_base64_string;
use crate::model::model::{Claims, Role};

lazy_static::lazy_static! {
    pub static ref JWT_SECRET: String = env::var("AUTH_SECRET")
    .unwrap_or_else(|_| generate_base64_string());

    pub static ref BASE_URL: String = env::var("DOMAIN")
        .unwrap_or_else(|_| "localhost".to_string());
}

pub struct AuthHelper;

impl AuthHelper {
    pub fn hash_password(password: &str) -> Result<String> {
        let hashed = hash(password, DEFAULT_COST)?;
        Ok(hashed)
    }

    pub fn verify_password(password: &str, hashed: &str) -> Result<bool> {
        let is_valid = verify(password, hashed)?;
        Ok(is_valid)
    }

    pub fn generate_token(user_id: Uuid, role: Role) -> Result<(String, String)> {
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            iss: BASE_URL.clone(),
            sub: user_id.to_string(),
            role: role.clone(),
            iat: Utc::now().timestamp() as usize,
            exp: expiration,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        )?;
        info!("Generated Auth token for user_id {}", user_id);

        let expiration = Utc::now()
            .checked_add_signed(Duration::days(7))
            .expect("valid timestamp")
            .timestamp() as usize;

        let refresh_claims = Claims {
            iss: BASE_URL.clone(),
            sub: user_id.to_string(),
            role: role,
            iat: Utc::now().timestamp() as usize,
            exp: expiration,
        };

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        )?;
        info!("Generated Refresh token for user_id {}", user_id);
        Ok((token, refresh_token))
    }

    pub fn validate_token(token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    pub fn extract_user_id_from_token(token: &str) -> Result<Uuid> {
        let claims = Self::validate_token(token)?;
        let user_id = Uuid::parse_str(&claims.sub)?;
        Ok(user_id)
    }

    pub fn extract_user_role_from_token(token: &str) -> Result<Role> {
        let claims = Self::validate_token(token)?;
        Ok(claims.role)
    }

    pub fn generate_email_verification_token(user_id: Uuid) -> String {
        let expiration = Utc::now()
            .checked_add_signed(Duration::minutes(15))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            iss: BASE_URL.clone(),
            sub: user_id.to_string(),
            role: Role::USER,
            iat: Utc::now().timestamp() as usize,
            exp: expiration,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        )
        .expect("Failed to generate email verification token");
        info!("Generated email verification token for user_id {}", user_id);
        token
    }
}
