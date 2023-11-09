incro!(State, on_event);

use incro::incro;
use incro::{
    evdev::{EventType, InputEvent, InputEventKind, Key},
    tokio::sync::Mutex,
    Methods, TaskHandle,
};

struct State {
    task: Option<TaskHandle>,
    shift_pressed: bool,
    s_pressed: bool,
    d_pressed: bool,
    a_pressed: bool,
    make_stairs: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            task: None,
            shift_pressed: false,
            s_pressed: false,
            d_pressed: false,
            a_pressed: false,
            make_stairs: true,
        }
    }
}

async fn on_event(methods: Methods, state_mutex: &'static Mutex<State>, event: InputEvent) -> bool {
    let mut state = state_mutex.lock().await;

    match event.kind() {
        InputEventKind::Key(Key::KEY_KP0) => {
            if event.value() == 0 {
                state.make_stairs = !state.make_stairs;
            }
            false
        }
        InputEventKind::Key(Key::KEY_LEFTSHIFT) => {
            if event.value() == 1 {
                state.shift_pressed = true;
            } else if event.value() == 0 {
                state.shift_pressed = false;
            }
            state.task.is_some() // ignore if bridging
        }
        InputEventKind::Key(Key::KEY_S) => {
            if event.value() == 1 {
                state.s_pressed = true;
            } else if event.value() == 0 {
                state.s_pressed = false;
            }
            state.task.is_some() // ignore if bridging
        }
        InputEventKind::Key(Key::KEY_D) => {
            if event.value() == 1 {
                state.d_pressed = true;
            } else if event.value() == 0 {
                state.d_pressed = false;
            }
            state.task.is_some() // ignore if bridging
        }
        InputEventKind::Key(Key::KEY_A) => {
            if event.value() == 1 {
                state.a_pressed = true;
            } else if event.value() == 0 {
                state.a_pressed = false;
            }
            state.task.is_some() // ignore if bridging
        }
        InputEventKind::Key(Key::KEY_LEFTALT) => {
            if event.value() == 1 {
                // Must be shifting + S
                if !(state.shift_pressed && state.s_pressed) {
                    return false;
                }

                if !state.a_pressed && !state.d_pressed {
                    drop(state);
                    // diagonal bridging
                    state_mutex.lock().await.task =
                        Some(methods.spawn(diagonal_bridging(methods, state_mutex)));
                } else {
                    drop(state);
                    // normal bridging
                    state_mutex.lock().await.task =
                        Some(methods.spawn(normal_bridging(methods, state_mutex)));
                }

                true
            } else if event.value() == 0 {
                if state.task.is_some() {
                    state.task = None;
                    // release mouse button
                    methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);
                    // release space
                    methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 0)]);

                    // either shift or unshift depending on real situation
                    methods.emit(&[InputEvent::new(
                        EventType::KEY,
                        Key::KEY_LEFTSHIFT.code(),
                        if state.shift_pressed { 1 } else { 0 },
                    )]);

                    // press or release S, D and A depending on real situation
                    methods.emit(&[InputEvent::new(
                        EventType::KEY,
                        Key::KEY_S.code(),
                        if state.s_pressed { 1 } else { 0 },
                    )]);
                    methods.emit(&[InputEvent::new(
                        EventType::KEY,
                        Key::KEY_D.code(),
                        if state.d_pressed { 1 } else { 0 },
                    )]);
                    methods.emit(&[InputEvent::new(
                        EventType::KEY,
                        Key::KEY_A.code(),
                        if state.a_pressed { 1 } else { 0 },
                    )]);

                    true
                } else {
                    false
                }
            } else {
                // Repeat event spam, ignore if bridging
                if state.task.is_some() {
                    true
                } else {
                    false
                }
            }
        }
        _ => false,
    }
}

async fn normal_bridging(methods: Methods, state_mutex: &'static Mutex<State>) {
    let mut i = 1;
    loop {
        // click
        methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)]);
        methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);

        if i % 3 == 0 && state_mutex.lock().await.make_stairs {
            // jump and place extra block to make stairs
            methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 1)]);

            // unshift
            methods.emit(&[InputEvent::new(
                EventType::KEY,
                Key::KEY_LEFTSHIFT.code(),
                0,
            )]);

            methods.precise_sleep(0, 60_000_000).await;

            // release space
            methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 0)]);
            // shift
            methods.emit(&[InputEvent::new(
                EventType::KEY,
                Key::KEY_LEFTSHIFT.code(),
                1,
            )]);

            methods.precise_sleep(0, 100_000_000).await;

            // click
            methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)]);
            methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);

            methods.precise_sleep(0, 350_000_000).await;

            // click
            methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)]);
            methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);

            i += 1;
        }

        // unshift
        methods.emit(&[InputEvent::new(
            EventType::KEY,
            Key::KEY_LEFTSHIFT.code(),
            0,
        )]);

        methods.precise_sleep(0, 230_642_344).await;

        // shift
        methods.emit(&[InputEvent::new(
            EventType::KEY,
            Key::KEY_LEFTSHIFT.code(),
            1,
        )]);

        methods.precise_sleep(0, 100_000_000).await;

        i += 1;
    }
}

async fn diagonal_bridging(methods: Methods, state_mutex: &'static Mutex<State>) {
    let mut i = 1;
    loop {
        // click
        methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)]);
        methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);

        methods.precise_sleep(0, 50_000_000).await;

        // click
        methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)]);
        methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);

        if i % 3 == 0 && state_mutex.lock().await.make_stairs {
            // jump and place extra block to make stairs
            methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 1)]);

            // unshift
            methods.emit(&[InputEvent::new(
                EventType::KEY,
                Key::KEY_LEFTSHIFT.code(),
                0,
            )]);

            methods.precise_sleep(0, 300_000_000).await;

            // release space
            methods.emit(&[InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 0)]);
            // shift
            methods.emit(&[InputEvent::new(
                EventType::KEY,
                Key::KEY_LEFTSHIFT.code(),
                1,
            )]);

            // methods.precise_sleep(0, 100_000_000).await;

            // click
            methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)]);
            methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);

            methods.precise_sleep(0, 250_000_000).await;

            // click
            methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)]);
            methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);

            methods.precise_sleep(0, 50_000_000).await;

            // click
            methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)]);
            methods.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);

            i += 1;
        }

        // unshift
        methods.emit(&[InputEvent::new(
            EventType::KEY,
            Key::KEY_LEFTSHIFT.code(),
            0,
        )]);

        methods.precise_sleep(0, 300_642_344).await;

        // shift
        methods.emit(&[InputEvent::new(
            EventType::KEY,
            Key::KEY_LEFTSHIFT.code(),
            1,
        )]);

        methods.precise_sleep(0, 100_000_000).await;

        i += 1;
    }
}
