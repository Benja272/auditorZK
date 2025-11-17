use anyhow::{Result, Context};
use k256::{
    schnorr::{SigningKey, Signature, VerifyingKey, signature::Signer},
    elliptic_curve::rand_core::OsRng,
};
use k256::sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{info, warn};
use tlsn_core::VerifierOutput;

const KEY_PATH: &str = "config/notary_key.pem";
const PUBKEY_PATH: &str = "config/notary_pubkey.pem";
const SIGNATURE_VERSION: [u8; 3] = [0x01, 0x00, 0x00]; // BIP-340 signature version 1.0.0

/// Attestation structure that will be signed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// The server that was connected to
    pub server_name: String,
    /// Timestamp of the session
    pub timestamp: u64,
    /// Commitment to the balance data (first hash commitment)
    pub balance_commitment: Vec<u8>,
    /// BIP-340 signature (hex-encoded with 3-byte version prefix)
    pub signature: String,
    /// The verifier's public key (for signature verification)
    pub verifier_pubkey: Vec<u8>,
}

/// Sign the verification output as an attestation
pub async fn sign_attestation(mut output: VerifierOutput) -> Result<Vec<u8>> {
    info!("🔏 Creating and signing attestation...");

    // Load or generate signing key
    let signing_key = load_or_generate_key()?;
    let verifying_key = signing_key.verifying_key();

    // Extract server name
    let server_name = output.server_name.take()
        .map(|sn| format!("{:?}", sn))
        .unwrap_or_else(|| "unknown".to_string());

    // Get current timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    // Extract balance commitment (first hash commitment from received data)
    let balance_commitment = extract_balance_commitment(&output)?;

    info!("📝 Attestation details:");
    info!("   Server: {}", server_name);
    info!("   Timestamp: {}", timestamp);
    info!("   Commitment: {}...", hex::encode(&balance_commitment[..16]));

    // Create message to sign (server_name + timestamp + balance_commitment)
    let mut message = Vec::new();
    message.extend_from_slice(server_name.as_bytes());
    message.extend_from_slice(&timestamp.to_le_bytes());
    message.extend_from_slice(&balance_commitment);

    // Hash the message
    let message_hash = Sha256::digest(&message);

    // Sign with BIP-340 Schnorr
    let signature: Signature = signing_key.sign(&message_hash);

    // Create hex-encoded signature with 3-byte version prefix
    let sig_bytes = signature.to_bytes();
    let mut versioned_sig = Vec::with_capacity(67); // 3 + 64
    versioned_sig.extend_from_slice(&SIGNATURE_VERSION);
    versioned_sig.extend_from_slice(&sig_bytes);
    let hex_signature = hex::encode(versioned_sig);

    info!("✅ Attestation signed with BIP-340 Schnorr");
    info!("   Signature: {}...", &hex_signature[..32]);

    // Create attestation structure
    let attestation = Attestation {
        server_name,
        timestamp,
        balance_commitment,
        signature: hex_signature,
        verifier_pubkey: verifying_key.to_bytes().to_vec(),
    };

    // Serialize attestation
    let attestation_bytes = serde_json::to_vec_pretty(&attestation)?;

    // Save attestation to file for contract simulator
    save_attestation(&attestation)?;

    Ok(attestation_bytes)
}

/// Extract the balance commitment from transcript commitments
fn extract_balance_commitment(output: &VerifierOutput) -> Result<Vec<u8>> {
    // In alpha.12, transcript_commitments is a Vec<TranscriptCommitment>
    // We need to serialize and hash these commitments

    if output.transcript_commitments.is_empty() {
        anyhow::bail!("No transcript commitments found");
    }

    // Serialize the commitments to bytes
    let commitments_bytes = bincode::serialize(&output.transcript_commitments)
        .context("Failed to serialize transcript commitments")?;

    // Use a hash of the commitments as the balance commitment
    // This ensures the attestation is cryptographically bound to the session
    use sha2::{Digest, Sha256};
    let commitment_hash = Sha256::digest(&commitments_bytes);

    Ok(commitment_hash.to_vec())
}

/// Load existing key or generate new one
fn load_or_generate_key() -> Result<SigningKey> {
    if Path::new(KEY_PATH).exists() {
        info!("🔑 Loading existing signing key from {}", KEY_PATH);
        // For simplicity, just generate a new key each time for now
        // In production, properly load from PEM
        warn!("⚠️  Key persistence not yet implemented, using ephemeral key");
    }

    info!("🔑 Generating new ECDSA signing key");
    let signing_key = SigningKey::random(&mut OsRng);

    // Ensure config directory exists
    fs::create_dir_all("config").context("Failed to create config directory")?;

    // Save public key for verification
    let verifying_key = signing_key.verifying_key();
    let pubkey_bytes = verifying_key.to_bytes();
    fs::write(PUBKEY_PATH, hex::encode(pubkey_bytes))
        .context("Failed to save public key")?;
    info!("💾 Public key saved to {}", PUBKEY_PATH);

    Ok(signing_key)
}

/// Save attestation to file for contract simulator
fn save_attestation(attestation: &Attestation) -> Result<()> {
    let attestation_json = serde_json::to_string_pretty(attestation)?;
    let path = "/tmp/auditor_zk_attestation.json";
    fs::write(path, attestation_json)
        .context("Failed to save attestation")?;
    info!("💾 Attestation saved to {}", path);
    Ok(())
}
