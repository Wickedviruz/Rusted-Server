use anyhow::Result;
use mysql_async::prelude::*;
use sha1::{Sha1, Digest};
use crate::connection::Database;

pub struct DatabaseManager {
    db: Database,
}

impl DatabaseManager {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Run query without expecting rows
    pub async fn execute(&self, query: &str) -> Result<()> {
        let mut conn = self.db.get_conn().await?;
        conn.query_drop(query).await?;
        Ok(())
    }

    /// Run query and fetch rows
    pub async fn query<T>(&self, query: &str) -> Result<Vec<T>>
    where
        T: mysql_async::prelude::FromRow + Send + 'static,
    {
        let mut conn = self.db.get_conn().await?;
        let result = conn.query(query).await?;
        Ok(result)
    }

    /// Check schema version in DB (som TFS gör)
    pub async fn check_version(&self) -> Result<i32> {
        let rows: Vec<(i32,)> = self.query("SELECT `version` FROM `schema_info`").await?;
        Ok(rows.into_iter().next().map(|(v,)| v).unwrap_or(0))
    }

    /// Placeholder för framtida migrations
    pub async fn migrate(&self) -> Result<()> {
        println!("Running migrations (TODO)...");
        Ok(())
    }

    pub async fn check_account(&self, name: &str, password: &str) -> Result<bool> {
        let mut conn = self.db.get_conn().await?;

        // Hasha lösenord på samma sätt som TFS (sha1)
        let mut hasher = Sha1::new();
        hasher.update(password.as_bytes());
        let hashed = format!("{:x}", hasher.finalize());

        let row: Option<(String,)> = conn
            .exec_first(
                "SELECT password FROM accounts WHERE name = :name",
                params! { "name" => name },
            )
            .await?;

        match row {
            Some((db_pass,)) => {
                println!("[DB] Found account={}, db_pass={}, client_hash={}", name, db_pass, hashed);
                Ok(db_pass == hashed)
            }
            None => {
                println!("[DB] No account found with name={}", name);
                Ok(false)
            }
        }
    }
}
