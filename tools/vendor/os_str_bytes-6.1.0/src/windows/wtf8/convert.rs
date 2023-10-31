use std::char;
use std::char::DecodeUtf16;
use std::num::NonZeroU16;

use crate::util::BYTE_SHIFT;
use crate::util::CONT_MASK;
use crate::util::CONT_TAG;

use super::CodePoints;
use super::Result;

const MIN_HIGH_SURROGATE: u16 = 0xD800;

const MIN_LOW_SURROGATE: u16 = 0xDC00;

const MIN_SURROGATE_CODE: u32 = (u16::MAX as u32) + 1;

macro_rules! static_assert {
    ( $condition:expr ) => {
        const _: () = [()][if $condition { 0 } else { 1 }];
    };
}

pub(in super::super) struct DecodeWide<I>
where
    I: Iterator<Item = u16>,
{
    iter: DecodeUtf16<I>,
    code_point: u32,
    shift: u8,
}

impl<I> DecodeWide<I>
where
    I: Iterator<Item = u16>,
{
    pub(in super::super) fn new<S>(string: S) -> Self
    where
        S: IntoIterator<IntoIter = I, Item = I::Item>,
    {
        Self {
            iter: char::decode_utf16(string),
            code_point: 0,
            shift: 0,
        }
    }
}

impl<I> Iterator for DecodeWide<I>
where
    I: Iterator<Item = u16>,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(shift) = self.shift.checked_sub(BYTE_SHIFT) {
            self.shift = shift;
            return Some(
                ((self.code_point >> self.shift) as u8 & CONT_MASK) | CONT_TAG,
            );
        }

        self.code_point = self
            .iter
            .next()?
            .map(Into::into)
            .unwrap_or_else(|x| x.unpaired_surrogate().into());

        macro_rules! decode {
            ( $tag:expr ) => {
                Some((self.code_point >> self.shift) as u8 | $tag)
            };
        }
        macro_rules! try_decode {
            ( $tag:expr , $upper_bound:expr ) => {
                if self.code_point < $upper_bound {
                    return decode!($tag);
                }
                self.shift += BYTE_SHIFT;
            };
        }
        try_decode!(0, 0x80);
        try_decode!(0xC0, 0x800);
        try_decode!(0xE0, MIN_SURROGATE_CODE);
        decode!(0xF0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.iter.size_hint();
        let shift = self.shift.into();
        (
            low.saturating_add(shift),
            high.and_then(|x| x.checked_mul(4))
                .and_then(|x| x.checked_add(shift)),
        )
    }
}

struct EncodeWide<I>
where
    I: Iterator<Item = u8>,
{
    iter: CodePoints<I>,
    surrogate: Option<NonZeroU16>,
}

impl<I> EncodeWide<I>
where
    I: Iterator<Item = u8>,
{
    pub(in super::super) fn new<S>(string: S) -> Self
    where
        S: IntoIterator<IntoIter = I, Item = I::Item>,
    {
        Self {
            iter: CodePoints::new(string),
            surrogate: None,
        }
    }
}

impl<I> Iterator for EncodeWide<I>
where
    I: Iterator<Item = u8>,
{
    type Item = Result<u16>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(surrogate) = self.surrogate.take() {
            return Some(Ok(surrogate.get()));
        }

        self.iter.next().map(|code_point| {
            code_point.map(|code_point| {
                code_point
                    .checked_sub(MIN_SURROGATE_CODE)
                    .map(|offset| {
                        static_assert!(MIN_LOW_SURROGATE != 0);

                        self.surrogate = Some(unsafe {
                            NonZeroU16::new_unchecked(
                                (offset & 0x3FF) as u16 | MIN_LOW_SURROGATE,
                            )
                        });
                        (offset >> 10) as u16 | MIN_HIGH_SURROGATE
                    })
                    .unwrap_or(code_point as u16)
            })
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.iter.inner_size_hint();
        let additional = self.surrogate.is_some().into();
        (
            (low.saturating_add(2) / 3).saturating_add(additional),
            high.and_then(|x| x.checked_add(additional)),
        )
    }
}

pub(in super::super) fn encode_wide(
    string: &[u8],
) -> impl '_ + Iterator<Item = Result<u16>> {
    EncodeWide::new(string.iter().copied())
}
