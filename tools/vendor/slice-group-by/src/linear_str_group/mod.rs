macro_rules! str_group_by_wrapped {
    (struct $name:ident, $elem:ty) => {
        impl<'a> $name<'a> {
            #[inline]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            #[inline]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[inline]
            pub fn remainder_len(&self) -> usize {
                self.0.remainder_len()
            }
        }

        impl<'a> std::iter::Iterator for $name<'a> {
            type Item = $elem;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }

            fn last(self) -> Option<Self::Item> {
                self.0.last()
            }
        }

        impl<'a> DoubleEndedIterator for $name<'a> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.0.next_back()
            }
        }

        impl<'a> std::iter::FusedIterator for $name<'a> { }
    }
}

mod linear_str_group;
mod linear_str_group_by;
mod linear_str_group_by_key;

pub use self::linear_str_group::{LinearStrGroup, LinearStrGroupMut};
pub use self::linear_str_group_by::{LinearStrGroupBy, LinearStrGroupByMut};
pub use self::linear_str_group_by_key::{LinearStrGroupByKey, LinearStrGroupByKeyMut};

fn str_as_ptr(string: &str) -> *const u8 {
    string.as_bytes().as_ptr()
}

fn str_as_mut_ptr(string: &mut str) -> *mut u8 {
    unsafe { string.as_bytes_mut().as_mut_ptr() }
}

unsafe fn str_from_raw_parts<'a>(data: *const u8, len: usize) -> &'a str {
    let slice = std::slice::from_raw_parts(data, len);
    std::str::from_utf8_unchecked(slice)
}

unsafe fn str_from_raw_parts_mut<'a>(data: *mut u8, len: usize) -> &'a mut str {
    let slice = std::slice::from_raw_parts_mut(data, len);
    std::str::from_utf8_unchecked_mut(slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_easy() {
        let string = "aaaabbbbbaacccc";

        let mut iter = LinearStrGroup::new(string);

        assert_eq!(iter.next(), Some("aaaa"));
        assert_eq!(iter.next(), Some("bbbbb"));
        assert_eq!(iter.next(), Some("aa"));
        assert_eq!(iter.next(), Some("cccc"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn str_mut_easy() {
        let mut string = String::from("aaaabbbbbaacccc");

        let mut iter = LinearStrGroupMut::new(&mut string);

        assert_eq!(iter.next().map(|s| &*s), Some("aaaa"));
        assert_eq!(iter.next().map(|s| &*s), Some("bbbbb"));
        assert_eq!(iter.next().map(|s| &*s), Some("aa"));
        assert_eq!(iter.next().map(|s| &*s), Some("cccc"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn str_kanji() {
        let string = "包包饰饰与与钥钥匙匙扣扣";

        let mut iter = LinearStrGroup::new(string);

        assert_eq!(iter.next(), Some("包包"));
        assert_eq!(iter.next(), Some("饰饰"));
        assert_eq!(iter.next(), Some("与与"));
        assert_eq!(iter.next(), Some("钥钥"));
        assert_eq!(iter.next(), Some("匙匙"));
        assert_eq!(iter.next(), Some("扣扣"));
        assert_eq!(iter.next(), None);
    }

    fn is_cjk(c: char) -> bool {
        (c >= '\u{2e80}' && c <= '\u{2eff}') ||
        (c >= '\u{2f00}' && c <= '\u{2fdf}') ||
        (c >= '\u{3040}' && c <= '\u{309f}') ||
        (c >= '\u{30a0}' && c <= '\u{30ff}') ||
        (c >= '\u{3100}' && c <= '\u{312f}') ||
        (c >= '\u{3200}' && c <= '\u{32ff}') ||
        (c >= '\u{3400}' && c <= '\u{4dbf}') ||
        (c >= '\u{4e00}' && c <= '\u{9fff}') ||
        (c >= '\u{f900}' && c <= '\u{faff}')
    }

    #[test]
    fn str_ascii_cjk() {
        let string = "abc包包bbccdd饰饰";

        let mut iter = LinearStrGroupBy::new(string, |a, b| is_cjk(a) == is_cjk(b));

        assert_eq!(iter.next(), Some("abc"));
        assert_eq!(iter.next(), Some("包包"));
        assert_eq!(iter.next(), Some("bbccdd"));
        assert_eq!(iter.next(), Some("饰饰"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn str_rev_easy() {
        let string = "aaaabbbbbaacccc";

        let mut iter = LinearStrGroup::new(string).rev();

        assert_eq!(iter.next(), Some("cccc"));
        assert_eq!(iter.next(), Some("aa"));
        assert_eq!(iter.next(), Some("bbbbb"));
        assert_eq!(iter.next(), Some("aaaa"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn str_mut_rev_easy() {
        let mut string = String::from("aaaabbbbbaacccc");

        let mut iter = LinearStrGroupMut::new(&mut string).rev();

        assert_eq!(iter.next().map(|s| &*s), Some("cccc"));
        assert_eq!(iter.next().map(|s| &*s), Some("aa"));
        assert_eq!(iter.next().map(|s| &*s), Some("bbbbb"));
        assert_eq!(iter.next().map(|s| &*s), Some("aaaa"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn str_rev_ascii_cjk() {
        let string = "abc包包bbccdd饰饰";

        let mut iter = LinearStrGroupBy::new(string, |a, b| is_cjk(a) == is_cjk(b)).rev();

        assert_eq!(iter.next(), Some("饰饰"));
        assert_eq!(iter.next(), Some("bbccdd"));
        assert_eq!(iter.next(), Some("包包"));
        assert_eq!(iter.next(), Some("abc"));
        assert_eq!(iter.next(), None);
    }
}
