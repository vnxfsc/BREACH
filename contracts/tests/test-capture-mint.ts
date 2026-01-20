/**
 * 捕捉→铸造完整流程测试
 * 测试后端 Solana NFT 铸造功能
 */

import { Keypair, Connection, PublicKey } from '@solana/web3.js';
import * as fs from 'fs';
import * as path from 'path';
import nacl from 'tweetnacl';
import bs58 from 'bs58';

const API_BASE = 'http://localhost:8080/api/v1';

// 加载钱包密钥对
function loadWallet(walletPath: string): Keypair {
    const expandedPath = walletPath.replace('~', process.env.HOME || '');
    const secretKey = JSON.parse(fs.readFileSync(expandedPath, 'utf-8'));
    return Keypair.fromSecretKey(new Uint8Array(secretKey));
}

// 签名消息（返回 base58 编码）
function signMessage(message: string, keypair: Keypair): string {
    const messageBytes = new TextEncoder().encode(message);
    const signature = nacl.sign.detached(messageBytes, keypair.secretKey);
    return bs58.encode(signature);
}

async function main() {
    console.log('='.repeat(60));
    console.log('🧪 捕捉→铸造完整流程测试');
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
    const healthData = await healthRes.json();
    console.log('✅ 后端健康:', healthData.status);

    // 3. 检查 Solana 服务
    const solanaInfoRes = await fetch(`${API_BASE}/solana/backend-info`);
    const solanaInfo = await solanaInfoRes.json();
    console.log('✅ Solana 服务:', JSON.stringify(solanaInfo, null, 2));

    // 4. 认证流程
    console.log('\n🔐 开始认证流程...');
    
    // 4.1 获取挑战
    const challengeRes = await fetch(`${API_BASE}/auth/challenge`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: wallet.publicKey.toBase58() })
    });
    const challengeData = await challengeRes.json();
    console.log('📝 挑战消息:', challengeData.message);

    // 4.2 签名挑战
    const signature = signMessage(challengeData.message, wallet);
    console.log('✍️  签名 (base58):', signature.substring(0, 30) + '...');

    // 4.3 认证
    const authRes = await fetch(`${API_BASE}/auth/authenticate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            wallet_address: wallet.publicKey.toBase58(),
            signature: signature,
            message: challengeData.message
        })
    });
    
    if (!authRes.ok) {
        const errText = await authRes.text();
        throw new Error(`认证失败: ${authRes.status} - ${errText}`);
    }
    
    const authData = await authRes.json();
    const token = authData.token;
    console.log('✅ 认证成功，获取 JWT');

    // 5. 查询附近的 Titans
    console.log('\n🗺️  查询附近的 Titans...');
    
    // 使用东京坐标（数据库中有 Titan）
    const playerLat = 35.69;
    const playerLng = 139.76;
    
    const titansRes = await fetch(
        `${API_BASE}/map/titans?lat=${playerLat}&lng=${playerLng}&radius=50000`,
        { headers: { 'Authorization': `Bearer ${token}` } }
    );
    const titans = await titansRes.json();
    
    if (!Array.isArray(titans) || titans.length === 0) {
        console.log('⚠️  附近没有 Titans，正在生成测试 Titan...');
        // 触发生成
        await fetch(`${API_BASE}/map/titans?lat=${playerLat}&lng=${playerLng}&radius=50000`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        throw new Error('没有可捕捉的 Titan，请等待生成后重试');
    }
    
    console.log(`✅ 发现 ${titans.length} 个 Titans`);
    const titan = titans[0];
    console.log('🎯 选择 Titan:', {
        id: titan.id,
        species_id: titan.species_id,
        element: titan.element,
        threat_class: titan.threat_class,
        location: titan.location
    });

    // 6. 请求捕捉授权
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
        const errText = await captureRequestRes.text();
        throw new Error(`捕捉授权请求失败: ${captureRequestRes.status} - ${errText}`);
    }
    
    const captureAuth = await captureRequestRes.json();
    console.log('✅ 捕捉授权:', {
        authorized: captureAuth.authorized,
        distance: captureAuth.distance,
        titan: captureAuth.titan ? {
            species_id: captureAuth.titan.species_id,
            element: captureAuth.titan.element
        } : null
    });

    if (!captureAuth.authorized) {
        throw new Error(`捕捉未授权: ${captureAuth.error || 'Unknown reason'}`);
    }

    // 7. 确认捕捉（启用链上铸造）
    console.log('\n⛓️  确认捕捉（链上铸造）...');
    const confirmRes = await fetch(`${API_BASE}/capture/confirm`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
            titan_id: titan.id,
            skip_blockchain: false  // 启用链上铸造
        })
    });
    
    const confirmData = await confirmRes.json();
    
    console.log('\n' + '='.repeat(60));
    console.log('📊 捕捉结果:');
    console.log('='.repeat(60));
    console.log(JSON.stringify(confirmData, null, 2));
    
    if (confirmRes.ok) {
        console.log('\n✅ 捕捉成功!');
        if (confirmData.titan_nft_address) {
            console.log('🎉 NFT 地址:', confirmData.titan_nft_address);
        }
        if (confirmData.breach_reward) {
            console.log('💰 BREACH 奖励:', confirmData.breach_reward);
        }
        if (confirmData.transaction_signature) {
            console.log('📝 交易签名:', confirmData.transaction_signature);
        }
    } else {
        console.log('\n❌ 捕捉失败');
        console.log('错误详情:', confirmData);
    }
    
    console.log('\n' + '='.repeat(60));
    console.log('测试完成');
    console.log('='.repeat(60));
}

main().catch(err => {
    console.error('\n❌ 测试失败:', err.message);
    process.exit(1);
});
