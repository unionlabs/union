use rand::distributions::Standard;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use rayon::prelude::*;

fn seeded_rng() -> XorShiftRng {
    let mut seed = <XorShiftRng as SeedableRng>::Seed::default();
    (0..).zip(seed.as_mut()).for_each(|(i, x)| *x = i);
    XorShiftRng::from_seed(seed)
}

#[test]
pub fn execute_strings() {
    let rng = seeded_rng();
    let s: String = rng.sample_iter::<char, _>(&Standard).take(1024).collect();

    let par_chars: String = s.par_chars().collect();
    assert_eq!(s, par_chars);

    let par_even: String = s.par_chars().filter(|&c| (c as u32) & 1 == 0).collect();
    let ser_even: String = s.chars().filter(|&c| (c as u32) & 1 == 0).collect();
    assert_eq!(par_even, ser_even);

    // test `FromParallelIterator<&char> for String`
    let vchars: Vec<char> = s.par_chars().collect();
    let par_chars: String = vchars.par_iter().collect();
    assert_eq!(s, par_chars);

    let par_bytes: Vec<u8> = s.par_bytes().collect();
    assert_eq!(s.as_bytes(), &*par_bytes);

    let par_utf16: Vec<u16> = s.par_encode_utf16().collect();
    let ser_utf16: Vec<u16> = s.encode_utf16().collect();
    assert_eq!(par_utf16, ser_utf16);

    let par_charind: Vec<_> = s.par_char_indices().collect();
    let ser_charind: Vec<_> = s.char_indices().collect();
    assert_eq!(par_charind, ser_charind);
}

#[test]
pub fn execute_strings_split() {
    // char testcases from examples in `str::split` etc.,
    // plus a large self-test for good measure.
    let tests = vec![
        ("Mary had a little lamb", ' '),
        ("", 'X'),
        ("lionXXtigerXleopard", 'X'),
        ("||||a||b|c", '|'),
        ("(///)", '/'),
        ("010", '0'),
        ("    a  b c", ' '),
        ("A.B.", '.'),
        ("A..B..", '.'),
        ("foo\r\nbar\n\nbaz\n", '\n'),
        ("foo\nbar\n\r\nbaz", '\n'),
        ("A few words", ' '),
        (" Mary   had\ta\u{2009}little  \n\t lamb", ' '),
        (include_str!("str.rs"), ' '),
    ];

    for &(string, separator) in &tests {
        let serial: Vec<_> = string.split(separator).collect();
        let parallel: Vec<_> = string.par_split(separator).collect();
        assert_eq!(serial, parallel);

        let pattern: &[char] = &['\u{0}', separator, '\u{1F980}'];
        let serial: Vec<_> = string.split(pattern).collect();
        let parallel: Vec<_> = string.par_split(pattern).collect();
        assert_eq!(serial, parallel);

        let serial_fn: Vec<_> = string.split(|c| c == separator).collect();
        let parallel_fn: Vec<_> = string.par_split(|c| c == separator).collect();
        assert_eq!(serial_fn, parallel_fn);
    }

    for &(string, separator) in &tests {
        let serial: Vec<_> = string.split_terminator(separator).collect();
        let parallel: Vec<_> = string.par_split_terminator(separator).collect();
        assert_eq!(serial, parallel);

        let pattern: &[char] = &['\u{0}', separator, '\u{1F980}'];
        let serial: Vec<_> = string.split_terminator(pattern).collect();
        let parallel: Vec<_> = string.par_split_terminator(pattern).collect();
        assert_eq!(serial, parallel);

        let serial: Vec<_> = string.split_terminator(|c| c == separator).collect();
        let parallel: Vec<_> = string.par_split_terminator(|c| c == separator).collect();
        assert_eq!(serial, parallel);
    }

    for &(string, _) in &tests {
        let serial: Vec<_> = string.lines().collect();
        let parallel: Vec<_> = string.par_lines().collect();
        assert_eq!(serial, parallel);
    }

    for &(string, _) in &tests {
        let serial: Vec<_> = string.split_whitespace().collect();
        let parallel: Vec<_> = string.par_split_whitespace().collect();
        assert_eq!(serial, parallel);
    }

    // try matching separators too!
    for &(string, separator) in &tests {
        let serial: Vec<_> = string.matches(separator).collect();
        let parallel: Vec<_> = string.par_matches(separator).collect();
        assert_eq!(serial, parallel);

        let pattern: &[char] = &['\u{0}', separator, '\u{1F980}'];
        let serial: Vec<_> = string.matches(pattern).collect();
        let parallel: Vec<_> = string.par_matches(pattern).collect();
        assert_eq!(serial, parallel);

        let serial_fn: Vec<_> = string.matches(|c| c == separator).collect();
        let parallel_fn: Vec<_> = string.par_matches(|c| c == separator).collect();
        assert_eq!(serial_fn, parallel_fn);
    }

    for &(string, separator) in &tests {
        let serial: Vec<_> = string.match_indices(separator).collect();
        let parallel: Vec<_> = string.par_match_indices(separator).collect();
        assert_eq!(serial, parallel);

        let pattern: &[char] = &['\u{0}', separator, '\u{1F980}'];
        let serial: Vec<_> = string.match_indices(pattern).collect();
        let parallel: Vec<_> = string.par_match_indices(pattern).collect();
        assert_eq!(serial, parallel);

        let serial_fn: Vec<_> = string.match_indices(|c| c == separator).collect();
        let parallel_fn: Vec<_> = string.par_match_indices(|c| c == separator).collect();
        assert_eq!(serial_fn, parallel_fn);
    }
}
