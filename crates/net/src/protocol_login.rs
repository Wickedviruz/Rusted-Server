use crate::{
    consts::*,
    connection::Connection,
    networkmessage::NetworkMessage,
    outputmessage::OutputMessage,
    protocol::{Protocol, ProtocolBase},
    rsa,
};
use db::{Database, DatabaseManager};
use anyhow::{anyhow, Result};
use std::sync::Weak;
use tokio::sync::Mutex;
use std::sync::Arc;

/// Minimal representant för vad vi plockar ut ur första paketet.
#[derive(Debug, Clone)]
pub struct LoginHandshake {
    pub version: u16,
    pub xtea_key: [u32; 4],
    pub account_name: String,
    pub password: String,
    pub auth_token: String,
}

/// Själva protokoll-klassen
pub struct ProtocolLogin {
    pub base: ProtocolBase,
    pub db: Database,
}

impl ProtocolLogin {
    pub fn new(connection: Weak<Mutex<Connection>>, db: Database) -> Self {
        Self {
            base: ProtocolBase::new(connection),
            db,
        }
    }

    fn get_connection(&self) -> &Weak<Mutex<Connection>> {
        self.base.get_connection()
    }

    /// Bygger och skickar character list till klienten (TFS-liknande).
    pub async fn get_character_list(
        &self,
        conn: &mut Connection,
        account_name: &str,
        password: &str,
        token: &str,
        _version: u16,
    ) -> Result<()> {
        // Normalt: kolla DB här. Vi mockar kontot.
        let characters = vec!["Sorcerer".to_string(), "Knight".to_string()];

        let mut output = OutputMessage::new();

        // MOTD (0x14)
        let motd = "Welcome to the Rust OT!";
        output.msg.add_byte(0x14);
        output.msg.add_string(&format!("1\n{}", motd));

        // Session key (0x28)
        output.msg.add_byte(0x28);
        let ticks = (chrono::Utc::now().timestamp() / 30) as i64;
        let session_key = format!("{}\n{}\n{}\n{}", account_name, password, token, ticks);
        output.msg.add_string(&session_key);

        // Character list (0x64)
        output.msg.add_byte(0x64);

        // Worlds
        output.msg.add_byte(1); // number of worlds
        output.msg.add_byte(0); // world id
        output.msg.add_string("RustOTS"); // server name
        output.msg.add_string("127.0.0.1"); // server IP
        output.msg.add::<u16>(7172); // game port
        output.msg.add_byte(0); // preview world = false

        // Characters
        output.msg.add_byte(characters.len() as u8);
        for name in &characters {
            output.msg.add_byte(0); // world-id
            output.msg.add_string(name);
        }

        // Premium days
        output.msg.add_byte(0);
        output.msg.add_byte(1); // has premium
        output.msg.add::<u32>(0); // days left (0 = unlimited if FREE_PREMIUM)

        // Add crypto header + length
        output.add_crypto_header(true);

        // Debug: dump hela paketet innan vi skickar
        println!(
            "[ProtocolLogin] Sending packet ({} bytes): {:?}",
            output.get_output_buffer().len(),
            output.get_output_buffer()
        );

        // Skicka ut
        let arc_msg = Arc::new(output);
        
        conn.send(arc_msg).await?;


        // Disconnect login (klienten öppnar ny mot game server)
        //conn.close();

        Ok(())
    }
}

impl Protocol for ProtocolLogin {
    fn on_recv_first_message(&mut self, msg: &mut NetworkMessage) {
        match parse_login_first_message(msg) {
            Ok(handshake) => {
                println!("Login attempt: {:?}", handshake);

                let db = self.db.clone();
                let base = self.base.clone();
                let handshake_cloned = handshake.clone();

                // Viktigt: vi spawnar *efter* att den här funktionen returnerat och accept-loopen släppt låset.
                tokio::spawn(async move {
                    let manager = DatabaseManager::new(db.clone());
                    match manager.check_account(&handshake_cloned.account_name, &handshake_cloned.password).await {
                        Ok(true) => {
                            println!("✅ Login OK for {}", handshake_cloned.account_name);

                            if let Some(conn_arc) = base.get_connection().upgrade() {
                                let mut conn = conn_arc.lock().await; // nu kan vi vänta på låset
                                let proto = ProtocolLogin { base: base.clone(), db: db.clone() };

                                if let Err(e) = proto.get_character_list(
                                    &mut conn,
                                    &handshake_cloned.account_name,
                                    &handshake_cloned.password,
                                    &handshake_cloned.auth_token,
                                    handshake_cloned.version,
                                ).await {
                                    eprintln!("Failed to send charlist: {}", e);
                                } else {
                                    println!("✅ Charlist skickad till klienten!");
                                }
                            }
                        }
                        Ok(false) => println!("❌ Login FAILED for {}", handshake_cloned.account_name),
                        Err(e) => eprintln!("DB error: {}", e),
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to parse login packet: {}", e);
            }
        }
    }

    fn on_recv_message(&mut self, _msg: &mut NetworkMessage) {
        // Efter login används ej
    }

    fn on_send_message(&self, _msg: &mut OutputMessage) {
        // Här kan vi lägga på XTEA/CRC om vi vill
    }
}

/// Kör TFS-liknande login-parsning på första klientpaketet.
/// - Läser OS, version
/// - Skippar signatures (17/12 bytes)
/// - RSA-dekrypterar första blocket för XTEA-key
/// - Version-check
/// - Läser account/password
/// - Hoppar till sista 128 bytes, RSA-dekrypterar och läser auth token
pub fn parse_login_first_message(msg: &mut NetworkMessage) -> Result<LoginHandshake> {
    println!(
        "[ProtocolLogin] Parsing login message, total length: {}",
        msg.get_length()
    );
    println!(
        "[ProtocolLogin] Buffer head: {:?}",
        &msg.buffer[..std::cmp::min(32, msg.get_length() as usize)]
    );

    // 1) OS (2 bytes, ignoreras)
    let os = msg.get_u16();
    println!(
        "[DEBUG] getU16() -> os={} (pos={}/{})",
        os,
        msg.get_buffer_position(),
        msg.get_length()
    );

    // 2) version (u16 LE)
    let version: u16 = msg.get_u16();
    println!(
        "[DEBUG] client version={} (pos={}/{})",
        version,
        msg.get_buffer_position(),
        msg.get_length()
    );

    // 3) Skip signatures (17 eller 12 bytes)
    if version >= 971 {
        msg.skip_bytes(17);
        println!(
            "[DEBUG] skipped 17 bytes (signatures) (pos={}/{})",
            msg.get_buffer_position(),
            msg.get_length()
        );
    } else {
        msg.skip_bytes(12);
        println!(
            "[DEBUG] skipped 12 bytes (signatures) (pos={}/{})",
            msg.get_buffer_position(),
            msg.get_length()
        );
    }

    // 4) RSA-decrypt första blocket direkt i msg-buffer
    if !ProtocolBase::rsa_decrypt(msg) {
        return Err(anyhow!("RSA #1 decrypt failed"));
    }

    // 5) Läs XTEA-nyckeln (4 * u32)
    let mut xtea_key = [0u32; 4];
    for i in 0..4 {
        xtea_key[i] = msg.get::<u32>();
        println!(
            "[DEBUG] XTEA key[{}] = {} (pos={}/{})",
            i,
            xtea_key[i],
            msg.get_buffer_position(),
            msg.get_length()
        );
    }

    // 6) Versiongränser
    if version < CLIENT_VERSION_MIN as u16 || version > CLIENT_VERSION_MAX as u16 {
        return Err(anyhow!(
            "Only clients with protocol {} allowed!",
            CLIENT_VERSION_STR
        ));
    }

    // 7) Account name
    let account_name = msg.get_string(None);
    println!(
        "[DEBUG] accountName='{}' (pos={}/{})",
        account_name,
        msg.get_buffer_position(),
        msg.get_length()
    );
    if account_name.is_empty() {
        return Err(anyhow!("Invalid account name."));
    }

    // 8) Password
    let password = msg.get_string(None);
    println!(
        "[DEBUG] password='{}' (pos={}/{})",
        password,
        msg.get_buffer_position(),
        msg.get_length()
    );
    if password.is_empty() {
        return Err(anyhow!("Invalid password."));
    }

    // 9) Hoppa fram till sista RSA-blocket (auth token)
    let total_len = msg.get_length() as i32;
    let cur_pos = msg.get_buffer_position() as i32;
    let tail_start = total_len - (rsa::RSA_BUFFER_LENGTH as i32);
    let to_skip = tail_start - cur_pos;
    if to_skip > 0 {
        msg.skip_bytes(to_skip as i16);
        println!(
            "[DEBUG] skipped {} bytes (to reach RSA#2) (pos={}/{})",
            to_skip,
            msg.get_buffer_position(),
            msg.get_length()
        );
    }

    if !ProtocolBase::rsa_decrypt(msg) {
        return Err(anyhow!("RSA #2 decrypt failed"));
    }

    // 10) Auth token (string)
    let auth_token = msg.get_string(None);
    println!(
        "[DEBUG] authToken='{}' (pos={}/{})",
        auth_token,
        msg.get_buffer_position(),
        msg.get_length()
    );

    Ok(LoginHandshake {
        version,
        xtea_key,
        account_name,
        password,
        auth_token,
    })
}