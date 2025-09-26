use crate::db::{Database, DbResult};
use crate::common::Config;
use anyhow::Result;
use mlua::Lua;
use std::path::Path;

pub struct DatabaseManager;

impl DatabaseManager {
    pub async fn table_exists(table_name: &str, config: &Config) -> Result<bool> {
        let db = Database::instance();
        let query = format!(
            "SELECT `TABLE_NAME` FROM `information_schema`.`tables` \
             WHERE `TABLE_SCHEMA` = '{}' AND `TABLE_NAME` = '{}' LIMIT 1",
            config.database.name, table_name
        );
        Ok(db.store_query(&query).await?.is_some())
    }

    pub async fn is_database_setup(config: &Config) -> Result<bool> {
        let db = Database::instance();
        let query = format!(
            "SELECT `TABLE_NAME` FROM `information_schema`.`tables` \
             WHERE `TABLE_SCHEMA` = '{}'",
            config.database.name
        );
        Ok(db.store_query(&query).await?.is_some())
    }

    pub async fn get_database_version(config: &Config) -> Result<i32> {
        if !Self::table_exists("server_config", config).await? {
            let db = Database::instance();
            db.execute(
                "CREATE TABLE `server_config` \
                 (`config` VARCHAR(50) NOT NULL, \
                  `value` VARCHAR(256) NOT NULL DEFAULT '', \
                  UNIQUE(`config`)) ENGINE=InnoDB",
            )
            .await?;
            db.execute("INSERT INTO `server_config` VALUES ('db_version', 0)")
                .await?;
            return Ok(0);
        }

        let mut version: i32 = 0;
        if Self::get_database_config("db_version", &mut version, config).await? {
            Ok(version)
        } else {
            Ok(-1)
        }
    }

    pub async fn update_database(config: &Config) -> Result<()> {
        let lua = Lua::new();

        // Här skulle du normalt registrera db/result API mot Lua,
        // men för nu håller vi det enkelt.
        // Anta att `data/migrations/{version}.lua` finns.

        let mut version = Self::get_database_version(config).await?;
        loop {
            let path = format!("data/migrations/{}.lua", version);
            if !Path::new(&path).exists() {
                break;
            }

            if let Err(e) = lua.load(&std::fs::read_to_string(&path)?).exec() {
                eprintln!(
                    "[Error - DatabaseManager::updateDatabase - Version: {}] {}",
                    version, e
                );
                break;
            }

            // simulera att Lua-skript returnerar true
            let continue_update = true;
            if !continue_update {
                break;
            }

            version += 1;
            println!("> Database has been updated to version {}.", version);
            Self::register_database_config("db_version", version, config).await?;
        }

        Ok(())
    }

    pub async fn optimize_tables(config: &Config) -> Result<bool> {
        let db = Database::instance();
        let query = format!(
            "SELECT `TABLE_NAME` FROM `information_schema`.`TABLES` \
             WHERE `TABLE_SCHEMA` = '{}' AND `DATA_FREE` > 0",
            config.database.name
        );

        if let Some(mut result) = db.store_query(&query).await? {
            loop {
                let table_name = result.get_string("TABLE_NAME").unwrap_or_default();
                print!("> Optimizing table {}...", table_name);

                let ok = db
                    .execute(&format!("OPTIMIZE TABLE `{}`", table_name))
                    .await
                    .is_ok();

                if ok {
                    println!(" [success]");
                } else {
                    println!(" [failed]");
                }

                if !result.next() {
                    break;
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_database_config(
        config_key: &str,
        out_value: &mut i32,
        config: &Config,
    ) -> Result<bool> {
        let db = Database::instance();
        let query = format!(
            "SELECT `value` FROM `server_config` WHERE `config` = '{}'",
            config_key
        );

        if let Some(result) = db.store_query(&query).await? {
            *out_value = result.get_number("value").unwrap_or(0);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn register_database_config(
        config_key: &str,
        value: i32,
        config: &Config,
    ) -> Result<()> {
        let mut tmp: i32 = 0;
        if !Self::get_database_config(config_key, &mut tmp, config).await? {
            let db = Database::instance();
            db.execute(&format!(
                "INSERT INTO `server_config` VALUES ('{}', '{}')",
                config_key, value
            ))
            .await?;
        } else {
            let db = Database::instance();
            db.execute(&format!(
                "UPDATE `server_config` SET `value` = '{}' WHERE `config` = '{}'",
                value, config_key
            ))
            .await?;
        }
        Ok(())
    }
}
