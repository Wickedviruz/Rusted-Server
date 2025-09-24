//! Timer / scheduler / event queue

// public API
pub struct TimerHandle {
    // ...
}

pub fn schedule_delay<F>(_delay_ms: u64, _task: F)
where
    F: FnOnce() + Send + 'static,
{
    unimplemented!()
}
