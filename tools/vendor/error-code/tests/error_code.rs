pub use error_code::{SystemError, PlainError, PosixError};

#[test]
fn it_works() {
    let error = PosixError::new(11);
    eprintln!("{:?}", error.to_string());
    eprintln!("{:?}", error);

    let error = PosixError::last();
    eprintln!("{}", error);

    let error = PlainError::new(11);
    eprintln!("{}", error);

    let error = SystemError::new(11);
    eprintln!("{:?}", error.to_string());

    let error = SystemError::last();
    eprintln!("{}", error);

    let error = PlainError::new(11);
    eprintln!("{}", error);

    let error = SystemError::unimplemented();
    eprintln!("{:?}", error);
    eprintln!("{:?}", error.to_string());
}

#[cfg(feature = "std")]
#[test]
fn convert_io_error() {
    use std::io::{ErrorKind, Error};
    let code = SystemError::unimplemented();
    let error: Error = code.into();
    assert_eq!(code, error);
    assert!(error.raw_os_error().is_some());
    let code2: SystemError = error.into();
    assert_eq!(code, code2);

    let error = Error::new(ErrorKind::Other, "lolka");
    let code: SystemError = error.into();
    assert_eq!(code.raw_code(), -1);
    assert_ne!(code, Error::new(ErrorKind::Other, "lolka"));
}
