use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    net::TcpListener,
    sync::Mutex,
};
use anyhow::Result;

use crate::services::service::ServiceBase;

/// Hanterar alla serviceports (game, login, status, osv.)
pub struct ServiceManager {
    acceptors: Arc<Mutex<HashMap<u16, Arc<ServicePort>>>>,
    running: Arc<Mutex<bool>>,
}

pub struct ServicePort {
    port: u16,
    services: Arc<Mutex<Vec<Box<dyn ServiceBase>>>>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            acceptors: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn is_running(&self) -> bool {
        futures::executor::block_on(async {
            let map = self.acceptors.lock().await;
            !map.is_empty()
        })
    }

    pub async fn add<S: ServiceBase + 'static>(&self, port: u16, service: S) -> Result<()> {
        if port == 0 {
            println!(
                "ERROR: No port provided for service {}. Service disabled.",
                service.protocol_name()
            );
            return Ok(());
        }

        let mut acc = self.acceptors.lock().await;
        let sp = acc.entry(port).or_insert_with(|| Arc::new(ServicePort::new(port)));
        sp.add_service(Box::new(service)).await?;
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        {
            let mut runflag = self.running.lock().await;
            *runflag = true;
        }

        // Blockerande loop: kör alla listeners parallellt
        let acceptors = self.acceptors.lock().await.clone();
        for (_, sp) in acceptors {
            let sp_clone = sp.clone();
            tokio::spawn(async move {
                if let Err(e) = sp_clone.run().await {
                    eprintln!("[ServicePort] Error: {e}");
                }
            });
        }

        Ok(())
    }

    pub async fn stop(&self) {
        let mut runflag = self.running.lock().await;
        *runflag = false;
        let mut acc = self.acceptors.lock().await;
        acc.clear();
    }
}

impl ServicePort {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            services: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_service(&self, svc: Box<dyn ServiceBase>) -> Result<()> {
        let mut s = self.services.lock().await;
        s.push(svc);
        Ok(())
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let addr: SocketAddr = format!("0.0.0.0:{}", self.port).parse()?;
        let listener = TcpListener::bind(addr).await?;
        println!("Listening on {}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let services = self.services.clone();
            tokio::spawn(async move {
                let svcs = services.lock().await;
                if let Some(svc) = svcs.first() {
                    if let Err(e) = svc.make_protocol(stream).await {
                        eprintln!("Protocol error: {e}");
                    }
                }
            });
        }
    }
}

