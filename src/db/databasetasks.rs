use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Notify};
use crate::db::{Database, DbResult};
use crate::tasks::Dispatcher;
use anyhow::Result;

type DbCallback = Box<dyn FnOnce(Option<DbResult>, bool) + Send + 'static>;

pub struct DatabaseTask {
    pub query: String,
    pub callback: Option<DbCallback>,
    pub store: bool,
}

pub struct DatabaseTasks {
    sender: mpsc::Sender<DatabaseTask>,
    shutdown_notify: Arc<Notify>,
    dispatcher: Arc<Dispatcher>,
}

impl DatabaseTasks {
    pub fn new(dispatcher: Arc<Dispatcher>) -> Self {
        let (tx, rx) = mpsc::channel::<DatabaseTask>(100);
        let shutdown_notify = Arc::new(Notify::new());

        let worker = Worker {
            rx,
            dispatcher: dispatcher.clone(),
            shutdown_notify: shutdown_notify.clone(),
        };
        tokio::spawn(worker.run());

        Self {
            sender: tx,
            shutdown_notify,
            dispatcher,
        }
    }

    pub async fn add_task<F>(&self, query: String, callback: Option<F>, store: bool)
    where
        F: FnOnce(Option<DbResult>, bool) + Send + 'static,
    {
        let task = DatabaseTask {
            query,
            callback: callback.map(|cb| Box::new(cb) as DbCallback),
            store,
        };
        let _ = self.sender.send(task).await;
    }

    pub async fn flush(&self) {
        // här kan vi vänta på att kön töms
        // enklast: polla tills kön är tom
        while !self.sender.is_closed() {
            // break när inga fler tasks väntar
            // (mer avancerat: ha räknare på antal pågående tasks)
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    pub async fn shutdown(&self) {
        // Stäng kön → worker avslutas när den tömmer
        self.sender.close_channel();
        self.shutdown_notify.notify_waiters();
    }
}

struct Worker {
    rx: mpsc::Receiver<DatabaseTask>,
    dispatcher: Arc<Dispatcher>,
    shutdown_notify: Arc<Notify>,
}

impl Worker {
    async fn run(mut self) {
        let db = Database::instance();
        if let Err(e) = db.connect().await {
            eprintln!("Failed to connect to database: {}", e);
            return;
        }

        println!(" MySQL {}", Database::get_client_version());

        while let Some(task) = self.rx.recv().await {
            let (result, success) = if task.store {
                match db.store_query(&task.query).await {
                    Ok(Some(res)) => (Some(res), true),
                    Ok(None) => (None, true),
                    Err(_) => (None, false),
                }
            } else {
                let ok = db.execute(&task.query).await.is_ok();
                (None, ok)
            };

            if let Some(cb) = task.callback {
                let disp = self.dispatcher.clone();
                disp.add_task(move || cb(result, success));
            }
        }

        self.shutdown_notify.notified().await;
    }
}
