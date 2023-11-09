incro!(State, on_event);

use incro::incro;
use incro::{
    evdev::{EventType, InputEvent, InputEventKind, Key},
    tokio::sync::Mutex,
    Methods, TaskHandle,
};

struct State {
    shift_pressed: bool,
    ctrl_pressed: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
        }
    }
}

async fn on_event(methods: Methods, state: &Mutex<State>, event: InputEvent) -> bool {
    let mut state = state.lock().await;

    match event.kind() {
        InputEventKind::Key(Key::KEY_LEFTSHIFT) => {
            if event.value() == 1 {
                state.shift_pressed = true;
            } else if event.value() == 0 {
                state.shift_pressed = false;
            }
            false
        }
        InputEventKind::Key(Key::KEY_LEFTCTRL) => {
            if event.value() == 1 {
                state.ctrl_pressed = true;
            } else if event.value() == 0 {
                state.ctrl_pressed = false;
            }
            false
        }
        InputEventKind::Key(
            Key::KEY_1
            | Key::KEY_2
            | Key::KEY_3
            | Key::KEY_4
            | Key::KEY_5
            | Key::KEY_6
            | Key::KEY_7
            | Key::KEY_8
            | Key::KEY_9,
        ) => {
            if event.value() == 1 {
                if state.shift_pressed {
                    // release shift
                    methods.emit(&[InputEvent::new(
                        EventType::KEY,
                        Key::KEY_LEFTSHIFT.code(),
                        0,
                    )]);
                }
                if state.ctrl_pressed {
                    // release ctrl
                    methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_LEFTCTRL.code(), 0)]);
                }

                false
            } else if event.value() == 0 {
                if state.shift_pressed {
                    // press shift
                    methods.emit(&[InputEvent::new(
                        EventType::KEY,
                        Key::KEY_LEFTSHIFT.code(),
                        1,
                    )]);
                }
                if state.ctrl_pressed {
                    // press ctrl
                    methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_LEFTCTRL.code(), 1)]);
                }

                false
            } else {
                false
            }
        }
        _ => false,
    }
}
