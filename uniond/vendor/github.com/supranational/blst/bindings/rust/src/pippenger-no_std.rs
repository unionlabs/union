// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;

macro_rules! pippenger_mult_impl {
    (
        $points:ident,
        $point:ty,
        $point_affine:ty,
        $to_affines:ident,
        $scratch_sizeof:ident,
        $multi_scalar_mult:ident,
        $tile_mult:ident,
        $add_or_double:ident,
        $double:ident,
        $test_mod:ident,
        $generator:ident,
        $mult:ident,
        $add:ident,
        $is_inf:ident,
        $in_group:ident,
    ) => {
        pub struct $points {
            points: Vec<$point_affine>,
        }

        impl<I: SliceIndex<[$point_affine]>> Index<I> for $points {
            type Output = I::Output;

            #[inline]
            fn index(&self, i: I) -> &Self::Output {
                &self.points[i]
            }
        }
        impl<I: SliceIndex<[$point_affine]>> IndexMut<I> for $points {
            #[inline]
            fn index_mut(&mut self, i: I) -> &mut Self::Output {
                &mut self.points[i]
            }
        }

        impl $points {
            #[inline]
            pub fn as_slice(&self) -> &[$point_affine] {
                self.points.as_slice()
            }

            pub fn from(points: &[$point]) -> Self {
                let npoints = points.len();
                let mut ret = Self {
                    points: Vec::with_capacity(npoints),
                };
                #[allow(clippy::uninit_vec)]
                unsafe { ret.points.set_len(npoints) };

                let p: [*const $point; 2] = [&points[0], ptr::null()];
                unsafe { $to_affines(&mut ret.points[0], &p[0], npoints) };
                ret
            }

            #[inline]
            pub fn mult(&self, scalars: &[u8], nbits: usize) -> $point {
                self.as_slice().mult(scalars, nbits)
            }

            #[inline]
            pub fn add(&self) -> $point {
                self.as_slice().add()
            }
        }

        impl MultiPoint for [$point_affine] {
            type Output = $point;

            fn mult(&self, scalars: &[u8], nbits: usize) -> $point {
                let npoints = self.len();
                let nbytes = (nbits + 7) / 8;

                if scalars.len() < nbytes * npoints {
                    panic!("scalars length mismatch");
                }

                let p: [*const $point_affine; 2] = [&self[0], ptr::null()];
                let s: [*const u8; 2] = [&scalars[0], ptr::null()];

                let mut ret = <$point>::default();
                unsafe {
                    let mut scratch: Vec<u64> =
                        Vec::with_capacity($scratch_sizeof(npoints) / 8);
                    #[allow(clippy::uninit_vec)]
                    scratch.set_len(scratch.capacity());
                    $multi_scalar_mult(
                        &mut ret,
                        &p[0],
                        npoints,
                        &s[0],
                        nbits,
                        &mut scratch[0],
                    );
                }
                ret
            }

            fn add(&self) -> $point {
                let npoints = self.len();

                let p: [*const _; 2] = [&self[0], ptr::null()];
                let mut ret = <$point>::default();
                unsafe { $add(&mut ret, &p[0], npoints) };

                ret
            }

            fn validate(&self) -> Result<(), BLST_ERROR> {
                for i in 0..self.len() {
                    if unsafe { $is_inf(&self[i]) } {
                        return Err(BLST_ERROR::BLST_PK_IS_INFINITY);
                    }
                    if !unsafe { $in_group(&self[i]) } {
                        return Err(BLST_ERROR::BLST_POINT_NOT_IN_GROUP);
                    }
                }
                Ok(())
            }
        }

        #[cfg(test)]
        pippenger_test_mod!(
            $test_mod,
            $points,
            $point,
            $add_or_double,
            $generator,
            $mult,
        );
    };
}

#[cfg(test)]
include!("pippenger-test_mod.rs");

pippenger_mult_impl!(
    p1_affines,
    blst_p1,
    blst_p1_affine,
    blst_p1s_to_affine,
    blst_p1s_mult_pippenger_scratch_sizeof,
    blst_p1s_mult_pippenger,
    blst_p1s_tile_pippenger,
    blst_p1_add_or_double,
    blst_p1_double,
    p1_multi_point,
    blst_p1_generator,
    blst_p1_mult,
    blst_p1s_add,
    blst_p1_affine_is_inf,
    blst_p1_affine_in_g1,
);

pippenger_mult_impl!(
    p2_affines,
    blst_p2,
    blst_p2_affine,
    blst_p2s_to_affine,
    blst_p2s_mult_pippenger_scratch_sizeof,
    blst_p2s_mult_pippenger,
    blst_p2s_tile_pippenger,
    blst_p2_add_or_double,
    blst_p2_double,
    p2_multi_point,
    blst_p2_generator,
    blst_p2_mult,
    blst_p2s_add,
    blst_p2_affine_is_inf,
    blst_p2_affine_in_g2,
);
