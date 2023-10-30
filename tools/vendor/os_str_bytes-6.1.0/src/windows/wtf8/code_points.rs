use std::iter::Peekable;
use std::mem;

use crate::util::is_continuation;
use crate::util::BYTE_SHIFT;
use crate::util::CONT_MASK;

use super::EncodingError;
use super::Result;

pub(in super::super) struct CodePoints<I>
where
    I: Iterator<Item = u8>,
{
    iter: Peekable<I>,
    surrogate: bool,
}

impl<I> CodePoints<I>
where
    I: Iterator<Item = u8>,
{
    pub(in super::super) fn new<S>(string: S) -> Self
    where
        S: IntoIterator<IntoIter = I, Item = I::Item>,
    {
        Self {
            iter: string.into_iter().peekable(),
            surrogate: false,
        }
    }

    fn consume_next(&mut self, code_point: &mut u32) -> Result<()> {
        if let Some(&byte) = self.iter.peek() {
            if !is_continuation(byte) {
                self.surrogate = false;
                // Not consuming this byte will be useful if this crate ever
                // offers a way to encode lossily.
                return Err(EncodingError::Byte(byte));
            }
            *code_point =
                (*code_point << BYTE_SHIFT) | u32::from(byte & CONT_MASK);

            let removed = self.iter.next();
            debug_assert_eq!(Some(byte), removed);
        } else {
            return Err(EncodingError::End());
        }
        Ok(())
    }

    pub(super) fn inner_size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I> Iterator for CodePoints<I>
where
    I: Iterator<Item = u8>,
{
    type Item = Result<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.iter.next()?;
        let mut code_point: u32 = byte.into();

        macro_rules! consume_next {
            () => {{
                if let Err(error) = self.consume_next(&mut code_point) {
                    return Some(Err(error));
                }
            }};
        }

        let prev_surrogate = mem::replace(&mut self.surrogate, false);

        let mut invalid = false;
        if !byte.is_ascii() {
            if byte < 0xC2 {
                return Some(Err(EncodingError::Byte(byte)));
            }

            if byte < 0xE0 {
                code_point &= 0x1F;
            } else {
                code_point &= 0x0F;
                consume_next!();

                if byte >= 0xF0 {
                    if code_point.wrapping_sub(0x10) >= 0x100 {
                        invalid = true;
                    }
                    consume_next!();

                // This condition is optimized to detect surrogate code points.
                } else if code_point & 0xFE0 == 0x360 {
                    if code_point & 0x10 == 0 {
                        self.surrogate = true;
                    } else if prev_surrogate {
                        // Decoding a broken surrogate pair would be lossy.
                        invalid = true;
                    }
                }

                if code_point < 0x20 {
                    invalid = true;
                }
            }
            consume_next!();
        }
        if invalid {
            return Some(Err(EncodingError::CodePoint(code_point)));
        }

        Some(Ok(code_point))
    }
}
