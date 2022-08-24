#[macro_export]
macro_rules! info {
    ($fmt:expr) => ($crate::serial_print!(concat!("[{}] ", $fmt, "\n"), module_path!()));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!("[{}] ", $fmt, "\n"), module_path!(), $($arg)*));
}

#[macro_export]
macro_rules! error {
    ($fmt:expr) => ($crate::serial_print!(concat!("ERROR: [{}] ", $fmt, "\n"), module_path!()));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!("ERROR: [{}] ", $fmt, "\n"), module_path!(), $($arg)*));
}

#[macro_export]
macro_rules! debug {
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

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
