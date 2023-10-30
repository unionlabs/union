use quickcheck::quickcheck;

use crate::{tests::memchr::testdata::memchr_tests, Memchr, Memchr2, Memchr3};

#[test]
fn memchr1_iter() {
    for test in memchr_tests() {
        test.iter_one(false, Memchr::new);
    }
}

#[test]
fn memchr2_iter() {
    for test in memchr_tests() {
        test.iter_two(false, Memchr2::new);
    }
}

#[test]
fn memchr3_iter() {
    for test in memchr_tests() {
        test.iter_three(false, Memchr3::new);
    }
}

#[test]
fn memrchr1_iter() {
    for test in memchr_tests() {
        test.iter_one(true, |n1, corpus| Memchr::new(n1, corpus).rev());
    }
}

#[test]
fn memrchr2_iter() {
    for test in memchr_tests() {
        test.iter_two(true, |n1, n2, corpus| {
            Memchr2::new(n1, n2, corpus).rev()
        })
    }
}

#[test]
fn memrchr3_iter() {
    for test in memchr_tests() {
        test.iter_three(true, |n1, n2, n3, corpus| {
            Memchr3::new(n1, n2, n3, corpus).rev()
        })
    }
}

quickcheck! {
    fn qc_memchr_double_ended_iter(
        needle: u8, data: Vec<u8>, take_side: Vec<bool>
    ) -> bool {
        // make nonempty
        let mut take_side = take_side;
        if take_side.is_empty() { take_side.push(true) };

        let iter = Memchr::new(needle, &data);
        let all_found = double_ended_take(
            iter, take_side.iter().cycle().cloned());

        all_found.iter().cloned().eq(positions1(needle, &data))
    }

    fn qc_memchr2_double_ended_iter(
        needle1: u8, needle2: u8, data: Vec<u8>, take_side: Vec<bool>
    ) -> bool {
        // make nonempty
        let mut take_side = take_side;
        if take_side.is_empty() { take_side.push(true) };

        let iter = Memchr2::new(needle1, needle2, &data);
        let all_found = double_ended_take(
            iter, take_side.iter().cycle().cloned());

        all_found.iter().cloned().eq(positions2(needle1, needle2, &data))
    }

    fn qc_memchr3_double_ended_iter(
        needle1: u8, needle2: u8, needle3: u8,
        data: Vec<u8>, take_side: Vec<bool>
    ) -> bool {
        // make nonempty
        let mut take_side = take_side;
        if take_side.is_empty() { take_side.push(true) };

        let iter = Memchr3::new(needle1, needle2, needle3, &data);
        let all_found = double_ended_take(
            iter, take_side.iter().cycle().cloned());

        all_found
            .iter()
            .cloned()
            .eq(positions3(needle1, needle2, needle3, &data))
    }

    fn qc_memchr1_iter(data: Vec<u8>) -> bool {
        let needle = 0;
        let answer = positions1(needle, &data);
        answer.eq(Memchr::new(needle, &data))
    }

    fn qc_memchr1_rev_iter(data: Vec<u8>) -> bool {
        let needle = 0;
        let answer = positions1(needle, &data);
        answer.rev().eq(Memchr::new(needle, &data).rev())
    }

    fn qc_memchr2_iter(data: Vec<u8>) -> bool {
        let needle1 = 0;
        let needle2 = 1;
        let answer = positions2(needle1, needle2, &data);
        answer.eq(Memchr2::new(needle1, needle2, &data))
    }

    fn qc_memchr2_rev_iter(data: Vec<u8>) -> bool {
        let needle1 = 0;
        let needle2 = 1;
        let answer = positions2(needle1, needle2, &data);
        answer.rev().eq(Memchr2::new(needle1, needle2, &data).rev())
    }

    fn qc_memchr3_iter(data: Vec<u8>) -> bool {
        let needle1 = 0;
        let needle2 = 1;
        let needle3 = 2;
        let answer = positions3(needle1, needle2, needle3, &data);
        answer.eq(Memchr3::new(needle1, needle2, needle3, &data))
    }

    fn qc_memchr3_rev_iter(data: Vec<u8>) -> bool {
        let needle1 = 0;
        let needle2 = 1;
        let needle3 = 2;
        let answer = positions3(needle1, needle2, needle3, &data);
        answer.rev().eq(Memchr3::new(needle1, needle2, needle3, &data).rev())
    }

    fn qc_memchr1_iter_size_hint(data: Vec<u8>) -> bool {
        // test that the size hint is within reasonable bounds
        let needle = 0;
        let mut iter = Memchr::new(needle, &data);
        let mut real_count = data
            .iter()
            .filter(|&&elt| elt == needle)
            .count();

        while let Some(index) = iter.next() {
            real_count -= 1;
            let (lower, upper) = iter.size_hint();
            assert!(lower <= real_count);
            assert!(upper.unwrap() >= real_count);
            assert!(upper.unwrap() <= data.len() - index);
        }
        true
    }
}

// take items from a DEI, taking front for each true and back for each false.
// Return a vector with the concatenation of the fronts and the reverse of the
// backs.
fn double_ended_take<I, J>(mut iter: I, take_side: J) -> Vec<I::Item>
where
    I: DoubleEndedIterator,
    J: Iterator<Item = bool>,
{
    let mut found_front = Vec::new();
    let mut found_back = Vec::new();

    for take_front in take_side {
        if take_front {
            if let Some(pos) = iter.next() {
                found_front.push(pos);
            } else {
                break;
            }
        } else {
            if let Some(pos) = iter.next_back() {
                found_back.push(pos);
            } else {
                break;
            }
        };
    }

    let mut all_found = found_front;
    all_found.extend(found_back.into_iter().rev());
    all_found
}

// return an iterator of the 0-based indices of haystack that match the needle
fn positions1<'a>(
    n1: u8,
    haystack: &'a [u8],
) -> Box<dyn DoubleEndedIterator<Item = usize> + 'a> {
    let it = haystack
        .iter()
        .enumerate()
        .filter(move |&(_, &b)| b == n1)
        .map(|t| t.0);
    Box::new(it)
}

fn positions2<'a>(
    n1: u8,
    n2: u8,
    haystack: &'a [u8],
) -> Box<dyn DoubleEndedIterator<Item = usize> + 'a> {
    let it = haystack
        .iter()
        .enumerate()
        .filter(move |&(_, &b)| b == n1 || b == n2)
        .map(|t| t.0);
    Box::new(it)
}

fn positions3<'a>(
    n1: u8,
    n2: u8,
    n3: u8,
    haystack: &'a [u8],
) -> Box<dyn DoubleEndedIterator<Item = usize> + 'a> {
    let it = haystack
        .iter()
        .enumerate()
        .filter(move |&(_, &b)| b == n1 || b == n2 || b == n3)
        .map(|t| t.0);
    Box::new(it)
}
