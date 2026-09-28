use duck_core::{AppPaths, Failure, fs::create_unique_dir};
use duck_platform::net::http_client;
use serde::Serialize;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

use crate::{
    cli::ProvisionArgs,
    cose::{DeviceKeys, RPC_CURVE_25519, build_csr, generate_ec_keypair},
    crypto_kdf::resolve_seed,
    http::{EekResponse, fetch_eek, submit_csr},
    keybox_xml::{CertificateChainSummary, parse_der_cert_chain, summarize_chain},
    verify::{VerifyReport, verify_csr},
};

use super::{ensure_request_context, path_details, random_bytes, resolve_runtime, write_output};

#[derive(Debug, Serialize)]
struct ProvisionChain {
    index: usize,
    path: String,
    summary: CertificateChainSummary,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProvisionData {
    mode: String,
    curve: String,
    cdi_leaf_pubkey_hex: String,
    challenge_hex: String,
    csr_path: String,
    csr_len: usize,
    protected_data_len: usize,
    local_verify: VerifyReport,
    cert_chains: Vec<ProvisionChain>,
    local_test_mode: bool,
    fetch_eek_error: Option<String>,
    server_submission_error: Option<String>,
}

/// Runs a full provisioning round trip. When the server is unreachable the CSR is still
/// built against a locally generated X25519 EEK ("local test mode") so it can be verified.
pub(crate) async fn run(paths: &AppPaths, args: &ProvisionArgs) -> Result<ProvisionData, Failure> {
    let resolved = resolve_runtime(paths, &args.shared, args.num_keys, None)?;
    let server_url = ensure_request_context(&resolved.profile)?;
    let keys = DeviceKeys::from_seed_with_curve(
        resolve_seed(&resolved.profile.key_source)?,
        resolved.profile.curve,
    );
    let client = http_client()?;

    let mut cose_pubs = Vec::new();
    for _ in 0..resolved.profile.num_keys {
        cose_pubs.push(generate_ec_keypair()?.cose_public);
    }

    let (eek, fetch_eek_error) =
        match fetch_eek(&client, &resolved.profile.fingerprint.value, &server_url).await {
            Ok(eek) => (eek, None),
            Err(error) => (local_test_eek()?, Some(format!("{error:#}"))),
        };

    let csr = build_csr(
        &keys,
        &eek.challenge,
        &cose_pubs,
        &eek.eek_public,
        &eek.eek_id,
        &resolved.profile.device,
        eek.eek_curve,
    )?;

    let run_dir = create_unique_dir(&paths.outputs_dir, "rkp-provision")?;
    let csr_path = run_dir.join("csr_output.cbor");
    let csr_details = path_details("csr_path", &csr_path);
    write_output(&csr_path, &csr.csr_bytes, Some(csr_details.clone()))?;

    let local_verify =
        verify_csr(&csr.csr_bytes).map_err(|error| Failure::with_details(error, csr_details))?;

    let (cert_chains, server_submission_error) =
        match submit_csr(&client, &csr.csr_bytes, &eek.challenge, &server_url).await {
            Ok(chains) => (chains, None),
            Err(error) => (Vec::new(), Some(format!("{error:#}"))),
        };

    let mut chain_data = Vec::new();
    for (index, chain) in cert_chains.iter().enumerate() {
        let chain_path = run_dir.join(format!("cert_chain_{index}.der"));
        write_output(&chain_path, chain, None)?;
        chain_data.push(ProvisionChain {
            index,
            path: chain_path.display().to_string(),
            summary: summarize_chain(&parse_der_cert_chain(chain)?),
        });
    }

    Ok(ProvisionData {
        mode: resolved.profile.key_source.mode_label().into(),
        curve: resolved.profile.curve.as_str().into(),
        cdi_leaf_pubkey_hex: keys.public_key_hex(),
        challenge_hex: eek.challenge_hex,
        csr_path: csr_path.display().to_string(),
        csr_len: csr.csr_bytes.len(),
        protected_data_len: csr.protected_data_len,
        local_verify,
        cert_chains: chain_data,
        local_test_mode: fetch_eek_error.is_some(),
        fetch_eek_error,
        server_submission_error,
    })
}

fn local_test_eek() -> anyhow::Result<EekResponse> {
    let challenge = random_bytes(32)?;
    let secret: [u8; 32] = random_bytes(32)?
        .try_into()
        .expect("random_bytes returns the requested length");
    let public_key = X25519PublicKey::from(&X25519StaticSecret::from(secret));

    Ok(EekResponse {
        challenge_hex: hex::encode(&challenge),
        challenge,
        eek_public_hex: hex::encode(public_key.as_bytes()),
        eek_public: public_key.as_bytes().to_vec(),
        eek_id: Vec::new(),
        eek_curve: RPC_CURVE_25519,
    })
}
