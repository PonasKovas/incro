incro::incro!(State, on_event);

use incro::{
    evdev::{EventType, InputEvent, InputEventKind, Key},
    Incro,
};
use std::{ops::ControlFlow, sync::Mutex, thread::sleep, time::Duration};

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

fn on_event(incro: Incro, state_mutex: &'static Mutex<State>, event: InputEvent) -> bool {
    let mut state = state_mutex.lock().unwrap();

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
            key @ (Key::KEY_1
            | Key::KEY_2
            | Key::KEY_3
            | Key::KEY_4
            | Key::KEY_5
            | Key::KEY_6
            | Key::KEY_7
            | Key::KEY_8
            | Key::KEY_9),
        ) => {
            if event.value() == 1 {
                drop(state);

                incro
                    .thread(move |incro| {
                        {
                            let state = state_mutex.lock().unwrap();
                            if state.shift_pressed {
                                // release shift
                                incro.emit(&[InputEvent::new(
                                    EventType::KEY,
                                    Key::KEY_LEFTSHIFT.code(),
                                    0,
                                )])?;
                            }
                            if state.ctrl_pressed {
                                // release ctrl
                                incro.emit(&[InputEvent::new(
                                    EventType::KEY,
                                    Key::KEY_LEFTCTRL.code(),
                                    0,
                                )])?;
                            }
                        }

                        // forward hotbar keypress
                        incro.emit(&[InputEvent::new(EventType::KEY, key.code(), 1)])?;

                        sleep(Duration::from_millis(50));

                        {
                            let state = state_mutex.lock().unwrap();
                            if state.shift_pressed {
                                // press shift
                                incro.emit(&[InputEvent::new(
                                    EventType::KEY,
                                    Key::KEY_LEFTSHIFT.code(),
                                    1,
                                )])?;
                            }
                            if state.ctrl_pressed {
                                // press ctrl
                                incro.emit(&[InputEvent::new(
                                    EventType::KEY,
                                    Key::KEY_LEFTCTRL.code(),
                                    1,
                                )])?;
                            }
                        }

                        // release hotbar key
                        incro.emit(&[InputEvent::new(EventType::KEY, key.code(), 0)])?;

                        ControlFlow::Continue(())
                    })
                    .detach();
            }

            true
        }
        _ => false,
    }
}
