use anyhow::Result;
use mysql_async::{prelude::*,Pool, Conn, Opts, OptsBuilder};

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn connect(
        host: &str,
        user: &str,
        password: &str,
        database: &str,
    ) -> Result<Self> {
        let opts = OptsBuilder::default()
            .ip_or_hostname(host.to_string())
            .user(Some(user.to_string()))
            .pass(Some(password.to_string()))
            .db_name(Some(database.to_string()));

        let pool = Pool::new(Opts::from(opts));
        Ok(Self { pool })
    }

    pub async fn get_conn(&self) -> Result<Conn> {
        Ok(self.pool.get_conn().await?)
    }

        /// Helper: run a query without expecting results
    pub async fn execute(&self, query: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        conn.query_drop(query).await?;
        Ok(())
    }

    /// Helper: run a query and fetch rows into Vec<T>
    pub async fn query<T>(&self, query: &str) -> Result<Vec<T>>
    where
        T: mysql_async::prelude::FromRow + Send + 'static,
    {
        let mut conn = self.get_conn().await?;
        Ok(conn.query(query).await?)
    }
}
