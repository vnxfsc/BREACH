#!/bin/bash
# ============================================
# BREACH Token Creation Script
# Creates $BREACH SPL Token on Solana
# ============================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           BREACH Token Creation Script                         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"

# Configuration
NETWORK="${1:-devnet}"  # Default to devnet
DECIMALS=9              # Same as SOL
TOTAL_SUPPLY=1000000000 # 1 billion tokens

# Token allocation (in tokens)
PLAY_TO_EARN=$((TOTAL_SUPPLY * 35 / 100))    # 35% = 350,000,000
ECOSYSTEM=$((TOTAL_SUPPLY * 25 / 100))        # 25% = 250,000,000
TEAM=$((TOTAL_SUPPLY * 15 / 100))             # 15% = 150,000,000
TREASURY=$((TOTAL_SUPPLY * 10 / 100))         # 10% = 100,000,000
LIQUIDITY=$((TOTAL_SUPPLY * 10 / 100))        # 10% = 100,000,000
ADVISORS=$((TOTAL_SUPPLY * 5 / 100))          # 5%  = 50,000,000

echo -e "\n${YELLOW}Network:${NC} $NETWORK"
echo -e "${YELLOW}Total Supply:${NC} $TOTAL_SUPPLY BREACH"
echo -e "${YELLOW}Decimals:${NC} $DECIMALS"

# Set network
echo -e "\n${GREEN}[1/6] Setting network to $NETWORK...${NC}"
solana config set --url $NETWORK

# Check balance
BALANCE=$(solana balance | awk '{print $1}')
echo -e "${YELLOW}Current balance:${NC} $BALANCE SOL"

if (( $(echo "$BALANCE < 0.5" | bc -l) )); then
    echo -e "${YELLOW}Requesting airdrop...${NC}"
    solana airdrop 2 || true
    sleep 2
fi

# Create token mint
echo -e "\n${GREEN}[2/6] Creating BREACH token mint...${NC}"
MINT_OUTPUT=$(spl-token create-token --decimals $DECIMALS 2>&1)
MINT_ADDRESS=$(echo "$MINT_OUTPUT" | grep "Creating token" | awk '{print $3}')

if [ -z "$MINT_ADDRESS" ]; then
    # Try alternative parsing
    MINT_ADDRESS=$(echo "$MINT_OUTPUT" | grep -oE '[1-9A-HJ-NP-Za-km-z]{32,44}' | head -1)
fi

echo -e "${GREEN}✓ Token Mint:${NC} $MINT_ADDRESS"

# Create token account for the wallet
echo -e "\n${GREEN}[3/6] Creating token account...${NC}"
ACCOUNT_OUTPUT=$(spl-token create-account $MINT_ADDRESS 2>&1)
TOKEN_ACCOUNT=$(echo "$ACCOUNT_OUTPUT" | grep "Creating account" | awk '{print $3}')

if [ -z "$TOKEN_ACCOUNT" ]; then
    TOKEN_ACCOUNT=$(spl-token accounts $MINT_ADDRESS 2>&1 | grep -oE '[1-9A-HJ-NP-Za-km-z]{32,44}' | head -1)
fi

echo -e "${GREEN}✓ Token Account:${NC} $TOKEN_ACCOUNT"

# Mint initial supply
echo -e "\n${GREEN}[4/6] Minting initial supply ($TOTAL_SUPPLY BREACH)...${NC}"
spl-token mint $MINT_ADDRESS $TOTAL_SUPPLY

# Verify balance
echo -e "\n${GREEN}[5/6] Verifying token balance...${NC}"
spl-token balance $MINT_ADDRESS

# Display allocation plan
echo -e "\n${GREEN}[6/6] Token Allocation Plan:${NC}"
echo -e "╔══════════════════════════════════════════════════════════════╗"
echo -e "║ Category        │ Allocation │ Amount                        ║"
echo -e "╠══════════════════════════════════════════════════════════════╣"
echo -e "║ Play-to-Earn    │ 35%        │ $PLAY_TO_EARN BREACH            ║"
echo -e "║ Ecosystem       │ 25%        │ $ECOSYSTEM BREACH            ║"
echo -e "║ Team (Vested)   │ 15%        │ $TEAM BREACH            ║"
echo -e "║ Treasury        │ 10%        │ $TREASURY BREACH            ║"
echo -e "║ Liquidity       │ 10%        │ $LIQUIDITY BREACH            ║"
echo -e "║ Advisors        │ 5%         │ $ADVISORS BREACH             ║"
echo -e "╚══════════════════════════════════════════════════════════════╝"

# Save token info
OUTPUT_FILE="../target/deploy/breach-token-info.json"
mkdir -p ../target/deploy

cat > $OUTPUT_FILE << EOF
{
  "network": "$NETWORK",
  "tokenName": "BREACH",
  "tokenSymbol": "BREACH",
  "decimals": $DECIMALS,
  "totalSupply": $TOTAL_SUPPLY,
  "mintAddress": "$MINT_ADDRESS",
  "tokenAccount": "$TOKEN_ACCOUNT",
  "createdAt": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "allocation": {
    "playToEarn": { "percentage": 35, "amount": $PLAY_TO_EARN },
    "ecosystem": { "percentage": 25, "amount": $ECOSYSTEM },
    "team": { "percentage": 15, "amount": $TEAM },
    "treasury": { "percentage": 10, "amount": $TREASURY },
    "liquidity": { "percentage": 10, "amount": $LIQUIDITY },
    "advisors": { "percentage": 5, "amount": $ADVISORS }
  }
}
EOF

echo -e "\n${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✓ BREACH Token created successfully!${NC}"
echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo -e "\n${YELLOW}Token Info:${NC}"
echo -e "  Mint Address:  ${BLUE}$MINT_ADDRESS${NC}"
echo -e "  Token Account: ${BLUE}$TOKEN_ACCOUNT${NC}"
echo -e "  Explorer:      https://explorer.solana.com/address/$MINT_ADDRESS?cluster=$NETWORK"
echo -e "\n${YELLOW}Saved to:${NC} $OUTPUT_FILE"

# Next steps
echo -e "\n${YELLOW}Next Steps:${NC}"
echo -e "  1. Add metadata using Metaplex (name, symbol, logo)"
echo -e "  2. Create allocation wallets and distribute tokens"
echo -e "  3. Add liquidity to Raydium/Orca"
echo -e "  4. Disable mint authority (optional, for fixed supply)"
