/**
 * 简单奖励测试 - 使用新钱包
 */

import { Keypair } from '@solana/web3.js';
import nacl from 'tweetnacl';
import bs58 from 'bs58';
import * as fs from 'fs';

const API_BASE = 'http://localhost:8080/api/v1';

// 加载主钱包（用于认证和支付）
function loadWallet(walletPath: string): Keypair {
    const expandedPath = walletPath.replace('~', process.env.HOME || '');
    const secretKey = JSON.parse(fs.readFileSync(expandedPath, 'utf-8'));
    return Keypair.fromSecretKey(new Uint8Array(secretKey));
}

// 签名
function signMessageBase58(message: string, keypair: Keypair): string {
    const messageBytes = new TextEncoder().encode(message);
    const signature = nacl.sign.detached(messageBytes, keypair.secretKey);
    return bs58.encode(signature);
}

// 认证
async function authenticate(wallet: Keypair): Promise<string> {
    const challengeRes = await fetch(`${API_BASE}/auth/challenge`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: wallet.publicKey.toBase58() })
    });
    const challengeData = await challengeRes.json();
    
    const signature = signMessageBase58(challengeData.message, wallet);
    
    const authRes = await fetch(`${API_BASE}/auth/authenticate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            wallet_address: wallet.publicKey.toBase58(),
            signature,
            message: challengeData.message
        })
    });
    
    const authData = await authRes.json();
    return authData.token;
}

async function main() {
    console.log('='.repeat(60));
    console.log('🎁 简单奖励测试');
    console.log('='.repeat(60));

    // 使用主钱包
    const wallet = loadWallet('~/.config/solana/mainnet-deploy-wallet.json');
    console.log('\n📁 钱包:', wallet.publicKey.toBase58());

    // 认证
    const token = await authenticate(wallet);
    console.log('✅ 认证成功\n');

    // 测试：分发 100 BREACH (Capture 奖励)
    console.log('🎁 分发 100 BREACH (Capture 奖励，1x倍数)');
    const res = await fetch(`${API_BASE}/game/reward/distribute`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
            reward_type: 0,  // Capture
            amount: 100_000_000_000  // 100 BREACH
        })
    });

    const result = await res.json();
    
    if (result.success) {
        console.log('✅ 分发成功！');
        console.log(`   交易签名: ${result.tx_signature}`);
        console.log(`   实际金额: ${result.amount / 1_000_000_000} BREACH`);
        console.log(`\n🔗 查看交易: https://explorer.solana.com/tx/${result.tx_signature}?cluster=devnet`);
    } else {
        console.log('❌ 失败:', result);
    }

    console.log('\n' + '='.repeat(60));
}

main().catch(err => {
    console.error('❌ 错误:', err.message);
    process.exit(1);
});
