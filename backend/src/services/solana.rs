//! Solana blockchain integration service
//!
//! Handles all on-chain operations including:
//! - Titan NFT minting
//! - $BREACH token transfers
//! - Game logic contract interactions

use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program::ID as SYSTEM_PROGRAM_ID,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TOKEN_PROGRAM_ID;

use crate::config::SolanaConfig;
use crate::error::{ApiResult, AppError};
use crate::models::Element;

/// Solana service for blockchain interactions
#[derive(Clone)]
pub struct SolanaService {
    config: SolanaConfig,
    rpc_client: std::sync::Arc<RpcClient>,
    backend_keypair: std::sync::Arc<Keypair>,
    titan_program_id: Pubkey,
    game_program_id: Pubkey,
    breach_token_mint: Pubkey,
}

/// Titan NFT data for minting
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct TitanMintData {
    pub element: u8,
    pub threat_class: u8,
    pub species_id: u32,
    pub genes: [u8; 32],
}

/// Battle record data for on-chain storage
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BattleRecordData {
    pub player: [u8; 32],
    pub titan_id: [u8; 32],
    pub battle_type: u8, // 0 = wild, 1 = pvp
    pub result: u8,      // 0 = loss, 1 = win, 2 = draw
    pub timestamp: i64,
}

/// Capture record data for on-chain storage
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CaptureRecordData {
    pub player: [u8; 32],
    pub titan_id: [u8; 32],
    pub location_hash: [u8; 8], // Geohash as bytes
    pub timestamp: i64,
}

/// Mint result containing transaction signature and NFT address
#[derive(Debug, Clone)]
pub struct MintResult {
    pub signature: String,
    pub mint_address: String,
    pub token_account: String,
}

/// Transfer result containing transaction signature
#[derive(Debug, Clone)]
pub struct TransferResult {
    pub signature: String,
    pub amount: u64,
}

impl SolanaService {
    /// Create a new Solana service
    pub fn new(config: &SolanaConfig) -> ApiResult<Self> {
        // Parse program IDs
        let titan_program_id = Pubkey::from_str(&config.titan_program_id)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid Titan program ID: {}", e)))?;
        
        let game_program_id = Pubkey::from_str(&config.game_program_id)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid Game program ID: {}", e)))?;
        
        let breach_token_mint = Pubkey::from_str(&config.breach_token_mint)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid BREACH token mint: {}", e)))?;

        // Load backend keypair
        let keypair_path = shellexpand::tilde(&config.backend_keypair_path).to_string();
        let keypair_bytes = std::fs::read(&keypair_path)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to read keypair: {}", e)))?;
        
        let keypair_json: Vec<u8> = serde_json::from_slice(&keypair_bytes)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid keypair format: {}", e)))?;
        
        let backend_keypair = Keypair::try_from(keypair_json.as_slice())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create keypair: {}", e)))?;

        // Create RPC client
        let rpc_client = RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        Ok(Self {
            config: config.clone(),
            rpc_client: std::sync::Arc::new(rpc_client),
            backend_keypair: std::sync::Arc::new(backend_keypair),
            titan_program_id,
            game_program_id,
            breach_token_mint,
        })
    }

    /// Create a new Solana service without loading keypair (for testing)
    pub fn new_without_keypair(config: &SolanaConfig) -> ApiResult<Self> {
        let titan_program_id = Pubkey::from_str(&config.titan_program_id)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid Titan program ID: {}", e)))?;
        
        let game_program_id = Pubkey::from_str(&config.game_program_id)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid Game program ID: {}", e)))?;
        
        let breach_token_mint = Pubkey::from_str(&config.breach_token_mint)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid BREACH token mint: {}", e)))?;

        let rpc_client = RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        // Generate a dummy keypair for testing
        let backend_keypair = Keypair::new();

        Ok(Self {
            config: config.clone(),
            rpc_client: std::sync::Arc::new(rpc_client),
            backend_keypair: std::sync::Arc::new(backend_keypair),
            titan_program_id,
            game_program_id,
            breach_token_mint,
        })
    }

    /// Get backend wallet public key
    pub fn backend_pubkey(&self) -> Pubkey {
        self.backend_keypair.pubkey()
    }

    /// Get current SOL balance for an address
    pub async fn get_balance(&self, address: &str) -> ApiResult<u64> {
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| AppError::BadRequest(format!("Invalid address: {}", e)))?;
        
        let balance = self.rpc_client.get_balance(&pubkey).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("RPC error: {}", e)))?;
        
        Ok(balance)
    }

    /// Get $BREACH token balance for an address
    pub async fn get_breach_balance(&self, address: &str) -> ApiResult<u64> {
        let owner = Pubkey::from_str(address)
            .map_err(|e| AppError::BadRequest(format!("Invalid address: {}", e)))?;
        
        let token_account = get_associated_token_address(&owner, &self.breach_token_mint);
        
        match self.rpc_client.get_token_account_balance(&token_account).await {
            Ok(balance) => {
                let amount = balance.amount.parse::<u64>().unwrap_or(0);
                Ok(amount)
            }
            Err(_) => Ok(0), // Account doesn't exist = 0 balance
        }
    }

    /// Mint a new Titan NFT
    /// 
    /// This calls the Titan NFT program to create a new NFT
    /// with the specified attributes.
    pub async fn mint_titan_nft(
        &self,
        player_wallet: &str,
        element: Element,
        threat_class: u8,
        species_id: u32,
        genes: [u8; 32],
    ) -> ApiResult<MintResult> {
        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;

        // Generate new mint keypair for the NFT
        let mint_keypair = Keypair::new();
        let mint_pubkey = mint_keypair.pubkey();

        // Derive PDA for titan data account
        let (titan_data_pda, _bump) = Pubkey::find_program_address(
            &[b"titan", mint_pubkey.as_ref()],
            &self.titan_program_id,
        );

        // Get player's associated token account
        let player_token_account = get_associated_token_address(&player, &mint_pubkey);

        // Prepare mint instruction data
        let mint_data = TitanMintData {
            element: element.as_u8(),
            threat_class,
            species_id,
            genes,
        };

        // Build instruction
        // Instruction layout: [discriminator(8)] + [data]
        let mut instruction_data = vec![0u8; 8]; // Discriminator for "mint_titan"
        instruction_data.extend(borsh::to_vec(&mint_data)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Borsh error: {}", e)))?);

        let accounts = vec![
            AccountMeta::new(mint_pubkey, true),           // mint (signer)
            AccountMeta::new(titan_data_pda, false),       // titan_data PDA
            AccountMeta::new(player_token_account, false), // player token account
            AccountMeta::new(player, false),               // player
            AccountMeta::new(self.backend_keypair.pubkey(), true), // authority (signer)
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        ];

        let instruction = Instruction {
            program_id: self.titan_program_id,
            accounts,
            data: instruction_data,
        };

        // Get recent blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get blockhash: {}", e)))?;

        // Create and sign transaction
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.backend_keypair.pubkey()),
            &[&*self.backend_keypair, &mint_keypair],
            recent_blockhash,
        );

        // Send transaction
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Transaction failed: {}", e)))?;

        Ok(MintResult {
            signature: signature.to_string(),
            mint_address: mint_pubkey.to_string(),
            token_account: player_token_account.to_string(),
        })
    }

    /// Transfer $BREACH tokens to a player as reward
    pub async fn transfer_breach_tokens(
        &self,
        recipient_wallet: &str,
        amount: u64,
    ) -> ApiResult<TransferResult> {
        let recipient = Pubkey::from_str(recipient_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid recipient wallet: {}", e)))?;

        // Get backend's token account
        let backend_token_account = get_associated_token_address(
            &self.backend_keypair.pubkey(),
            &self.breach_token_mint,
        );

        // Get or create recipient's token account
        let recipient_token_account = get_associated_token_address(&recipient, &self.breach_token_mint);

        // Check if recipient account exists, if not, create it
        let account_exists = self.rpc_client
            .get_account(&recipient_token_account)
            .await
            .is_ok();

        let mut instructions = Vec::new();

        // Create associated token account if needed
        if !account_exists {
            let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
                &self.backend_keypair.pubkey(),
                &recipient,
                &self.breach_token_mint,
                &TOKEN_PROGRAM_ID,
            );
            instructions.push(create_ata_ix);
        }

        // Transfer instruction
        let transfer_ix = spl_token::instruction::transfer(
            &TOKEN_PROGRAM_ID,
            &backend_token_account,
            &recipient_token_account,
            &self.backend_keypair.pubkey(),
            &[],
            amount,
        ).map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create transfer: {}", e)))?;

        instructions.push(transfer_ix);

        // Get recent blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get blockhash: {}", e)))?;

        // Create and sign transaction
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&self.backend_keypair.pubkey()),
            &[&*self.backend_keypair],
            recent_blockhash,
        );

        // Send transaction
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Token transfer failed: {}", e)))?;

        Ok(TransferResult {
            signature: signature.to_string(),
            amount,
        })
    }

    /// Record a capture on the Game Logic program
    pub async fn record_capture(
        &self,
        player_wallet: &str,
        titan_mint: &str,
        geohash: &str,
    ) -> ApiResult<String> {
        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;
        
        let titan_id = Pubkey::from_str(titan_mint)
            .map_err(|e| AppError::BadRequest(format!("Invalid titan mint: {}", e)))?;

        // Derive PDA for capture record
        let (capture_record_pda, _bump) = Pubkey::find_program_address(
            &[b"capture", player.as_ref(), titan_id.as_ref()],
            &self.game_program_id,
        );

        // Prepare capture data
        let mut location_hash = [0u8; 8];
        let geohash_bytes = geohash.as_bytes();
        let len = geohash_bytes.len().min(8);
        location_hash[..len].copy_from_slice(&geohash_bytes[..len]);

        let capture_data = CaptureRecordData {
            player: player.to_bytes(),
            titan_id: titan_id.to_bytes(),
            location_hash,
            timestamp: chrono::Utc::now().timestamp(),
        };

        // Build instruction
        let mut instruction_data = vec![1u8; 8]; // Discriminator for "record_capture"
        instruction_data.extend(borsh::to_vec(&capture_data)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Borsh error: {}", e)))?);

        let accounts = vec![
            AccountMeta::new(capture_record_pda, false),
            AccountMeta::new_readonly(player, false),
            AccountMeta::new_readonly(titan_id, false),
            AccountMeta::new(self.backend_keypair.pubkey(), true), // backend authority
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];

        let instruction = Instruction {
            program_id: self.game_program_id,
            accounts,
            data: instruction_data,
        };

        let recent_blockhash = self.rpc_client.get_latest_blockhash().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get blockhash: {}", e)))?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.backend_keypair.pubkey()),
            &[&*self.backend_keypair],
            recent_blockhash,
        );

        let signature = self.rpc_client.send_and_confirm_transaction(&transaction).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Record capture failed: {}", e)))?;

        Ok(signature.to_string())
    }

    /// Record a battle result on the Game Logic program
    pub async fn record_battle(
        &self,
        player_wallet: &str,
        titan_mint: &str,
        battle_type: u8,
        result: u8,
    ) -> ApiResult<String> {
        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;
        
        let titan_id = Pubkey::from_str(titan_mint)
            .map_err(|e| AppError::BadRequest(format!("Invalid titan mint: {}", e)))?;

        // Derive PDA for battle record
        let timestamp = chrono::Utc::now().timestamp();
        let (battle_record_pda, _bump) = Pubkey::find_program_address(
            &[
                b"battle",
                player.as_ref(),
                &timestamp.to_le_bytes(),
            ],
            &self.game_program_id,
        );

        let battle_data = BattleRecordData {
            player: player.to_bytes(),
            titan_id: titan_id.to_bytes(),
            battle_type,
            result,
            timestamp,
        };

        // Build instruction
        let mut instruction_data = vec![2u8; 8]; // Discriminator for "record_battle"
        instruction_data.extend(borsh::to_vec(&battle_data)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Borsh error: {}", e)))?);

        let accounts = vec![
            AccountMeta::new(battle_record_pda, false),
            AccountMeta::new_readonly(player, false),
            AccountMeta::new_readonly(titan_id, false),
            AccountMeta::new(self.backend_keypair.pubkey(), true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];

        let instruction = Instruction {
            program_id: self.game_program_id,
            accounts,
            data: instruction_data,
        };

        let recent_blockhash = self.rpc_client.get_latest_blockhash().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get blockhash: {}", e)))?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.backend_keypair.pubkey()),
            &[&*self.backend_keypair],
            recent_blockhash,
        );

        let signature = self.rpc_client.send_and_confirm_transaction(&transaction).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Record battle failed: {}", e)))?;

        Ok(signature.to_string())
    }

    /// Add experience to a player's Titan
    pub async fn add_titan_experience(
        &self,
        titan_mint: &str,
        experience: u64,
    ) -> ApiResult<String> {
        let titan = Pubkey::from_str(titan_mint)
            .map_err(|e| AppError::BadRequest(format!("Invalid titan mint: {}", e)))?;

        // Derive PDA for titan data
        let (titan_data_pda, _bump) = Pubkey::find_program_address(
            &[b"titan", titan.as_ref()],
            &self.titan_program_id,
        );

        // Build instruction
        let mut instruction_data = vec![3u8; 8]; // Discriminator for "add_experience"
        instruction_data.extend(experience.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(titan_data_pda, false),
            AccountMeta::new_readonly(titan, false),
            AccountMeta::new(self.backend_keypair.pubkey(), true),
        ];

        let instruction = Instruction {
            program_id: self.titan_program_id,
            accounts,
            data: instruction_data,
        };

        let recent_blockhash = self.rpc_client.get_latest_blockhash().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get blockhash: {}", e)))?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.backend_keypair.pubkey()),
            &[&*self.backend_keypair],
            recent_blockhash,
        );

        let signature = self.rpc_client.send_and_confirm_transaction(&transaction).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Add experience failed: {}", e)))?;

        Ok(signature.to_string())
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, signature: &str) -> ApiResult<Option<bool>> {
        use solana_sdk::signature::Signature;
        
        let sig = Signature::from_str(signature)
            .map_err(|e| AppError::BadRequest(format!("Invalid signature: {}", e)))?;
        
        match self.rpc_client.get_signature_status(&sig).await {
            Ok(Some(status)) => Ok(Some(status.is_ok())),
            Ok(None) => Ok(None), // Transaction not found
            Err(e) => Err(AppError::Internal(anyhow::anyhow!("RPC error: {}", e))),
        }
    }

    /// Airdrop SOL (devnet only)
    #[cfg(debug_assertions)]
    pub async fn request_airdrop(&self, address: &str, lamports: u64) -> ApiResult<String> {
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| AppError::BadRequest(format!("Invalid address: {}", e)))?;
        
        let signature = self.rpc_client.request_airdrop(&pubkey, lamports).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Airdrop failed: {}", e)))?;
        
        Ok(signature.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SolanaConfig {
        SolanaConfig {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            ws_url: "wss://api.devnet.solana.com".to_string(),
            titan_program_id: "3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7".to_string(),
            game_program_id: "DLk2GnDu9AYn7PeLprEDHDYH9UWKENX47UqqfeiQBaSX".to_string(),
            breach_token_mint: "CSH2Vz4MbgTLzB9SYJ7gBwNsyu7nKpbvEJzKQLgmmjt4".to_string(),
            backend_keypair_path: "~/.config/solana/id.json".to_string(),
        }
    }

    #[test]
    fn test_create_service_without_keypair() {
        let config = test_config();
        let service = SolanaService::new_without_keypair(&config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_titan_mint_data_serialization() {
        let data = TitanMintData {
            element: 1,
            threat_class: 3,
            species_id: 42,
            genes: [0u8; 32],
        };
        
        let bytes = borsh::to_vec(&data).unwrap();
        assert!(!bytes.is_empty());
        
        let decoded: TitanMintData = BorshDeserialize::try_from_slice(&bytes).unwrap();
        assert_eq!(decoded.element, 1);
        assert_eq!(decoded.threat_class, 3);
        assert_eq!(decoded.species_id, 42);
    }

    #[test]
    fn test_capture_record_data_serialization() {
        let data = CaptureRecordData {
            player: [1u8; 32],
            titan_id: [2u8; 32],
            location_hash: [0x78, 0x6e, 0x37, 0x37, 0x68, 0x00, 0x00, 0x00], // "xn77h"
            timestamp: 1700000000,
        };
        
        let bytes = borsh::to_vec(&data).unwrap();
        assert!(!bytes.is_empty());
        
        let decoded: CaptureRecordData = BorshDeserialize::try_from_slice(&bytes).unwrap();
        assert_eq!(decoded.timestamp, 1700000000);
    }

    #[test]
    fn test_battle_record_data_serialization() {
        let data = BattleRecordData {
            player: [1u8; 32],
            titan_id: [2u8; 32],
            battle_type: 1,
            result: 1,
            timestamp: 1700000000,
        };
        
        let bytes = borsh::to_vec(&data).unwrap();
        let decoded: BattleRecordData = BorshDeserialize::try_from_slice(&bytes).unwrap();
        assert_eq!(decoded.battle_type, 1);
        assert_eq!(decoded.result, 1);
    }
}
