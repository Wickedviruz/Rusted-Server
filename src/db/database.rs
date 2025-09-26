use anyhow::{anyhow, Result};
use mysql_async::{prelude::Queryable, Pool, Row};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// =======================
/// Database (Singleton)
/// =======================
#[derive(Clone)]
pub struct Database {
    pool: Pool,
    max_packet_size: Arc<Mutex<u64>>,
}

static INSTANCE: OnceCell<Database> = OnceCell::new();

impl Database {
    /// Initiera singletonen utifrån en MySQL-URL (tex: mysql://user:pass@host:port/db)
    /// Returnerar &Database (singleton) när den är initierad.
    pub async fn connect_url(url: &str) -> Result<&'static Database> {
        let pool = Pool::new(url);
        // hämtar en connection för att verifiera och läsa variabler
        let mut conn = pool.get_conn().await.map_err(|e| {
            anyhow!("Failed to initialize MySQL pool/conn: {e}")
        })?;

        // Hämta max_allowed_packet
        let row: Option<(String, String)> =
            conn.query_first("SHOW VARIABLES LIKE 'max_allowed_packet'")
                .await
                .map_err(|e| anyhow!("MySQL error: {e}"))?;
        let max = row
            .map(|(_, v)| v.parse::<u64>().unwrap_or(1048576))
            .unwrap_or(1048576);

        let db = Database {
            pool,
            max_packet_size: Arc::new(Mutex::new(max)),
        };
        Ok(INSTANCE.get_or_init(|| db))
    }

    /// Alternativ init: bygg URL av separata parametrar (liknar TFS configfält)
    pub async fn connect_with_params(
        host: &str,
        user: &str,
        pass: &str,
        db: &str,
        port: u16,
        socket: Option<&str>,
    ) -> Result<&'static Database> {
        let url = if let Some(sock) = socket {
            // MySQL sock-stöd via URL är implementation-/platform-beroende,
            // men vi behåller parametern så signaturen speglar TFS.
            format!("mysql://{user}:{pass}@{host}:{port}/{db}?socket={sock}")
        } else {
            format!("mysql://{user}:{pass}@{host}:{port}/{db}")
        };
        Self::connect_url(&url).await
    }

    /// Hämta singleton (måste ha initierats med connect_* först)
    pub fn instance() -> &'static Database {
        INSTANCE.get().expect("Database not connected")
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Kör kommando utan result-set (INSERT/UPDATE/DELETE/DDL).
    pub async fn execute(&self, query: &str) -> Result<()> {
        let mut conn = self.pool.get_conn().await?;
        // TFS loopar på reconnect/timeout; mysql_async hanterar mycket av detta i poolen.
        conn.query_drop(query).await?;
        Ok(())
    }

    /// Kör SELECT och returnera ett DbResult (None om tomt).
    pub async fn store_query(&self, query: &str) -> Result<Option<DbResult>> {
        let mut conn = self.pool.get_conn().await?;
        let rows: Vec<Row> = conn.query(query).await?;
        if rows.is_empty() {
            Ok(None)
        } else {
            Ok(Some(DbResult::new(rows)))
        }
    }

    /// Sista auto_increment ID (best effort).
    pub async fn last_insert_id(&self) -> Result<u64> {
        let mut conn = self.pool.get_conn().await?;
        Ok(conn.last_insert_id().unwrap_or(0))
    }

    /// Server-version (TFS skriver ut MySQL-version efter connect).
    pub async fn server_version(&self) -> String {
        let mut conn = match self.pool.get_conn().await {
            Ok(c) => c,
            Err(_) => return "unknown".into(),
        };
        let row: Option<(String,)> = conn.query_first("SELECT VERSION()").await.ok().flatten();
        row.map(|(v,)| v).unwrap_or_else(|| "unknown".into())
    }

    /// max_allowed_packet som lästes i connect.
    pub async fn max_packet_size(&self) -> u64 {
        *self.max_packet_size.lock().await
    }

    /// Grov escapning (TFS har mysql_real_escape_string & kvoterar med `'`).
    /// Här ger vi ett **enkelt** motsvarande beteende. För seriös kod:
    /// använd parametriserade queries. Denna används bara för att spegla API:t.
    pub fn escape_string(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('\'');
        for ch in s.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '\'' => out.push_str("\\'"),
                '\0' => out.push_str("\\0"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\x08' => out.push_str("\\b"),
                '\t' => out.push_str("\\t"),
                _ => out.push(ch),
            }
        }
        out.push('\'');
        out
    }

    /// Motsvarighet till escapeBlob.
    pub fn escape_blob(&self, bytes: &[u8]) -> String {
        let s = String::from_utf8_lossy(bytes);
        self.escape_string(&s)
    }
}

/// =======================
/// DbResult (SELECT-result)
/// =======================
pub struct DbResult {
    rows: Vec<Row>,
    cursor: usize,
    columns: HashMap<String, usize>,
}

impl DbResult {
    pub fn new(rows: Vec<Row>) -> Self {
        let mut columns = HashMap::new();
        if let Some(first) = rows.first() {
            for (i, col) in first.columns_ref().iter().enumerate() {
                columns.insert(col.name_str().to_string(), i);
            }
        }
        Self {
            rows,
            cursor: 0,
            columns,
        }
    }

    pub fn has_next(&self) -> bool {
        self.cursor < self.rows.len()
    }

    pub fn next(&mut self) -> bool {
        if self.cursor + 1 < self.rows.len() {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    pub fn get_string(&self, col: &str) -> String {
        let idx = match self.columns.get(col) {
            Some(i) => *i,
            None => {
                eprintln!("[Error - DbResult::get_string] Column '{col}' does not exist in result set.");
                return String::new();
            }
        };
        match self.rows.get(self.cursor).and_then(|row| row.get::<String, _>(idx)) {
            Some(s) => s,
            None => String::new(),
        }
    }

    pub fn get_stream(&self, col: &str) -> Option<Vec<u8>> {
        let idx = *self.columns.get(col)?;
        self.rows
            .get(self.cursor)
            .and_then(|row| row.as_ref(idx))
            .map(|val| val.as_bytes().map(|b| b.to_vec()))
            .flatten()
    }

    pub fn get_number<T: std::str::FromStr + Default>(&self, col: &str) -> T {
        let s = self.get_string(col);
        s.parse::<T>().unwrap_or_default()
    }
}

/// =======================
/// DbInsert (bufferar VALUES)
/// =======================
pub struct DbInsert {
    query: String,
    values: Vec<String>,
    // length används i TFS för att respektera max_packet_size; vi kan approximera
    total_len: usize,
}

impl DbInsert {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            values: Vec::new(),
            total_len: query.len(),
        }
    }

    pub async fn add_row(&mut self, row: &str) -> Result<()> {
        let row_len = row.len();
        self.total_len += row_len + 3; // ,(row)
        // Om vi skulle vilja respektera max_packet_size:
        // if self.total_len > Database::instance().max_packet_size().await as usize {
        //     self.execute().await?;
        // }
        self.values.push(format!("({row})"));
        Ok(())
    }

    pub async fn add_row_from(&mut self, row_buf: &mut String) -> Result<()> {
        let r = std::mem::take(row_buf);
        self.add_row(&r).await
    }

    pub async fn execute(&mut self) -> Result<()> {
        if self.values.is_empty() {
            return Ok(());
        }
        let sql = format!("{} {}", self.query, self.values.join(","));
        Database::instance().execute(&sql).await?;
        self.values.clear();
        self.total_len = self.query.len();
        Ok(())
    }
}

/// =======================
/// DbTransaction (RAII)
/// =======================
pub struct DbTransaction {
    active: bool,
}

impl DbTransaction {
    pub async fn begin() -> Result<Self> {
        Database::instance().execute("BEGIN").await?;
        Ok(Self { active: true })
    }

    pub async fn commit(mut self) -> Result<()> {
        Database::instance().execute("COMMIT").await?;
        self.active = false;
        Ok(())
    }

    pub async fn rollback(mut self) -> Result<()> {
        Database::instance().execute("ROLLBACK").await?;
        self.active = false;
        Ok(())
    }
}

impl Drop for DbTransaction {
    fn drop(&mut self) {
        if self.active {
            // best effort rollback
            let _ = futures::executor::block_on(Database::instance().execute("ROLLBACK"));
        }
    }
}
