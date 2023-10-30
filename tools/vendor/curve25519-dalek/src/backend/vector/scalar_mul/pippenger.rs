// -*- mode: rust; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2019 Oleg Andreev
// See LICENSE for licensing information.
//
// Authors:
// - Oleg Andreev <oleganza@gmail.com>

#![allow(non_snake_case)]

use core::borrow::Borrow;

use backend::vector::{CachedPoint, ExtendedPoint};
use edwards::EdwardsPoint;
use scalar::Scalar;
use traits::{Identity, VartimeMultiscalarMul};

#[allow(unused_imports)]
use prelude::*;

/// Implements a version of Pippenger's algorithm.
///
/// See the documentation in the serial `scalar_mul::pippenger` module for details.
pub struct Pippenger;

#[cfg(any(feature = "alloc", feature = "std"))]
impl VartimeMultiscalarMul for Pippenger {
    type Point = EdwardsPoint;

    fn optional_multiscalar_mul<I, J>(scalars: I, points: J) -> Option<EdwardsPoint>
    where
        I: IntoIterator,
        I::Item: Borrow<Scalar>,
        J: IntoIterator<Item = Option<EdwardsPoint>>,
    {
        let mut scalars = scalars.into_iter();
        let size = scalars.by_ref().size_hint().0;
        let w = if size < 500 {
            6
        } else if size < 800 {
            7
        } else {
            8
        };

        let max_digit: usize = 1 << w;
        let digits_count: usize = Scalar::to_radix_2w_size_hint(w);
        let buckets_count: usize = max_digit / 2; // digits are signed+centered hence 2^w/2, excluding 0-th bucket

        // Collect optimized scalars and points in a buffer for repeated access
        // (scanning the whole collection per each digit position).
        let scalars = scalars
            .into_iter()
            .map(|s| s.borrow().to_radix_2w(w));

        let points = points
            .into_iter()
            .map(|p| p.map(|P| CachedPoint::from(ExtendedPoint::from(P))));

        let scalars_points = scalars
            .zip(points)
            .map(|(s, maybe_p)| maybe_p.map(|p| (s, p)))
            .collect::<Option<Vec<_>>>()?;

        // Prepare 2^w/2 buckets.
        // buckets[i] corresponds to a multiplication factor (i+1).
        let mut buckets: Vec<ExtendedPoint> = (0..buckets_count)
            .map(|_| ExtendedPoint::identity())
            .collect();

        let mut columns = (0..digits_count).rev().map(|digit_index| {
            // Clear the buckets when processing another digit.
            for i in 0..buckets_count {
                buckets[i] = ExtendedPoint::identity();
            }

            // Iterate over pairs of (point, scalar)
            // and add/sub the point to the corresponding bucket.
            // Note: if we add support for precomputed lookup tables,
            // we'll be adding/subtractiong point premultiplied by `digits[i]` to buckets[0].
            for (digits, pt) in scalars_points.iter() {
                // Widen digit so that we don't run into edge cases when w=8.
                let digit = digits[digit_index] as i16;
                if digit > 0 {
                    let b = (digit - 1) as usize;
                    buckets[b] = &buckets[b] + pt;
                } else if digit < 0 {
                    let b = (-digit - 1) as usize;
                    buckets[b] = &buckets[b] - pt;
                }
            }

            // Add the buckets applying the multiplication factor to each bucket.
            // The most efficient way to do that is to have a single sum with two running sums:
            // an intermediate sum from last bucket to the first, and a sum of intermediate sums.
            //
            // For example, to add buckets 1*A, 2*B, 3*C we need to add these points:
            //   C
            //   C B
            //   C B A   Sum = C + (C+B) + (C+B+A)
            let mut buckets_intermediate_sum = buckets[buckets_count - 1];
            let mut buckets_sum = buckets[buckets_count - 1];
            for i in (0..(buckets_count - 1)).rev() {
                buckets_intermediate_sum =
                    &buckets_intermediate_sum + &CachedPoint::from(buckets[i]);
                buckets_sum = &buckets_sum + &CachedPoint::from(buckets_intermediate_sum);
            }

            buckets_sum
        });

        // Take the high column as an initial value to avoid wasting time doubling the identity element in `fold()`.
        // `unwrap()` always succeeds because we know we have more than zero digits.
        let hi_column = columns.next().unwrap();

        Some(
            columns
                .fold(hi_column, |total, p| {
                    &total.mul_by_pow_2(w as u32) + &CachedPoint::from(p)
                })
                .into(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use constants;
    use scalar::Scalar;

    #[test]
    fn test_vartime_pippenger() {
        // Reuse points across different tests
        let mut n = 512;
        let x = Scalar::from(2128506u64).invert();
        let y = Scalar::from(4443282u64).invert();
        let points: Vec<_> = (0..n)
            .map(|i| constants::ED25519_BASEPOINT_POINT * Scalar::from(1 + i as u64))
            .collect();
        let scalars: Vec<_> = (0..n)
            .map(|i| x + (Scalar::from(i as u64) * y)) // fast way to make ~random but deterministic scalars
            .collect();

        let premultiplied: Vec<EdwardsPoint> = scalars
            .iter()
            .zip(points.iter())
            .map(|(sc, pt)| sc * pt)
            .collect();

        while n > 0 {
            let scalars = &scalars[0..n].to_vec();
            let points = &points[0..n].to_vec();
            let control: EdwardsPoint = premultiplied[0..n].iter().sum();

            let subject = Pippenger::vartime_multiscalar_mul(scalars.clone(), points.clone());

            assert_eq!(subject.compress(), control.compress());

            n = n / 2;
        }
    }
}
