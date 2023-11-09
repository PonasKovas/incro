use evdev::{uinput::VirtualDevice, InputEvent};
use std::sync::Mutex;

/// Simulates events
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EventSender {
    ptr: *const (),
    send: unsafe extern "C" fn(ptr: *const (), events: *const InputEvent, len: usize),
}

impl EventSender {
    pub const fn new(virtual_device: *const Mutex<VirtualDevice>) -> Self {
        unsafe extern "C" fn send(ptr: *const (), events: *const InputEvent, len: usize) {
            let virtual_device = (ptr as *const Mutex<VirtualDevice>).as_ref().unwrap();
            let events = std::slice::from_raw_parts(events, len);

            virtual_device.lock().unwrap().emit(events).unwrap();
        }

        Self {
            ptr: virtual_device as *const (),
            send,
        }
    }
    pub fn send(&self, events: &[InputEvent]) {
        unsafe { (self.send)(self.ptr, events.as_ptr(), events.len()) }
    }
}

unsafe impl Send for EventSender {}
unsafe impl Sync for EventSender {}
