//! TODO: Support the segment for XCOFF when auxiliary file header and loader section is ready.

use core::fmt::Debug;
use core::str;

use crate::read::{self, ObjectSegment, ReadRef, Result};
use crate::xcoff;

use super::{FileHeader, XcoffFile};

/// An iterator over the segments of an `XcoffFile32`.
pub type XcoffSegmentIterator32<'data, 'file, R = &'data [u8]> =
    XcoffSegmentIterator<'data, 'file, xcoff::FileHeader32, R>;
/// An iterator over the segments of an `XcoffFile64`.
pub type XcoffSegmentIterator64<'data, 'file, R = &'data [u8]> =
    XcoffSegmentIterator<'data, 'file, xcoff::FileHeader64, R>;

/// An iterator over the segments of an `XcoffFile`.
#[derive(Debug)]
pub struct XcoffSegmentIterator<'data, 'file, Xcoff, R = &'data [u8]>
where
    'data: 'file,
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    #[allow(unused)]
    pub(super) file: &'file XcoffFile<'data, Xcoff, R>,
}

impl<'data, 'file, Xcoff, R> Iterator for XcoffSegmentIterator<'data, 'file, Xcoff, R>
where
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    type Item = XcoffSegment<'data, 'file, Xcoff, R>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

/// A segment of an `XcoffFile32`.
pub type XcoffSegment32<'data, 'file, R = &'data [u8]> =
    XcoffSegment<'data, 'file, xcoff::FileHeader32, R>;
/// A segment of an `XcoffFile64`.
pub type XcoffSegment64<'data, 'file, R = &'data [u8]> =
    XcoffSegment<'data, 'file, xcoff::FileHeader64, R>;

/// A loadable section of an `XcoffFile`.
#[derive(Debug)]
pub struct XcoffSegment<'data, 'file, Xcoff, R = &'data [u8]>
where
    'data: 'file,
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    #[allow(unused)]
    pub(super) file: &'file XcoffFile<'data, Xcoff, R>,
}

impl<'data, 'file, Xcoff, R> XcoffSegment<'data, 'file, Xcoff, R>
where
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
}

impl<'data, 'file, Xcoff, R> read::private::Sealed for XcoffSegment<'data, 'file, Xcoff, R>
where
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
}

impl<'data, 'file, Xcoff, R> ObjectSegment<'data> for XcoffSegment<'data, 'file, Xcoff, R>
where
    Xcoff: FileHeader,
    R: ReadRef<'data>,
{
    fn address(&self) -> u64 {
        unreachable!();
    }

    fn size(&self) -> u64 {
        unreachable!();
    }

    fn align(&self) -> u64 {
        unreachable!();
    }

    fn file_range(&self) -> (u64, u64) {
        unreachable!();
    }

    fn data(&self) -> Result<&'data [u8]> {
        unreachable!();
    }

    fn data_range(&self, _address: u64, _size: u64) -> Result<Option<&'data [u8]>> {
        unreachable!();
    }

    fn name_bytes(&self) -> Result<Option<&[u8]>> {
        unreachable!();
    }

    fn name(&self) -> Result<Option<&str>> {
        unreachable!();
    }

    fn flags(&self) -> crate::SegmentFlags {
        unreachable!();
    }
}
