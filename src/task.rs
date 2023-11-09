use std::future::Future;

use async_ffi::{FfiFuture, FutureExt};
use tokio::{spawn, task::JoinHandle};

/// Allows to spawn tasks
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TaskSpawner {
    spawn: unsafe extern "C" fn(task: FfiFuture<()>) -> TaskHandle,
}

/// Allows to cancel (abort) a task. Either drop it or use `abort()` method.
#[repr(C)]
pub struct TaskHandle {
    handle: *mut (),
    abort: unsafe extern "C" fn(handle: *mut ()),
}

impl TaskSpawner {
    pub const fn new() -> Self {
        unsafe extern "C" fn spawn_fn(task: FfiFuture<()>) -> TaskHandle {
            TaskHandle::new(spawn(async move { task.await }))
        }
        Self { spawn: spawn_fn }
    }
    pub fn spawn<F: Future<Output = ()> + Send + 'static>(&self, task: F) -> TaskHandle {
        unsafe { (self.spawn)(task.into_ffi()) }
    }
}

unsafe impl Send for TaskSpawner {}
unsafe impl Sync for TaskSpawner {}

impl TaskHandle {
    fn new(handle: JoinHandle<()>) -> Self {
        unsafe extern "C" fn abort(handle: *mut ()) {
            let handle = Box::from_raw(handle as *mut JoinHandle<()>);
            handle.abort();
            drop(handle);
        }
        Self {
            handle: Box::into_raw(Box::new(handle)) as *mut (),
            abort,
        }
    }
    pub fn abort(self) {
        drop(self)
    }
}

impl Drop for TaskHandle {
    fn drop(&mut self) {
        unsafe {
            (self.abort)(self.handle);
        }
    }
}

unsafe impl Send for TaskHandle {}
unsafe impl Sync for TaskHandle {}
