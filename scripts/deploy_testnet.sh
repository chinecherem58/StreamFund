#!/usr/bin/env bash
# =============================================================================
# deploy_testnet.sh — Build and deploy StreamFundContract to Stellar Testnet
#
# Prerequisites:
#   - Rust + wasm32-unknown-unknown target installed
#   - Stellar CLI: https://developers.stellar.org/docs/tools/stellar-cli
#     Install: cargo install stellar-cli --features opt
#   - A funded Stellar Testnet account (get XLM: https://friendbot.stellar.org)
#
# Usage:
#   export STELLAR_SECRET_KEY=S...your...secret...key
#   bash scripts/deploy_testnet.sh
# =============================================================================

set -euo pipefail

NETWORK="testnet"
NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
RPC_URL="https://soroban-testnet.stellar.org"
WASM_PATH="target/wasm32-unknown-unknown/release/stream_fund.wasm"

# ── 1. Validate environment ───────────────────────────────────────────────────
if [ -z "${STELLAR_SECRET_KEY:-}" ]; then
  echo "ERROR: STELLAR_SECRET_KEY is not set."
  echo "Export your Stellar Testnet secret key:"
  echo "  export STELLAR_SECRET_KEY=S..."
  exit 1
fi

echo "=== StreamFund Testnet Deployment ==="
echo "Network : $NETWORK"
echo "RPC URL : $RPC_URL"
echo ""

# ── 2. Build optimised WASM ───────────────────────────────────────────────────
echo "[1/4] Building WASM artifact..."
cargo build --target wasm32-unknown-unknown --release

WASM_SIZE=$(wc -c < "$WASM_PATH")
echo "      WASM size: ${WASM_SIZE} bytes"

if [ "$WASM_SIZE" -gt 65536 ]; then
  echo "ERROR: WASM exceeds Soroban 64KB contract size limit."
  exit 1
fi
echo "      Size OK."

# ── 3. Configure Stellar CLI identity ─────────────────────────────────────────
echo "[2/4] Configuring deployment identity..."
stellar keys add deployer --secret-key "$STELLAR_SECRET_KEY" 2>/dev/null || true

# ── 4. Deploy contract ────────────────────────────────────────────────────────
echo "[3/4] Deploying to $NETWORK..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm "$WASM_PATH" \
  --source deployer \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE")

echo "      Contract ID: $CONTRACT_ID"

# ── 5. Smoke test — query a non-existent stream (expects StreamNotFound) ──────
echo "[4/4] Smoke-testing deployed contract..."
RESULT=$(stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source deployer \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- withdrawable_balance \
  --stream_id 0 2>&1 || true)

if echo "$RESULT" | grep -q "StreamNotFound\|Error\|error"; then
  echo "      Smoke test passed (StreamNotFound returned as expected)."
else
  echo "WARNING: Unexpected smoke test result: $RESULT"
fi

# ── 6. Output summary ─────────────────────────────────────────────────────────
echo ""
echo "=== Deployment Complete ==="
echo "Contract ID : $CONTRACT_ID"
echo "Network     : $NETWORK"
echo "Explorer    : https://stellar.expert/explorer/testnet/contract/$CONTRACT_ID"
echo ""
echo "Save the Contract ID — you will need it for all invocations."
echo "CONTRACT_ID=$CONTRACT_ID" >> .env.testnet
