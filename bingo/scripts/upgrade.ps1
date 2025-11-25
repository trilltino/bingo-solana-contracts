# Bingo Solana Program Upgrade Script (PowerShell)
# This script upgrades an existing program with the same program ID

$ErrorActionPreference = "Stop"

# Colors for output
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

Write-ColorOutput Green "========================================"
Write-ColorOutput Green "Bingo Program Upgrade Script"
Write-ColorOutput Green "========================================"
Write-Output ""

# Check if Solana CLI is installed
if (-not (Get-Command solana -ErrorAction SilentlyContinue)) {
    Write-ColorOutput Red "Error: solana CLI is not installed"
    Write-Output "Install from: https://docs.solana.com/cli/install-solana-cli-tools"
    exit 1
}

# Check if Anchor CLI is installed
if (-not (Get-Command anchor -ErrorAction SilentlyContinue)) {
    Write-ColorOutput Red "Error: anchor CLI is not installed"
    Write-Output "Install from: https://www.anchor-lang.com/docs/installation"
    exit 1
}

Write-ColorOutput Green "✓ Tools installed"
Write-Output ""

# Get cluster from argument or default to devnet
$CLUSTER = if ($args.Count -gt 0) { $args[0] } else { "devnet" }

Write-ColorOutput Yellow "Cluster: $CLUSTER"
Write-Output ""

# Verify cluster configuration
Write-Output "Current Solana configuration:"
solana config get
Write-Output ""

# Check wallet balance
$balanceOutput = solana balance
$balance = ($balanceOutput -split ' ')[0]
Write-Output "Wallet balance: $balance SOL"

if ([double]$balance -lt 0.1) {
    Write-ColorOutput Red "Warning: Low balance. You need at least ~2 SOL for upgrade"

    if ($CLUSTER -eq "devnet") {
        Write-Output "Requesting airdrop..."
        solana airdrop 2
        Write-ColorOutput Green "✓ Airdrop successful"
    } else {
        Write-Output "Please fund your wallet before upgrading on $CLUSTER"
        exit 1
    }
}

Write-Output ""

# Get the program ID from Anchor.toml
$anchorToml = Get-Content "Anchor.toml" -Raw
$programId = $null

if ($CLUSTER -eq "devnet") {
    if ($anchorToml -match '\[programs\.devnet\]\s+bingo\s*=\s*"([^"]+)"') {
        $programId = $matches[1]
    }
} elseif ($CLUSTER -eq "mainnet" -or $CLUSTER -eq "mainnet-beta") {
    if ($anchorToml -match '\[programs\.mainnet[^]]*\]\s+bingo\s*=\s*"([^"]+)"') {
        $programId = $matches[1]
    }
} else {
    if ($anchorToml -match "\[programs\.$CLUSTER\]\s+bingo\s*=\s*`"([^`"]+)`"") {
        $programId = $matches[1]
    }
}

if (-not $programId) {
    Write-ColorOutput Red "Error: Could not find program ID for cluster $CLUSTER in Anchor.toml"
    exit 1
}

Write-ColorOutput Green "Program ID: $programId"
Write-Output ""

# Verify program exists on-chain
Write-ColorOutput Yellow "Verifying program exists on $CLUSTER..."
$programInfo = solana program show $programId --url $CLUSTER 2>&1

if ($programInfo -match "not found") {
    Write-ColorOutput Red "Error: Program $programId not found on $CLUSTER"
    Write-Output "Use deploy.sh for initial deployment instead"
    exit 1
}

Write-ColorOutput Green "✓ Program found on-chain"
Write-Output ""

# Build the program
Write-ColorOutput Yellow "Building program..."
anchor build

if ($LASTEXITCODE -ne 0) {
    Write-ColorOutput Red "Build failed"
    exit 1
}

Write-ColorOutput Green "✓ Build successful"
Write-Output ""

# Verify the program ID in lib.rs matches
$libRs = Get-Content "programs/bingo/src/lib.rs" -Raw
if ($libRs -match 'declare_id!\("([^"]+)"\)') {
    $libRsProgramId = $matches[1]
    if ($libRsProgramId -ne $programId) {
        Write-ColorOutput Red "Error: Program ID mismatch!"
        Write-Output "  Anchor.toml: $programId"
        Write-Output "  lib.rs: $libRsProgramId"
        Write-Output "Please ensure they match before upgrading"
        exit 1
    }
}

Write-ColorOutput Green "✓ Program ID verified"
Write-Output ""

# Upgrade the program
Write-ColorOutput Yellow "Upgrading program on $CLUSTER..."
anchor upgrade target/deploy/bingo.so --program-id target/deploy/bingo-keypair.json --provider.cluster $CLUSTER

if ($LASTEXITCODE -ne 0) {
    Write-ColorOutput Red "Upgrade failed"
    exit 1
}

Write-ColorOutput Green "✓ Upgrade successful"
Write-Output ""

# Copy IDL to frontend (if directory exists)
Write-ColorOutput Yellow "Updating IDL..."
$frontendIdlDir = "../../src/idl"
if (Test-Path (Split-Path $frontendIdlDir)) {
    New-Item -ItemType Directory -Force -Path $frontendIdlDir | Out-Null
    Copy-Item "target/idl/bingo.json" "$frontendIdlDir/" -ErrorAction SilentlyContinue
    Write-ColorOutput Green "✓ IDL updated"
} else {
    Write-ColorOutput Yellow "Frontend directory not found, skipping IDL copy"
}

Write-Output ""
Write-ColorOutput Green "========================================"
Write-ColorOutput Green "Upgrade Complete!"
Write-ColorOutput Green "========================================"
Write-Output ""
Write-ColorOutput Green "Program ID: $programId"
Write-ColorOutput Yellow "Cluster: $CLUSTER"
Write-Output ""
Write-Output "View on Solana Explorer:"
if ($CLUSTER -eq "mainnet" -or $CLUSTER -eq "mainnet-beta") {
    Write-Output "https://explorer.solana.com/address/$programId"
} else {
    Write-Output "https://explorer.solana.com/address/$programId?cluster=$CLUSTER"
}
Write-Output ""
Write-Output "✅ Program upgraded successfully with the same program ID!"
Write-Output ""

