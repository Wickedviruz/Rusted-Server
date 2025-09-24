use crate::{
    connection::Connection,
    networkmessage::NetworkMessage,
    outputmessage::OutputMessage,
    rsa,
    xtea,
};
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

/// Trait motsvarande C++ `Protocol`-bas.
pub trait Protocol: Send + Sync {
    fn on_connect(&mut self) {}
    fn on_recv_first_message(&mut self, msg: &mut NetworkMessage);
    fn on_recv_message(&mut self, msg: &mut NetworkMessage);
    fn on_send_message(&self, _msg: &mut OutputMessage) {}

    /// Obs: ingen async här längre!
    fn disconnect(&self, conn: &Arc<Mutex<Connection>>) {
        // Här kan vi trigga att anslutningen stängs, men utan await.
        // Connection::close är sync, så vi kallar den direkt.
        let mut conn = conn.blocking_lock(); // tokio::Mutex har .blocking_lock()
        conn.close();
    }

    fn get_ip(&self, conn: &Arc<Connection>) -> u32 {
        conn.get_ip()
    }
}

/// Bas som håller gemensamt state (XTEA, checksum etc.)
#[derive(Clone)]
pub struct ProtocolBase {
    pub connection: Weak<Mutex<Connection>>,
    pub output_buffer: Option<Arc<Mutex<OutputMessage>>>,
    pub key: Option<xtea::RoundKeys>,
    pub encryption_enabled: bool,
    pub checksum_enabled: bool,
    pub raw_messages: bool,
}

impl ProtocolBase {
    pub fn new(connection: Weak<Mutex<Connection>>) -> Self {
        Self {
            connection,
            output_buffer: None,
            key: None,
            encryption_enabled: false,
            checksum_enabled: true,
            raw_messages: false,
        }
    }

    pub fn enable_xtea(&mut self) {
        self.encryption_enabled = true;
    }
    pub fn set_xtea_key(&mut self, key: xtea::Key) {
        self.key = Some(xtea::expand_key(&key));
    }
    pub fn disable_checksum(&mut self) {
        self.checksum_enabled = false;
    }
    pub fn set_raw_messages(&mut self, v: bool) {
        self.raw_messages = v;
    }
    pub fn get_connection(&self) -> &Weak<Mutex<Connection>> {
        &self.connection
    }

    /// RSA-decrypt block i `NetworkMessage`
    pub fn rsa_decrypt(msg: &mut NetworkMessage) -> bool {
        if (msg.get_length() - msg.get_buffer_position()) < 128 {
            return false;
        }
        let buf = &mut msg.buffer[msg.position as usize..msg.position as usize + 128];
        if rsa::decrypt(buf).is_err() {
            return false;
        }
        msg.get_byte() == 0
    }
}
