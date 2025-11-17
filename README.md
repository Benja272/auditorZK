# AuditorZK - Proof of Reserves for Tokenized RWA

Privacy-preserving proof of reserves system for the Midnight blockchain using TLSNotary.

## Project Structure

```
auditorZK/
├── verifier-server/    # Rust WebSocket verifier (✅ COMPLETE)
├── prover-client/      # TypeScript browser prover (✅ COMPLETE)
└── contract-simulator/ # On-chain verification simulator (TODO)
```

## Architecture

The system proves financial reserves from **Plaid's sandbox API** without revealing exact balances:

```
Browser Prover (React + tlsn-js)
    ↓ WebSocket
Verifier Server (Rust, port 7047)
    ↓ MPC-TLS Protocol
Browser ↔ Plaid Sandbox API (sandbox.plaid.com)
    ↓ Signed Attestation
Smart Contract (Midnight/Compact - TODO)
```

**Privacy Model:**
- **Public on-chain:** User qualified (yes/no), threshold amount, commitment hash
- **Private:** Actual balance amount, API credentials, MPC secrets

## Quick Start

### Prerequisites

- **Rust** (stable): Install from [rustup.rs](https://rustup.rs/)
- **Node.js** v18+ and npm
- **Plaid sandbox credentials** (free at [plaid.com/docs](https://plaid.com/docs/))

### Setup

#### 1. Configure Plaid Credentials

Create `.env` file in `prover-client/`:

```bash
cd auditorZK/prover-client
cp .env.example .env
```

Edit `.env` with your Plaid sandbox credentials:
```env
REACT_APP_PLAID_CLIENT_ID=your_client_id
REACT_APP_PLAID_SECRET=your_sandbox_secret
REACT_APP_PLAID_ACCESS_TOKEN=your_access_token
REACT_APP_PLAID_ACCOUNT_ID=your_account_id
```

#### 2. Start Verifier Server

```bash
cd auditorZK/verifier-server
cargo run --release
```

Output:
```
🔐 AuditorZK Verifier Server
📡 Listening on: 0.0.0.0:7047
✅ Ready to verify TLS sessions
```

#### 3. Start WebSocket Proxy

**Required for browser to make TCP connections to Plaid.**

```bash
# Install once
cargo install wstcp

# Run proxy for Plaid sandbox
wstcp --bind-addr 127.0.0.1:55688 sandbox.plaid.com:443
```

Output:
```
Listening on ws://127.0.0.1:55688
Proxying to sandbox.plaid.com:443
```

#### 4. Start Prover Client

```bash
cd auditorZK/prover-client
npm install  # First time only
npm start
```

Browser opens at `http://localhost:3000`

### Usage

1. **Configure** (defaults should work):
   - Verifier URL: `ws://localhost:7047`
   - Proxy URL: `ws://localhost:55688`
   - Target: `https://sandbox.plaid.com/accounts/balance/get`
   - Threshold: `$10,000`

2. **Click "Start Proof of Reserves"**

3. **Watch the flow**:
   - Initialize TLSNotary WASM
   - Connect to verifier via WebSocket
   - Establish MPC-TLS session with Plaid
   - Send authenticated API request
   - Parse balance from response
   - Create cryptographic commitment
   - Receive signed attestation

4. **View results**:
   - Total balance across accounts
   - Qualification status (balance > threshold)
   - Privacy guarantees displayed

5. **Verify attestation**:
   ```bash
   cat /tmp/auditor_zk_attestation.json
   ```

## How It Works

### TLSNotary MPC-TLS Protocol

1. **Prover** (browser) connects to **Verifier** (Rust server)
2. Together they run **MPC protocol** to establish TLS with Plaid
3. Prover sends authenticated request to Plaid API
4. Response data is **committed** using MPC-derived secret
5. Verifier **signs attestation** binding commitment to TLS session
6. Prover can later prove `balance > threshold` with ZK proof

### Privacy Guarantees

**What's revealed:**
- User has balance above threshold ✅
- Connection was to `sandbox.plaid.com` ✅
- Timestamp of proof ✅

**What's hidden:**
- Exact balance amount 🔒
- API credentials 🔒
- Account details 🔒

### Cryptographic Commitment

```rust
commitment = SHA256(balance_data || mpc_secret)
```

The attestation signature proves:
1. TLS session was with Plaid
2. Commitment was created during that session
3. Cannot be forged or replayed

Later, a ZK circuit proves:
```
H(balance || secret) == commitment  // Knowledge of preimage
balance > threshold                  // Meets requirement
```

## Troubleshooting

### Environment variables not loading

**Error:** `client_id: undefined` in debug logs

**Solution:** Restart `npm start` after editing `.env` file. The `dotenv-webpack` plugin loads environment variables at build time.

### WebSocket connection failed (1006)

**Error:** `CloseEvent { code: 1006 }`

**Solution:** Make sure `wstcp` proxy is running:
```bash
wstcp --bind-addr 127.0.0.1:55688 sandbox.plaid.com:443
```

### Invalid Plaid credentials

**Error:** `INVALID_CREDENTIALS` from Plaid API

**Solution:**
1. Verify credentials at [Plaid Dashboard](https://dashboard.plaid.com/team/keys)
2. Use **sandbox** environment credentials (not development/production)
3. Ensure access token is valid for the account_id

### Port conflicts

Ports used:
- **7047**: Verifier server
- **55688**: WebSocket proxy
- **3000**: Prover client dev server

Check with:
```bash
lsof -i :7047
lsof -i :55688
lsof -i :3000
```

## Next Steps

1. ✅ Verifier server with MPC-TLS
2. ✅ Browser prover with Plaid integration
3. ⏳ Midnight/Compact smart contract for on-chain verification
4. ⏳ ZK circuit for balance threshold proof
5. ⏳ Integration tests
6. ⏳ Production deployment guide

## Technical Details

- **TLSNotary version:** v0.1.0-alpha.12 (tlsn-js) / v0.1.0-alpha.13 (Rust)
- **Signature scheme:** ECDSA with secp256k1
- **Commitment:** SHA256 hash
- **MPC protocol:** Garbled circuits for TLS handshake
- **Target blockchain:** Midnight (Compact smart contracts)

## License

(To be determined)

Contrato de alquiler de dust sobre pruebas de reservas en cardano.