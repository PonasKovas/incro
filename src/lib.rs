pub const INCRO_VERSION: u64 = 1; // bumped on each incompatible ABI change

pub use evdev;
pub use scopeguard;

use evdev::InputEvent;
use event_sender::EventSender;
use std::ops::ControlFlow;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
    Arc,
};

mod event_sender;
mod r#macro;

/// Incro handle, contains all available methods for the callback
#[derive(Clone)]
#[repr(C)]
pub struct Incro {
    event_sender: EventSender,
    parent_thread: Option<ThreadHandle>,
}

/// Handle of an Incro-style spawned thread
#[derive(Clone)]
pub struct ThreadHandle {
    should_stop: Arc<AtomicBool>,
}

impl Incro {
    #[doc(hidden)]
    pub fn new(event_sender: Sender<Vec<InputEvent>>) -> Self {
        Self {
            event_sender: EventSender::new(event_sender),
            parent_thread: None,
        }
    }
    #[must_use]
    /// Emits fake events
    pub fn emit(&self, events: &[InputEvent]) -> ControlFlow<()> {
        if let Some(parent_thread) = &self.parent_thread {
            if parent_thread.should_stop.load(Ordering::SeqCst) {
                return ControlFlow::Break(());
            }
        }
        self.force_emit(events);

        ControlFlow::Continue(())
    }
    /// Emits fake events even if thread of `Incro` must be stopped
    pub fn force_emit(&self, events: &[InputEvent]) {
        self.event_sender.send(events);
    }
    /// Spawns a thread
    pub fn thread<F: FnOnce(Incro) -> ControlFlow<()> + Send + 'static>(
        &self,
        f: F,
    ) -> ThreadHandle {
        let mut incro = self.clone();

        let atomic = Arc::new(AtomicBool::new(false));
        let handle = ThreadHandle {
            should_stop: Arc::clone(&atomic),
        };

        incro.parent_thread = Some(handle.clone());
        std::thread::spawn(move || f(incro));

        handle
    }
}

impl ThreadHandle {
    pub fn detach(self) {
        std::mem::forget(self);
    }
    pub fn stop(self) {}
}

impl Drop for ThreadHandle {
    fn drop(&mut self) {
        self.should_stop
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}
