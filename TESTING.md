# AuditorZK - Manual Testing Results

## Test Date: 2025-11-14

### ✅ Test 1: Mock Plaid Server

**Status**: PASSED

**Server**: `http://127.0.0.1:8443`

**Tests Performed**:

1. **Health Check Endpoint**:
```bash
$ curl http://127.0.0.1:8443/health
```
**Result**:
```json
{
  "status": "ok",
  "service": "mock-plaid-server"
}
```
✅ Server responds correctly

2. **Balance Endpoint**:
```bash
$ curl http://127.0.0.1:8443/balance
```
**Result**:
```json
{
  "accounts": [
    {
      "balances": {
        "available": 15234.50,
        "current": 15234.50
      },
      "name": "Checking Account"
    },
    {
      "balances": {
        "available": 5678.25,
        "current": 5678.25
      },
      "name": "Savings Account"
    }
  ]
}
```
✅ Returns mock balance data ($20,912.75 total)

**Server Logs**:
```
INFO mock_plaid_server: 🏦 Mock Plaid Server - AuditorZK
INFO mock_plaid_server: ================================
INFO mock_plaid_server: 📡 Server listening on http://127.0.0.1:8443
INFO mock_plaid_server: ✅ Ready to serve requests
INFO mock_plaid_server: 📥 Received balance request
INFO mock_plaid_server: ✅ Returning balance data
```

### ✅ Test 2: Verifier Server Startup

**Status**: PASSED

**Server**: `ws://0.0.0.0:7047`

**Tests Performed**:

1. **Server Compilation**:
```bash
$ cd verifier-server && cargo check
```
✅ Compiles without errors

2. **Server Startup**:
```bash
$ cargo run
```
**Result**:
```
INFO auditor_zk_verifier: 🔐 AuditorZK Verifier Server
INFO auditor_zk_verifier: ================================
INFO auditor_zk_verifier: 📡 Listening on: 0.0.0.0:7047
INFO auditor_zk_verifier: ✅ Ready to verify TLS sessions from prover clients
```
✅ Server starts successfully

3. **Port Accessibility**:
```bash
$ python3 -c "import socket; ..."
```
**Result**:
```
✅ Verifier server is listening on port 7047
```
✅ WebSocket server accepts connections

## Summary

### Working Components ✅
1. **Mock Plaid Server**: Serving balance data on port 8443
2. **Verifier Server**: WebSocket server listening on port 7047
3. **Project Structure**: All code compiles and runs

### Next Steps 🚧
1. **Prover Client**: Build TypeScript browser client using tlsn-js
2. **Integration Test**: Connect prover → verifier → mock Plaid
3. **Attestation Test**: Verify attestation signing and saving
4. **Contract Simulator**: Build on-chain verification simulator

## How to Run

### Terminal 1: Mock Plaid Server
```bash
cd auditorZK/mock-plaid-server
cargo run
```

### Terminal 2: Verifier Server
```bash
cd auditorZK/verifier-server
cargo run
```

### Test the Servers
```bash
# Test mock Plaid
curl http://127.0.0.1:8443/health
curl http://127.0.0.1:8443/balance | jq .

# Test verifier (requires WebSocket client)
# Will be tested once prover client is built
```

## Configuration

### Mock Plaid Server
- Port: 8443
- Endpoints: `/health`, `/balance`
- Data source: `../mock-data/balance.json`

### Verifier Server
- Port: 7047
- Protocol: WebSocket
- Max sent: 4KB
- Max received: 16KB
- Attestation output: `/tmp/auditor_zk_attestation.json`
- Public key: `config/notary_pubkey.pem`

## Known Limitations

- Mock server uses HTTP (not HTTPS) - TLS will be added when needed for TLSNotary
- Verifier server hasn't been tested with actual prover connection yet
- No end-to-end integration test yet (waiting for prover client)
