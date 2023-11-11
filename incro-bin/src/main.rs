use anyhow::bail;
use anyhow::Context;
use incro::INCRO_VERSION;
use libloading::{Library, Symbol};
use notify::event::CreateKind;
use notify::event::DataChange;
use notify::event::ModifyKind;
use notify::event::RemoveKind;
use notify::event::RenameMode;
use notify::Event;
use notify::RecursiveMode;
use notify::Watcher;
use path_absolutize::Absolutize;
use std::path::Path;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use tracing::span;

use std::{collections::BTreeMap, os::unix::prelude::OsStrExt, path::PathBuf};
use tracing::event;
use tracing::instrument;
use tracing::Level;
use tracing_subscriber::EnvFilter;

mod absorber;
mod emitter;

const MACROS_DIRECTORY: &str = "macros";

fn main() {
    if let Err(e) = run() {
        event!(Level::ERROR, "{}", e);
    }
}

fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env()?,
        )
        .init();

    // Protection from input deadlocks during development
    #[cfg(debug_assertions)]
    thread::spawn(|| {
        use std::time::Duration;

        thread::sleep(Duration::from_secs(5));
        event!(Level::WARN, "Exiting due to debug 5 second timeout.");
        std::process::exit(0);
    });

    let macros = Arc::new(RwLock::new(BTreeMap::<PathBuf, Library>::new()));

    // `macros` lock scope
    {
        let mut macros = macros.write().unwrap();

        for entry in std::fs::read_dir(MACROS_DIRECTORY)? {
            let entry = entry?;

            load_macro(&mut macros, entry.path().absolutize()?.into_owned())?;
        }
    }

    // Watch macros directory to add new macros on the fly
    let macros_clone = Arc::clone(&macros);
    let mut watcher =
        notify::recommended_watcher(move |res| filesystem_event_handler(res, &macros_clone))?;
    watcher.watch(Path::new(MACROS_DIRECTORY), RecursiveMode::NonRecursive)?;

    let (shutdown_sender, shutdown_receiver) = mpsc::sync_channel(0);

    // Initialize fake event emitter
    let event_sender = emitter::emitter(shutdown_sender.clone())?;

    // Initialize an event absorber for each device
    for (_path, mut device) in
        evdev::enumerate().filter(|(_path, device)| device.supported_keys().is_some())
    {
        if device.name() == Some("Incro") {
            continue; // dont absorb self...
        }

        event!(Level::INFO, path = ?_path, name = ?device.name());

        device.grab()?;

        let event_sender = event_sender.clone();
        let macros = Arc::clone(&macros);
        let shutdown = shutdown_sender.clone();

        thread::spawn(move || {
            if let Err(e) = absorber::absorber_thread(event_sender, &macros, device) {
                event!(Level::ERROR, "Absorber thread error: {}", e);
            }

            // if the absorber thread returns that means something's wrong
            // and its time to end the party... 😔
            let _ = shutdown.send(());
        });
    }

    event!(Level::INFO, "All devices grabbed.");

    let _ = shutdown_receiver.recv();

    Ok(())
}

#[instrument(skip(macros))]
fn load_macro(macros: &mut BTreeMap<PathBuf, Library>, path: PathBuf) -> anyhow::Result<()> {
    if path.is_dir()
        || path
            .file_name()
            .context("invalid path")?
            .as_bytes()
            .starts_with(b".")
    {
        // skip directories and hidden files (starting with .)
        event!(Level::DEBUG, "skipping");
        return Ok(());
    }

    let lib = unsafe { libloading::Library::new(&path)? };

    // make sure lib has the required symbols and version is ok
    let version: Symbol<*const u64> = unsafe { lib.get(b"INCRO_VERSION")? };
    let _event: Symbol<extern "C" fn()> = unsafe { lib.get(b"incro_event")? };

    let version = unsafe { version.as_ref() }.context("Null pointer as version")?;
    if *version != INCRO_VERSION {
        event!(
            Level::ERROR,
            "Incompatible version! Macro {version}, but host {INCRO_VERSION}."
        );
        bail!("Incompatible macro version");
    }

    macros.insert(path, lib);

    event!(Level::INFO, "Macro loaded.");

    Ok(())
}

fn filesystem_event_handler(
    res: Result<Event, notify::Error>,
    macros: &RwLock<BTreeMap<PathBuf, Library>>,
) {
    let load = |macros: &mut BTreeMap<PathBuf, Library>, path: PathBuf| {
        if let Err(e) = load_macro(macros, path) {
            event!(Level::ERROR, "Error loading macro: {}", e);
        }
    };

    let remove = |macros: &mut BTreeMap<PathBuf, Library>, path| {
        if macros.remove(path).is_none() {
            event!(Level::WARN, "Macro was supposed to be loaded but wasnt");
        }
    };

    match res {
        Ok(event) => {
            let _span = span!(Level::INFO, "filesystem event", path = ?event.paths[0]).entered();

            match &event.kind {
                notify::EventKind::Create(CreateKind::File) => {
                    event!(Level::INFO, "Loading macro (file added)");

                    load(&mut macros.write().unwrap(), event.paths[0].clone());
                }
                notify::EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                    event!(Level::INFO, "Reloading macro (file contents changed)");

                    let mut macros = macros.write().unwrap();

                    remove(&mut macros, &event.paths[0]);

                    load(&mut macros, event.paths[0].clone());
                }
                notify::EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                    let was_hidden = event.paths[0]
                        .file_name()
                        .unwrap()
                        .as_bytes()
                        .starts_with(b".");
                    let is_hidden = event.paths[1]
                        .file_name()
                        .unwrap()
                        .as_bytes()
                        .starts_with(b".");

                    if was_hidden && !is_hidden {
                        event!(Level::INFO, "Loading macro (file unhidden)");

                        load(&mut macros.write().unwrap(), event.paths[1].clone());
                    } else if !was_hidden && is_hidden {
                        event!(Level::INFO, "Unloading macro (file hidden)");

                        remove(&mut macros.write().unwrap(), &event.paths[0]);
                    }
                }
                notify::EventKind::Remove(RemoveKind::File) => {
                    event!(Level::INFO, "Removing macro (file removed)");

                    if macros.write().unwrap().remove(&event.paths[0]).is_none() {
                        event!(Level::ERROR, "Tried to unload macro that was not loaded");
                    }
                }
                _ => {}
            }
        }
        Err(e) => event!(Level::ERROR, error = %e, "watch error"),
    }
}
