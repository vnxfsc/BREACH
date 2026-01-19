//! Authentication service

use chrono::{Duration, Utc};
use ed25519_dalek::{PublicKey, Signature, Verifier};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::error::{ApiResult, AppError};
use crate::models::PlayerSession;

/// Authentication service
#[derive(Clone)]
pub struct AuthService {
    config: AppConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

/// Challenge message for wallet authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub message: String,
    pub nonce: String,
    pub expires_at: i64,
}

/// Authentication request
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub wallet_address: String,
    pub signature: String,
    pub message: String,
}

/// Authentication response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_at: i64,
    pub player_id: String,
}

impl AuthService {
    pub fn new(config: AppConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.auth.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.auth.jwt_secret.as_bytes());

        Self {
            config,
            encoding_key,
            decoding_key,
        }
    }

    /// Generate a challenge message for wallet signing
    pub fn generate_challenge(&self, wallet_address: &str) -> AuthChallenge {
        let nonce = uuid::Uuid::new_v4().to_string();
        let expires_at = Utc::now().timestamp() + 300; // 5 minutes

        let message = format!(
            "BREACH Authentication\n\nWallet: {}\nNonce: {}\nExpires: {}",
            wallet_address, nonce, expires_at
        );

        AuthChallenge {
            message,
            nonce,
            expires_at,
        }
    }

    /// Verify a signed message from a Solana wallet
    pub fn verify_signature(
        &self,
        wallet_address: &str,
        message: &str,
        signature_base58: &str,
    ) -> ApiResult<bool> {
        // Decode wallet address (public key)
        let pubkey_bytes = bs58::decode(wallet_address)
            .into_vec()
            .map_err(|_| AppError::InvalidSignature)?;

        if pubkey_bytes.len() != 32 {
            return Err(AppError::InvalidSignature);
        }

        let pubkey_array: [u8; 32] = pubkey_bytes
            .try_into()
            .map_err(|_| AppError::InvalidSignature)?;

        let verifying_key = PublicKey::from_bytes(&pubkey_array)
            .map_err(|_| AppError::InvalidSignature)?;

        // Decode signature
        let signature_bytes = bs58::decode(signature_base58)
            .into_vec()
            .map_err(|_| AppError::InvalidSignature)?;

        if signature_bytes.len() != 64 {
            return Err(AppError::InvalidSignature);
        }

        let signature_array: [u8; 64] = signature_bytes
            .try_into()
            .map_err(|_| AppError::InvalidSignature)?;

        let signature = Signature::from_bytes(&signature_array)
            .map_err(|_| AppError::InvalidSignature)?;

        // Verify
        verifying_key
            .verify(message.as_bytes(), &signature)
            .map_err(|_| AppError::InvalidSignature)?;

        Ok(true)
    }

    /// Generate a JWT token for an authenticated player
    pub fn generate_token(&self, player_id: uuid::Uuid, wallet_address: &str) -> ApiResult<String> {
        let expiry = Utc::now() + Duration::hours(self.config.auth.jwt_expiry_hours as i64);

        let claims = PlayerSession {
            player_id,
            wallet_address: wallet_address.to_string(),
            exp: expiry.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(e.into()))
    }

    /// Verify and decode a JWT token
    pub fn verify_token(&self, token: &str) -> ApiResult<PlayerSession> {
        let token_data = decode::<PlayerSession>(token, &self.decoding_key, &Validation::default())
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
                _ => AppError::Unauthorized,
            })?;

        Ok(token_data.claims)
    }
}
