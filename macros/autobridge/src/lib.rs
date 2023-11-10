incro::incro!(State, on_event);

use incro::{
    evdev::{EventType, InputEvent, InputEventKind, Key},
    Incro, ThreadHandle,
};
use std::{ops::ControlFlow, sync::Mutex, thread::sleep, time::Duration};

struct State {
    thread: Option<ThreadHandle>,
    shift_pressed: bool,
    s_pressed: bool,
    d_pressed: bool,
    a_pressed: bool,
    make_stairs: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            thread: None,
            shift_pressed: false,
            s_pressed: false,
            d_pressed: false,
            a_pressed: false,
            make_stairs: true,
        }
    }
}

fn on_event(incro: Incro, state_mutex: &'static Mutex<State>, event: InputEvent) -> bool {
    let mut state = state_mutex.lock().unwrap();

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
            state.thread.is_some() // ignore if bridging
        }
        InputEventKind::Key(Key::KEY_S) => {
            if event.value() == 1 {
                state.s_pressed = true;
            } else if event.value() == 0 {
                state.s_pressed = false;
            }
            state.thread.is_some() // ignore if bridging
        }
        InputEventKind::Key(Key::KEY_D) => {
            if event.value() == 1 {
                state.d_pressed = true;
            } else if event.value() == 0 {
                state.d_pressed = false;
            }
            state.thread.is_some() // ignore if bridging
        }
        InputEventKind::Key(Key::KEY_A) => {
            if event.value() == 1 {
                state.a_pressed = true;
            } else if event.value() == 0 {
                state.a_pressed = false;
            }
            state.thread.is_some() // ignore if bridging
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
                    state_mutex.lock().unwrap().thread =
                        Some(incro.thread(move |incro| diagonal_bridging(incro, state_mutex)));
                } else {
                    drop(state);
                    // normal bridging
                    state_mutex.lock().unwrap().thread =
                        Some(incro.thread(move |incro| normal_bridging(incro, state_mutex)));
                }

                true
            } else if event.value() == 0 {
                if state.thread.is_some() {
                    state.thread = None;
                    // release mouse button
                    let _ =
                        incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)]);
                    // release space
                    let _ =
                        incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 0)]);

                    // either shift or unshift depending on real situation
                    let _ = incro.emit(&[InputEvent::new(
                        EventType::KEY,
                        Key::KEY_LEFTSHIFT.code(),
                        if state.shift_pressed { 1 } else { 0 },
                    )]);

                    // press or release S, D and A depending on real situation
                    let _ = incro.emit(&[InputEvent::new(
                        EventType::KEY,
                        Key::KEY_S.code(),
                        if state.s_pressed { 1 } else { 0 },
                    )]);
                    let _ = incro.emit(&[InputEvent::new(
                        EventType::KEY,
                        Key::KEY_D.code(),
                        if state.d_pressed { 1 } else { 0 },
                    )]);
                    let _ = incro.emit(&[InputEvent::new(
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
                if state.thread.is_some() {
                    true
                } else {
                    false
                }
            }
        }
        _ => false,
    }
}

fn normal_bridging(incro: Incro, state_mutex: &'static Mutex<State>) -> ControlFlow<()> {
    let mut i = 1;
    loop {
        // click
        incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)])?;
        incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)])?;

        if i % 3 == 0 && state_mutex.lock().unwrap().make_stairs {
            // jump and place extra block to make stairs
            incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 1)])?;

            // unshift
            incro.emit(&[InputEvent::new(
                EventType::KEY,
                Key::KEY_LEFTSHIFT.code(),
                0,
            )])?;

            sleep(Duration::from_millis(60));

            // release space
            incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 0)])?;
            // shift
            incro.emit(&[InputEvent::new(
                EventType::KEY,
                Key::KEY_LEFTSHIFT.code(),
                1,
            )])?;

            sleep(Duration::from_millis(100));

            // click
            incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)])?;
            incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)])?;

            sleep(Duration::from_millis(350));

            // click
            incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)])?;
            incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)])?;

            i += 1;
        }

        // unshift
        incro.emit(&[InputEvent::new(
            EventType::KEY,
            Key::KEY_LEFTSHIFT.code(),
            0,
        )])?;

        sleep(Duration::from_millis(230));

        // shift
        incro.emit(&[InputEvent::new(
            EventType::KEY,
            Key::KEY_LEFTSHIFT.code(),
            1,
        )])?;

        sleep(Duration::from_millis(100));

        i += 1;
    }
}

fn diagonal_bridging(incro: Incro, state_mutex: &'static Mutex<State>) -> ControlFlow<()> {
    let mut i = 1;
    loop {
        // click
        incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)])?;
        incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)])?;

        sleep(Duration::from_millis(50));

        // click
        incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)])?;
        incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)])?;

        if i % 3 == 0 && state_mutex.lock().unwrap().make_stairs {
            // jump and place extra block to make stairs
            incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 1)])?;

            // unshift
            incro.emit(&[InputEvent::new(
                EventType::KEY,
                Key::KEY_LEFTSHIFT.code(),
                0,
            )])?;

            sleep(Duration::from_millis(300));

            // release space
            incro.emit(&[InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 0)])?;
            // shift
            incro.emit(&[InputEvent::new(
                EventType::KEY,
                Key::KEY_LEFTSHIFT.code(),
                1,
            )])?;

            // click
            incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)])?;
            incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)])?;

            sleep(Duration::from_millis(250));

            // click
            incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)])?;
            incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)])?;

            sleep(Duration::from_millis(50));

            // click
            incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 1)])?;
            incro.emit(&[InputEvent::new(EventType::KEY, Key::BTN_RIGHT.code(), 0)])?;

            i += 1;
        }

        // unshift
        incro.emit(&[InputEvent::new(
            EventType::KEY,
            Key::KEY_LEFTSHIFT.code(),
            0,
        )])?;

        sleep(Duration::from_millis(300));

        // shift
        incro.emit(&[InputEvent::new(
            EventType::KEY,
            Key::KEY_LEFTSHIFT.code(),
            1,
        )])?;

        sleep(Duration::from_millis(100));

        i += 1;
    }
}
