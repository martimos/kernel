// adapted from the rust std crate
#[macro_export]
macro_rules! dbg {
    () => {
        #[cfg(not(debug_assertions))]
        $crate::serial_println!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        #[cfg(not(debug_assertions))]
        match $val {
            tmp => {
                $crate::serial_println!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        #[cfg(not(debug_assertions))]
        ($($crate::dbg!($val)),+,)
    };
}
