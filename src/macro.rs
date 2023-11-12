#[macro_export]
macro_rules! incro {
    ($state:ty, $entry:path) => {
        const _: () = {
            use ::std::sync::OnceLock;
            use $crate::evdev;
            use $crate::Incro;

            #[no_mangle]
            static INCRO_VERSION: u64 = $crate::INCRO_VERSION;

            static STATE: OnceLock<$state> = OnceLock::new();

            #[no_mangle]
            extern "C" fn incro_event(incro: Incro, event: evdev::InputEvent) -> bool {
                let state = STATE.get_or_init(|| <$state as ::std::default::Default>::default());

                $entry(incro, state, event)
            }
        };
    };
}
