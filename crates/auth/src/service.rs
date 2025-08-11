use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use time::OffsetDateTime;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::models::Claims;
use core::{config::AuthConfig, error::Result};

#[derive(Clone)]
pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    jwt_expiration: u64,
    argon2: Argon2<'static>,
}

impl AuthService {
    pub fn new(config: &AuthConfig) -> Result<Self> {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());

        let argon2 = Argon2::default();

        Ok(Self {
            encoding_key,
            decoding_key,
            jwt_expiration: config.jwt_expiration,
            argon2,
        })
    }

    #[instrument(skip(self, password))]
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);

        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                error!("Failed to hash password: {}", e);
                anyhow::anyhow!("Password hashing failed")
            })?;

        Ok(password_hash.to_string())
    }

    #[instrument(skip(self, password, hash))]
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| {
                error!("Failed to parse password hash: {}", e);
                anyhow::anyhow!("Invalid password hash")
            })?;

        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => {
                error!("Password verification error: {}", e);
                Err(anyhow::anyhow!("Password verification failed").into())
            }
        }
    }

    #[instrument(skip(self))]
    pub fn generate_token(
        &self,
        user_id: Uuid,
        username: String,
        email: String,
        roles: Vec<String>,
    ) -> Result<String> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let expiration = now + self.jwt_expiration as i64;

        let claims = Claims {
            sub: user_id,
            username,
            email,
            roles,
            exp: expiration,
            iat: now,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| {
                error!("Failed to encode JWT: {}", e);
                anyhow::anyhow!("Token generation failed").into()
            })
    }

    #[instrument(skip(self, token))]
    pub async fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::default();

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| {
                error!("Failed to decode JWT: {}", e);
                anyhow::anyhow!("Invalid token")
            })?;

        // Check if token is expired
        let now = OffsetDateTime::now_utc().unix_timestamp();
        if token_data.claims.exp < now {
            return Err(anyhow::anyhow!("Token expired").into());
        }

        Ok(token_data.claims)
    }

    #[instrument(skip(self))]
    pub fn extract_token_from_header<'a>(&self, auth_header: &'a str) -> Result<&'a str> {
        auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(move || anyhow::anyhow!("Invalid authorization header format").into())
    }

    pub fn jwt_expiration(&self) -> u64 {
        self.jwt_expiration
    }
}
