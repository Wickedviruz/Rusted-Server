use tracing_subscriber::fmt;
use tracing_appender::rolling;
use once_cell::sync::OnceCell;

static GUARD: OnceCell<tracing_appender::non_blocking::WorkerGuard> = OnceCell::new();

/// Initiera global logger (stdout + fil)
pub fn init() {
    let file_appender = rolling::daily("logs", "server.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    fmt()
        .with_writer(non_blocking)
        .with_target(false)
        .init();

    // Behåll guard globalt så att loggar flushas korrekt
    GUARD.set(guard).ok();
}
