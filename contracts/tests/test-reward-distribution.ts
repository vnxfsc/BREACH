/**
 * BREACH 代币奖励分发测试
 * 
 * 测试奖励分发功能
 */

import { Keypair } from '@solana/web3.js';
import * as fs from 'fs';
import nacl from 'tweetnacl';
import bs58 from 'bs58';

const API_BASE = 'http://localhost:8080/api/v1';

// 加载钱包
function loadWallet(walletPath: string): Keypair {
    const expandedPath = walletPath.replace('~', process.env.HOME || '');
    const secretKey = JSON.parse(fs.readFileSync(expandedPath, 'utf-8'));
    return Keypair.fromSecretKey(new Uint8Array(secretKey));
}

// 签名消息（base58）用于认证
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

// 分发奖励
async function distributeReward(
    token: string,
    rewardType: number,
    amount: number
): Promise<any> {
    const res = await fetch(`${API_BASE}/game/reward/distribute`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
            reward_type: rewardType,
            amount: amount
        })
    });

    if (!res.ok) {
        const err = await res.text();
        throw new Error(`Failed: ${err}`);
    }

    return res.json();
}

// 查询 BREACH 余额
async function getBREACHBalance(wallet: string): Promise<number> {
    const res = await fetch(`${API_BASE}/solana/breach-balance/${wallet}`);
    if (!res.ok) {
        return 0;
    }
    const data = await res.json();
    return data.balance || 0;
}

async function main() {
    console.log('='.repeat(60));
    console.log('💰 BREACH 代币奖励分发测试');
    console.log('='.repeat(60));

    // 加载钱包
    const walletPath = '~/.config/solana/mainnet-deploy-wallet.json';
    const wallet = loadWallet(walletPath);
    console.log('\n📁 钱包:', wallet.publicKey.toBase58());

    // 查询初始余额
    console.log('\n💵 查询 BREACH 余额...');
    const initialBalance = await getBREACHBalance(wallet.publicKey.toBase58());
    console.log(`   初始余额: ${initialBalance / 1_000_000_000} BREACH`);

    // 认证
    console.log('\n🔐 认证中...');
    const token = await authenticate(wallet);
    console.log('✅ 认证成功');

    // 测试奖励类型（使用更大金额以便观察）
    const rewardTypes = [
        { type: 0, name: 'Capture', multiplier: '1x', amount: 10_000_000_000 },     // 10 BREACH
        { type: 1, name: 'Battle Win', multiplier: '2x', amount: 10_000_000_000 },  // 10 * 2 = 20 BREACH
        { type: 2, name: 'Daily Bonus', multiplier: '5x', amount: 10_000_000_000 }, // 10 * 5 = 50 BREACH
    ];

    for (const reward of rewardTypes) {
        console.log('\n' + '─'.repeat(60));
        console.log(`🎁 测试 ${reward.name} 奖励 (${reward.multiplier})`);
        console.log(`   基础金额: ${reward.amount / 1_000_000_000} BREACH`);

        try {
            const result = await distributeReward(token, reward.type, reward.amount);
            console.log('✅ 分发成功');
            console.log('   交易签名:', result.tx_signature);
            
            // 等待确认
            await new Promise(resolve => setTimeout(resolve, 2000));
            
            // 查询余额
            const newBalance = await getBREACHBalance(wallet.publicKey.toBase58());
            const change = (newBalance - initialBalance) / 1_000_000_000;
            console.log(`   新余额: ${newBalance / 1_000_000_000} BREACH (+${change.toFixed(2)})`);
        } catch (err: any) {
            console.log('❌ 失败:', err.message);
        }
    }

    // 最终余额
    console.log('\n' + '='.repeat(60));
    const finalBalance = await getBREACHBalance(wallet.publicKey.toBase58());
    const totalEarned = (finalBalance - initialBalance) / 1_000_000_000;
    console.log(`💰 最终余额: ${finalBalance / 1_000_000_000} BREACH`);
    console.log(`📈 总收益: ${totalEarned.toFixed(2)} BREACH`);
    console.log('='.repeat(60));
}

main().catch(err => {
    console.error('❌ 测试失败:', err.message);
    process.exit(1);
});
