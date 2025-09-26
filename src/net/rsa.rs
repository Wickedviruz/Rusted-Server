use anyhow::{anyhow, Result};
use base64::Engine as _;
use lazy_static::lazy_static;
use rsa::{pkcs1::DecodeRsaPrivateKey, RsaPrivateKey,  traits::{PrivateKeyParts, PublicKeyParts}};
use std::{sync::Mutex};
use num_bigint_dig::BigUint;

pub const RSA_BUFFER_LENGTH: usize = 128;

lazy_static! {
    static ref PKEY: Mutex<Option<RsaPrivateKey>> = Mutex::new(None);
}

pub fn load_pem_file<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    let pem = std::fs::read_to_string(path)?;
    load_pem_str(&pem)
}

pub fn load_pem_str(pem: &str) -> Result<()> {
    const HEADER: &str = "-----BEGIN RSA PRIVATE KEY-----";
    const FOOTER: &str = "-----END RSA PRIVATE KEY-----";

    let start: usize = pem.find(HEADER).ok_or_else(|| anyhow!("Missing RSA PEM header"))? + HEADER.len();
    let end: usize = pem.find(FOOTER).ok_or_else(|| anyhow!("Missing RSA PEM footer"))?;

    let b64_region: String = pem[start..end]
        .lines()
        .map(|l: &str| l.trim())
        .collect::<String>();

    let der: Vec<u8> = base64::engine::general_purpose::STANDARD
        .decode(&b64_region)
        .map_err(|e: base64::DecodeError| anyhow!("Base64 decode failed: {}", e))?;

    let key: RsaPrivateKey = RsaPrivateKey::from_pkcs1_der(&der)
        .map_err(|e: rsa::pkcs1::Error| anyhow!("Failed to parse PKCS#1 DER: {}", e))?;

    let mut g: std::sync::MutexGuard<'_, Option<RsaPrivateKey>> = PKEY.lock().unwrap();
    *g = Some(key);
    Ok(())
}

/// Dekryptera ett block (128 bytes) in-place, utan padding.
pub fn decrypt(buf: &mut [u8]) -> Result<()> {
    if buf.len() != RSA_BUFFER_LENGTH {
        return Err(anyhow!("RSA decrypt expects exactly 128 bytes"));
    }

    let key = PKEY
        .lock()
        .unwrap()
        .as_ref()
        .cloned()
        .ok_or_else(|| anyhow!("RSA key not loaded"))?;

    let n = key.n().clone();
    let d = key.d().clone();

    let c = BigUint::from_bytes_be(buf);
    let m = c.modpow(&d, &n);

    let m_bytes = m.to_bytes_be();
    buf.fill(0);
    buf[RSA_BUFFER_LENGTH - m_bytes.len()..].copy_from_slice(&m_bytes);

    Ok(())
}

pub fn load_pem(path: &str) -> Result<()> {
    load_pem_file(path)
}

pub fn rsa_decrypt(buf: &mut [u8]) -> Result<()> {
    decrypt(buf)
}