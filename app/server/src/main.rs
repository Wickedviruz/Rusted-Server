use anyhow::Result;
use db::Database;
use net::run_login_server;

#[tokio::main]
async fn main() -> Result<()> {
    let db = Database::connect("127.0.0.1", "tfs", "tfs", "tfs").await?;

    let conn = db.get_conn().await?;
    let version = conn.server_version();
    println!("DB connection OK: {:?}", version);

    run_login_server(db).await?;

    core::hello_core();

    Ok(())
}
