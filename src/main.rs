mod common;
mod net;
mod protocols;
mod services;
mod scheduler;
mod tasks;
mod db;

use std::path::PathBuf;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::oneshot;

// local crates
use crate::common::{logger, Result, Config};
use crate::common::init_logger as logger;
use crate::common::tracing::info;
use crate::services::ServiceManager;


use tasks::Dispatcher;
use scheduler::Scheduler;

static LOADER_SIGNAL: Lazy<(tokio::sync::Mutex<bool>, tokio::sync::Notify)> =
    Lazy::new(|| (tokio::sync::Mutex::new(false), tokio::sync::Notify::new()));


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    info!("Starting Rusted-Server…");

    // ServiceManager – körs när servern är redo
    let service_manager = Arc::new(ServiceManager::new());

    // Starta dispatcher/scheduler (bakgrundstasks)
    let dispatcher = Arc::new(Dispatcher::new());
    let scheduler = Arc::new(Scheduler::new(dispatcher.clone()));

        // Kör main_loader i bakgrund (som g_dispatcher.addTask(mainLoader))
    let (tx, rx) = oneshot::channel();
    let sm_clone = Arc::clone(&service_manager);
    let disp_clone = dispatcher.clone();
    let sched_clone = scheduler.clone();
    tokio::spawn(async move {
        if let Err(e) = main_loader(sm_clone, disp_clone, sched_clone).await {
            eprintln!("Startup failed: {}", e);
            let _ = tx.send(false);
            return;
        }
        let _ = tx.send(true);
    });

    // Vänta på loadern
    let ok = rx.await.unwrap_or(false);

    if ok && service_manager.is_running() {
        println!(">> Server Online!\n");
        service_manager.run().await?;
    } else {
        println!(">> No services running. The server is NOT online.");
        scheduler.shutdown().await;
        dispatcher.shutdown().await;
    }

    Ok(())
}

fn print_server_version() -> anyhow::Result<()> {
    let server_name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let build_date = std::env::var("BUILD_DATE").unwrap_or_else(|_| "unknown".into());
    let target = std::env::var("BUILD_TARGET").unwrap_or_else(|_| "unknown".into());
    let git_hash = option_env!("GIT_HASH").unwrap_or("unknown");
    let rustc_version = rustc_version_runtime::version_meta().short_version_string;
    let lua = mlua::Lua::new();
    let lua_version: String = lua.load("return _VERSION").eval()?;

    println!("{} - Version {}", server_name, version);
    println!("Git commit: {}", git_hash);
    println!("Compiled with Rust {}", rustc_version);
    println!("Compiled on {} for platform {}", build_date, target);
    println!("Linked with {}", lua_version);
    println!();
    println!("A server developed by Wickedviruz & contributors");
    println!("Visit our repo for updates: https://github.com/Wickedviruz/Rusted-Server");
    println!();

    Ok(())
}


async fn main_loader(
    manager: Arc<ServiceManager>,
    dispatcher: Arc<Dispatcher>,
    scheduler: Arc<Scheduler>,
) -> anyhow::Result<()> {

    // 1. Logging
    logger::init();
    print_server_version()?;
    info!("Starting Rusted-Server…");

    // 2. Config
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        //.join("..")
        //.join("..")
        .join("config.lua")
        .canonicalize()?; // ger absolut path
    println!(">> Loading config");
    //let content = std::fs::read_to_string(&config_path)?;
    let config = common::Config::load(config_path.to_str().unwrap())?;

    // 3. Init crypto (RSA/XTEA)
    let pem_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        //.join("..")
        //.join("..")
        .join("key.pem")
        .canonicalize()?;

    println!(">> Loading RSA key");
    net::rsa::load_pem_file(&pem_path)?;

    // 4. DB
    println!(">> Establishing database connection...");

    if !Database::instance().connect().await.is_ok() {
        eprintln!("Failed to connect to database.");
        return Ok(());
    }

    println!(" MySQL {}", Database::get_client_version());

    println!(">> Running database manager");

    if !DatabaseManager::is_database_setup(&config).await? {
        eprintln!("The database you specified is empty, please import schema.sql.");
        return Ok(());
    }

    DatabaseManager::update_database(&config).await?;

    if config.optimize_database {
        if !DatabaseManager::optimize_tables(&config).await? {
            println!("> No tables were optimized.");
        }
    }


    // 5. Load game assets
    //rules::vocation::load("data/vocations.xml")?;
    //items::loader::load_all("data/items")?;
    //scripting::script_manager::load_scripts("data/scripts")?;
    //entities::monster::load("data/monsters")?;
    //world::map::load(config.map)?;
    info!("Game data loaded");

    // 6. Setup services
    //let mut service_manager = services::ServiceManager::new();
    //manager.add(config.login_protocol_port, LoginProtocol::new()).await?;

    //service_manager.add(protocols-game::GameProtocol::new(config.game_port));
    //service_manager.add(protocols_status::StatusProtocol::new(config.status_port));

    // 7. Game state
    //world::World::set_state(world::GameState::Normal);

    println!("Server online at {}:{}", config.ip, config.game_protocol_port);

    // 8. Run services (blocking loop)
    //service_manager.run()?;

    Ok(())
}

pub fn startup_error_message(error_str: &str) {
    println!("> ERROR: {}", error_str);
    let (_, cvar) = &*LOADER_SIGNAL;
    cvar.notify_waiters();
}