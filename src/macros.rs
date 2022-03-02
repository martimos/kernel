// adapted from the rust std crate
// #[macro_export]
// macro_rules! dbg {
//     () => {
//         #[cfg(debug_assertions)]
//         $crate::serial_println!("[{}:{}]", file!(), line!())
//     };
//     ($val:expr) => {
//         #[cfg(debug_assertions)]
//         $crate::serial_println!(concat!("[{}:{}] ", $val), file!(), line!());
//     };
//     ($val:expr $(,)?) => {
//         // Use of `match` here is intentional because it affects the lifetimes
//         // of temporaries - https://stackoverflow.com/a/48732525/1063961
//         #[cfg(debug_assertions)]
//         match $val {
//             tmp => {
//                 $crate::serial_println!("[{}:{}] {} = {:#?}",
//                     file!(), line!(), stringify!($val), &tmp);
//                 tmp
//             }
//         }
//     };
//     ($($val:expr),+ $(,)?) => {
//         #[cfg(debug_assertions)]
//         ($($crate::dbg!($val)),+,)
//     };
// }

#[macro_export]
macro_rules! info {
    ($fmt:expr) => ($crate::serial_print!(concat!("[{}] ", $fmt, "\n"), module_path!()));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!("[{}] ", $fmt, "\n"), module_path!(), $($arg)*));
}

#[macro_export]
macro_rules! dbg {
    ($fmt:expr) => {
        #[cfg(debug_assertions)]
        ($crate::serial_print!(concat!("[{}:{}] ", $fmt, "\n"), file!(), line!()))
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        ($crate::serial_print!(
            concat!("[{}:{}] ", $fmt, "\n"), file!(), line!(), $($arg)*))
    };
}
