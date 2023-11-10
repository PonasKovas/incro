incro::incro!(State, on_event);

use incro::{
    evdev::{EventType, InputEvent, InputEventKind, Key},
    Incro, ThreadHandle,
};
use std::{sync::Mutex, thread::sleep, time::Duration};
struct State {
    shift_pressed: bool,
    ctrl_pressed: bool,
    thread: Option<ThreadHandle>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
            thread: None,
        }
    }
}

fn on_event(incro: Incro, state_mutex: &'static Mutex<State>, event: InputEvent) -> bool {
    let mut state = state_mutex.lock().unwrap();

    match event.kind() {
        InputEventKind::Key(Key::KEY_LEFTSHIFT) => {
            if event.value() == 1 {
                state.shift_pressed = true;
            } else if event.value() == 0 {
                state.shift_pressed = false;
            }
        }
        InputEventKind::Key(Key::KEY_LEFTCTRL) => {
            if event.value() == 1 {
                state.ctrl_pressed = true;
            } else if event.value() == 0 {
                state.ctrl_pressed = false;
            }
        }
        InputEventKind::Key(Key::KEY_LEFTALT) => {
            if event.value() == 1 {
                // Must not be shifting
                if state.shift_pressed {
                    return false;
                }

                state.thread = Some(incro.thread(|incro| {
                    // release ctrl if pressed
                    if state_mutex.lock().unwrap().ctrl_pressed {
                        incro.emit(&[InputEvent::new(
                            EventType::KEY,
                            Key::KEY_LEFTCTRL.code(),
                            0,
                        )])?;
                    }

                    loop {
                        // select block
                        incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_2.code(), 1)])?;
                        incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_2.code(), 0)])?;

                        // place
                        incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)])?;
                        incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)])?;

                        sleep(Duration::from_millis(20));

                        // select main weapon
                        incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_1.code(), 1)])?;
                        incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_1.code(), 0)])?;

                        sleep(Duration::from_millis(100));
                    }
                }));
            } else if event.value() == 0 {
                if state.thread.take().is_some() {
                    // release
                    let _ = incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_2.code(), 0)]);
                    let _ =
                        incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);

                    // select main weapon
                    let _ = incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_1.code(), 1)]);
                    let _ = incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_1.code(), 0)]);

                    // press ctrl if was originally pressed
                    if state.ctrl_pressed {
                        let _ = incro.emit(&[InputEvent::new(
                            EventType::KEY,
                            Key::KEY_LEFTCTRL.code(),
                            1,
                        )]);
                    }
                }
            }
        }
        _ => {}
    }

    false
}
