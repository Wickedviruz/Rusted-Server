use std::sync::Arc;
use anyhow::Result;
use tokio::net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}};
use tokio::sync::{mpsc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::networkmessage::NetworkMessage;
use crate::outputmessage::OutputMessage;
use crate::protocol::Protocol;
use crate::tools::adler32;

const SEND_QUEUE_SIZE: usize = 128;

pub struct Connection {
    pub reader: Option<OwnedReadHalf>,
    pub writer_tx: mpsc::Sender<Arc<OutputMessage>>,
    pub protocol: Option<Arc<Mutex<dyn Protocol + Send>>>,
    pub received_first: bool,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Arc<Mutex<Self>> {
        let (reader, writer) = stream.into_split();
        let (tx, rx) = mpsc::channel(SEND_QUEUE_SIZE);

        let conn = Arc::new(Mutex::new(Connection {
            reader: Some(reader),
            writer_tx: tx,
            protocol: None,
            received_first: false,
        }));

        // Starta read-loop
        {
            let conn_clone = conn.clone();
            tokio::spawn(async move {
                // vi flyttar ut reader ur structen här
                let mut reader = {
                    let mut lock = conn_clone.lock().await;
                    lock.reader.take().expect("Reader already taken")
                };
                if let Err(e) = Self::read_loop(conn_clone.clone(), reader).await {
                    eprintln!("[Connection] Read loop error: {}", e);
                }
            });
        }

        // Starta write-loop
        tokio::spawn(async move {
            if let Err(e) = Self::write_loop(writer, rx).await {
                eprintln!("[Connection] Write loop error: {}", e);
            }
        });

        conn
    }

    pub async fn close(&mut self) {
        self.writer_tx.closed().await;
    }

    pub fn set_protocol(&mut self, proto: Arc<Mutex<dyn Protocol + Send>>) {
        self.protocol = Some(proto);
    }

    pub fn get_ip(&self) -> u32 {
        // OBS: vi har inte reader kvar i structen efter att read_loop tar ownership.
        // Så vi får hämta peer_addr från protokollet istället om vi behöver senare.
        0
    }

    pub async fn send(&self, msg: Arc<OutputMessage>) -> Result<()> {
        self.writer_tx
            .send(msg)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    async fn read_loop(conn: Arc<Mutex<Self>>, mut reader: OwnedReadHalf) -> Result<()> {
    loop {
        // 1) Läs header (2 bytes)
        let mut header = [0u8; NetworkMessage::HEADER_LENGTH];
        if reader.read_exact(&mut header).await.is_err() {
            return Err(anyhow::anyhow!("client disconnected"));
        }
        println!("[Connection] Header bytes: {:?}", header);

        // 2) Body length (LE)
        let body_len = u16::from_le_bytes(header) as usize;
        println!("[Connection] Body-längd enligt header: {}", body_len);

        // 3) Läs body
        let mut body = vec![0u8; body_len];
        reader.read_exact(&mut body).await?;
        println!(
            "[Connection] Body {} bytes läst. Body head: {:?}",
            body_len,
            &body[..std::cmp::min(32, body_len)]
        );

        // 4) Bygg upp msg på samma sätt som gamla accept()
        let mut msg = NetworkMessage::new();
        msg.buffer[0..2].copy_from_slice(&header);
        msg.buffer[2..2 + body_len].copy_from_slice(&body);

        msg.set_length((body_len + NetworkMessage::HEADER_LENGTH) as u16);
        msg.position = NetworkMessage::HEADER_LENGTH as u16; // starta efter headern

        // 5) Adler32 checksum-kontroll (precis som gamla koden)
        let cur = msg.get_buffer_position() as usize; // borde vara 2
        let total = msg.get_length() as usize;        // 2 + body_len
        let len_for_adler = total.saturating_sub(cur + NetworkMessage::CHECKSUM_LENGTH);

        let calc_adler = if len_for_adler > 0 {
            adler32(&msg.buffer[(cur + NetworkMessage::CHECKSUM_LENGTH)..(cur + NetworkMessage::CHECKSUM_LENGTH + len_for_adler)])
        } else {
            0
        };

        // läs mottagen checksum (flyttar pos med 4)
        let recv_adler = msg.get_u32();
        if recv_adler != calc_adler {
            // TFS fallback: inte en checksum → backa 4
            msg.skip_bytes(-(NetworkMessage::CHECKSUM_LENGTH as i16));
        }

        // 6) Dispatch till protokollet
        let mut conn_lock = conn.lock().await;
        if let Some(proto) = conn_lock.protocol.clone() {
            let mut proto = proto.lock().await;
            if !conn_lock.received_first {
                println!("[Connection] Dispatchar till on_recv_first_message()");
                proto.on_recv_first_message(&mut msg);
                conn_lock.received_first = true;
            } else {
                println!("[Connection] Dispatchar till on_recv_message()");
                proto.on_recv_message(&mut msg);
            }
        }
    }
}
    async fn write_loop(mut writer: OwnedWriteHalf, mut rx: mpsc::Receiver<Arc<OutputMessage>>) -> Result<()> {
        while let Some(msg) = rx.recv().await {
            let buf = msg.get_output_buffer();
            println!("[Connection] Writing {} bytes to client", buf.len());
            writer.write_all(buf).await?;
        }
        Ok(())
    }
}
