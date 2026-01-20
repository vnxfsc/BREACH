/**
 * Titan 操作测试
 * 
 * 测试 Level Up、Evolve、Fuse、Transfer 等操作
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

// 签名并提交交易
async function signAndSubmit(
    token: string,
    wallet: Keypair,
    buildData: { serialized_transaction: string, message_to_sign: string }
): Promise<any> {
    // 签名
    const messageToSign = Buffer.from(buildData.message_to_sign, 'base64');
    const signature = signTransactionMessage(messageToSign, wallet);

    // 提交
    const submitRes = await fetch(`${API_BASE}/titan/submit`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
            serialized_transaction: buildData.serialized_transaction,
            user_signature: signature,
        })
    });

    return submitRes.json();
}

async function testLevelUp(token: string, wallet: Keypair, titanId: number) {
    console.log('\n📈 测试 Level Up...');
    
    const buildRes = await fetch(`${API_BASE}/titan/level-up/build`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ titan_id: titanId })
    });

    if (!buildRes.ok) {
        const err = await buildRes.text();
        console.log('❌ 构建失败:', err);
        return;
    }

    const buildData = await buildRes.json();
    console.log('✅ 交易已构建');

    const result = await signAndSubmit(token, wallet, buildData);
    console.log('📊 结果:', result);
}

async function testEvolve(token: string, wallet: Keypair, titanId: number, newSpeciesId: number) {
    console.log('\n🦋 测试 Evolve...');
    
    const buildRes = await fetch(`${API_BASE}/titan/evolve/build`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ 
            titan_id: titanId,
            new_species_id: newSpeciesId
        })
    });

    if (!buildRes.ok) {
        const err = await buildRes.text();
        console.log('❌ 构建失败:', err);
        return;
    }

    const buildData = await buildRes.json();
    console.log('✅ 交易已构建');

    const result = await signAndSubmit(token, wallet, buildData);
    console.log('📊 结果:', result);
}

async function testTransfer(token: string, wallet: Keypair, titanId: number, toWallet: string) {
    console.log('\n🔄 测试 Transfer...');
    
    const buildRes = await fetch(`${API_BASE}/titan/transfer/build`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ 
            titan_id: titanId,
            to_wallet: toWallet
        })
    });

    if (!buildRes.ok) {
        const err = await buildRes.text();
        console.log('❌ 构建失败:', err);
        return;
    }

    const buildData = await buildRes.json();
    console.log('✅ 交易已构建');

    const result = await signAndSubmit(token, wallet, buildData);
    console.log('📊 结果:', result);
}

async function testFuse(token: string, wallet: Keypair, titanAId: number, titanBId: number) {
    console.log('\n🔮 测试 Fuse...');
    
    const buildRes = await fetch(`${API_BASE}/titan/fuse/build`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({ 
            titan_a_id: titanAId,
            titan_b_id: titanBId
        })
    });

    if (!buildRes.ok) {
        const err = await buildRes.text();
        console.log('❌ 构建失败:', err);
        return;
    }

    const buildData = await buildRes.json();
    console.log('✅ 交易已构建');
    console.log('   offspring_id:', buildData.offspring_id);
    console.log('   offspring_pda:', buildData.offspring_pda);

    const result = await signAndSubmit(token, wallet, buildData);
    console.log('📊 结果:', result);
}

async function main() {
    console.log('='.repeat(60));
    console.log('🎮 Titan 操作 API 测试');
    console.log('='.repeat(60));

    // 加载钱包
    const walletPath = '~/.config/solana/mainnet-deploy-wallet.json';
    const wallet = loadWallet(walletPath);
    console.log('\n📁 钱包:', wallet.publicKey.toBase58());

    // 认证
    console.log('\n🔐 认证中...');
    const token = await authenticate(wallet);
    console.log('✅ 认证成功');

    // 测试参数（需要根据实际链上数据调整）
    const testTitanId = 65558; // 之前铸造的 Titan ID
    
    // 测试各个操作
    // 注意: 这些测试可能会因为条件不满足而失败（比如经验值不够升级）
    
    console.log('\n' + '─'.repeat(60));
    console.log('测试 Level Up (需要足够经验值)');
    console.log('⚠️ 跳过: 新铸造的 Titan 经验值为 0，无法升级');
    // await testLevelUp(token, wallet, testTitanId);

    console.log('\n' + '─'.repeat(60));
    console.log('测试 Evolve (需要等级 >= 30)');
    console.log('⚠️ 跳过: 需要等级 >= 30');
    // await testEvolve(token, wallet, testTitanId, 5104);

    console.log('\n' + '─'.repeat(60));
    console.log('测试 Transfer');
    // 创建一个新钱包作为接收者
    const receiver = Keypair.generate();
    console.log('📤 接收者钱包:', receiver.publicKey.toBase58());
    await testTransfer(token, wallet, testTitanId, receiver.publicKey.toBase58());

    console.log('\n' + '─'.repeat(60));
    console.log('测试 Fuse (需要两个同元素、等级 >= 20 的 Titan)');
    console.log('⚠️ 跳过: 需要两个同元素且等级 >= 20 的 Titan');
    // await testFuse(token, wallet, testTitanId, ANOTHER_TITAN_ID);

    console.log('\n' + '='.repeat(60));
    console.log('测试完成');
    console.log('='.repeat(60));
}

main().catch(err => {
    console.error('❌ 测试失败:', err.message);
    process.exit(1);
});
