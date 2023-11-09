use anyhow::bail;
use async_ffi::{FfiFuture, FutureExt};
use evdev::{InputEvent, InputEventKind, Synchronization};
use incro::Methods;
use incro::INCRO_VERSION;
use libloading::{Library, Symbol};
use std::sync::Mutex;
use std::time::Instant;
use std::{collections::BTreeMap, os::unix::prelude::OsStrExt, path::PathBuf, time::Duration};
use tokio::{spawn, sync::Mutex as TokioMutex};
use tracing::info;
use tracing::instrument;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let macros: &TokioMutex<BTreeMap<PathBuf, Library>> =
        Box::leak(Box::new(TokioMutex::new(BTreeMap::new())));

    {
        let mut macros = macros.lock().await;
        let mut read_dir = tokio::fs::read_dir("macros").await?;
        while let Some(entry) = read_dir.next_entry().await? {
            if entry.file_type().await?.is_dir() || entry.file_name().as_bytes().starts_with(b".") {
                // skip directories and hidden files (starting with .)
                continue;
            }

            load_macro(&mut *macros, entry.path()).await?;
        }
    }

    // spawn(async move {
    //     // watch macro dir and add/remove macros on demand
    // });

    let mut device_builder = evdev::uinput::VirtualDeviceBuilder::new()
        .unwrap()
        .name("Incro");

    for (_path, device) in
        evdev::enumerate().filter(|(_path, device)| device.supported_keys().is_some())
    {
        if let Some(keys) = device.supported_keys() {
            device_builder = device_builder.with_keys(keys).unwrap()
        }
        if let Some(axes) = device.supported_relative_axes() {
            device_builder = device_builder.with_relative_axes(axes).unwrap()
        }
    }

    let virtual_device = Box::leak(Box::new(Mutex::new(device_builder.build().unwrap())));
    let methods = Methods::new(virtual_device);

    let mut tasks = Vec::new();
    for (_path, mut device) in
        evdev::enumerate().filter(|(_path, device)| device.supported_keys().is_some())
    {
        let handle = spawn(async move {
            device.grab().unwrap();

            let mut event_stream = device.into_event_stream().unwrap();

            let mut events_stack = Vec::new();

            loop {
                let event = event_stream.next_event().await.unwrap();

                if event.kind() == InputEventKind::Synchronization(Synchronization::SYN_REPORT) {
                    methods.emit(&events_stack);
                    events_stack.clear();
                    continue;
                }

                let mut forward = true;
                for (_path, lib) in macros.lock().await.iter() {
                    let entry: Symbol<
                        extern "C" fn(methods: Methods, event: InputEvent) -> FfiFuture<bool>,
                    > = unsafe { lib.get(b"incro_event").unwrap() };

                    if (*entry)(methods, event).await {
                        forward = false;
                    }
                }

                if forward {
                    events_stack.push(event);
                }
            }
        });
        tasks.push(handle);
    }

    info!("All devices grabbed.");

    for handle in tasks {
        handle.await.unwrap();
    }

    Ok(())
}

#[instrument(skip(macros))]
async fn load_macro(macros: &mut BTreeMap<PathBuf, Library>, path: PathBuf) -> anyhow::Result<()> {
    let lib = unsafe { libloading::Library::new(&path)? };
    // make sure lib has the required symbols and version is ok
    let version: Symbol<*const u64> = unsafe { lib.get(b"INCRO_VERSION")? };
    let _event: Symbol<extern "C" fn()> = unsafe { lib.get(b"incro_event")? };

    let version = unsafe { **version };
    if version != INCRO_VERSION {
        bail!("Incompatible version! Macro {version}, but host {INCRO_VERSION}.");
    }

    macros.insert(path, lib);

    info!("Macro loaded.");

    Ok(())
}
