use std::fmt;
use std::io;


pub trait AnyWrite {
    type wstr: ?Sized;
    type Error;

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> Result<(), Self::Error>;

    fn write_str(&mut self, s: &Self::wstr) -> Result<(), Self::Error>;
}


impl<'a> AnyWrite for fmt::Write + 'a {
    type wstr = str;
    type Error = fmt::Error;

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> Result<(), Self::Error> {
        fmt::Write::write_fmt(self, fmt)
    }

    fn write_str(&mut self, s: &Self::wstr) -> Result<(), Self::Error> {
        fmt::Write::write_str(self, s)
    }
}


impl<'a> AnyWrite for io::Write + 'a {
    type wstr = [u8];
    type Error = io::Error;

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> Result<(), Self::Error> {
        io::Write::write_fmt(self, fmt)
    }

    fn write_str(&mut self, s: &Self::wstr) -> Result<(), Self::Error> {
        io::Write::write_all(self, s)
    }
}
