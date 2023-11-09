use async_ffi::{FfiFuture, FutureExt};
use std::time::{Duration, Instant};

pub extern "C" fn sleep(secs: u64, nanos: u32) -> FfiFuture<()> {
    async move { tokio::time::sleep(Duration::new(secs, nanos)).await }.into_ffi()
}

pub extern "C" fn precise_sleep(secs: u64, nanos: u32) -> FfiFuture<()> {
    let start_time = Instant::now();

    async move {
        let duration = Duration::new(secs, nanos);

        // Use tokio sleep timer to sleep until 4 ms remain, then begin busy wait
        if let Some(duration) = duration.checked_sub(Duration::from_micros(4_000)) {
            tokio::time::sleep(duration).await
        }

        // Spin sleep 😝
        while start_time.elapsed() < duration {}
    }
    .into_ffi()
}
