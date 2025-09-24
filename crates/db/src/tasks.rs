use anyhow::Result;
use crate::Database;
use tokio::time::{interval, Duration};

pub struct DatabaseTasks {
    db: Database,
}

impl DatabaseTasks {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Startar en bakgrundsuppgift som t.ex. sparar online tid var 5:e minut
    pub async fn run_online_saver(&self) {
        let mut ticker = interval(Duration::from_secs(300));
        loop {
            ticker.tick().await;
            if let Err(e) = self.save_online_time().await {
                eprintln!("Error in save_online_time: {e}");
            }
        }
    }

    async fn save_online_time(&self) -> Result<()> {
        // Placeholder – här körs riktig SQL update
        self.db.execute("UPDATE players SET onlinetime = onlinetime + 300 WHERE online = 1").await?;
        println!("Updated online time for online players.");
        Ok(())
    }
}
