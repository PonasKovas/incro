#[macro_export]
macro_rules! incro {
    ($state:ty, $entry:path) => {
        const _: () = {
            use ::std::sync::OnceLock;
            use $crate::async_ffi::FfiFuture;
            use $crate::async_ffi::FutureExt;
            use $crate::evdev::InputEvent;
            use $crate::tokio::sync::Mutex;
            use $crate::Methods;

            #[no_mangle]
            static INCRO_VERSION: u64 = $crate::INCRO_VERSION;

            #[no_mangle]
            extern "C" fn incro_event(methods: Methods, event: InputEvent) -> FfiFuture<bool> {
                static STATE: OnceLock<Mutex<$state>> = OnceLock::new();
                let state = STATE
                    .get_or_init(|| Mutex::new(<$state as ::std::default::Default>::default()));

                $entry(methods, state, event).into_ffi()
            }
        };
    };
}
