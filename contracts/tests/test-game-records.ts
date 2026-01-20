/**
 * Game Logic 链上记录测试
 * 
 * 测试 Record Capture、Record Battle、Add Experience
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

// 签名交易消息（base64）
function signTransactionMessage(messageBytes: Uint8Array, keypair: Keypair): string {
    const signature = nacl.sign.detached(messageBytes, keypair.secretKey);
    return Buffer.from(signature).toString('base64');
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

// 签名并提交双签名交易
async function signAndSubmitDual(
    token: string,
    wallet: Keypair,
    buildData: { serialized_transaction: string, message_to_sign: string }
): Promise<any> {
    // 签名
    const messageToSign = Buffer.from(buildData.message_to_sign, 'base64');
    const signature = signTransactionMessage(messageToSign, wallet);

    // 提交
    const submitRes = await fetch(`${API_BASE}/game/submit`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
            serialized_transaction: buildData.serialized_transaction,
            player_signature: signature,
        })
    });

    return submitRes.json();
}

async function testAddExperience(token: string, wallet: Keypair, titanId: number, expAmount: number) {
    console.log('\n📈 测试 Add Experience...');
    console.log(`   titan_id: ${titanId}, exp_amount: ${expAmount}`);
    
    const buildRes = await fetch(`${API_BASE}/game/experience/build`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ 
            titan_id: titanId,
            exp_amount: expAmount
        })
    });

    if (!buildRes.ok) {
        const err = await buildRes.text();
        console.log('❌ 构建失败:', err);
        return;
    }

    const buildData = await buildRes.json();
    console.log('✅ 交易已构建');

    const result = await signAndSubmitDual(token, wallet, buildData);
    console.log('📊 结果:', result);
    return result;
}

async function testRecordCapture(token: string, wallet: Keypair, titanId: number) {
    console.log('\n📍 测试 Record Capture...');
    console.log(`   titan_id: ${titanId}`);
    
    const buildRes = await fetch(`${API_BASE}/game/capture/build`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ 
            titan_id: titanId,
            location_lat: 31230000,  // 31.23°N
            location_lng: 121470000, // 121.47°E
            threat_class: 2,
            element_type: 1
        })
    });

    if (!buildRes.ok) {
        const err = await buildRes.text();
        console.log('❌ 构建失败:', err);
        return;
    }

    const buildData = await buildRes.json();
    console.log('✅ 交易已构建');
    console.log('   capture_id:', buildData.capture_id);

    const result = await signAndSubmitDual(token, wallet, buildData);
    console.log('📊 结果:', result);
    return result;
}

async function testRecordBattle(
    token: string, 
    wallet: Keypair, 
    opponentWallet: string,
    titanId: number, 
    opponentTitanId: number
) {
    console.log('\n⚔️ 测试 Record Battle...');
    console.log(`   titan_id: ${titanId} vs opponent_titan_id: ${opponentTitanId}`);
    
    const buildRes = await fetch(`${API_BASE}/game/battle/build`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ 
            opponent_wallet: opponentWallet,
            titan_id: titanId,
            opponent_titan_id: opponentTitanId,
            winner: 0, // 玩家获胜
            exp_gained: 100,
            opponent_exp_gained: 50,
            location_lat: 31230000,
            location_lng: 121470000
        })
    });

    if (!buildRes.ok) {
        const err = await buildRes.text();
        console.log('❌ 构建失败:', err);
        return;
    }

    const buildData = await buildRes.json();
    console.log('✅ 交易已构建');
    console.log('   battle_id:', buildData.battle_id);

    const result = await signAndSubmitDual(token, wallet, buildData);
    console.log('📊 结果:', result);
    return result;
}

async function main() {
    console.log('='.repeat(60));
    console.log('🎮 Game Logic 链上记录测试');
    console.log('='.repeat(60));

    // 加载钱包
    const walletPath = '~/.config/solana/mainnet-deploy-wallet.json';
    const wallet = loadWallet(walletPath);
    console.log('\n📁 钱包:', wallet.publicKey.toBase58());

    // 认证
    console.log('\n🔐 认证中...');
    const token = await authenticate(wallet);
    console.log('✅ 认证成功');

    // 测试参数
    const testTitanId = 65559; // 使用最新铸造的 Titan

    // 测试 Add Experience
    console.log('\n' + '─'.repeat(60));
    console.log('测试 Add Experience (给 Titan 添加经验值)');
    const expResult = await testAddExperience(token, wallet, testTitanId, 500);
    
    if (expResult?.success) {
        console.log('\n✅ 经验值添加成功！');
        console.log('   现在可以测试 Level Up 了');
    }

    // 测试 Record Capture (可选)
    // console.log('\n' + '─'.repeat(60));
    // await testRecordCapture(token, wallet, testTitanId);

    // 测试 Record Battle (可选，需要对手)
    // console.log('\n' + '─'.repeat(60));
    // const opponent = Keypair.generate();
    // await testRecordBattle(token, wallet, opponent.publicKey.toBase58(), testTitanId, 65558);

    console.log('\n' + '='.repeat(60));
    console.log('测试完成');
    console.log('='.repeat(60));
}

main().catch(err => {
    console.error('❌ 测试失败:', err.message);
    process.exit(1);
});
