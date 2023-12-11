use std::{
    fs::File,
    hash::Hash,
    ops::{Deref, Index, RangeBounds},
    path::{Path, PathBuf},
};

use bytes::{Buf, Bytes};

use crate::{mmap::MmappedSlice, MmapError};

/// A cheaply cloneable owned buffer that may be backed by a memory-mapped file
/// or [`Bytes`] in memory.
#[derive(Debug, Clone, Default)]
pub struct OwnedBuffer {
    repr: Repr,
}

impl OwnedBuffer {
    pub const fn new() -> Self {
        OwnedBuffer {
            repr: Repr::InMemory(Bytes::new()),
        }
    }

    pub const fn from_static(bytes: &'static [u8]) -> Self {
        OwnedBuffer {
            repr: Repr::InMemory(Bytes::from_static(bytes)),
        }
    }

    pub fn from_bytes(bytes: impl Into<Bytes>) -> Self {
        bytes.into().into()
    }

    pub fn mmap(path: impl AsRef<Path>) -> Result<Self, MmapError> {
        let mmap = MmappedSlice::from_path(path.as_ref())?;
        Ok(mmap.into())
    }

    pub fn from_file(file: &File) -> Result<Self, MmapError> {
        let mmap = MmappedSlice::from_file(file)?;
        Ok(mmap.into())
    }

    pub fn into_bytes(self) -> Bytes {
        match self.repr {
            Repr::InMemory(b) => b,
            Repr::Mmap(m) => Bytes::from(m.as_slice().to_vec()),
        }
    }

    pub fn as_bytes(&self) -> Option<&Bytes> {
        self.repr.as_bytes()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        match &self.repr {
            Repr::InMemory(b) => b,
            Repr::Mmap(m) => m.as_slice(),
        }
    }

    #[inline]
    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        self.repr.slice(range).into()
    }

    pub fn is_mmapped(&self) -> bool {
        matches!(self.repr, Repr::Mmap(_))
    }
}

impl Deref for OwnedBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl AsRef<[u8]> for OwnedBuffer {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl PartialEq for OwnedBuffer {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialEq<[u8]> for OwnedBuffer {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

impl PartialEq<&[u8]> for OwnedBuffer {
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_slice() == *other
    }
}

impl<const N: usize> PartialEq<[u8; N]> for OwnedBuffer {
    fn eq(&self, other: &[u8; N]) -> bool {
        self.as_slice() == other
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for OwnedBuffer {
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.as_slice() == *other
    }
}

impl PartialEq<Vec<u8>> for OwnedBuffer {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_slice() == other
    }
}

impl PartialEq<&Vec<u8>> for OwnedBuffer {
    fn eq(&self, other: &&Vec<u8>) -> bool {
        self.as_slice() == *other
    }
}

impl PartialEq<OwnedBuffer> for [u8] {
    fn eq(&self, other: &OwnedBuffer) -> bool {
        other == self
    }
}

impl PartialEq<OwnedBuffer> for &[u8] {
    fn eq(&self, other: &OwnedBuffer) -> bool {
        other == self
    }
}

impl<const N: usize> PartialEq<OwnedBuffer> for [u8; N] {
    fn eq(&self, other: &OwnedBuffer) -> bool {
        other == self
    }
}

impl<const N: usize> PartialEq<OwnedBuffer> for &[u8; N] {
    fn eq(&self, other: &OwnedBuffer) -> bool {
        other == self
    }
}

impl PartialEq<OwnedBuffer> for Vec<u8> {
    fn eq(&self, other: &OwnedBuffer) -> bool {
        other == self
    }
}

impl PartialEq<OwnedBuffer> for &Vec<u8> {
    fn eq(&self, other: &OwnedBuffer) -> bool {
        other == self
    }
}

impl<'a> IntoIterator for &'a OwnedBuffer {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl IntoIterator for OwnedBuffer {
    type Item = u8;
    type IntoIter = OwnedIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        OwnedIntoIter {
            buffer: self,
            next_index: 0,
        }
    }
}

impl<I> Index<I> for OwnedBuffer
where
    [u8]: Index<I>,
{
    type Output = <[u8] as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl Eq for OwnedBuffer {}

impl Ord for OwnedBuffer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl PartialOrd for OwnedBuffer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for OwnedBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl Buf for OwnedBuffer {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    fn advance(&mut self, cnt: usize) {
        match &mut self.repr {
            Repr::InMemory(b) => b.advance(cnt),
            Repr::Mmap(m) => m.advance(cnt),
        }
    }
}

impl From<Bytes> for OwnedBuffer {
    fn from(value: Bytes) -> Self {
        OwnedBuffer {
            repr: Repr::InMemory(value),
        }
    }
}

impl From<&[u8]> for OwnedBuffer {
    fn from(value: &[u8]) -> Self {
        value.to_vec().into()
    }
}

impl From<Vec<u8>> for OwnedBuffer {
    fn from(value: Vec<u8>) -> Self {
        Bytes::from(value).into()
    }
}

impl From<&Vec<u8>> for OwnedBuffer {
    fn from(value: &Vec<u8>) -> Self {
        value.as_slice().into()
    }
}

impl From<OwnedBuffer> for Vec<u8> {
    fn from(value: OwnedBuffer) -> Self {
        value.to_vec()
    }
}

impl From<Repr> for OwnedBuffer {
    fn from(repr: Repr) -> Self {
        OwnedBuffer { repr }
    }
}

impl From<MmappedSlice> for OwnedBuffer {
    fn from(mmap: MmappedSlice) -> Self {
        Repr::Mmap(mmap).into()
    }
}

impl TryFrom<&Path> for OwnedBuffer {
    type Error = MmapError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        OwnedBuffer::mmap(value)
    }
}

impl TryFrom<PathBuf> for OwnedBuffer {
    type Error = MmapError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        OwnedBuffer::mmap(value)
    }
}

impl TryFrom<&PathBuf> for OwnedBuffer {
    type Error = MmapError;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        OwnedBuffer::mmap(value)
    }
}

impl TryFrom<&std::fs::File> for OwnedBuffer {
    type Error = MmapError;

    fn try_from(file: &std::fs::File) -> Result<Self, Self::Error> {
        OwnedBuffer::from_file(file)
    }
}

impl TryFrom<std::fs::File> for OwnedBuffer {
    type Error = MmapError;

    fn try_from(file: std::fs::File) -> Result<Self, Self::Error> {
        OwnedBuffer::from_file(&file)
    }
}

impl TryFrom<OwnedBuffer> for Bytes {
    type Error = OwnedBuffer;

    fn try_from(value: OwnedBuffer) -> Result<Self, Self::Error> {
        if let Repr::InMemory(bytes) = value.repr {
            Ok(bytes)
        } else {
            Err(value)
        }
    }
}

#[derive(Debug, Clone)]
enum Repr {
    InMemory(Bytes),
    Mmap(MmappedSlice),
}

impl Repr {
    #[inline]
    fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        match self {
            Repr::InMemory(b) => Repr::InMemory(b.slice(range)),
            Repr::Mmap(m) => Repr::Mmap(m.slice(range)),
        }
    }

    #[inline]
    fn as_bytes(&self) -> Option<&Bytes> {
        match self {
            Repr::InMemory(b) => Some(b),
            _ => None,
        }
    }
}

impl Default for Repr {
    fn default() -> Self {
        Repr::InMemory(Bytes::new())
    }
}

#[derive(Debug, Clone)]
pub struct OwnedIntoIter {
    buffer: OwnedBuffer,
    next_index: usize,
}

impl Iterator for OwnedIntoIter {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let &byte = self.buffer.as_slice().get(self.next_index)?;
        self.next_index += 1;
        Some(byte)
    }
}
