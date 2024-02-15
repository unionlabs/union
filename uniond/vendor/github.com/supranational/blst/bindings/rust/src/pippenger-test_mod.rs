// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

macro_rules! pippenger_test_mod {
    (
        $test_mod:ident,
        $points:ident,
        $point:ty,
        $add_or_double:ident,
        $generator:ident,
        $mult:ident,
    ) => {
        mod $test_mod {
            use super::*;
            use rand::{RngCore, SeedableRng};
            use rand_chacha::ChaCha20Rng;

            #[test]
            fn test_mult() {
                const npoints: usize = 2000;
                const nbits: usize = 160;
                const nbytes: usize = (nbits + 7) / 8;

                let mut scalars = Box::new([0u8; nbytes * npoints]);
                ChaCha20Rng::from_seed([0u8; 32]).fill_bytes(scalars.as_mut());

                let mut points: Vec<$point> = Vec::with_capacity(npoints);
                unsafe { points.set_len(points.capacity()) };

                let mut naive = <$point>::default();
                for i in 0..npoints {
                    unsafe {
                        let mut t = <$point>::default();
                        $mult(
                            &mut points[i],
                            $generator(),
                            &scalars[i * nbytes],
                            core::cmp::min(32, nbits),
                        );
                        $mult(&mut t, &points[i], &scalars[i * nbytes], nbits);
                        $add_or_double(&mut naive, &naive, &t);
                    }
                    if i < 27 {
                        let points = $points::from(&points[0..i + 1]);
                        assert_eq!(naive, points.mult(scalars.as_ref(), nbits));
                    }
                }

                let points = $points::from(&points);

                assert_eq!(naive, points.mult(scalars.as_ref(), nbits));
            }

            #[test]
            fn test_add() {
                const npoints: usize = 2000;
                const nbits: usize = 32;
                const nbytes: usize = (nbits + 7) / 8;

                let mut scalars = Box::new([0u8; nbytes * npoints]);
                ChaCha20Rng::from_seed([0u8; 32]).fill_bytes(scalars.as_mut());

                let mut points: Vec<$point> = Vec::with_capacity(npoints);
                unsafe { points.set_len(points.capacity()) };

                let mut naive = <$point>::default();
                for i in 0..npoints {
                    unsafe {
                        $mult(
                            &mut points[i],
                            $generator(),
                            &scalars[i * nbytes],
                            32,
                        );
                        $add_or_double(&mut naive, &naive, &points[i]);
                    }
                }

                let points = $points::from(&points);
                assert_eq!(naive, points.add());
            }
        }
    };
}
