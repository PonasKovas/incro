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
    task: Option<TaskHandle>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
            task: None,
        }
    }
}

async fn on_event(methods: Methods, state_mutex: &'static Mutex<State>, event: InputEvent) -> bool {
    let mut state = state_mutex.lock().await;

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

                state.task = Some(methods.spawn(async move {
                    // release ctrl if pressed
                    if state_mutex.lock().await.ctrl_pressed {
                        methods.emit(&[InputEvent::new(
                            EventType::KEY,
                            Key::KEY_LEFTCTRL.code(),
                            0,
                        )]);
                    }

                    loop {
                        // select block
                        methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_2.code(), 1)]);
                        methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_2.code(), 0)]);

                        // place
                        methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)]);
                        methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);

                        methods.sleep(0, 20_000_000).await;

                        // select main weapon
                        methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_1.code(), 1)]);
                        methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_1.code(), 0)]);

                        methods.sleep(0, 100_000_000).await;
                    }
                }));
            } else if event.value() == 0 {
                if state.task.take().is_some() {
                    // release
                    methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_2.code(), 0)]);
                    methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);

                    // select main weapon
                    methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_1.code(), 1)]);
                    methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_1.code(), 0)]);

                    // press ctrl if was originally pressed
                    if state.ctrl_pressed {
                        methods.emit(&[InputEvent::new(
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
