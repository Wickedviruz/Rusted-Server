use anyhow::{anyhow, Result};
use bytes::{Buf, BytesMut};

#[inline]
pub fn get_u8(buf: &mut BytesMut) -> Result<u8> {
    if buf.len() < 1 { return Err(anyhow!("buffer underflow (u8)")); }
    Ok(buf.split_to(1)[0])
}

#[inline]
pub fn get_u16_le(buf: &mut BytesMut) -> Result<u16> {
    if buf.len() < 2 { return Err(anyhow!("buffer underflow (u16)")); }
    let v = u16::from_le_bytes([buf[0], buf[1]]);
    buf.advance(2);
    Ok(v)
}

#[inline]
pub fn get_u32_le(buf: &mut BytesMut) -> Result<u32> {
    if buf.len() < 4 { return Err(anyhow!("buffer underflow (u32)")); }
    let v = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    buf.advance(4);
    Ok(v)
}

/// Tibia string: u16 length (LE) + bytes
pub fn get_tibia_string(buf: &mut BytesMut) -> Result<String> {
    let len = get_u16_le(buf)? as usize;
    if buf.len() < len { return Err(anyhow!("buffer underflow (string)")); }
    let bytes = buf.split_to(len).to_vec();
    String::from_utf8(bytes).map_err(|_| anyhow!("invalid utf8 in string"))
}
