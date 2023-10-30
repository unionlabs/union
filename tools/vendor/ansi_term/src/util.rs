use display::*;
use std::ops::Deref;

/// Return a substring of the given ANSIStrings sequence, while keeping the formatting.
pub fn sub_string<'a>(start: usize, len: usize, strs: &ANSIStrings<'a>) -> Vec<ANSIString<'static>> {
    let mut vec = Vec::new();
    let mut pos = start;
    let mut len_rem = len;

    for i in strs.0.iter() {
        let fragment = i.deref();
        let frag_len = fragment.len();
        if pos >= frag_len {
            pos -= frag_len;
            continue;
        }
        if len_rem <= 0 {
            break;
        }

        let end = pos + len_rem;
        let pos_end = if end >= frag_len { frag_len } else { end };

        vec.push(i.style_ref().paint(String::from(&fragment[pos..pos_end])));

        if end <= frag_len {
            break;
        }

        len_rem -= pos_end - pos;
        pos = 0;
    }

    vec
}

/// Return a concatenated copy of `strs` without the formatting, as an allocated `String`.
pub fn unstyle(strs: &ANSIStrings) -> String {
    let mut s = String::new();

    for i in strs.0.iter() {
        s += &i.deref();
    }

    s
}

/// Return the unstyled length of ANSIStrings. This is equaivalent to `unstyle(strs).len()`.
pub fn unstyled_len(strs: &ANSIStrings) -> usize {
    let mut l = 0;
    for i in strs.0.iter() {
        l += i.deref().len();
    }
    l
}

#[cfg(test)]
mod test {
    use Colour::*;
    use display::*;
    use super::*;

    #[test]
    fn test() {
        let l = [
            Black.paint("first"),
            Red.paint("-second"),
            White.paint("-third"),
        ];
        let a = ANSIStrings(&l);
        assert_eq!(unstyle(&a), "first-second-third");
        assert_eq!(unstyled_len(&a), 18);

        let l2 = [
            Black.paint("st"),
            Red.paint("-second"),
            White.paint("-t"),
        ];
        assert_eq!(sub_string(3, 11, &a).as_slice(), &l2);
    }
}
