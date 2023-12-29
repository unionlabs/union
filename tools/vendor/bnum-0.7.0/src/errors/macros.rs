macro_rules! err_prefix {
    () => {
        "(bnum)"
    };
}

pub(crate) use err_prefix;

macro_rules! err_msg {
    ($msg: expr) => {
        concat!(crate::errors::err_prefix!(), " ", $msg)
    };
}

pub(crate) use err_msg;

macro_rules! div_zero {
    () => {
        panic!(crate::errors::err_msg!("attempt to divide by zero"))
    };
}

pub(crate) use div_zero;

macro_rules! rem_zero {
    () => {
        panic!(crate::errors::err_msg!(
            "attempt to calculate remainder with a divisor of zero"
        ))
    };
}

pub(crate) use rem_zero;

// TODO: this will become unnecessary when `const_option` is stabilised: https://github.com/rust-lang/rust/issues/67441.
macro_rules! option_expect {
    ($option: expr, $msg: expr) => {
        match $option {
            Some(value) => value,
            _ => panic!($msg),
        }
    };
}
pub(crate) use option_expect;

macro_rules! result_expect {
    ($option: expr, $msg: expr) => {
        match $option {
            Ok(value) => value,
            _ => panic!($msg),
        }
    };
}
pub(crate) use result_expect;
