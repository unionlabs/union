macro_rules! w {
    ($buf:expr, $to_w:expr) => {
        match $buf.write_all($to_w) {
            Ok(..) => (),
            Err(..) => panic!("Failed to write to generated file"),
        }
    };
}

#[cfg(feature = "debug")]
macro_rules! debug {
    ($($arg:tt)*) => {
        eprint!("[{:>w$}] \t", module_path!(), w = 28);
        eprintln!($($arg)*)
    }
}

#[cfg(not(feature = "debug"))]
macro_rules! debug {
    ($($arg:tt)*) => {};
}
