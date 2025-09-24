
use std::path::PathBuf;
use std::sync::{Condvar, Mutex};
use once_cell::sync::Lazy;
use colored::*;

// local crates
use common::{logger, Result, Config};
use common::tracing::{info, error};
use services;
mod banner;

static LOADER_SIGNAL: Lazy<(Mutex<bool>, Condvar)> = Lazy::new(|| (Mutex::new(false), Condvar::new()));


fn main() -> anyhow::Result<()> {

    // 1. Logging
    logger::init();
    banner::print_banner()?;
    info!("Starting Rusted-Server…");

    // 2. Config
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("config.lua")
        .canonicalize()?; // ger absolut path
    println!(">> Loading config");
    let content = std::fs::read_to_string(&config_path)?;
    let config = common::Config::load(config_path.to_str().unwrap())?;

    // 3. Init crypto (RSA/XTEA)
    let pem_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("key.pem")
        .canonicalize()?;

    println!(">> Loading RSA key");
    net::rsa::load_pem_file(&pem_path)?;

    // 4. DB
    //persistence::database::connect(&config.database)?;
    //persistence::database::migrate()?; // kör migrations
    //info!("Database connected and migrated");

    // 5. Load game assets
    //rules::vocation::load("data/vocations.xml")?;
    //items::loader::load_all("data/items")?;
    //scripting::script_manager::load_scripts("data/scripts")?;
    //entities::monster::load("data/monsters")?;
    //world::map::load(config.map)?;
    info!("Game data loaded");

    // 6. Setup services
    //let mut service_manager = services::ServiceManager::new();
    //service_manager.add(protocols-login::LoginProtocol::new(config.login_port));
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
    println!("{} {}", "> ERROR:".red().bold(), error_str);
    let (lock, cvar) = &*LOADER_SIGNAL;
    let mut started = lock.lock().unwrap();
    *started = true;
    cvar.notify_all();
}
