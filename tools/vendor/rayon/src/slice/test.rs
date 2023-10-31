#![cfg(test)]

use crate::prelude::*;
use rand::distributions::Uniform;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::cmp::Ordering::{Equal, Greater, Less};

macro_rules! sort {
    ($f:ident, $name:ident) => {
        #[test]
        fn $name() {
            let rng = &mut thread_rng();

            for len in (0..25).chain(500..501) {
                for &modulus in &[5, 10, 100] {
                    let dist = Uniform::new(0, modulus);
                    for _ in 0..100 {
                        let v: Vec<i32> = rng.sample_iter(&dist).take(len).collect();

                        // Test sort using `<` operator.
                        let mut tmp = v.clone();
                        tmp.$f(|a, b| a.cmp(b));
                        assert!(tmp.windows(2).all(|w| w[0] <= w[1]));

                        // Test sort using `>` operator.
                        let mut tmp = v.clone();
                        tmp.$f(|a, b| b.cmp(a));
                        assert!(tmp.windows(2).all(|w| w[0] >= w[1]));
                    }
                }
            }

            // Test sort with many duplicates.
            for &len in &[1_000, 10_000, 100_000] {
                for &modulus in &[5, 10, 100, 10_000] {
                    let dist = Uniform::new(0, modulus);
                    let mut v: Vec<i32> = rng.sample_iter(&dist).take(len).collect();

                    v.$f(|a, b| a.cmp(b));
                    assert!(v.windows(2).all(|w| w[0] <= w[1]));
                }
            }

            // Test sort with many pre-sorted runs.
            for &len in &[1_000, 10_000, 100_000] {
                let len_dist = Uniform::new(0, len);
                for &modulus in &[5, 10, 1000, 50_000] {
                    let dist = Uniform::new(0, modulus);
                    let mut v: Vec<i32> = rng.sample_iter(&dist).take(len).collect();

                    v.sort();
                    v.reverse();

                    for _ in 0..5 {
                        let a = rng.sample(&len_dist);
                        let b = rng.sample(&len_dist);
                        if a < b {
                            v[a..b].reverse();
                        } else {
                            v.swap(a, b);
                        }
                    }

                    v.$f(|a, b| a.cmp(b));
                    assert!(v.windows(2).all(|w| w[0] <= w[1]));
                }
            }

            // Sort using a completely random comparison function.
            // This will reorder the elements *somehow*, but won't panic.
            let mut v: Vec<_> = (0..100).collect();
            v.$f(|_, _| *[Less, Equal, Greater].choose(&mut thread_rng()).unwrap());
            v.$f(|a, b| a.cmp(b));
            for i in 0..v.len() {
                assert_eq!(v[i], i);
            }

            // Should not panic.
            [0i32; 0].$f(|a, b| a.cmp(b));
            [(); 10].$f(|a, b| a.cmp(b));
            [(); 100].$f(|a, b| a.cmp(b));

            let mut v = [0xDEAD_BEEFu64];
            v.$f(|a, b| a.cmp(b));
            assert!(v == [0xDEAD_BEEF]);
        }
    };
}

sort!(par_sort_by, test_par_sort);
sort!(par_sort_unstable_by, test_par_sort_unstable);

#[test]
fn test_par_sort_stability() {
    for len in (2..25).chain(500..510).chain(50_000..50_010) {
        for _ in 0..10 {
            let mut counts = [0; 10];

            // Create a vector like [(6, 1), (5, 1), (6, 2), ...],
            // where the first item of each tuple is random, but
            // the second item represents which occurrence of that
            // number this element is, i.e. the second elements
            // will occur in sorted order.
            let mut rng = thread_rng();
            let mut v: Vec<_> = (0..len)
                .map(|_| {
                    let n: usize = rng.gen_range(0..10);
                    counts[n] += 1;
                    (n, counts[n])
                })
                .collect();

            // Only sort on the first element, so an unstable sort
            // may mix up the counts.
            v.par_sort_by(|&(a, _), &(b, _)| a.cmp(&b));

            // This comparison includes the count (the second item
            // of the tuple), so elements with equal first items
            // will need to be ordered with increasing
            // counts... i.e. exactly asserting that this sort is
            // stable.
            assert!(v.windows(2).all(|w| w[0] <= w[1]));
        }
    }
}

#[test]
fn test_par_chunks_exact_remainder() {
    let v: &[i32] = &[0, 1, 2, 3, 4];
    let c = v.par_chunks_exact(2);
    assert_eq!(c.remainder(), &[4]);
    assert_eq!(c.len(), 2);
}

#[test]
fn test_par_chunks_exact_mut_remainder() {
    let v: &mut [i32] = &mut [0, 1, 2, 3, 4];
    let mut c = v.par_chunks_exact_mut(2);
    assert_eq!(c.remainder(), &[4]);
    assert_eq!(c.len(), 2);
    assert_eq!(c.into_remainder(), &[4]);

    let mut c = v.par_chunks_exact_mut(2);
    assert_eq!(c.take_remainder(), &[4]);
    assert_eq!(c.take_remainder(), &[]);
    assert_eq!(c.len(), 2);
}

#[test]
fn test_par_rchunks_exact_remainder() {
    let v: &[i32] = &[0, 1, 2, 3, 4];
    let c = v.par_rchunks_exact(2);
    assert_eq!(c.remainder(), &[0]);
    assert_eq!(c.len(), 2);
}

#[test]
fn test_par_rchunks_exact_mut_remainder() {
    let v: &mut [i32] = &mut [0, 1, 2, 3, 4];
    let mut c = v.par_rchunks_exact_mut(2);
    assert_eq!(c.remainder(), &[0]);
    assert_eq!(c.len(), 2);
    assert_eq!(c.into_remainder(), &[0]);

    let mut c = v.par_rchunks_exact_mut(2);
    assert_eq!(c.take_remainder(), &[0]);
    assert_eq!(c.take_remainder(), &[]);
    assert_eq!(c.len(), 2);
}
