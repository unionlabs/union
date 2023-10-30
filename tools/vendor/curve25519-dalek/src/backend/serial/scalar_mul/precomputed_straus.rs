// -*- mode: rust; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2019 Henry de Valence.
// See LICENSE for licensing information.
//
// Authors:
// - Henry de Valence <hdevalence@hdevalence.ca>

//! Precomputation for Straus's method.

#![allow(non_snake_case)]

use core::borrow::Borrow;

use backend::serial::curve_models::{
    AffineNielsPoint, CompletedPoint, ProjectiveNielsPoint, ProjectivePoint,
};
use edwards::EdwardsPoint;
use scalar::Scalar;
use traits::Identity;
use traits::VartimePrecomputedMultiscalarMul;
use window::{NafLookupTable5, NafLookupTable8};

#[allow(unused_imports)]
use prelude::*;

pub struct VartimePrecomputedStraus {
    static_lookup_tables: Vec<NafLookupTable8<AffineNielsPoint>>,
}

impl VartimePrecomputedMultiscalarMul for VartimePrecomputedStraus {
    type Point = EdwardsPoint;

    fn new<I>(static_points: I) -> Self
    where
        I: IntoIterator,
        I::Item: Borrow<Self::Point>,
    {
        Self {
            static_lookup_tables: static_points
                .into_iter()
                .map(|P| NafLookupTable8::<AffineNielsPoint>::from(P.borrow()))
                .collect(),
        }
    }

    fn optional_mixed_multiscalar_mul<I, J, K>(
        &self,
        static_scalars: I,
        dynamic_scalars: J,
        dynamic_points: K,
    ) -> Option<Self::Point>
    where
        I: IntoIterator,
        I::Item: Borrow<Scalar>,
        J: IntoIterator,
        J::Item: Borrow<Scalar>,
        K: IntoIterator<Item = Option<Self::Point>>,
    {
        let static_nafs = static_scalars
            .into_iter()
            .map(|c| c.borrow().non_adjacent_form(5))
            .collect::<Vec<_>>();
        let dynamic_nafs: Vec<_> = dynamic_scalars
            .into_iter()
            .map(|c| c.borrow().non_adjacent_form(5))
            .collect::<Vec<_>>();

        let dynamic_lookup_tables = dynamic_points
            .into_iter()
            .map(|P_opt| P_opt.map(|P| NafLookupTable5::<ProjectiveNielsPoint>::from(&P)))
            .collect::<Option<Vec<_>>>()?;

        let sp = self.static_lookup_tables.len();
        let dp = dynamic_lookup_tables.len();
        assert_eq!(sp, static_nafs.len());
        assert_eq!(dp, dynamic_nafs.len());

        // We could save some doublings by looking for the highest
        // nonzero NAF coefficient, but since we might have a lot of
        // them to search, it's not clear it's worthwhile to check.
        let mut S = ProjectivePoint::identity();
        for j in (0..256).rev() {
            let mut R: CompletedPoint = S.double();

            for i in 0..dp {
                let t_ij = dynamic_nafs[i][j];
                if t_ij > 0 {
                    R = &R.to_extended() + &dynamic_lookup_tables[i].select(t_ij as usize);
                } else if t_ij < 0 {
                    R = &R.to_extended() - &dynamic_lookup_tables[i].select(-t_ij as usize);
                }
            }

            for i in 0..sp {
                let t_ij = static_nafs[i][j];
                if t_ij > 0 {
                    R = &R.to_extended() + &self.static_lookup_tables[i].select(t_ij as usize);
                } else if t_ij < 0 {
                    R = &R.to_extended() - &self.static_lookup_tables[i].select(-t_ij as usize);
                }
            }

            S = R.to_projective();
        }

        Some(S.to_extended())
    }
}
