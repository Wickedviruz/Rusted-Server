use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time;
use std::collections::HashMap;

use crate::tasks::Dispatcher;

#[derive(Clone)]
pub struct Scheduler {
    dispatcher: Arc<Dispatcher>, 
    events: Arc<Mutex<HashMap<u32, ()>>>,
    last_id: Arc<Mutex<u32>>,
}

impl Scheduler {
    pub fn new(dispatcher: Arc<Dispatcher>) -> Self {
        Self {
            dispatcher,
            events: Arc::new(Mutex::new(HashMap::new())),
            last_id: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn add_event<F>(&self, delay_ms: u64, f: F) -> u32
    where
        F: FnOnce() + Send + 'static,
    {
        let mut id_lock = self.last_id.lock().await;
        *id_lock += 1;
        let id = *id_lock;
        drop(id_lock);

        {
            let mut ev = self.events.lock().await;
            ev.insert(id, ());
        }

        let dispatcher = self.dispatcher.clone();
        let events = self.events.clone();

        task::spawn(async move {
            time::sleep(Duration::from_millis(delay_ms)).await;

            let mut ev = events.lock().await;
            if ev.remove(&id).is_some() {
                dispatcher.add_task(f);
            }
        });

        id
    }

    pub async fn stop_event(&self, id: u32) {
        let mut ev = self.events.lock().await;
        ev.remove(&id);
    }

    pub async fn shutdown(&self) {
        let mut ev = self.events.lock().await;
        ev.clear();
    }
}
