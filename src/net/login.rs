use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use bytes::BytesMut;
use anyhow::{Result, anyhow};

use crate::rsa;
use crate::types::AccountLogin;

/// XTEA konstanter
const DELTA: u32 = 0x9E3779B9;
const NUM_ROUNDS: u32 = 32;

/// XTEA decrypt helper
fn xtea_decrypt(buf: &mut [u8], key: &[u32; 4]) {
    for chunk in buf.chunks_mut(8) {
        let mut v0 = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let mut v1 = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        let mut sum: u32 = DELTA.wrapping_mul(NUM_ROUNDS);

        for _ in 0..NUM_ROUNDS {
            v1 = v1.wrapping_sub(
                ((v0 << 4) ^ (v0 >> 5)).wrapping_add(v0)
                    ^ (sum.wrapping_add(key[((sum >> 11) & 3) as usize])),
            );
            sum = sum.wrapping_sub(DELTA);
            v0 = v0.wrapping_sub(
                ((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1)
                    ^ (sum.wrapping_add(key[(sum & 3) as usize])),
            );
        }

        chunk[..4].copy_from_slice(&v0.to_le_bytes());
        chunk[4..].copy_from_slice(&v1.to_le_bytes());
    }
}

/// Parse login-paketet med RSA + XTEA (som TFS gör)
pub fn parse_login_packet_with_rsa(mut packet: BytesMut) -> Result<AccountLogin> {
    if packet.len() < 128 {
        return Err(anyhow!("RSA block too short"));
    }

    // --- steg 1: RSA block ---
    let mut rsa_block = packet.split_to(128);
    rsa::decrypt(rsa_block.as_mut())?; // muterar buffern direkt

    if rsa_block.len() != 128 {
        return Err(anyhow!(
            "RSA decrypt wrong size: got {}, expected 128",
            rsa_block.len()
        ));
    }

    // Skippa padding 0x00
    let mut i = 0;
    while i < rsa_block.len() && rsa_block[i] == 0 {
        i += 1;
    }
    if i + 86 > rsa_block.len() {
        return Err(anyhow!("RSA block too short after skipping padding"));
    }

    // XTEA key
    let mut xtea = [0u32; 4];
    for j in 0..4 {
        let start = i + j * 4;
        xtea[j] = u32::from_le_bytes([
            rsa_block[start],
            rsa_block[start + 1],
            rsa_block[start + 2],
            rsa_block[start + 3],
        ]);
    }

    // --- steg 2: Resten av paketet (XTEA-krypterat) ---
    let mut encrypted_payload = packet.to_vec();

    if encrypted_payload.len() < 3 {
        return Err(anyhow!("Payload too short before XTEA"));
    }
    // ta bort 2 bytes length + 1 byte opcode
    let mut encrypted_payload = encrypted_payload.split_off(3);

    if encrypted_payload.len() % 8 != 0 {
        return Err(anyhow!(
            "Encrypted payload not multiple of 8: {} bytes",
            encrypted_payload.len()
        ));
    }

    // Dekryptera in place
    xtea_decrypt(&mut encrypted_payload, &xtea);

    // --- steg 3: plocka ut login-fält från decrypted payload ---
    if encrypted_payload.len() < 70 {
        return Err(anyhow!("XTEA decrypted payload too short"));
    }

    let os = u16::from_le_bytes([encrypted_payload[0], encrypted_payload[1]]);
    let version = u16::from_le_bytes([encrypted_payload[2], encrypted_payload[3]]);

    // Account name string (Tibia string = u16 length + bytes)
    let name_len = u16::from_le_bytes([encrypted_payload[6], encrypted_payload[7]]) as usize;
    let account_name = String::from_utf8(encrypted_payload[8..8 + name_len].to_vec())
        .map_err(|_| anyhow!("Invalid UTF-8 in account name"))?;

    let pass_offset = 8 + name_len;
    let pass_len = u16::from_le_bytes([
        encrypted_payload[pass_offset],
        encrypted_payload[pass_offset + 1],
    ]) as usize;
    let password = String::from_utf8(
        encrypted_payload[pass_offset + 2..pass_offset + 2 + pass_len].to_vec(),
    )
    .map_err(|_| anyhow!("Invalid UTF-8 in password"))?;

    Ok(AccountLogin {
        account_name,
        password,
        xtea,
        os,
        version,
    })
}

pub async fn send_login_error(socket: &mut TcpStream) -> Result<()> {
    let msg = "Invalid account or password";
    let mut data = Vec::new();
    data.push(0x0A); // opcode for error
    data.extend_from_slice(&(msg.len() as u16).to_le_bytes());
    data.extend_from_slice(msg.as_bytes());

    socket.write_all(&data).await?;
    Ok(())
}
