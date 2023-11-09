incro!(State, on_event);

use incro::incro;
use incro::{
    evdev::{EventType, InputEvent, InputEventKind, Key},
    tokio::sync::Mutex,
    Methods, TaskHandle,
};

struct State {
    task: Option<TaskHandle>,
}
impl Default for State {
    fn default() -> Self {
        Self { task: None }
    }
}

const CPS: u32 = 15;

async fn on_event(methods: Methods, state: &Mutex<State>, event: InputEvent) -> bool {
    let mut state = state.lock().await;

    match event.kind() {
        InputEventKind::Key(Key::BTN_MIDDLE) => {
            if event.value() == 1 {
                state.task = Some(methods.spawn(async move {
                    loop {
                        // Click LEFT
                        methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_LEFT.code(), 1)]);
                        methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_LEFT.code(), 0)]);

                        methods.sleep(0, 1_000_000_000 / CPS).await;
                    }
                }));
            } else if event.value() == 0 {
                state.task = None;

                methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_LEFT.code(), 0)]);
            }
            true
        }

        _ => false,
    }
}
