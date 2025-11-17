use anyhow::{Result, bail};
use tracing::{info, warn};
use tlsn_core::VerifierOutput;

/// Validate that the connection was to a Plaid server or localhost (for testing)
pub fn validate_plaid_connection(output: &VerifierOutput) -> Result<()> {
    info!("🏦 Validating server connection...");

    // Check server identity
    let server_name = output.server_name.as_ref()
        .ok_or_else(|| anyhow::anyhow!("No server name provided"))?;

    // In alpha.12, ServerName has an as_str() method
    let name_str = server_name.as_str();

    // Accept Plaid domains or localhost for testing
    let is_valid = name_str.ends_with(".plaid.com") ||
        name_str == "production.plaid.com" ||
        name_str == "sandbox.plaid.com" ||
        name_str == "development.plaid.com" ||
        name_str == "localhost" ||
        name_str == "127.0.0.1";

    if !is_valid {
        warn!("❌ Server is not a valid Plaid or test domain: {}", name_str);
        bail!("Server must be a Plaid API endpoint or localhost for testing");
    }

    info!("✅ Confirmed valid server: {}", name_str);

    // Validate we received commitments (required for privacy-preserving proofs)
    if output.transcript_commitments.is_empty() {
        warn!("❌ No transcript commitments provided");
        bail!("Prover must commit to transcript data using selective disclosure");
    }

    // Validate at least one Hash commitment exists (for balance proof)
    let has_hash_commitment = output.transcript_commitments.iter().any(|c| {
        matches!(c, tlsn_core::transcript::TranscriptCommitment::Hash(_))
    });

    if !has_hash_commitment {
        warn!("❌ No Hash commitments found");
        bail!("Prover must provide at least one SHA256 hash commitment for balance data");
    }

    info!("✅ {} transcript commitment(s) received", output.transcript_commitments.len());

    // Validate SHA256 algorithm is used
    for (i, commitment) in output.transcript_commitments.iter().enumerate() {
        match commitment {
            tlsn_core::transcript::TranscriptCommitment::Hash(hash) => {
                use tlsn_core::hash::HashAlgId;
                if hash.hash.alg != HashAlgId::SHA256 {
                    warn!("❌ Commitment {} uses {:?}, expected SHA256", i, hash.hash.alg);
                    bail!("All hash commitments must use SHA256 algorithm");
                }
                info!("   ✅ Commitment {}: SHA256 hash ({:?})", i, hash.direction);
            }
            tlsn_core::transcript::TranscriptCommitment::Encoding(_) => {
                info!("   ℹ️  Commitment {}: Encoding (Merkle tree)", i);
            }
            _ => {
                info!("   ℹ️  Commitment {}: Unknown type", i);
            }
        }
    }

    // PRIVACY MODE: Handle both selective disclosure and partial revelation
    if let Some(transcript) = &output.transcript {
        info!("⚠️  PRIVACY WARNING: Partial transcript was revealed");
        info!("   The verifier can see {} bytes sent, {} bytes received",
              transcript.sent_unsafe().len(),
              transcript.received_unsafe().len());

        // If transcript is revealed, do basic validation
        let recv = String::from_utf8_lossy(transcript.received_unsafe());

        if recv.contains("\"accounts\"") || recv.contains("\"balances\"") {
            info!("   ℹ️  Detected Plaid API response structure in revealed data");
        } else if recv.contains("HTTP") {
            info!("   ℹ️  Valid HTTP response in revealed data");
        }
    } else {
        info!("🔒 Full selective disclosure mode: No transcript revealed");
        info!("   ✅ Maximum privacy: Verifier cannot see balance data");
    }

    Ok(())
}

/// Analyze and log commitment details for debugging
#[allow(dead_code)]
pub fn analyze_commitments(output: &VerifierOutput) {
    info!("📊 Commitment Analysis:");
    info!("  Total commitments: {}", output.transcript_commitments.len());
    // Note: In alpha.12, transcript_commitments is just a Vec<u8>
    // For detailed analysis, we'd need to deserialize the commitment structure
}
