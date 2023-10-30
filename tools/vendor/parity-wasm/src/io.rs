//! Simple abstractions for the IO operations.
//!
//! Basically it just a replacement for the std::io that is usable from
//! the `no_std` environment.

#[cfg(feature = "std")]
use std::io;

/// IO specific error.
#[derive(Debug)]
pub enum Error {
	/// Some unexpected data left in the buffer after reading all data.
	TrailingData,

	/// Unexpected End-Of-File
	UnexpectedEof,

	/// Invalid data is encountered.
	InvalidData,

	#[cfg(feature = "std")]
	Io(std::io::Error),
}

/// IO specific Result.
pub type Result<T> = core::result::Result<T, Error>;

pub trait Write {
	/// Write a buffer of data into this write.
	///
	/// All data is written at once.
	fn write(&mut self, buf: &[u8]) -> Result<()>;
}

pub trait Read {
	/// Read a data from this read to a buffer.
	///
	/// If there is not enough data in this read then `UnexpectedEof` will be returned.
	fn read(&mut self, buf: &mut [u8]) -> Result<()>;
}

/// Reader that saves the last position.
pub struct Cursor<T> {
	inner: T,
	pos: usize,
}

impl<T> Cursor<T> {
	pub fn new(inner: T) -> Cursor<T> {
		Cursor { inner, pos: 0 }
	}

	pub fn position(&self) -> usize {
		self.pos
	}
}

impl<T: AsRef<[u8]>> Read for Cursor<T> {
	fn read(&mut self, buf: &mut [u8]) -> Result<()> {
		let slice = self.inner.as_ref();
		let remainder = slice.len() - self.pos;
		let requested = buf.len();
		if requested > remainder {
			return Err(Error::UnexpectedEof)
		}
		buf.copy_from_slice(&slice[self.pos..(self.pos + requested)]);
		self.pos += requested;
		Ok(())
	}
}

#[cfg(not(feature = "std"))]
impl Write for alloc::vec::Vec<u8> {
	fn write(&mut self, buf: &[u8]) -> Result<()> {
		self.extend(buf);
		Ok(())
	}
}

#[cfg(feature = "std")]
impl<T: io::Read> Read for T {
	fn read(&mut self, buf: &mut [u8]) -> Result<()> {
		self.read_exact(buf).map_err(Error::Io)
	}
}

#[cfg(feature = "std")]
impl<T: io::Write> Write for T {
	fn write(&mut self, buf: &[u8]) -> Result<()> {
		self.write_all(buf).map_err(Error::Io)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn cursor() {
		let mut cursor = Cursor::new(vec![0xFFu8, 0x7Fu8]);
		assert_eq!(cursor.position(), 0);

		let mut buf = [0u8];
		assert!(cursor.read(&mut buf[..]).is_ok());
		assert_eq!(cursor.position(), 1);
		assert_eq!(buf[0], 0xFFu8);
		assert!(cursor.read(&mut buf[..]).is_ok());
		assert_eq!(buf[0], 0x7Fu8);
		assert_eq!(cursor.position(), 2);
	}

	#[test]
	fn overflow_in_cursor() {
		let mut cursor = Cursor::new(vec![0u8]);
		let mut buf = [0, 1, 2];
		assert!(cursor.read(&mut buf[..]).is_err());
	}
}
