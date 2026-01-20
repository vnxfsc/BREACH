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

use serde::Serialize;

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

/// Titan NFT data for minting (matches contract MintTitanData)
/// Total size: 88 bytes (packed)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct TitanMintData {
    pub species_id: u16,        // 2 bytes
    pub threat_class: u8,       // 1 byte
    pub element_type: u8,       // 1 byte
    pub power: u8,              // 1 byte
    pub fortitude: u8,          // 1 byte
    pub velocity: u8,           // 1 byte
    pub resonance: u8,          // 1 byte
    pub genes: [u8; 6],         // 6 bytes
    pub capture_lat: i32,       // 4 bytes
    pub capture_lng: i32,       // 4 bytes
    pub nonce: u64,             // 8 bytes
    pub signature: [u8; 64],    // 64 bytes - placeholder, not verified on-chain
}

impl TitanMintData {
    // 序列化为字节数组
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(88);
        bytes.extend_from_slice(&self.species_id.to_le_bytes());
        bytes.push(self.threat_class);
        bytes.push(self.element_type);
        bytes.push(self.power);
        bytes.push(self.fortitude);
        bytes.push(self.velocity);
        bytes.push(self.resonance);
        bytes.extend_from_slice(&self.genes);
        bytes.extend_from_slice(&self.capture_lat.to_le_bytes());
        bytes.extend_from_slice(&self.capture_lng.to_le_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes.extend_from_slice(&self.signature);
        bytes
    }
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
    /// 
    /// Account layout (must match contract):
    /// [0] payer - 玩家钱包 (签名者)
    /// [1] config_account - Config PDA
    /// [2] player_account - Player PDA  
    /// [3] titan_account - Titan PDA
    /// [4] capture_authority - 后端钱包 (签名者)
    /// [5] system_program
    pub async fn mint_titan_nft(
        &self,
        player_wallet: &str,
        element: Element,
        threat_class: u8,
        species_id: u32,
        genes: [u8; 32],
    ) -> ApiResult<MintResult> {
        // 注意：当前实现使用后端钱包作为 payer (测试用)
        // 真正的玩家钱包地址仅作为记录保留
        let _player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;
        
        // 临时方案：使用后端作为 payer
        // TODO: 生产环境应修改合约支持 mint_titan_for_player 或使用前端签名
        let payer = self.backend_keypair.pubkey();
        
        tracing::info!(
            "Minting Titan NFT: player={}, payer(backend)={}, element={:?}, threat_class={}, species_id={}",
            player_wallet, payer, element, threat_class, species_id
        );

        // 获取当前 config 来确定 titan_id
        let (config_pda, _) = Pubkey::find_program_address(
            &[b"config"],
            &self.titan_program_id,
        );
        
        // 读取 config 获取 total_titans_minted
        let config_account = self.rpc_client.get_account(&config_pda).await
            .map_err(|e| {
                tracing::error!("Failed to get config account: {}", e);
                AppError::Internal(anyhow::anyhow!("Failed to get config: {}", e))
            })?;
        
        tracing::debug!("Config account data length: {}", config_account.data.len());
        
        // GlobalConfig 布局 (packed, 无填充):
        // - discriminator: [u8; 8] - offset 0-7
        // - authority: Pubkey - offset 8-39
        // - treasury: Pubkey - offset 40-71
        // - breach_mint: Pubkey - offset 72-103
        // - capture_authority: Pubkey - offset 104-135
        // - capture_fee_bps: u16 - offset 136-137
        // - marketplace_fee_bps: u16 - offset 138-139
        // - fusion_fee_bps: u16 - offset 140-141
        // - max_titans_per_wallet: u16 - offset 142-143
        // - capture_cooldown_seconds: u32 - offset 144-147
        // - paused: bool - offset 148
        // - bump: u8 - offset 149
        // - total_titans_minted: u64 - offset 150-157
        // 总大小: 182 bytes
        let total_minted = if config_account.data.len() >= 158 {
            u64::from_le_bytes(config_account.data[150..158].try_into().unwrap_or([0u8; 8]))
        } else {
            tracing::warn!("Config account too small, assuming total_minted=0");
            0
        };
        let titan_id = total_minted + 1;
        tracing::info!("Next Titan ID: {}", titan_id);

        // Derive Player PDA (使用后端作为 payer)
        let (player_pda, _) = Pubkey::find_program_address(
            &[b"player", payer.as_ref()],
            &self.titan_program_id,
        );
        tracing::debug!("Player PDA (for backend): {}", player_pda);

        // Derive Titan PDA
        let titan_id_bytes = titan_id.to_le_bytes();
        let (titan_pda, _) = Pubkey::find_program_address(
            &[b"titan", &titan_id_bytes],
            &self.titan_program_id,
        );
        tracing::debug!("Titan PDA: {}", titan_pda);

        // 生成随机属性 (power, fortitude, velocity, resonance)
        // 在单独的块中使用 rng，避免跨 await 点
        let (power, fortitude, velocity, resonance) = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            (
                rng.gen_range(10..100u8),
                rng.gen_range(10..100u8),
                rng.gen_range(10..100u8),
                rng.gen_range(10..100u8),
            )
        };

        // 生成 6 字节基因
        let mut genes_6: [u8; 6] = [0u8; 6];
        genes_6.copy_from_slice(&genes[..6]);

        // 准备 mint 指令数据 (instruction discriminator 1 = mint_titan)
        let mint_data = TitanMintData {
            species_id: species_id as u16,
            threat_class,
            element_type: element.as_u8(),
            power,
            fortitude,
            velocity,
            resonance,
            genes: genes_6,
            capture_lat: 35658600,  // 东京纬度 * 10^6
            capture_lng: 139745200, // 东京经度 * 10^6
            nonce: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: [0u8; 64], // 占位符，合约不验证
        };

        // 构建指令数据: discriminator(1) + MintTitanData
        let mut instruction_data = vec![1u8]; // 1 = mint_titan instruction
        instruction_data.extend(mint_data.to_bytes());

        // 账户列表 (必须匹配合约)
        // 临时方案：后端既是 payer 又是 capture_authority
        let accounts = vec![
            AccountMeta::new(payer, true),                            // [0] payer (后端签名者，临时方案)
            AccountMeta::new(config_pda, false),                      // [1] config_account
            AccountMeta::new(player_pda, false),                      // [2] player_account
            AccountMeta::new(titan_pda, false),                       // [3] titan_account
            AccountMeta::new(self.backend_keypair.pubkey(), true),    // [4] capture_authority (后端签名者)
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),      // [5] system_program
        ];
        
        tracing::debug!(
            "Mint instruction accounts: payer={}, config={}, player_pda={}, titan_pda={}, capture_auth={}",
            payer, config_pda, player_pda, titan_pda, self.backend_keypair.pubkey()
        );

        let instruction = Instruction {
            program_id: self.titan_program_id,
            accounts,
            data: instruction_data,
        };

        // 获取最新 blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get blockhash: {}", e)))?;

        // 注意: 这里需要玩家签名，但后端无法获取玩家私钥
        // 在实际应用中，应该使用交易预签名或由前端发起交易
        // 目前的实现是后端代付，玩家不需要签名
        // 这需要修改合约或使用不同的方式
        
        // 暂时使用后端作为 payer (这需要合约支持)
        // 创建交易，后端作为唯一签名者
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.backend_keypair.pubkey()),
            &[&*self.backend_keypair],
            recent_blockhash,
        );

        // 发送交易
        tracing::info!("Sending mint transaction to Solana...");
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction).await
            .map_err(|e| {
                tracing::error!("Mint transaction failed: {:?}", e);
                AppError::Internal(anyhow::anyhow!("Mint transaction failed: {}", e))
            })?;
        
        tracing::info!("Mint transaction successful: {}", signature);

        Ok(MintResult {
            signature: signature.to_string(),
            mint_address: titan_pda.to_string(), // Titan PDA 作为 NFT 地址
            token_account: player_pda.to_string(), // Player PDA
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

    // ═══════════════════════════════════════════════════════════════════════════
    // 生产级交易构建 API（支持前端签名）
    // ═══════════════════════════════════════════════════════════════════════════

    /// 构建 NFT 铸造交易（未签名）
    /// 
    /// 返回序列化的交易，供前端签名
    /// 流程：
    /// 1. 后端构建交易，返回 base64 编码的交易
    /// 2. 前端用钱包签名
    /// 3. 前端调用 submit_signed_transaction 提交
    pub async fn build_mint_transaction(
        &self,
        player_wallet: &str,
        element: Element,
        threat_class: u8,
        species_id: u32,
        genes: [u8; 32],
        capture_lat: i32,
        capture_lng: i32,
    ) -> ApiResult<BuildTransactionResult> {
        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;
        
        tracing::info!(
            "Building mint transaction: player={}, element={:?}, threat_class={}, species_id={}",
            player_wallet, element, threat_class, species_id
        );

        // 获取 config PDA
        let (config_pda, _) = Pubkey::find_program_address(
            &[b"config"],
            &self.titan_program_id,
        );
        
        // 读取 config 获取 total_titans_minted
        let config_account = self.rpc_client.get_account(&config_pda).await
            .map_err(|e| {
                tracing::error!("Failed to get config account: {}", e);
                AppError::Internal(anyhow::anyhow!("Failed to get config: {}", e))
            })?;
        
        // 从 config 读取 total_titans_minted (offset 150-157)
        let total_minted = if config_account.data.len() >= 158 {
            u64::from_le_bytes(config_account.data[150..158].try_into().unwrap_or([0u8; 8]))
        } else {
            0
        };
        let titan_id = total_minted + 1;
        tracing::info!("Next Titan ID: {}", titan_id);

        // Derive Player PDA (基于玩家钱包)
        let (player_pda, _) = Pubkey::find_program_address(
            &[b"player", player.as_ref()],
            &self.titan_program_id,
        );

        // Derive Titan PDA
        let titan_id_bytes = titan_id.to_le_bytes();
        let (titan_pda, _) = Pubkey::find_program_address(
            &[b"titan", &titan_id_bytes],
            &self.titan_program_id,
        );

        // 生成随机属性
        let (power, fortitude, velocity, resonance) = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            (
                rng.gen_range(10..100u8),
                rng.gen_range(10..100u8),
                rng.gen_range(10..100u8),
                rng.gen_range(10..100u8),
            )
        };

        // 构建 MintTitanData
        let mut genes_6: [u8; 6] = [0u8; 6];
        genes_6.copy_from_slice(&genes[..6]);

        let mint_data = TitanMintData {
            species_id: species_id as u16,
            threat_class,
            element_type: element.as_u8(),
            power,
            fortitude,
            velocity,
            resonance,
            genes: genes_6,
            capture_lat,
            capture_lng,
            nonce: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: [0u8; 64],
        };

        // 构建指令数据
        let mut instruction_data = vec![1u8]; // discriminator = 1 (mint_titan)
        instruction_data.extend(mint_data.to_bytes());

        // 账户列表 - 玩家作为 payer 和签名者
        let accounts = vec![
            AccountMeta::new(player, true),                            // [0] payer (玩家签名)
            AccountMeta::new(config_pda, false),                       // [1] config_account
            AccountMeta::new(player_pda, false),                       // [2] player_account
            AccountMeta::new(titan_pda, false),                        // [3] titan_account
            AccountMeta::new(self.backend_keypair.pubkey(), true),     // [4] capture_authority (后端签名)
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),       // [5] system_program
        ];

        let instruction = Instruction {
            program_id: self.titan_program_id,
            accounts,
            data: instruction_data,
        };

        // 获取最新 blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get blockhash: {}", e)))?;

        // 创建未签名的交易
        use solana_sdk::message::Message;
        let message = Message::new_with_blockhash(
            &[instruction],
            Some(&player), // fee payer 是玩家
            &recent_blockhash,
        );
        
        // 创建带有空签名槽的交易
        let mut transaction = Transaction::new_unsigned(message);
        // 初始化签名数组（空签名）
        let num_signers = transaction.message.header.num_required_signatures as usize;
        transaction.signatures = vec![solana_sdk::signature::Signature::default(); num_signers];

        // 使用 bincode 序列化整个交易
        let serialized_tx = bincode::serialize(&transaction)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to serialize transaction: {}", e)))?;
        
        // Base64 编码
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        let encoded_tx = BASE64.encode(&serialized_tx);

        tracing::info!(
            "Built mint transaction: player={}, titan_pda={}, tx_len={}, signers={}",
            player, titan_pda, encoded_tx.len(), num_signers
        );

        // 同时返回用于签名的消息字节（前端签名需要）
        let message_to_sign = transaction.message.serialize();
        let encoded_message = BASE64.encode(&message_to_sign);

        Ok(BuildTransactionResult {
            serialized_transaction: encoded_tx,
            message_to_sign: encoded_message,
            recent_blockhash: recent_blockhash.to_string(),
            titan_pda: titan_pda.to_string(),
            player_pda: player_pda.to_string(),
            titan_id,
        })
    }

    /// 提交已签名的交易
    /// 
    /// 接收前端签名和原始交易，后端添加签名并广播
    /// 
    /// 流程：
    /// 1. 前端对 message_to_sign 进行签名
    /// 2. 前端把签名 + 原始交易发给后端
    /// 3. 后端验证签名、添加自己的签名、广播
    pub async fn submit_signed_transaction(
        &self,
        serialized_transaction: &str,
        player_signature: &str,
        player_wallet: &str,
    ) -> ApiResult<SubmitTransactionResult> {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        use solana_sdk::signature::Signature;
        
        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;

        // 解码原始交易
        let tx_bytes = BASE64.decode(serialized_transaction)
            .map_err(|e| AppError::BadRequest(format!("Invalid base64 transaction: {}", e)))?;
        
        // 反序列化交易
        let mut transaction: Transaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| AppError::BadRequest(format!("Invalid transaction format: {}", e)))?;

        // 解码玩家签名
        let sig_bytes = BASE64.decode(player_signature)
            .map_err(|e| AppError::BadRequest(format!("Invalid base64 signature: {}", e)))?;
        
        if sig_bytes.len() != 64 {
            return Err(AppError::BadRequest(format!("Invalid signature length: {}", sig_bytes.len())));
        }
        
        let player_sig = Signature::try_from(sig_bytes.as_slice())
            .map_err(|e| AppError::BadRequest(format!("Invalid signature: {}", e)))?;

        tracing::info!(
            "Received transaction with {} signatures, {} account keys",
            transaction.signatures.len(),
            transaction.message.account_keys.len()
        );

        // 验证玩家签名
        let message_bytes = transaction.message.serialize();
        
        if !player_sig.verify(player.as_ref(), &message_bytes) {
            return Err(AppError::BadRequest("Invalid player signature".to_string()));
        }
        
        tracing::info!("Player signature verified for {}", player);

        // 找到玩家和后端在签名数组中的位置
        let player_sig_idx = transaction.message.account_keys
            .iter()
            .position(|k| k == &player)
            .ok_or_else(|| AppError::BadRequest("Player not found in account keys".to_string()))?;
        
        let backend_sig_idx = transaction.message.account_keys
            .iter()
            .position(|k| k == &self.backend_keypair.pubkey())
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Backend not found in account keys")))?;

        // 设置玩家签名
        if player_sig_idx < transaction.signatures.len() {
            transaction.signatures[player_sig_idx] = player_sig;
        } else {
            return Err(AppError::BadRequest("Player signature index out of bounds".to_string()));
        }
        
        // 添加后端签名
        let backend_sig = self.backend_keypair.sign_message(&message_bytes);
        
        if backend_sig_idx < transaction.signatures.len() {
            transaction.signatures[backend_sig_idx] = backend_sig;
        } else {
            return Err(AppError::Internal(anyhow::anyhow!("Backend signature index out of bounds")));
        }

        tracing::info!("Submitting transaction with {} signatures", transaction.signatures.len());

        // 发送交易
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction).await
            .map_err(|e| {
                tracing::error!("Transaction submission failed: {:?}", e);
                AppError::Internal(anyhow::anyhow!("Transaction failed: {}", e))
            })?;

        tracing::info!("Transaction submitted successfully: {}", signature);

        Ok(SubmitTransactionResult {
            signature: signature.to_string(),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Titan 操作（Level Up, Evolve, Fuse, Transfer）
    // ═══════════════════════════════════════════════════════════════════════════

    /// 构建 Level Up 交易
    /// 
    /// 消耗经验值升级 Titan
    pub async fn build_level_up_transaction(
        &self,
        player_wallet: &str,
        titan_id: u64,
    ) -> ApiResult<SimpleTransactionResult> {
        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;

        // Derive Titan PDA
        let titan_id_bytes = titan_id.to_le_bytes();
        let (titan_pda, _) = Pubkey::find_program_address(
            &[b"titan", &titan_id_bytes],
            &self.titan_program_id,
        );

        // 构建指令 (discriminator = 2)
        let instruction_data = vec![2u8]; // LEVEL_UP

        let accounts = vec![
            AccountMeta::new(player, true),       // [0] owner (signer)
            AccountMeta::new(titan_pda, false),   // [1] titan_account
        ];

        let instruction = Instruction {
            program_id: self.titan_program_id,
            accounts,
            data: instruction_data,
        };

        self.build_simple_transaction(&player, instruction).await
    }

    /// 构建 Evolve 交易
    /// 
    /// 进化 Titan 到更高形态（需要等级 >= 30）
    pub async fn build_evolve_transaction(
        &self,
        player_wallet: &str,
        titan_id: u64,
        new_species_id: u16,
    ) -> ApiResult<SimpleTransactionResult> {
        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;

        // Derive Titan PDA
        let titan_id_bytes = titan_id.to_le_bytes();
        let (titan_pda, _) = Pubkey::find_program_address(
            &[b"titan", &titan_id_bytes],
            &self.titan_program_id,
        );

        // 构建指令 (discriminator = 3 + EvolveData)
        let mut instruction_data = vec![3u8]; // EVOLVE
        instruction_data.extend(new_species_id.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(player, true),       // [0] owner (signer)
            AccountMeta::new(titan_pda, false),   // [1] titan_account
        ];

        let instruction = Instruction {
            program_id: self.titan_program_id,
            accounts,
            data: instruction_data,
        };

        self.build_simple_transaction(&player, instruction).await
    }

    /// 构建 Fuse 交易
    /// 
    /// 融合两个 Titan 创建新的（需要同元素，等级 >= 20）
    pub async fn build_fuse_transaction(
        &self,
        player_wallet: &str,
        titan_a_id: u64,
        titan_b_id: u64,
    ) -> ApiResult<FuseTransactionResult> {
        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;

        // 获取 config
        let (config_pda, _) = Pubkey::find_program_address(
            &[b"config"],
            &self.titan_program_id,
        );

        // 读取 config 获取下一个 titan_id
        let config_account = self.rpc_client.get_account(&config_pda).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get config: {}", e)))?;
        
        let total_minted = if config_account.data.len() >= 158 {
            u64::from_le_bytes(config_account.data[150..158].try_into().unwrap_or([0u8; 8]))
        } else {
            0
        };
        let offspring_id = total_minted + 1;

        // Derive PDAs
        let titan_a_bytes = titan_a_id.to_le_bytes();
        let (titan_a_pda, _) = Pubkey::find_program_address(
            &[b"titan", &titan_a_bytes],
            &self.titan_program_id,
        );

        let titan_b_bytes = titan_b_id.to_le_bytes();
        let (titan_b_pda, _) = Pubkey::find_program_address(
            &[b"titan", &titan_b_bytes],
            &self.titan_program_id,
        );

        let offspring_bytes = offspring_id.to_le_bytes();
        let (offspring_pda, _) = Pubkey::find_program_address(
            &[b"titan", &offspring_bytes],
            &self.titan_program_id,
        );

        // 构建指令 (discriminator = 4)
        let instruction_data = vec![4u8]; // FUSE

        let accounts = vec![
            AccountMeta::new(player, true),                        // [0] owner (signer)
            AccountMeta::new(config_pda, false),                   // [1] config_account
            AccountMeta::new(titan_a_pda, false),                  // [2] titan_a
            AccountMeta::new(titan_b_pda, false),                  // [3] titan_b
            AccountMeta::new(offspring_pda, false),                // [4] offspring
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),   // [5] system_program
        ];

        let instruction = Instruction {
            program_id: self.titan_program_id,
            accounts,
            data: instruction_data,
        };

        let result = self.build_simple_transaction(&player, instruction).await?;

        Ok(FuseTransactionResult {
            serialized_transaction: result.serialized_transaction,
            message_to_sign: result.message_to_sign,
            recent_blockhash: result.recent_blockhash,
            offspring_id,
            offspring_pda: offspring_pda.to_string(),
        })
    }

    /// 构建 Transfer 交易
    /// 
    /// 转移 Titan 所有权给另一个玩家
    pub async fn build_transfer_transaction(
        &self,
        from_wallet: &str,
        to_wallet: &str,
        titan_id: u64,
    ) -> ApiResult<SimpleTransactionResult> {
        let from_owner = Pubkey::from_str(from_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid from wallet: {}", e)))?;
        let to_owner = Pubkey::from_str(to_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid to wallet: {}", e)))?;

        // 获取 config
        let (config_pda, _) = Pubkey::find_program_address(
            &[b"config"],
            &self.titan_program_id,
        );

        // Derive Titan PDA
        let titan_id_bytes = titan_id.to_le_bytes();
        let (titan_pda, _) = Pubkey::find_program_address(
            &[b"titan", &titan_id_bytes],
            &self.titan_program_id,
        );

        // Derive Player PDAs
        let (from_player_pda, _) = Pubkey::find_program_address(
            &[b"player", from_owner.as_ref()],
            &self.titan_program_id,
        );
        let (to_player_pda, _) = Pubkey::find_program_address(
            &[b"player", to_owner.as_ref()],
            &self.titan_program_id,
        );

        // 构建指令 (discriminator = 5)
        let instruction_data = vec![5u8]; // TRANSFER

        let accounts = vec![
            AccountMeta::new(from_owner, true),       // [0] from_owner (signer)
            AccountMeta::new_readonly(to_owner, false), // [1] to_owner
            AccountMeta::new_readonly(config_pda, false), // [2] config
            AccountMeta::new(titan_pda, false),       // [3] titan
            AccountMeta::new(from_player_pda, false), // [4] from_player
            AccountMeta::new(to_player_pda, false),   // [5] to_player
        ];

        let instruction = Instruction {
            program_id: self.titan_program_id,
            accounts,
            data: instruction_data,
        };

        self.build_simple_transaction(&from_owner, instruction).await
    }

    /// 通用交易构建辅助函数
    async fn build_simple_transaction(
        &self,
        payer: &Pubkey,
        instruction: Instruction,
    ) -> ApiResult<SimpleTransactionResult> {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        use solana_sdk::message::Message;

        let recent_blockhash = self.rpc_client.get_latest_blockhash().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get blockhash: {}", e)))?;

        let message = Message::new_with_blockhash(
            &[instruction],
            Some(payer),
            &recent_blockhash,
        );

        let mut transaction = Transaction::new_unsigned(message);
        let num_signers = transaction.message.header.num_required_signatures as usize;
        transaction.signatures = vec![solana_sdk::signature::Signature::default(); num_signers];

        let serialized_tx = bincode::serialize(&transaction)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to serialize: {}", e)))?;
        
        let message_to_sign = transaction.message.serialize();

        Ok(SimpleTransactionResult {
            serialized_transaction: BASE64.encode(&serialized_tx),
            message_to_sign: BASE64.encode(&message_to_sign),
            recent_blockhash: recent_blockhash.to_string(),
        })
    }

    /// 提交只需用户签名的交易（Level Up, Evolve, Transfer 等）
    pub async fn submit_user_signed_transaction(
        &self,
        serialized_transaction: &str,
        user_signature: &str,
        user_wallet: &str,
    ) -> ApiResult<SubmitTransactionResult> {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        use solana_sdk::signature::Signature;
        
        let user = Pubkey::from_str(user_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid user wallet: {}", e)))?;

        // 解码交易
        let tx_bytes = BASE64.decode(serialized_transaction)
            .map_err(|e| AppError::BadRequest(format!("Invalid base64 transaction: {}", e)))?;
        
        let mut transaction: Transaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| AppError::BadRequest(format!("Invalid transaction format: {}", e)))?;

        // 解码用户签名
        let sig_bytes = BASE64.decode(user_signature)
            .map_err(|e| AppError::BadRequest(format!("Invalid base64 signature: {}", e)))?;
        
        if sig_bytes.len() != 64 {
            return Err(AppError::BadRequest("Invalid signature length".to_string()));
        }
        
        let user_sig = Signature::try_from(sig_bytes.as_slice())
            .map_err(|e| AppError::BadRequest(format!("Invalid signature: {}", e)))?;

        // 验证签名
        let message_bytes = transaction.message.serialize();
        if !user_sig.verify(user.as_ref(), &message_bytes) {
            return Err(AppError::BadRequest("Invalid user signature".to_string()));
        }

        // 设置签名（用户是唯一签名者）
        if !transaction.signatures.is_empty() {
            transaction.signatures[0] = user_sig;
        }

        // 发送交易
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction).await
            .map_err(|e| {
                tracing::error!("Transaction failed: {:?}", e);
                AppError::Internal(anyhow::anyhow!("Transaction failed: {}", e))
            })?;

        Ok(SubmitTransactionResult {
            signature: signature.to_string(),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Game Logic 操作（Record Capture, Record Battle, Add Experience）
    // ═══════════════════════════════════════════════════════════════════════════

    /// 记录捕获到链上
    /// 
    /// 需要玩家和后端双签名
    pub async fn record_capture_onchain(
        &self,
        player_wallet: &str,
        titan_id: u64,
        location_lat: i32,
        location_lng: i32,
        threat_class: u8,
        element_type: u8,
    ) -> ApiResult<RecordCaptureResult> {
        let backend_keypair = &self.backend_keypair;

        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;

        // 获取 game config
        let (game_config_pda, _) = Pubkey::find_program_address(
            &[b"game_config"],
            &self.game_program_id,
        );

        // 读取 config 获取 total_captures
        let config_account = self.rpc_client.get_account(&game_config_pda).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get game config: {}", e)))?;
        
        // total_captures 在 offset 220 处 (228 - 8)
        let total_captures = if config_account.data.len() >= 228 {
            u64::from_le_bytes(config_account.data[212..220].try_into().unwrap_or([0u8; 8]))
        } else {
            0
        };
        let capture_id = total_captures + 1;

        // Derive capture record PDA
        let capture_id_bytes = capture_id.to_le_bytes();
        let (capture_record_pda, _) = Pubkey::find_program_address(
            &[b"capture", &capture_id_bytes],
            &self.game_program_id,
        );

        // 构建指令数据 (discriminator = 1)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut instruction_data = vec![1u8]; // RECORD_CAPTURE
        instruction_data.extend(titan_id.to_le_bytes());
        instruction_data.extend(location_lat.to_le_bytes());
        instruction_data.extend(location_lng.to_le_bytes());
        instruction_data.push(threat_class);
        instruction_data.push(element_type);
        instruction_data.extend(timestamp.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(player, true),                        // [0] player (signer)
            AccountMeta::new_readonly(backend_keypair.pubkey(), true), // [1] backend_authority (signer)
            AccountMeta::new(game_config_pda, false),              // [2] config
            AccountMeta::new(capture_record_pda, false),           // [3] capture_record
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),   // [4] system_program
        ];

        let instruction = Instruction {
            program_id: self.game_program_id,
            accounts,
            data: instruction_data,
        };

        // 构建双签名交易
        let result = self.build_dual_signed_transaction(&player, instruction).await?;

        Ok(RecordCaptureResult {
            serialized_transaction: result.serialized_transaction,
            message_to_sign: result.message_to_sign,
            recent_blockhash: result.recent_blockhash,
            capture_id,
            capture_record_pda: capture_record_pda.to_string(),
        })
    }

    /// 记录战斗到链上
    /// 
    /// 需要玩家A和后端双签名
    pub async fn record_battle_onchain(
        &self,
        player_a_wallet: &str,
        player_b_wallet: &str,
        titan_a_id: u64,
        titan_b_id: u64,
        winner: u8,
        exp_gained_a: u32,
        exp_gained_b: u32,
        location_lat: i32,
        location_lng: i32,
    ) -> ApiResult<RecordBattleResult> {
        let backend_keypair = &self.backend_keypair;

        let player_a = Pubkey::from_str(player_a_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player A wallet: {}", e)))?;
        let player_b = Pubkey::from_str(player_b_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player B wallet: {}", e)))?;

        // 获取 game config
        let (game_config_pda, _) = Pubkey::find_program_address(
            &[b"game_config"],
            &self.game_program_id,
        );

        // 读取 config 获取 total_battles
        let config_account = self.rpc_client.get_account(&game_config_pda).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get game config: {}", e)))?;
        
        // total_battles 在 offset 204 处
        let total_battles = if config_account.data.len() >= 228 {
            u64::from_le_bytes(config_account.data[204..212].try_into().unwrap_or([0u8; 8]))
        } else {
            0
        };
        let battle_id = total_battles + 1;

        // Derive battle record PDA
        let battle_id_bytes = battle_id.to_le_bytes();
        let (battle_record_pda, _) = Pubkey::find_program_address(
            &[b"battle", &battle_id_bytes],
            &self.game_program_id,
        );

        // 构建指令数据 (discriminator = 2)
        let mut instruction_data = vec![2u8]; // RECORD_BATTLE
        instruction_data.extend(titan_a_id.to_le_bytes());
        instruction_data.extend(titan_b_id.to_le_bytes());
        instruction_data.push(winner);
        instruction_data.extend(exp_gained_a.to_le_bytes());
        instruction_data.extend(exp_gained_b.to_le_bytes());
        instruction_data.extend(location_lat.to_le_bytes());
        instruction_data.extend(location_lng.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(player_a, true),                      // [0] player_a (signer)
            AccountMeta::new_readonly(player_b, false),            // [1] player_b
            AccountMeta::new_readonly(backend_keypair.pubkey(), true), // [2] backend_authority (signer)
            AccountMeta::new(game_config_pda, false),              // [3] config
            AccountMeta::new(battle_record_pda, false),            // [4] battle_record
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),   // [5] system_program
        ];

        let instruction = Instruction {
            program_id: self.game_program_id,
            accounts,
            data: instruction_data,
        };

        // 构建双签名交易
        let result = self.build_dual_signed_transaction(&player_a, instruction).await?;

        Ok(RecordBattleResult {
            serialized_transaction: result.serialized_transaction,
            message_to_sign: result.message_to_sign,
            recent_blockhash: result.recent_blockhash,
            battle_id,
            battle_record_pda: battle_record_pda.to_string(),
        })
    }

    /// 添加经验值到 Titan
    /// 
    /// 通过 game_logic 合约 CPI 调用 titan_nft 的 add_experience
    pub async fn add_experience_onchain(
        &self,
        player_wallet: &str,
        titan_id: u64,
        exp_amount: u32,
    ) -> ApiResult<AddExperienceResult> {
        let backend_keypair = &self.backend_keypair;

        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;

        // 获取 game config
        let (game_config_pda, _) = Pubkey::find_program_address(
            &[b"game_config"],
            &self.game_program_id,
        );

        // 获取 titan config
        let (titan_config_pda, _) = Pubkey::find_program_address(
            &[b"config"],
            &self.titan_program_id,
        );

        // 获取 titan PDA
        let titan_id_bytes = titan_id.to_le_bytes();
        let (titan_pda, _) = Pubkey::find_program_address(
            &[b"titan", &titan_id_bytes],
            &self.titan_program_id,
        );

        // 构建指令数据 (discriminator = 3)
        let mut instruction_data = vec![3u8]; // ADD_EXPERIENCE
        instruction_data.extend(titan_id.to_le_bytes());
        instruction_data.extend(exp_amount.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(player, true),                        // [0] player (signer)
            AccountMeta::new_readonly(backend_keypair.pubkey(), true), // [1] backend_authority (signer)
            AccountMeta::new(game_config_pda, false),              // [2] game_config
            AccountMeta::new(titan_pda, false),                    // [3] titan
            AccountMeta::new_readonly(titan_config_pda, false),    // [4] titan_config
            AccountMeta::new_readonly(self.titan_program_id, false), // [5] titan_program
        ];

        let instruction = Instruction {
            program_id: self.game_program_id,
            accounts,
            data: instruction_data,
        };

        // 构建双签名交易
        let result = self.build_dual_signed_transaction(&player, instruction).await?;

        Ok(AddExperienceResult {
            serialized_transaction: result.serialized_transaction,
            message_to_sign: result.message_to_sign,
            recent_blockhash: result.recent_blockhash,
            titan_id,
            exp_amount,
        })
    }

    /// 构建需要双签名的交易（玩家 + 后端）
    async fn build_dual_signed_transaction(
        &self,
        payer: &Pubkey,
        instruction: Instruction,
    ) -> ApiResult<SimpleTransactionResult> {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        use solana_sdk::message::Message;

        let recent_blockhash = self.rpc_client.get_latest_blockhash().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get blockhash: {}", e)))?;

        let message = Message::new_with_blockhash(
            &[instruction],
            Some(payer),
            &recent_blockhash,
        );

        let mut transaction = Transaction::new_unsigned(message);
        let num_signers = transaction.message.header.num_required_signatures as usize;
        transaction.signatures = vec![solana_sdk::signature::Signature::default(); num_signers];

        let serialized_tx = bincode::serialize(&transaction)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to serialize: {}", e)))?;
        
        let message_to_sign = transaction.message.serialize();

        Ok(SimpleTransactionResult {
            serialized_transaction: BASE64.encode(&serialized_tx),
            message_to_sign: BASE64.encode(&message_to_sign),
            recent_blockhash: recent_blockhash.to_string(),
        })
    }

    /// 分发 BREACH 代币奖励
    /// 
    /// 只需后端签名（直接执行）
    pub async fn distribute_breach_reward(
        &self,
        player_wallet: &str,
        reward_type: u8,
        amount: u64,
    ) -> ApiResult<SubmitTransactionResult> {
        let backend_keypair = &self.backend_keypair;
        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;

        // 获取 game config
        let (game_config_pda, _) = Pubkey::find_program_address(
            &[b"game_config"],
            &self.game_program_id,
        );

        // 读取 config 获取 reward_pool
        let config_account = self.rpc_client.get_account(&game_config_pda).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get game config: {}", e)))?;
        
        // reward_pool 在 offset 136 处 (32*4 + 32 + 8)
        if config_account.data.len() < 168 {
            return Err(AppError::Internal(anyhow::anyhow!("Invalid config account size")));
        }
        let reward_pool_bytes = &config_account.data[136..168];
        let reward_pool = Pubkey::new_from_array(
            reward_pool_bytes.try_into()
                .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid reward pool pubkey")))?
        );

        // 获取或创建玩家的 BREACH token 账户
        let player_token_account = spl_associated_token_account::get_associated_token_address(
            &player,
            &self.breach_token_mint,
        );

        // 检查玩家 token 账户是否存在
        let account_exists = self.rpc_client.get_account(&player_token_account).await.is_ok();
        
        let mut instructions = Vec::new();

        // 如果账户不存在，创建 ATA
        if !account_exists {
            let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
                &backend_keypair.pubkey(), // payer
                &player,                   // wallet
                &self.breach_token_mint,   // mint
                &spl_token::ID,
            );
            instructions.push(create_ata_ix);
        }

        // 构建 distribute_reward 指令 (discriminator = 4)
        let mut instruction_data = vec![4u8]; // DISTRIBUTE_REWARD
        instruction_data.push(reward_type);
        instruction_data.extend(amount.to_le_bytes());

        let distribute_ix = Instruction {
            program_id: self.game_program_id,
            accounts: vec![
                AccountMeta::new_readonly(backend_keypair.pubkey(), true), // [0] backend_authority (signer)
                AccountMeta::new(game_config_pda, false),                  // [1] config
                AccountMeta::new(reward_pool, false),                      // [2] reward_pool
                AccountMeta::new(player_token_account, false),             // [3] player_token_account
                AccountMeta::new_readonly(spl_token::ID, false),           // [4] token_program
            ],
            data: instruction_data,
        };
        instructions.push(distribute_ix);

        // 构建并发送交易
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to get blockhash: {}", e)))?;

        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&backend_keypair.pubkey()),
            &[backend_keypair],
            recent_blockhash,
        );

        tracing::info!("Distributing {} BREACH reward (type {}) to {}", 
            amount as f64 / 1_000_000_000.0, reward_type, player_wallet);

        let signature = self.rpc_client.send_and_confirm_transaction(&transaction).await
            .map_err(|e| {
                tracing::error!("Reward distribution failed: {:?}", e);
                AppError::Internal(anyhow::anyhow!("Reward distribution failed: {}", e))
            })?;

        Ok(SubmitTransactionResult {
            signature: signature.to_string(),
        })
    }

    /// 提交双签名交易（玩家签名 + 后端签名）
    pub async fn submit_dual_signed_transaction(
        &self,
        serialized_transaction: &str,
        player_signature: &str,
        player_wallet: &str,
    ) -> ApiResult<SubmitTransactionResult> {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        use solana_sdk::signature::Signature;

        let backend_keypair = &self.backend_keypair;
        
        let player = Pubkey::from_str(player_wallet)
            .map_err(|e| AppError::BadRequest(format!("Invalid player wallet: {}", e)))?;

        // 解码交易
        let tx_bytes = BASE64.decode(serialized_transaction)
            .map_err(|e| AppError::BadRequest(format!("Invalid base64 transaction: {}", e)))?;
        
        let mut transaction: Transaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| AppError::BadRequest(format!("Invalid transaction format: {}", e)))?;

        // 解码玩家签名
        let player_sig_bytes = BASE64.decode(player_signature)
            .map_err(|e| AppError::BadRequest(format!("Invalid base64 signature: {}", e)))?;
        
        if player_sig_bytes.len() != 64 {
            return Err(AppError::BadRequest("Invalid signature length".to_string()));
        }
        
        let player_sig = Signature::try_from(player_sig_bytes.as_slice())
            .map_err(|e| AppError::BadRequest(format!("Invalid signature: {}", e)))?;

        // 验证玩家签名
        let message_bytes = transaction.message.serialize();
        if !player_sig.verify(player.as_ref(), &message_bytes) {
            return Err(AppError::BadRequest("Invalid player signature".to_string()));
        }

        // 后端签名
        let backend_sig = backend_keypair.sign_message(&message_bytes);

        // 设置签名（玩家是第一个签名者，后端是第二个）
        // 签名顺序由 account_keys 中的顺序决定
        let account_keys = &transaction.message.account_keys;
        for (i, key) in account_keys.iter().enumerate() {
            if i >= transaction.signatures.len() {
                break;
            }
            if *key == player {
                transaction.signatures[i] = player_sig;
            } else if *key == backend_keypair.pubkey() {
                transaction.signatures[i] = backend_sig;
            }
        }

        tracing::info!("Submitting dual-signed transaction");

        // 发送交易
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction).await
            .map_err(|e| {
                tracing::error!("Transaction failed: {:?}", e);
                AppError::Internal(anyhow::anyhow!("Transaction failed: {}", e))
            })?;

        Ok(SubmitTransactionResult {
            signature: signature.to_string(),
        })
    }
}

/// Record Capture 结果
#[derive(Debug, Clone, Serialize)]
pub struct RecordCaptureResult {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
    pub capture_id: u64,
    pub capture_record_pda: String,
}

/// Record Battle 结果
#[derive(Debug, Clone, Serialize)]
pub struct RecordBattleResult {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
    pub battle_id: u64,
    pub battle_record_pda: String,
}

/// Add Experience 结果
#[derive(Debug, Clone, Serialize)]
pub struct AddExperienceResult {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
    pub titan_id: u64,
    pub exp_amount: u32,
}

/// 简单交易结果（只需用户签名）
#[derive(Debug, Clone, Serialize)]
pub struct SimpleTransactionResult {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
}

/// Fuse 交易结果
#[derive(Debug, Clone, Serialize)]
pub struct FuseTransactionResult {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
    pub offspring_id: u64,
    pub offspring_pda: String,
}

/// 构建交易的返回结果
#[derive(Debug, Clone, Serialize)]
pub struct BuildTransactionResult {
    /// Base64 编码的序列化交易（含空签名槽，bincode 格式）
    pub serialized_transaction: String,
    /// Base64 编码的消息字节（用于前端签名）
    pub message_to_sign: String,
    /// 最近的 blockhash
    pub recent_blockhash: String,
    /// Titan PDA 地址（mint_address）
    pub titan_pda: String,
    /// Player PDA 地址
    pub player_pda: String,
    /// Titan ID
    pub titan_id: u64,
}

/// 提交交易的返回结果
#[derive(Debug, Clone, Serialize)]
pub struct SubmitTransactionResult {
    /// 交易签名
    pub signature: String,
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
