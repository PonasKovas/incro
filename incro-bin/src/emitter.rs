use anyhow::Result;
use evdev::{uinput::VirtualDevice, InputEvent};
use std::{
    sync::mpsc::{self, Receiver, Sender, SyncSender},
    thread,
};
use tracing::{event, Level};

pub(crate) fn emitter(shutdown: SyncSender<()>) -> Result<Sender<Vec<InputEvent>>> {
    let mut device_builder = evdev::uinput::VirtualDeviceBuilder::new()?.name("Incro");

    for (_path, device) in
        evdev::enumerate().filter(|(_path, device)| device.supported_keys().is_some())
    {
        if let Some(keys) = device.supported_keys() {
            device_builder = device_builder.with_keys(keys)?;
        }
        if let Some(axes) = device.supported_relative_axes() {
            device_builder = device_builder.with_relative_axes(axes)?;
        }
    }

    let device = device_builder.build().unwrap();

    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        emitter_thread(device, receiver);

        // if the emitter thread returns that means an error ocurred and its time to end the party...
        let _ = shutdown.send(());
    });

    Ok(sender)
}

#[tracing::instrument(skip_all)]
fn emitter_thread(mut device: VirtualDevice, event_receiver: Receiver<Vec<InputEvent>>) {
    while let Ok(events) = event_receiver.recv() {
        // event!(Level::DEBUG, event = ?event.kind());
        if let Err(e) = device.emit(&events) {
            event!(Level::ERROR, "Error emitting uinput event: {}", e);
        }
    }

    event!(Level::ERROR, "Event emitter channel dead");
}
