/**
 * 前端签名捕捉流程测试
 * 
 * 测试生产级的交易签名流程:
 * 1. 构建交易 (build-transaction)
 * 2. 钱包签名
 * 3. 提交交易 (submit-transaction)
 */

import { Keypair } from '@solana/web3.js';
import * as fs from 'fs';
import nacl from 'tweetnacl';
import bs58 from 'bs58';

const API_BASE = 'http://localhost:8080/api/v1';

// 加载钱包密钥对
function loadWallet(walletPath: string): Keypair {
    const expandedPath = walletPath.replace('~', process.env.HOME || '');
    const secretKey = JSON.parse(fs.readFileSync(expandedPath, 'utf-8'));
    return Keypair.fromSecretKey(new Uint8Array(secretKey));
}

// 签名消息（返回 base58 编码）用于认证
function signMessageBase58(message: string, keypair: Keypair): string {
    const messageBytes = new TextEncoder().encode(message);
    const signature = nacl.sign.detached(messageBytes, keypair.secretKey);
    return bs58.encode(signature);
}

// 签名交易消息（返回 base64 编码）
function signTransactionMessage(messageBytes: Uint8Array, keypair: Keypair): string {
    const signature = nacl.sign.detached(messageBytes, keypair.secretKey);
    return Buffer.from(signature).toString('base64');
}

async function main() {
    console.log('='.repeat(60));
    console.log('🔐 前端签名捕捉流程测试');
    console.log('='.repeat(60));

    // 1. 加载钱包
    const walletPath = '~/.config/solana/mainnet-deploy-wallet.json';
    console.log('\n📁 加载钱包:', walletPath);
    const wallet = loadWallet(walletPath);
    console.log('✅ 钱包地址:', wallet.publicKey.toBase58());

    // 2. 检查后端状态
    console.log('\n📡 检查后端状态...');
    const healthRes = await fetch('http://localhost:8080/health');
    if (!healthRes.ok) {
        throw new Error('后端不可用');
    }
    console.log('✅ 后端健康');

    // 3. 认证流程
    console.log('\n🔐 开始认证流程...');
    
    const challengeRes = await fetch(`${API_BASE}/auth/challenge`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: wallet.publicKey.toBase58() })
    });
    const challengeData = await challengeRes.json();
    
    const authSignature = signMessageBase58(challengeData.message, wallet);
    
    const authRes = await fetch(`${API_BASE}/auth/authenticate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            wallet_address: wallet.publicKey.toBase58(),
            signature: authSignature,
            message: challengeData.message
        })
    });
    
    if (!authRes.ok) {
        throw new Error(`认证失败: ${await authRes.text()}`);
    }
    
    const authData = await authRes.json();
    const token = authData.token;
    console.log('✅ 认证成功');

    // 4. 查询 Titans
    console.log('\n🗺️  查询附近的 Titans...');
    const playerLat = 35.69;
    const playerLng = 139.76;
    
    const titansRes = await fetch(
        `${API_BASE}/map/titans?lat=${playerLat}&lng=${playerLng}&radius=50000`,
        { headers: { 'Authorization': `Bearer ${token}` } }
    );
    const titans = await titansRes.json();
    
    if (!Array.isArray(titans) || titans.length === 0) {
        throw new Error('没有可捕捉的 Titan');
    }
    
    console.log(`✅ 发现 ${titans.length} 个 Titans`);
    const titan = titans[0];
    console.log('🎯 选择 Titan:', {
        id: titan.id,
        species_id: titan.species_id,
        element: titan.element,
        threat_class: titan.threat_class,
    });

    // 5. 请求捕捉授权
    console.log('\n📋 请求捕捉授权...');
    const captureRequestRes = await fetch(`${API_BASE}/capture/request`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
            titan_id: titan.id,
            player_location: {
                lat: titan.location.lat,
                lng: titan.location.lng,
                accuracy: 10.0
            }
        })
    });
    
    if (!captureRequestRes.ok) {
        throw new Error(`捕捉授权失败: ${await captureRequestRes.text()}`);
    }
    
    const captureAuth = await captureRequestRes.json();
    if (!captureAuth.authorized) {
        throw new Error(`捕捉未授权: ${captureAuth.error || 'Unknown'}`);
    }
    console.log('✅ 捕捉已授权');

    // 6. 构建交易
    console.log('\n🔨 构建铸造交易...');
    const buildRes = await fetch(`${API_BASE}/capture/build-transaction`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
            titan_id: titan.id,
            capture_lat: titan.location.lat,
            capture_lng: titan.location.lng,
        })
    });
    
    if (!buildRes.ok) {
        const errText = await buildRes.text();
        throw new Error(`构建交易失败: ${buildRes.status} - ${errText}`);
    }
    
    const buildData = await buildRes.json();
    console.log('✅ 交易已构建:', {
        titan_pda: buildData.titan_pda,
        player_pda: buildData.player_pda,
        titan_id: buildData.titan_id,
        blockhash: buildData.recent_blockhash.substring(0, 20) + '...',
    });

    // 7. 签名交易
    console.log('\n✍️  签名交易...');
    
    // 使用后端提供的 message_to_sign 进行签名
    const messageToSign = Buffer.from(buildData.message_to_sign, 'base64');
    const signatureBytes = nacl.sign.detached(messageToSign, wallet.secretKey);
    
    // Base64 编码签名
    const playerSignature = Buffer.from(signatureBytes).toString('base64');
    
    console.log('✅ 交易已签名');
    console.log('   签名 (前30字节 hex):', Buffer.from(signatureBytes).toString('hex').substring(0, 60) + '...');

    // 8. 提交交易
    console.log('\n📤 提交已签名交易...');
    const submitRes = await fetch(`${API_BASE}/capture/submit-transaction`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
            player_signature: playerSignature,
            serialized_transaction: buildData.serialized_transaction, // 原始未修改的交易
            titan_id: titan.id,
            titan_pda: buildData.titan_pda,
        })
    });
    
    const submitData = await submitRes.json();
    
    console.log('\n' + '='.repeat(60));
    console.log('📊 捕捉结果:');
    console.log('='.repeat(60));
    console.log(JSON.stringify(submitData, null, 2));
    
    if (submitRes.ok && submitData.success) {
        console.log('\n🎉 捕捉成功!');
        console.log('📝 交易签名:', submitData.tx_signature);
        console.log('🏆 NFT 地址:', submitData.mint_address);
        if (submitData.breach_reward) {
            console.log('💰 BREACH 奖励:', submitData.breach_reward / 1_000_000_000, 'BREACH');
        }
    } else {
        console.log('\n❌ 捕捉失败');
        console.log('错误详情:', submitData);
    }
    
    console.log('\n' + '='.repeat(60));
    console.log('测试完成');
    console.log('='.repeat(60));
}

main().catch(err => {
    console.error('\n❌ 测试失败:', err.message);
    process.exit(1);
});
