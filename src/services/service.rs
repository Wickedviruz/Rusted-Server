use async_trait::async_trait;
use tokio::net::TcpStream;
use anyhow::Result;

/// Motsvarar ServiceBase
#[async_trait]
pub trait ServiceBase: Send + Sync {
    fn is_single_socket(&self) -> bool { false }
    fn is_checksummed(&self) -> bool { false }
    fn protocol_identifier(&self) -> u8 { 0 }
    fn protocol_name(&self) -> &'static str;

    async fn make_protocol(&self, stream: TcpStream) -> Result<()>;
}

/// Motsvarar Service<ProtocolType>
pub struct Service<P: Protocol> {
    _marker: std::marker::PhantomData<P>,
}

impl<P: Protocol> Service<P> {
    pub fn new() -> Self {
        Self { _marker: std::marker::PhantomData }
    }
}

#[async_trait::async_trait]
impl<P: Protocol> ServiceBase for Service<P> {
    fn is_single_socket(&self) -> bool { P::SERVER_SENDS_FIRST }
    fn is_checksummed(&self) -> bool { P::USE_CHECKSUM }
    fn protocol_identifier(&self) -> u8 { P::PROTOCOL_IDENTIFIER }
    fn protocol_name(&self) -> &'static str { P::protocol_name() }

    async fn make_protocol(&self, stream: TcpStream) -> Result<()> {
        P::handle(stream).await
    }
}

/// Trait för själva protokollen (Game, Login, Status osv.)
#[async_trait]
pub trait Protocol: Send + Sync + 'static {
    const SERVER_SENDS_FIRST: bool = false;
    const USE_CHECKSUM: bool = false;
    const PROTOCOL_IDENTIFIER: u8 = 0;

    fn protocol_name() -> &'static str;
    async fn handle(stream: TcpStream) -> Result<()>;
}
