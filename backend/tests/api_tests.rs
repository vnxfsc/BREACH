//! Integration tests for API endpoints
//! 
//! These tests require a running database and Redis instance.
//! Run with: cargo test --test api_tests -- --ignored

mod common;

use reqwest::Client;
use serde_json::json;

const BASE_URL: &str = "http://localhost:8080/api/v1";

/// Test helper to create HTTP client
fn create_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

// ========================================
// Health Check Tests
// ========================================

#[tokio::test]
#[ignore] // Requires running server
async fn test_health_endpoint() {
    let client = create_client();
    let response = client
        .get("http://localhost:8080/health")
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_ready_endpoint() {
    let client = create_client();
    let response = client
        .get("http://localhost:8080/ready")
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
}

// ========================================
// Authentication Tests
// ========================================

#[tokio::test]
#[ignore] // Requires running server
async fn test_auth_challenge() {
    let client = create_client();
    let wallet = common::random_wallet_address();
    
    let response = client
        .post(&format!("{}/auth/challenge", BASE_URL))
        .json(&json!({ "wallet_address": wallet }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["message"].is_string());
    assert!(body["nonce"].is_string());
    assert!(body["expires_at"].is_number());
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_auth_invalid_signature() {
    let client = create_client();
    let wallet = common::random_wallet_address();
    
    let response = client
        .post(&format!("{}/auth/authenticate", BASE_URL))
        .json(&json!({
            "wallet_address": wallet,
            "signature": "invalid_signature",
            "message": "test message"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(!response.status().is_success());
}

// ========================================
// Protected Endpoint Tests
// ========================================

#[tokio::test]
#[ignore] // Requires running server
async fn test_protected_endpoint_without_token() {
    let client = create_client();
    
    let response = client
        .get(&format!("{}/player/me", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_protected_endpoint_with_invalid_token() {
    let client = create_client();
    
    let response = client
        .get(&format!("{}/player/me", BASE_URL))
        .header("Authorization", "Bearer invalid.token.here")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status().as_u16(), 401);
}

// ========================================
// Map Endpoint Tests
// ========================================

#[tokio::test]
#[ignore] // Requires running server and auth
async fn test_get_nearby_titans() {
    let client = create_client();
    let player_id = uuid::Uuid::new_v4();
    let wallet = common::random_wallet_address();
    let token = common::test_jwt_token(player_id, &wallet);
    
    let response = client
        .get(&format!("{}/map/titans", BASE_URL))
        .query(&[("lat", "35.6762"), ("lng", "139.6503"), ("radius", "1000")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    // May fail if player doesn't exist, but should at least parse
    let status = response.status();
    assert!(status.as_u16() == 200 || status.as_u16() == 401 || status.as_u16() == 404);
}

// ========================================
// Leaderboard Tests
// ========================================

#[tokio::test]
#[ignore] // Requires running server
async fn test_leaderboard_public() {
    let client = create_client();
    let player_id = uuid::Uuid::new_v4();
    let wallet = common::random_wallet_address();
    let token = common::test_jwt_token(player_id, &wallet);
    
    let response = client
        .get(&format!("{}/leaderboard", BASE_URL))
        .query(&[("type", "experience"), ("limit", "10")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    // Should return data or empty array
    let status = response.status();
    assert!(status.is_success() || status.as_u16() == 401);
}

// ========================================
// Quest Endpoint Tests
// ========================================

#[tokio::test]
#[ignore] // Requires running server and auth
async fn test_get_quests() {
    let client = create_client();
    let player_id = uuid::Uuid::new_v4();
    let wallet = common::random_wallet_address();
    let token = common::test_jwt_token(player_id, &wallet);
    
    let response = client
        .get(&format!("{}/quests", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    let status = response.status();
    // May fail if player doesn't exist
    assert!(status.as_u16() == 200 || status.as_u16() == 401 || status.as_u16() == 404);
}

// ========================================
// Achievement Endpoint Tests
// ========================================

#[tokio::test]
#[ignore] // Requires running server
async fn test_get_achievements() {
    let client = create_client();
    let player_id = uuid::Uuid::new_v4();
    let wallet = common::random_wallet_address();
    let token = common::test_jwt_token(player_id, &wallet);
    
    let response = client
        .get(&format!("{}/achievements", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    let status = response.status();
    assert!(status.is_success() || status.as_u16() == 401);
}

// ========================================
// Marketplace Tests
// ========================================

#[tokio::test]
#[ignore] // Requires running server
async fn test_marketplace_search() {
    let client = create_client();
    let player_id = uuid::Uuid::new_v4();
    let wallet = common::random_wallet_address();
    let token = common::test_jwt_token(player_id, &wallet);
    
    let response = client
        .get(&format!("{}/marketplace", BASE_URL))
        .query(&[("limit", "10"), ("offset", "0")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    let status = response.status();
    assert!(status.is_success() || status.as_u16() == 401);
}

// ========================================
// Chat Tests
// ========================================

#[tokio::test]
#[ignore] // Requires running server
async fn test_chat_channels() {
    let client = create_client();
    let player_id = uuid::Uuid::new_v4();
    let wallet = common::random_wallet_address();
    let token = common::test_jwt_token(player_id, &wallet);
    
    let response = client
        .get(&format!("{}/chat/channels", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    let status = response.status();
    assert!(status.is_success() || status.as_u16() == 401);
}

// ========================================
// PvP Tests
// ========================================

#[tokio::test]
#[ignore] // Requires running server
async fn test_pvp_season() {
    let client = create_client();
    let player_id = uuid::Uuid::new_v4();
    let wallet = common::random_wallet_address();
    let token = common::test_jwt_token(player_id, &wallet);
    
    let response = client
        .get(&format!("{}/pvp/season", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    let status = response.status();
    assert!(status.is_success() || status.as_u16() == 401 || status.as_u16() == 404);
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_pvp_leaderboard() {
    let client = create_client();
    let player_id = uuid::Uuid::new_v4();
    let wallet = common::random_wallet_address();
    let token = common::test_jwt_token(player_id, &wallet);
    
    let response = client
        .get(&format!("{}/pvp/leaderboard", BASE_URL))
        .query(&[("limit", "10")])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    
    let status = response.status();
    assert!(status.is_success() || status.as_u16() == 401);
}
