/**
 * Marketplace 功能测试
 * 
 * 演示市场listing创建、查询和取消（链下功能）
 * 
 * 注意：完整的链上购买需要escrow合约（未实现）
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
    console.log('🏪 Marketplace 功能测试');
    console.log('='.repeat(60));

    // 加载钱包
    const wallet = loadWallet('~/.config/solana/mainnet-deploy-wallet.json');
    console.log('\n📁 钱包:', wallet.publicKey.toBase58());

    // 认证
    console.log('\n🔐 认证中...');
    const token = await authenticate(wallet);
    console.log('✅ 认证成功');

    // 查询市场listings
    console.log('\n' + '─'.repeat(60));
    console.log('📋 查询市场Listings...');
    
    const searchRes = await fetch(`${API_BASE}/marketplace?limit=5`, {
        headers: { 'Authorization': `Bearer ${token}` }
    });
    
    if (searchRes.ok) {
        const results = await searchRes.json();
        console.log(`✅ 找到 ${results.total_count} 个listings`);
        if (results.listings.length > 0) {
            console.log('\n前几个listings:');
            results.listings.forEach((listing: any, i: number) => {
                console.log(`  ${i + 1}. ${listing.titan.element} Titan (Level ${listing.titan.level})`);
                console.log(`     价格: ${listing.price / 1_000_000_000} BREACH`);
                console.log(`     卖家: ${listing.seller_username || listing.seller_id}`);
            });
        } else {
            console.log('  (当前没有active listings)');
        }
    } else {
        console.log('❌ 查询失败:', await searchRes.text());
    }

    // 查询我的listings
    console.log('\n' + '─'.repeat(60));
    console.log('📦 查询我的Listings...');
    
    const myListingsRes = await fetch(`${API_BASE}/marketplace/my-listings`, {
        headers: { 'Authorization': `Bearer ${token}` }
    });
    
    if (myListingsRes.ok) {
        const myListings = await myListingsRes.json();
        console.log(`✅ 我有 ${myListings.length} 个listings`);
        if (myListings.length > 0) {
            myListings.forEach((listing: any, i: number) => {
                console.log(`  ${i + 1}. ${listing.titan.element} Titan - ${listing.price / 1_000_000_000} BREACH`);
                console.log(`     状态: ${listing.status}`);
            });
        }
    } else {
        console.log('❌ 查询失败:', await myListingsRes.text());
    }

    // 查询市场统计
    console.log('\n' + '─'.repeat(60));
    console.log('📊 市场统计...');
    
    const statsRes = await fetch(`${API_BASE}/marketplace/stats`, {
        headers: { 'Authorization': `Bearer ${token}` }
    });
    
    if (statsRes.ok) {
        const stats = await statsRes.json();
        console.log('✅ 市场数据:');
        console.log(`   总Listings: ${stats.total_listings}`);
        console.log(`   Active: ${stats.active_listings}`);
        console.log(`   24h交易量: ${(stats.total_volume_24h / 1_000_000_000).toFixed(2)} BREACH`);
        console.log(`   24h销售: ${stats.total_sales_24h}`);
        if (stats.floor_price) {
            console.log(`   地板价: ${(stats.floor_price / 1_000_000_000).toFixed(2)} BREACH`);
        }
    } else {
        console.log('❌ 查询失败:', await statsRes.text());
    }

    console.log('\n' + '='.repeat(60));
    console.log('ℹ️  注意事项:');
    console.log('   • 当前市场是链下数据库实现');
    console.log('   • 链上购买需要escrow合约（未实现）');
    console.log('   • 可以通过API创建/取消listings和查询市场');
    console.log('   • 完整的去中心化市场需要链上escrow程序');
    console.log('='.repeat(60));
}

main().catch(err => {
    console.error('❌ 测试失败:', err.message);
    process.exit(1);
});
