
use std::sync::Arc;
use tokio::sync::{mpsc, Notify};
use tokio::task;

type TaskFn = Box<dyn FnOnce() + Send + 'static>;

pub struct Task {
    func: Option<TaskFn>,
}

impl Task {
    pub fn new(f: TaskFn) -> Self {
        Self { func: Some(f) }
    }

    pub fn run(mut self) {
        if let Some(f) = self.func.take() {
            f();
        }
    }
}

/// Dispatcher motsvarar g_dispatcher i TFS
#[derive(Clone)]
pub struct Dispatcher {
    tx: mpsc::UnboundedSender<Task>,
    notify: Arc<Notify>,
}

impl Dispatcher {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Task>();
        let notify = Arc::new(Notify::new());
        let notify_loop = notify.clone();

        // starta bakgrundsloop
        task::spawn(async move {
            loop {
                if let Some(task) = rx.recv().await {
                    task.run();
                } else {
                    break; // channel stängd
                }
            }
            notify_loop.notify_waiters();
        });

        Self { tx, notify }
    }

    pub fn add_task<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let _ = self.tx.send(Task::new(Box::new(f)));
    }

    pub async fn shutdown(&self) {
        drop(&self.tx);
        self.notify.notified().await;
    }
}
