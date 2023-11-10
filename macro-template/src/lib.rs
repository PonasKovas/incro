incro::incro!(State, on_event);

use incro::{
    evdev::{EventType, InputEvent, InputEventKind, Key},
    Incro, ThreadHandle,
};
use std::{sync::Mutex, thread::sleep, time::Duration};

struct State {
    thread: Option<ThreadHandle>,
}
impl Default for State {
    fn default() -> Self {
        Self { thread: None }
    }
}

const CPS: u64 = 15;

fn on_event(incro: Incro, state: &Mutex<State>, event: InputEvent) -> bool {
    let mut state = state.lock().unwrap();

    match event.kind() {
        InputEventKind::Key(Key::BTN_MIDDLE) => {
            if event.value() == 1 {
                incro::scopeguard::defer! {
                    // let _ = incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_LEFT.code(), 0)]);
                };

                state.thread = Some(incro.thread(move |incro| {
                    loop {
                        // Click LEFT
                        // incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_LEFT.code(), 1)])?;
                        // incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_LEFT.code(), 0)])?;

                        sleep(Duration::from_nanos(1_000_000_000 / CPS));
                    }
                }));
            } else if event.value() == 0 {
                state.thread = None;
            }
            true
        }

        _ => false,
    }
}
