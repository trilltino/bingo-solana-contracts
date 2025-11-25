#!/bin/bash

# Bingo Solana Program Upgrade Script
# This script upgrades an existing program with the same program ID

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Bingo Program Upgrade Script${NC}"
echo -e "${GREEN}========================================${NC}"
echo

# Check if Solana CLI is installed
if ! command -v solana &> /dev/null; then
    echo -e "${RED}Error: solana CLI is not installed${NC}"
    echo "Install from: https://docs.solana.com/cli/install-solana-cli-tools"
    exit 1
fi

# Check if Anchor CLI is installed
if ! command -v anchor &> /dev/null; then
    echo -e "${RED}Error: anchor CLI is not installed${NC}"
    echo "Install from: https://www.anchor-lang.com/docs/installation"
    exit 1
fi

echo -e "${GREEN}✓${NC} Tools installed"
echo

# Get cluster from argument or default to devnet
CLUSTER=${1:-devnet}

echo -e "${YELLOW}Cluster: ${CLUSTER}${NC}"
echo

# Verify cluster configuration
echo "Current Solana configuration:"
solana config get
echo

# Check wallet balance
BALANCE=$(solana balance | awk '{print $1}')
echo -e "Wallet balance: ${BALANCE} SOL"

if (( $(echo "$BALANCE < 0.1" | bc -l) )); then
    echo -e "${RED}Warning: Low balance. You need at least ~2 SOL for upgrade${NC}"

    if [ "$CLUSTER" = "devnet" ]; then
        echo "Requesting airdrop..."
        solana airdrop 2
        echo -e "${GREEN}✓${NC} Airdrop successful"
    else
        echo "Please fund your wallet before upgrading on $CLUSTER"
        exit 1
    fi
fi

echo

# Get the program ID from Anchor.toml
if [ "$CLUSTER" = "devnet" ]; then
    PROGRAM_ID=$(grep -A 1 "\[programs.devnet\]" Anchor.toml | grep "bingo" | cut -d'"' -f2)
elif [ "$CLUSTER" = "mainnet" ] || [ "$CLUSTER" = "mainnet-beta" ]; then
    PROGRAM_ID=$(grep -A 1 "\[programs.mainnet\]" Anchor.toml | grep "bingo" | cut -d'"' -f2)
    if [ -z "$PROGRAM_ID" ]; then
        PROGRAM_ID=$(grep -A 1 "\[programs.mainnet-beta\]" Anchor.toml | grep "bingo" | cut -d'"' -f2)
    fi
else
    PROGRAM_ID=$(grep -A 1 "\[programs.${CLUSTER}\]" Anchor.toml | grep "bingo" | cut -d'"' -f2)
fi

if [ -z "$PROGRAM_ID" ]; then
    echo -e "${RED}Error: Could not find program ID for cluster ${CLUSTER} in Anchor.toml${NC}"
    exit 1
fi

echo -e "Program ID: ${GREEN}${PROGRAM_ID}${NC}"
echo

# Verify program exists on-chain
echo -e "${YELLOW}Verifying program exists on ${CLUSTER}...${NC}"
PROGRAM_INFO=$(solana program show "$PROGRAM_ID" --url "$CLUSTER" 2>&1 || true)

if echo "$PROGRAM_INFO" | grep -q "not found"; then
    echo -e "${RED}Error: Program ${PROGRAM_ID} not found on ${CLUSTER}${NC}"
    echo "Use deploy.sh for initial deployment instead"
    exit 1
fi

echo -e "${GREEN}✓${NC} Program found on-chain"
echo

# Check upgrade authority
UPGRADE_AUTHORITY=$(echo "$PROGRAM_INFO" | grep "Upgrade Authority" | awk '{print $3}' || echo "")
if [ -n "$UPGRADE_AUTHORITY" ]; then
    CURRENT_WALLET=$(solana address)
    echo -e "Upgrade Authority: ${YELLOW}${UPGRADE_AUTHORITY}${NC}"
    echo -e "Current Wallet: ${YELLOW}${CURRENT_WALLET}${NC}"
    
    if [ "$UPGRADE_AUTHORITY" != "$CURRENT_WALLET" ]; then
        echo -e "${RED}Warning: Current wallet is not the upgrade authority${NC}"
        echo "You may need to use the upgrade authority keypair"
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
fi

echo

# Build the program
echo -e "${YELLOW}Building program...${NC}"
anchor build

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Build successful"
echo

# Verify the program ID in lib.rs matches
LIB_RS_PROGRAM_ID=$(grep "declare_id!" programs/bingo/src/lib.rs | cut -d'"' -f2)
if [ "$LIB_RS_PROGRAM_ID" != "$PROGRAM_ID" ]; then
    echo -e "${RED}Error: Program ID mismatch!${NC}"
    echo "  Anchor.toml: ${PROGRAM_ID}"
    echo "  lib.rs: ${LIB_RS_PROGRAM_ID}"
    echo "Please ensure they match before upgrading"
    exit 1
fi

echo -e "${GREEN}✓${NC} Program ID verified"
echo

# Upgrade the program
echo -e "${YELLOW}Upgrading program on ${CLUSTER}...${NC}"
anchor upgrade target/deploy/bingo.so --program-id target/deploy/bingo-keypair.json --provider.cluster "$CLUSTER"

if [ $? -ne 0 ]; then
    echo -e "${RED}Upgrade failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Upgrade successful"
echo

# Copy IDL to frontend (if directory exists)
echo -e "${YELLOW}Updating IDL...${NC}"
FRONTEND_IDL_DIR="../../src/idl"
if [ -d "$(dirname "$FRONTEND_IDL_DIR")" ]; then
    mkdir -p "$FRONTEND_IDL_DIR"
    cp target/idl/bingo.json "$FRONTEND_IDL_DIR/" 2>/dev/null || echo -e "${YELLOW}Warning: Could not copy IDL${NC}"
    echo -e "${GREEN}✓${NC} IDL updated"
else
    echo -e "${YELLOW}Frontend directory not found, skipping IDL copy${NC}"
fi

echo
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Upgrade Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo
echo -e "Program ID: ${GREEN}${PROGRAM_ID}${NC}"
echo -e "Cluster: ${YELLOW}${CLUSTER}${NC}"
echo
echo "View on Solana Explorer:"
if [ "$CLUSTER" = "mainnet" ] || [ "$CLUSTER" = "mainnet-beta" ]; then
    echo "https://explorer.solana.com/address/${PROGRAM_ID}"
else
    echo "https://explorer.solana.com/address/${PROGRAM_ID}?cluster=${CLUSTER}"
fi
echo
echo "✅ Program upgraded successfully with the same program ID!"
echo

