#!/bin/bash

# Fundraisely Solana Program Deployment Script
# This script handles the complete deployment workflow

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Fundraisely Deployment Script${NC}"
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
    echo -e "${RED}Warning: Low balance. You need at least ~2 SOL for deployment${NC}"

    if [ "$CLUSTER" = "devnet" ]; then
        echo "Requesting airdrop..."
        solana airdrop 2
        echo -e "${GREEN}✓${NC} Airdrop successful"
    else
        echo "Please fund your wallet before deploying to $CLUSTER"
        exit 1
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

# Get the program ID from the keypair
PROGRAM_KEYPAIR="target/deploy/fundraisely-keypair.json"

if [ ! -f "$PROGRAM_KEYPAIR" ]; then
    echo -e "${RED}Error: Program keypair not found at $PROGRAM_KEYPAIR${NC}"
    echo "Run 'anchor build' first"
    exit 1
fi

PROGRAM_ID=$(solana-keygen pubkey "$PROGRAM_KEYPAIR")
echo -e "Program ID: ${GREEN}${PROGRAM_ID}${NC}"
echo

# Deploy the program
echo -e "${YELLOW}Deploying to ${CLUSTER}...${NC}"
anchor deploy --provider.cluster "$CLUSTER"

if [ $? -ne 0 ]; then
    echo -e "${RED}Deployment failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Deployment successful"
echo

# Update Anchor.toml with program ID
echo -e "${YELLOW}Updating Anchor.toml...${NC}"
sed -i.bak "s/fundraisely = \".*\"/fundraisely = \"$PROGRAM_ID\"/" Anchor.toml
echo -e "${GREEN}✓${NC} Anchor.toml updated"

# Update lib.rs with program ID
echo -e "${YELLOW}Updating lib.rs...${NC}"
LIB_RS="programs/fundraisely/src/lib.rs"
sed -i.bak "s/declare_id!(\".*\")/declare_id!(\"$PROGRAM_ID\")/" "$LIB_RS"
echo -e "${GREEN}✓${NC} lib.rs updated"

# Rebuild to regenerate IDL with correct program ID
echo -e "${YELLOW}Rebuilding with updated program ID...${NC}"
anchor build
echo -e "${GREEN}✓${NC} Rebuild complete"

# Copy IDL to frontend
echo -e "${YELLOW}Copying IDL to frontend...${NC}"
FRONTEND_IDL_DIR="../../src/idl"
mkdir -p "$FRONTEND_IDL_DIR"
cp target/idl/fundraisely.json "$FRONTEND_IDL_DIR/"
echo -e "${GREEN}✓${NC} IDL copied to $FRONTEND_IDL_DIR"

# Update frontend .env
FRONTEND_ENV="../../.env"
if [ -f "$FRONTEND_ENV" ]; then
    if grep -q "VITE_PROGRAM_ID=" "$FRONTEND_ENV"; then
        sed -i.bak "s/VITE_PROGRAM_ID=.*/VITE_PROGRAM_ID=$PROGRAM_ID/" "$FRONTEND_ENV"
    else
        echo "VITE_PROGRAM_ID=$PROGRAM_ID" >> "$FRONTEND_ENV"
    fi
    echo -e "${GREEN}✓${NC} Frontend .env updated"
else
    echo -e "${YELLOW}Warning: Frontend .env not found at $FRONTEND_ENV${NC}"
    echo "Please manually set VITE_PROGRAM_ID=$PROGRAM_ID"
fi

echo
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Deployment Complete!${NC}"
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
echo "Next steps:"
echo "1. Run tests: anchor test"
echo "2. Initialize global config (see docs/DEPLOYMENT.md)"
echo "3. Start the frontend: cd ../.. && npm run dev"
echo
