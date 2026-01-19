//! Authentication service

use chrono::{Duration, Utc};
use ed25519_dalek::{Signature, VerifyingKey, Verifier};
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

        let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
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

        let signature = Signature::from_bytes(&signature_array);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AppConfig {
        AppConfig::default()
    }

    // ========================================
    // Challenge Generation Tests
    // ========================================

    #[test]
    fn test_generate_challenge() {
        let config = test_config();
        let auth_service = AuthService::new(config);
        
        let wallet = "TestWallet123456789";
        let challenge = auth_service.generate_challenge(wallet);
        
        assert!(challenge.message.contains(wallet));
        assert!(challenge.message.contains(&challenge.nonce));
        assert!(!challenge.nonce.is_empty());
        assert!(challenge.expires_at > Utc::now().timestamp());
    }

    #[test]
    fn test_generate_challenge_unique_nonce() {
        let config = test_config();
        let auth_service = AuthService::new(config);
        
        let wallet = "TestWallet123456789";
        let challenge1 = auth_service.generate_challenge(wallet);
        let challenge2 = auth_service.generate_challenge(wallet);
        
        assert_ne!(challenge1.nonce, challenge2.nonce);
    }

    #[test]
    fn test_challenge_expires_in_5_minutes() {
        let config = test_config();
        let auth_service = AuthService::new(config);
        
        let wallet = "TestWallet123456789";
        let challenge = auth_service.generate_challenge(wallet);
        
        let now = Utc::now().timestamp();
        let diff = challenge.expires_at - now;
        
        // Should expire in ~300 seconds (5 minutes)
        assert!(diff >= 299 && diff <= 301);
    }

    // ========================================
    // JWT Token Tests
    // ========================================

    #[test]
    fn test_generate_and_verify_token() {
        let config = test_config();
        let auth_service = AuthService::new(config);
        
        let player_id = uuid::Uuid::new_v4();
        let wallet = "TestWallet123456789";
        
        let token = auth_service.generate_token(player_id, wallet).unwrap();
        assert!(!token.is_empty());
        
        let session = auth_service.verify_token(&token).unwrap();
        assert_eq!(session.player_id, player_id);
        assert_eq!(session.wallet_address, wallet);
    }

    #[test]
    fn test_token_has_expiry() {
        let config = test_config();
        let auth_service = AuthService::new(config);
        
        let player_id = uuid::Uuid::new_v4();
        let wallet = "TestWallet123456789";
        
        let token = auth_service.generate_token(player_id, wallet).unwrap();
        let session = auth_service.verify_token(&token).unwrap();
        
        // Token should expire in ~24 hours
        let now = Utc::now().timestamp();
        let diff = session.exp - now;
        
        // Allow some tolerance
        assert!(diff > 23 * 3600 && diff <= 24 * 3600 + 60);
    }

    #[test]
    fn test_invalid_token() {
        let config = test_config();
        let auth_service = AuthService::new(config);
        
        let result = auth_service.verify_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_wrong_secret() {
        let config1 = test_config();
        let mut config2 = test_config();
        config2.auth.jwt_secret = "different_secret_key".to_string();
        
        let auth_service1 = AuthService::new(config1);
        let auth_service2 = AuthService::new(config2);
        
        let player_id = uuid::Uuid::new_v4();
        let wallet = "TestWallet123456789";
        
        // Generate with service 1
        let token = auth_service1.generate_token(player_id, wallet).unwrap();
        
        // Verify with service 2 (different secret)
        let result = auth_service2.verify_token(&token);
        assert!(result.is_err());
    }

    // ========================================
    // Signature Verification Tests
    // ========================================

    #[test]
    fn test_verify_signature_invalid_wallet() {
        let config = test_config();
        let auth_service = AuthService::new(config);
        
        // Invalid base58
        let result = auth_service.verify_signature("!!!invalid!!!", "message", "signature");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_signature_wrong_length_wallet() {
        let config = test_config();
        let auth_service = AuthService::new(config);
        
        // Valid base58 but wrong length
        let result = auth_service.verify_signature("ABC", "message", "signature");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_signature_invalid_signature_format() {
        let config = test_config();
        let auth_service = AuthService::new(config);
        
        // Valid 32-byte base58 public key (dummy)
        let wallet = "11111111111111111111111111111111";
        
        let result = auth_service.verify_signature(wallet, "message", "!!!invalid!!!");
        assert!(result.is_err());
    }

    // ========================================
    // AuthChallenge Tests
    // ========================================

    #[test]
    fn test_auth_challenge_serialize() {
        let challenge = AuthChallenge {
            message: "Test message".to_string(),
            nonce: "test-nonce".to_string(),
            expires_at: 1700000000,
        };
        
        let json = serde_json::to_string(&challenge).unwrap();
        assert!(json.contains("Test message"));
        assert!(json.contains("test-nonce"));
        assert!(json.contains("1700000000"));
    }

    // ========================================
    // AuthRequest Tests
    // ========================================

    #[test]
    fn test_auth_request_deserialize() {
        let json = r#"{
            "wallet_address": "TestWallet123",
            "signature": "TestSignature456",
            "message": "Test message"
        }"#;
        
        let request: AuthRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.wallet_address, "TestWallet123");
        assert_eq!(request.signature, "TestSignature456");
        assert_eq!(request.message, "Test message");
    }

    // ========================================
    // AuthResponse Tests
    // ========================================

    #[test]
    fn test_auth_response_serialize() {
        let response = AuthResponse {
            token: "jwt.token.here".to_string(),
            expires_at: 1700000000,
            player_id: "player-123".to_string(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("jwt.token.here"));
        assert!(json.contains("1700000000"));
        assert!(json.contains("player-123"));
    }
}
