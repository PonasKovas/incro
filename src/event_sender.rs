use evdev::InputEvent;
use std::sync::mpsc::Sender;

/// Basically FFI safe version of mpsc channel Sender<InputEvent>
#[repr(C)]
pub(crate) struct EventSender {
    channel: *const (), // raw Box<Sender<InputEvent>>
    send: unsafe extern "C" fn(channel: *const (), events: *const InputEvent, events_n: usize),
    clone: unsafe extern "C" fn(channel: *const ()) -> EventSender,
    drop: unsafe extern "C" fn(channel: *const ()),
}

impl EventSender {
    pub(crate) fn new(channel: Sender<Vec<InputEvent>>) -> Self {
        let channel = Box::into_raw(Box::new(channel)) as *const ();

        unsafe extern "C" fn send(channel: *const (), events: *const InputEvent, event_n: usize) {
            (channel as *const Sender<Vec<InputEvent>>)
                .as_ref()
                .unwrap()
                .send(unsafe { std::slice::from_raw_parts(events, event_n).to_vec() })
                .expect("Event sender channel dead");
        }

        unsafe extern "C" fn clone(channel: *const ()) -> EventSender {
            let sender = (channel as *const Sender<Vec<InputEvent>>)
                .as_ref()
                .unwrap()
                .clone();

            EventSender::new(sender)
        }

        unsafe extern "C" fn drop(channel: *const ()) {
            std::mem::drop(Box::from_raw(channel as *mut Sender<Vec<InputEvent>>));
        }

        Self {
            channel,
            send,
            clone,
            drop,
        }
    }
    pub(crate) fn send(&self, event: &[InputEvent]) {
        unsafe { (self.send)(self.channel, event.as_ptr(), event.len()) }
    }
}

impl Clone for EventSender {
    fn clone(&self) -> Self {
        unsafe { (self.clone)(self.channel) }
    }
}

impl Drop for EventSender {
    fn drop(&mut self) {
        unsafe { (self.drop)(self.channel) }
    }
}

unsafe impl Send for EventSender {}
unsafe impl Sync for EventSender {}
