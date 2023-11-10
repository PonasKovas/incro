use anyhow::{bail, Result};
use evdev::{Device, InputEvent, InputEventKind, Synchronization};
use incro::Incro;
use libloading::{Library, Symbol};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{mpsc::Sender, RwLock},
};
use tracing::{event, Level};

/// Monitors device input and runs the macros
#[tracing::instrument(skip_all, fields(device = device.name().unwrap_or("<unnamed>")))]
pub(crate) fn absorber_thread(
    event_sender: Sender<Vec<InputEvent>>,
    macros: &RwLock<BTreeMap<PathBuf, Library>>,
    mut device: Device,
) -> Result<()> {
    loop {
        let mut stack = Vec::new();

        for event in device.fetch_events()? {
            // Synchronization events
            if event.kind() == InputEventKind::Synchronization(Synchronization::SYN_REPORT) {
                // send stack
                event_sender.send(stack.clone())?;
                stack.clear();

                continue;
            }

            event!(Level::DEBUG, event = ?event.kind());

            // Lock the macros RwLock
            let macros = match macros.read() {
                Ok(lock) => lock,
                Err(_) => bail!("Macros RwLock poisoned"),
            };

            // Whether to re-emit the absorbed event or ignore it (if false)
            let mut forward = true;
            for (_path, lib) in macros.iter() {
                let incro = Incro::new(event_sender.clone());

                unsafe {
                    let entry: Symbol<
                        unsafe extern "C" fn(incro: Incro, event: InputEvent) -> bool,
                    > = lib.get(b"incro_event")?;

                    if (*entry)(incro, event) {
                        forward = false;
                    }
                }
            }

            if forward {
                stack.push(event);
            }
        }
    }
}
