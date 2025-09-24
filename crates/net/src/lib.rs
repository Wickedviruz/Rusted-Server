use anyhow::Result;
use tokio::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use db::Database;

pub mod packet;
pub mod types;
pub mod rsa;
pub mod xtea;
pub mod networkmessage;
pub mod consts;
pub mod protocol_login;
pub mod connection;
pub mod outputmessage;
pub mod tools;
pub mod protocol;

use connection::Connection;
use protocol_login::ProtocolLogin;

pub async fn run_login_server(db: Database) -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:7171").await?;
    println!("Login server listening on port 7171...");

    // Bygg path på ett OS-säkert sätt
    let pem_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("key.pem");

    println!("Loading RSA key");
    rsa::load_pem_file(&pem_path)?;

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Client connected: {}", addr);

        let db = db.clone();

        tokio::spawn(async move {
            // Skapa en Connection (returnerar redan Arc<Mutex<Connection>> och startar read/write-loopar)
            let conn_arc = Connection::new(socket);

            // Skapa ProtocolLogin och koppla på connection
            let login_proto = Arc::new(Mutex::new(ProtocolLogin::new(
                Arc::downgrade(&conn_arc),
                db.clone(),
            )));

            conn_arc.lock().await.set_protocol(login_proto);
        });
    }
}
