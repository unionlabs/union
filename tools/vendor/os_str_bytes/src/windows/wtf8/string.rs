use crate::util;

const SURROGATE_LENGTH: usize = 3;

pub(crate) fn ends_with(string: &[u8], mut suffix: &[u8]) -> bool {
    let index = if let Some(index) = string.len().checked_sub(suffix.len()) {
        index
    } else {
        return false;
    };
    if let Some(&byte) = string.get(index) {
        if util::is_continuation(byte) {
            let index = expect_encoded!(index.checked_sub(1));
            let mut wide_surrogate =
                if let Some(surrogate) = suffix.get(..SURROGATE_LENGTH) {
                    super::encode_wide(surrogate)
                } else {
                    return false;
                };
            let surrogate_wchar = wide_surrogate
                .next()
                .expect("failed decoding non-empty suffix");

            if wide_surrogate.next().is_some()
                || super::encode_wide(&string[index..])
                    .take_while(Result::is_ok)
                    .nth(1)
                    != Some(surrogate_wchar)
            {
                return false;
            }
            suffix = &suffix[SURROGATE_LENGTH..];
        }
    }
    string.ends_with(suffix)
}

pub(crate) fn starts_with(string: &[u8], mut prefix: &[u8]) -> bool {
    if let Some(&byte) = string.get(prefix.len()) {
        if util::is_continuation(byte) {
            let index = if let Some(index) =
                prefix.len().checked_sub(SURROGATE_LENGTH)
            {
                index
            } else {
                return false;
            };
            let (substring, surrogate) = prefix.split_at(index);
            let mut wide_surrogate = super::encode_wide(surrogate);
            let surrogate_wchar = wide_surrogate
                .next()
                .expect("failed decoding non-empty prefix");

            if surrogate_wchar.is_err()
                || wide_surrogate.next().is_some()
                || super::encode_wide(&string[index..])
                    .next()
                    .expect("failed decoding non-empty substring")
                    != surrogate_wchar
            {
                return false;
            }
            prefix = substring;
        }
    }
    string.starts_with(prefix)
}
