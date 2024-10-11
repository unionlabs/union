// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::bitwidth::BitWidth;
use crate::flexbuffer_type::FlexBufferType;
use crate::{Blob, Buffer};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::Rem;
use std::str::FromStr;
mod de;
mod iter;
mod map;
mod serialize;
mod vector;
pub use de::DeserializationError;
pub use iter::ReaderIterator;
pub use map::{MapReader, MapReaderIndexer};
pub use vector::VectorReader;

/// All the possible errors when reading a flexbuffer.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Error {
    /// One of the following data errors occured:
    ///
    /// *    The read flexbuffer had an offset that pointed outside the flexbuffer.
    /// *    The 'negative indicies' where length and map keys are stored were out of bounds
    /// *    The buffer was too small to contain a flexbuffer root.
    FlexbufferOutOfBounds,
    /// Failed to parse a valid FlexbufferType and Bitwidth from a type byte.
    InvalidPackedType,
    /// Flexbuffer type of the read data does not match function used.
    UnexpectedFlexbufferType {
        expected: FlexBufferType,
        actual: FlexBufferType,
    },
    /// BitWidth type of the read data does not match function used.
    UnexpectedBitWidth {
        expected: BitWidth,
        actual: BitWidth,
    },
    /// Read a flexbuffer offset or length that overflowed usize.
    ReadUsizeOverflowed,
    /// Tried to index a type that's not one of the Flexbuffer vector types.
    CannotIndexAsVector,
    /// Tried to index a Flexbuffer vector or map out of bounds.
    IndexOutOfBounds,
    /// A Map was indexed with a key that it did not contain.
    KeyNotFound,
    /// Failed to parse a Utf8 string.
    /// The Option will be `None` if and only if this Error was deserialized.
    // NOTE: std::str::Utf8Error does not implement Serialize, Deserialize, nor Default. We tell
    // serde to skip the field and default to None. We prefer to have the boxed error so it can be
    // used with std::error::Error::source, though another (worse) option could be to drop that
    // information.
    Utf8Error(#[serde(skip)] Option<Box<std::str::Utf8Error>>),
    /// get_slice failed because the given data buffer is misaligned.
    AlignmentError,
    InvalidRootWidth,
    InvalidMapKeysVectorWidth,
}
impl std::convert::From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Utf8Error(Some(Box::new(e)))
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedBitWidth { expected, actual } => write!(
                f,
                "Error reading flexbuffer: Expected bitwidth: {:?}, found bitwidth: {:?}",
                expected, actual
            ),
            Self::UnexpectedFlexbufferType { expected, actual } => write!(
                f,
                "Error reading flexbuffer: Expected type: {:?}, found type: {:?}",
                expected, actual
            ),
            _ => write!(f, "Error reading flexbuffer: {:?}", self),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Self::Utf8Error(Some(e)) = self {
            Some(e)
        } else {
            None
        }
    }
}

pub trait ReadLE: crate::private::Sealed + std::marker::Sized {
    const VECTOR_TYPE: FlexBufferType;
    const WIDTH: BitWidth;
}
macro_rules! rle {
    ($T: ty, $VECTOR_TYPE: ident, $WIDTH: ident) => {
        impl ReadLE for $T {
            const VECTOR_TYPE: FlexBufferType = FlexBufferType::$VECTOR_TYPE;
            const WIDTH: BitWidth = BitWidth::$WIDTH;
        }
    };
}
rle!(u8, VectorUInt, W8);
rle!(u16, VectorUInt, W16);
rle!(u32, VectorUInt, W32);
rle!(u64, VectorUInt, W64);
rle!(i8, VectorInt, W8);
rle!(i16, VectorInt, W16);
rle!(i32, VectorInt, W32);
rle!(i64, VectorInt, W64);
rle!(f32, VectorFloat, W32);
rle!(f64, VectorFloat, W64);

macro_rules! as_default {
    ($as: ident, $get: ident, $T: ty) => {
        pub fn $as(&self) -> $T {
            self.$get().unwrap_or_default()
        }
    };
}

/// `Reader`s allow access to data stored in a Flexbuffer.
///
/// Each reader represents a single address in the buffer so data is read lazily. Start a reader
/// by calling `get_root` on your flexbuffer `&[u8]`.
///
/// - The `get_T` methods return a `Result<T, Error>`. They return an OK value if and only if the
/// flexbuffer type matches `T`. This is analogous to the behavior of Rust's json library, though
/// with Result instead of Option.
/// - The `as_T` methods will try their best to return to a value of type `T`
/// (by casting or even parsing a string if necessary) but ultimately returns `T::default` if it
/// fails. This behavior is analogous to that of flexbuffers C++.
pub struct Reader<B> {
    fxb_type: FlexBufferType,
    width: BitWidth,
    address: usize,
    buffer: B,
}

impl<B: Buffer> Clone for Reader<B> {
    fn clone(&self) -> Self {
        Reader {
            fxb_type: self.fxb_type,
            width: self.width,
            address: self.address,
            buffer: self.buffer.shallow_copy(),
        }
    }
}

impl<B: Buffer> Default for Reader<B> {
    fn default() -> Self {
        Reader {
            fxb_type: FlexBufferType::default(),
            width: BitWidth::default(),
            address: usize::default(),
            buffer: B::empty(),
        }
    }
}

// manual implementation of Debug because buffer slice can't be automatically displayed
impl<B> std::fmt::Debug for Reader<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // skips buffer field
        f.debug_struct("Reader")
            .field("fxb_type", &self.fxb_type)
            .field("width", &self.width)
            .field("address", &self.address)
            .finish()
    }
}

macro_rules! try_cast_fn {
    ($name: ident, $full_width: ident, $Ty: ident) => {
        pub fn $name(&self) -> $Ty {
            self.$full_width().try_into().unwrap_or_default()
        }
    };
}

fn safe_sub(a: usize, b: usize) -> Result<usize, Error> {
    a.checked_sub(b).ok_or(Error::FlexbufferOutOfBounds)
}

fn deref_offset(buffer: &[u8], address: usize, width: BitWidth) -> Result<usize, Error> {
    let off = read_usize(buffer, address, width);
    safe_sub(address, off)
}

impl<B: Buffer> Reader<B> {
    fn new(
        buffer: B,
        mut address: usize,
        mut fxb_type: FlexBufferType,
        width: BitWidth,
        parent_width: BitWidth,
    ) -> Result<Self, Error> {
        if fxb_type.is_reference() {
            address = deref_offset(&buffer, address, parent_width)?;
            // Indirects were dereferenced.
            if let Some(t) = fxb_type.to_direct() {
                fxb_type = t;
            }
        }
        Ok(Reader {
            address,
            fxb_type,
            width,
            buffer,
        })
    }

    /// Parses the flexbuffer from the given buffer. Assumes the flexbuffer root is the last byte
    /// of the buffer.
    pub fn get_root(buffer: B) -> Result<Self, Error> {
        let end = buffer.len();
        if end < 3 {
            return Err(Error::FlexbufferOutOfBounds);
        }
        // Last byte is the root width.
        let root_width = BitWidth::from_nbytes(buffer[end - 1]).ok_or(Error::InvalidRootWidth)?;
        // Second last byte is root type.
        let (fxb_type, width) = unpack_type(buffer[end - 2])?;
        // Location of root data. (BitWidth bits before root type)
        let address = safe_sub(end - 2, root_width.n_bytes())?;
        Self::new(buffer, address, fxb_type, width, root_width)
    }

    /// Convenience function to get the underlying buffer. By using `shallow_copy`, this preserves
    /// the lifetime that the underlying buffer has.
    pub fn buffer(&self) -> B {
        self.buffer.shallow_copy()
    }

    /// Returns the FlexBufferType of this Reader.
    pub fn flexbuffer_type(&self) -> FlexBufferType {
        self.fxb_type
    }

    /// Returns the bitwidth of this Reader.
    pub fn bitwidth(&self) -> BitWidth {
        self.width
    }

    /// Returns the length of the Flexbuffer. If the type has no length, or if an error occurs,
    /// 0 is returned.
    pub fn length(&self) -> usize {
        if let Some(len) = self.fxb_type.fixed_length_vector_length() {
            len
        } else if self.fxb_type.has_length_slot() && self.address >= self.width.n_bytes() {
            read_usize(
                &self.buffer,
                self.address - self.width.n_bytes(),
                self.width,
            )
        } else {
            0
        }
    }
    /// Returns true if the flexbuffer is aligned to 8 bytes. This guarantees, for valid
    /// flexbuffers, that the data is correctly aligned in memory and slices can be read directly
    /// e.g. with `get_f64s` or `get_i16s`.
    #[inline]
    pub fn is_aligned(&self) -> bool {
        (self.buffer.as_ptr() as usize).rem(8) == 0
    }

    as_default!(as_vector, get_vector, VectorReader<B>);
    as_default!(as_map, get_map, MapReader<B>);

    fn expect_type(&self, ty: FlexBufferType) -> Result<(), Error> {
        if self.fxb_type == ty {
            Ok(())
        } else {
            Err(Error::UnexpectedFlexbufferType {
                expected: ty,
                actual: self.fxb_type,
            })
        }
    }
    fn expect_bw(&self, bw: BitWidth) -> Result<(), Error> {
        if self.width == bw {
            Ok(())
        } else {
            Err(Error::UnexpectedBitWidth {
                expected: bw,
                actual: self.width,
            })
        }
    }

    /// Directly reads a slice of type `T` where `T` is one of `u8,u16,u32,u64,i8,i16,i32,i64,f32,f64`.
    /// Returns Err if the type, bitwidth, or memory alignment does not match. Since the bitwidth is
    /// dynamic, its better to use a VectorReader unless you know your data and performance is critical.
    #[cfg(target_endian = "little")]
    #[deprecated(
        since = "0.3.0",
        note = "This function is unsafe - if this functionality is needed use `Reader::buffer::align_to`"
    )]
    pub fn get_slice<T: ReadLE>(&self) -> Result<&[T], Error> {
        if self.flexbuffer_type().typed_vector_type() != T::VECTOR_TYPE.typed_vector_type() {
            self.expect_type(T::VECTOR_TYPE)?;
        }
        if self.bitwidth().n_bytes() != std::mem::size_of::<T>() {
            self.expect_bw(T::WIDTH)?;
        }
        let end = self.address + self.length() * std::mem::size_of::<T>();
        let slice: &[u8] = self
            .buffer
            .get(self.address..end)
            .ok_or(Error::FlexbufferOutOfBounds)?;

        // `align_to` is required because the point of this function is to directly hand back a
        // slice of scalars. This can fail because Rust's default allocator is not 16byte aligned
        // (though in practice this only happens for small buffers).
        let (pre, mid, suf) = unsafe { slice.align_to::<T>() };
        if pre.is_empty() && suf.is_empty() {
            Ok(mid)
        } else {
            Err(Error::AlignmentError)
        }
    }

    /// Returns the value of the reader if it is a boolean.
    /// Otherwise Returns error.
    pub fn get_bool(&self) -> Result<bool, Error> {
        self.expect_type(FlexBufferType::Bool)?;
        Ok(
            self.buffer[self.address..self.address + self.width.n_bytes()]
                .iter()
                .any(|&b| b != 0),
        )
    }

    /// Gets the length of the key if this type is a key.
    ///
    /// Otherwise, returns an error.
    #[inline]
    fn get_key_len(&self) -> Result<usize, Error> {
        self.expect_type(FlexBufferType::Key)?;
        let (length, _) = self.buffer[self.address..]
            .iter()
            .enumerate()
            .find(|(_, &b)| b == b'\0')
            .unwrap_or((0, &0));
        Ok(length)
    }

    /// Retrieves the string value up until the first `\0` character.
    pub fn get_key(&self) -> Result<B::BufferString, Error> {
        let bytes = self
            .buffer
            .slice(self.address..self.address + self.get_key_len()?)
            .ok_or(Error::IndexOutOfBounds)?;
        Ok(bytes.buffer_str()?)
    }

    pub fn get_blob(&self) -> Result<Blob<B>, Error> {
        self.expect_type(FlexBufferType::Blob)?;
        Ok(Blob(
            self.buffer
                .slice(self.address..self.address + self.length())
                .ok_or(Error::IndexOutOfBounds)?,
        ))
    }

    pub fn as_blob(&self) -> Blob<B> {
        self.get_blob().unwrap_or_else(|_| Blob(B::empty()))
    }

    /// Retrieves str pointer, errors if invalid UTF-8, or the provided index
    /// is out of bounds.
    pub fn get_str(&self) -> Result<B::BufferString, Error> {
        self.expect_type(FlexBufferType::String)?;
        let bytes = self
            .buffer
            .slice(self.address..self.address + self.length());
        Ok(bytes.ok_or(Error::ReadUsizeOverflowed)?.buffer_str()?)
    }

    fn get_map_info(&self) -> Result<(usize, BitWidth), Error> {
        self.expect_type(FlexBufferType::Map)?;
        if 3 * self.width.n_bytes() >= self.address {
            return Err(Error::FlexbufferOutOfBounds);
        }
        let keys_offset_address = self.address - 3 * self.width.n_bytes();
        let keys_width = {
            let kw_addr = self.address - 2 * self.width.n_bytes();
            let kw = read_usize(&self.buffer, kw_addr, self.width);
            BitWidth::from_nbytes(kw).ok_or(Error::InvalidMapKeysVectorWidth)
        }?;
        Ok((keys_offset_address, keys_width))
    }

    pub fn get_map(&self) -> Result<MapReader<B>, Error> {
        let (keys_offset_address, keys_width) = self.get_map_info()?;
        let keys_address = deref_offset(&self.buffer, keys_offset_address, self.width)?;
        // TODO(cneo): Check that vectors length equals keys length.
        Ok(MapReader {
            buffer: self.buffer.shallow_copy(),
            values_address: self.address,
            values_width: self.width,
            keys_address,
            keys_width,
            length: self.length(),
        })
    }

    /// Tries to read a FlexBufferType::UInt. Returns Err if the type is not a UInt or if the
    /// address is out of bounds.
    pub fn get_u64(&self) -> Result<u64, Error> {
        self.expect_type(FlexBufferType::UInt)?;
        let cursor = self
            .buffer
            .get(self.address..self.address + self.width.n_bytes());
        match self.width {
            BitWidth::W8 => cursor.map(|s| s[0] as u8).map(Into::into),
            BitWidth::W16 => cursor
                .and_then(|s| s.try_into().ok())
                .map(<u16>::from_le_bytes)
                .map(Into::into),
            BitWidth::W32 => cursor
                .and_then(|s| s.try_into().ok())
                .map(<u32>::from_le_bytes)
                .map(Into::into),
            BitWidth::W64 => cursor
                .and_then(|s| s.try_into().ok())
                .map(<u64>::from_le_bytes),
        }
        .ok_or(Error::FlexbufferOutOfBounds)
    }
    /// Tries to read a FlexBufferType::Int. Returns Err if the type is not a UInt or if the
    /// address is out of bounds.
    pub fn get_i64(&self) -> Result<i64, Error> {
        self.expect_type(FlexBufferType::Int)?;
        let cursor = self
            .buffer
            .get(self.address..self.address + self.width.n_bytes());
        match self.width {
            BitWidth::W8 => cursor.map(|s| s[0] as i8).map(Into::into),
            BitWidth::W16 => cursor
                .and_then(|s| s.try_into().ok())
                .map(<i16>::from_le_bytes)
                .map(Into::into),
            BitWidth::W32 => cursor
                .and_then(|s| s.try_into().ok())
                .map(<i32>::from_le_bytes)
                .map(Into::into),
            BitWidth::W64 => cursor
                .and_then(|s| s.try_into().ok())
                .map(<i64>::from_le_bytes),
        }
        .ok_or(Error::FlexbufferOutOfBounds)
    }
    /// Tries to read a FlexBufferType::Float. Returns Err if the type is not a UInt, if the
    /// address is out of bounds, or if its a f16 or f8 (not currently supported).
    pub fn get_f64(&self) -> Result<f64, Error> {
        self.expect_type(FlexBufferType::Float)?;
        let cursor = self
            .buffer
            .get(self.address..self.address + self.width.n_bytes());
        match self.width {
            BitWidth::W8 | BitWidth::W16 => return Err(Error::InvalidPackedType),
            BitWidth::W32 => cursor
                .and_then(|s| s.try_into().ok())
                .map(f32_from_le_bytes)
                .map(Into::into),
            BitWidth::W64 => cursor
                .and_then(|s| s.try_into().ok())
                .map(f64_from_le_bytes),
        }
        .ok_or(Error::FlexbufferOutOfBounds)
    }
    pub fn as_bool(&self) -> bool {
        use FlexBufferType::*;
        match self.fxb_type {
            Bool => self.get_bool().unwrap_or_default(),
            UInt => self.as_u64() != 0,
            Int => self.as_i64() != 0,
            Float => self.as_f64().abs() > std::f64::EPSILON,
            String | Key => !self.as_str().is_empty(),
            Null => false,
            Blob => self.length() != 0,
            ty if ty.is_vector() => self.length() != 0,
            _ => unreachable!(),
        }
    }
    /// Returns a u64, casting if necessary. For Maps and Vectors, their length is
    /// returned. If anything fails, 0 is returned.
    pub fn as_u64(&self) -> u64 {
        match self.fxb_type {
            FlexBufferType::UInt => self.get_u64().unwrap_or_default(),
            FlexBufferType::Int => self
                .get_i64()
                .unwrap_or_default()
                .try_into()
                .unwrap_or_default(),
            FlexBufferType::Float => self.get_f64().unwrap_or_default() as u64,
            FlexBufferType::String => {
                if let Ok(s) = self.get_str() {
                    if let Ok(f) = u64::from_str(&s) {
                        return f;
                    }
                }
                0
            }
            _ if self.fxb_type.is_vector() => self.length() as u64,
            _ => 0,
        }
    }
    try_cast_fn!(as_u32, as_u64, u32);
    try_cast_fn!(as_u16, as_u64, u16);
    try_cast_fn!(as_u8, as_u64, u8);

    /// Returns an i64, casting if necessary. For Maps and Vectors, their length is
    /// returned. If anything fails, 0 is returned.
    pub fn as_i64(&self) -> i64 {
        match self.fxb_type {
            FlexBufferType::Int => self.get_i64().unwrap_or_default(),
            FlexBufferType::UInt => self
                .get_u64()
                .unwrap_or_default()
                .try_into()
                .unwrap_or_default(),
            FlexBufferType::Float => self.get_f64().unwrap_or_default() as i64,
            FlexBufferType::String => {
                if let Ok(s) = self.get_str() {
                    if let Ok(f) = i64::from_str(&s) {
                        return f;
                    }
                }
                0
            }
            _ if self.fxb_type.is_vector() => self.length() as i64,
            _ => 0,
        }
    }
    try_cast_fn!(as_i32, as_i64, i32);
    try_cast_fn!(as_i16, as_i64, i16);
    try_cast_fn!(as_i8, as_i64, i8);

    /// Returns an f64, casting if necessary. For Maps and Vectors, their length is
    /// returned. If anything fails, 0 is returned.
    pub fn as_f64(&self) -> f64 {
        match self.fxb_type {
            FlexBufferType::Int => self.get_i64().unwrap_or_default() as f64,
            FlexBufferType::UInt => self.get_u64().unwrap_or_default() as f64,
            FlexBufferType::Float => self.get_f64().unwrap_or_default(),
            FlexBufferType::String => {
                if let Ok(s) = self.get_str() {
                    if let Ok(f) = f64::from_str(&s) {
                        return f;
                    }
                }
                0.0
            }
            _ if self.fxb_type.is_vector() => self.length() as f64,
            _ => 0.0,
        }
    }
    pub fn as_f32(&self) -> f32 {
        self.as_f64() as f32
    }

    /// Returns empty string if you're not trying to read a string.
    pub fn as_str(&self) -> B::BufferString {
        match self.fxb_type {
            FlexBufferType::String => self.get_str().unwrap_or_else(|_| B::empty_str()),
            FlexBufferType::Key => self.get_key().unwrap_or_else(|_| B::empty_str()),
            _ => B::empty_str(),
        }
    }

    pub fn get_vector(&self) -> Result<VectorReader<B>, Error> {
        if !self.fxb_type.is_vector() {
            self.expect_type(FlexBufferType::Vector)?;
        };
        Ok(VectorReader {
            reader: self.clone(),
            length: self.length(),
        })
    }
}

impl<B: Buffer> fmt::Display for Reader<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FlexBufferType::*;
        match self.flexbuffer_type() {
            Null => write!(f, "null"),
            UInt => write!(f, "{}", self.as_u64()),
            Int => write!(f, "{}", self.as_i64()),
            Float => write!(f, "{}", self.as_f64()),
            Key | String => write!(f, "{:?}", &self.as_str() as &str),
            Bool => write!(f, "{}", self.as_bool()),
            Blob => write!(f, "blob"),
            Map => {
                write!(f, "{{")?;
                let m = self.as_map();
                let mut pairs = m.iter_keys().zip(m.iter_values());
                if let Some((k, v)) = pairs.next() {
                    write!(f, "{:?}: {}", &k as &str, v)?;
                    for (k, v) in pairs {
                        write!(f, ", {:?}: {}", &k as &str, v)?;
                    }
                }
                write!(f, "}}")
            }
            t if t.is_vector() => {
                write!(f, "[")?;
                let mut elems = self.as_vector().iter();
                if let Some(first) = elems.next() {
                    write!(f, "{}", first)?;
                    for e in elems {
                        write!(f, ", {}", e)?;
                    }
                }
                write!(f, "]")
            }
            _ => unreachable!("Display not implemented for {:?}", self),
        }
    }
}

// TODO(cneo): Use <f..>::from_le_bytes when we move past rustc 1.39.
fn f32_from_le_bytes(bytes: [u8; 4]) -> f32 {
    let bits = <u32>::from_le_bytes(bytes);
    <f32>::from_bits(bits)
}

fn f64_from_le_bytes(bytes: [u8; 8]) -> f64 {
    let bits = <u64>::from_le_bytes(bytes);
    <f64>::from_bits(bits)
}

fn read_usize(buffer: &[u8], address: usize, width: BitWidth) -> usize {
    let cursor = &buffer[address..];
    match width {
        BitWidth::W8 => cursor[0] as usize,
        BitWidth::W16 => cursor
            .get(0..2)
            .and_then(|s| s.try_into().ok())
            .map(<u16>::from_le_bytes)
            .unwrap_or_default() as usize,
        BitWidth::W32 => cursor
            .get(0..4)
            .and_then(|s| s.try_into().ok())
            .map(<u32>::from_le_bytes)
            .unwrap_or_default() as usize,
        BitWidth::W64 => cursor
            .get(0..8)
            .and_then(|s| s.try_into().ok())
            .map(<u64>::from_le_bytes)
            .unwrap_or_default() as usize,
    }
}

fn unpack_type(ty: u8) -> Result<(FlexBufferType, BitWidth), Error> {
    let w = BitWidth::try_from(ty & 3u8).map_err(|_| Error::InvalidPackedType)?;
    let t = FlexBufferType::try_from(ty >> 2).map_err(|_| Error::InvalidPackedType)?;
    Ok((t, w))
}
