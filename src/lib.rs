pub const INCRO_VERSION: u64 = 0; // bumped on each incompatible ABI change

pub use async_ffi;
pub use evdev;
pub use task::TaskHandle;
pub use tokio;

use async_ffi::FfiFuture;
use evdev::{uinput::VirtualDevice, InputEvent};
use event_sender::EventSender;
use std::{future::Future, sync::Mutex};
use task::TaskSpawner;

mod event_sender;
mod r#macro;
mod sleep;
mod task;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Methods {
    event_sender: EventSender,
    task_spawner: TaskSpawner,
    sleep: extern "C" fn(secs: u64, nanos: u32) -> FfiFuture<()>,
    precise_sleep: extern "C" fn(secs: u64, nanos: u32) -> FfiFuture<()>,
}

impl Methods {
    pub fn new(virtual_device: *const Mutex<VirtualDevice>) -> Self {
        Self {
            event_sender: EventSender::new(virtual_device),
            task_spawner: TaskSpawner::new(),
            sleep: sleep::sleep,
            precise_sleep: sleep::precise_sleep,
        }
    }
    pub fn emit(&self, events: &[InputEvent]) {
        self.event_sender.send(events);
    }
    pub fn spawn<F: Future<Output = ()> + Send + 'static>(&self, future: F) -> TaskHandle {
        self.task_spawner.spawn(future)
    }
    pub async fn sleep(&self, secs: u64, nanos: u32) {
        (self.sleep)(secs, nanos).await
    }
    pub async fn precise_sleep(&self, secs: u64, nanos: u32) {
        (self.precise_sleep)(secs, nanos).await
    }
}
