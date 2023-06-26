// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::any::Any;
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::*;
use std::num::Wrapping;
use std::sync::{mpsc::channel, Arc, Barrier};
use zeroize::Zeroize;

#[cfg(not(feature = "no-threads"))]
trait ThreadPoolExt {
    fn joined_execute<'any, F>(&self, job: F)
    where
        F: FnOnce() + Send + 'any;
}
#[cfg(not(feature = "no-threads"))]
mod mt {
    use super::*;
    use core::mem::transmute;
    use std::sync::{Mutex, Once};
    use threadpool::ThreadPool;

    pub fn da_pool() -> ThreadPool {
        static INIT: Once = Once::new();
        static mut POOL: *const Mutex<ThreadPool> =
            0 as *const Mutex<ThreadPool>;

        INIT.call_once(|| {
            let pool = Mutex::new(ThreadPool::default());
            unsafe { POOL = transmute(Box::new(pool)) };
        });
        unsafe { (*POOL).lock().unwrap().clone() }
    }

    type Thunk<'any> = Box<dyn FnOnce() + Send + 'any>;

    impl ThreadPoolExt for ThreadPool {
        fn joined_execute<'scope, F>(&self, job: F)
        where
            F: FnOnce() + Send + 'scope,
        {
            // Bypass 'lifetime limitations by brute force. It works,
            // because we explicitly join the threads...
            self.execute(unsafe {
                transmute::<Thunk<'scope>, Thunk<'static>>(Box::new(job))
            })
        }
    }
}

#[cfg(feature = "no-threads")]
mod mt {
    pub struct EmptyPool {}

    pub fn da_pool() -> EmptyPool {
        EmptyPool {}
    }

    impl EmptyPool {
        pub fn max_count(&self) -> usize {
            1
        }
        pub fn joined_execute<'scope, F>(&self, job: F)
        where
            F: FnOnce() + Send + 'scope,
        {
            job()
        }
    }
}

include!("bindings.rs");

impl PartialEq for blst_p1 {
    fn eq(&self, other: &Self) -> bool {
        unsafe { blst_p1_is_equal(self, other) }
    }
}

impl PartialEq for blst_p1_affine {
    fn eq(&self, other: &Self) -> bool {
        unsafe { blst_p1_affine_is_equal(self, other) }
    }
}

impl PartialEq for blst_p2 {
    fn eq(&self, other: &Self) -> bool {
        unsafe { blst_p2_is_equal(self, other) }
    }
}

impl PartialEq for blst_p2_affine {
    fn eq(&self, other: &Self) -> bool {
        unsafe { blst_p2_affine_is_equal(self, other) }
    }
}

impl Default for blst_fp12 {
    fn default() -> Self {
        unsafe { *blst_fp12_one() }
    }
}

impl PartialEq for blst_fp12 {
    fn eq(&self, other: &Self) -> bool {
        unsafe { blst_fp12_is_equal(self, other) }
    }
}

impl core::ops::Mul for blst_fp12 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut out = MaybeUninit::<blst_fp12>::uninit();
        unsafe {
            blst_fp12_mul(out.as_mut_ptr(), &self, &other);
            out.assume_init()
        }
    }
}

impl core::ops::MulAssign for blst_fp12 {
    fn mul_assign(&mut self, other: Self) {
        unsafe { blst_fp12_mul(self, self, &other) }
    }
}

impl blst_fp12 {
    pub fn miller_loop(q: &blst_p2_affine, p: &blst_p1_affine) -> Self {
        let mut out = MaybeUninit::<blst_fp12>::uninit();
        unsafe {
            blst_miller_loop(out.as_mut_ptr(), q, p);
            out.assume_init()
        }
    }

    pub fn final_exp(&self) -> Self {
        let mut out = MaybeUninit::<blst_fp12>::uninit();
        unsafe {
            blst_final_exp(out.as_mut_ptr(), self);
            out.assume_init()
        }
    }

    pub fn in_group(&self) -> bool {
        unsafe { blst_fp12_in_group(self) }
    }

    pub fn finalverify(a: &Self, b: &Self) -> bool {
        unsafe { blst_fp12_finalverify(a, b) }
    }

    pub fn to_bendian(&self) -> [u8; 48 * 12] {
        let mut out = MaybeUninit::<[u8; 48 * 12]>::uninit();
        unsafe {
            blst_bendian_from_fp12(out.as_mut_ptr() as *mut u8, self);
            out.assume_init()
        }
    }
}

impl blst_scalar {
    pub fn hash_to(msg: &[u8], dst: &[u8]) -> Option<Self> {
        unsafe {
            let mut out = <Self>::default();
            let mut elem: [u8; 48] = MaybeUninit::uninit().assume_init();
            blst_expand_message_xmd(
                elem.as_mut_ptr(),
                elem.len(),
                msg.as_ptr(),
                msg.len(),
                dst.as_ptr(),
                dst.len(),
            );
            if blst_scalar_from_be_bytes(&mut out, elem.as_ptr(), elem.len()) {
                Some(out)
            } else {
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct Pairing {
    v: Box<[u64]>,
}

impl Pairing {
    pub fn new(hash_or_encode: bool, dst: &[u8]) -> Self {
        let v: Vec<u64> = vec![0; unsafe { blst_pairing_sizeof() } / 8];
        let mut obj = Self {
            v: v.into_boxed_slice(),
        };
        obj.init(hash_or_encode, dst);
        obj
    }

    pub fn init(&mut self, hash_or_encode: bool, dst: &[u8]) {
        unsafe {
            blst_pairing_init(
                self.ctx(),
                hash_or_encode,
                dst.as_ptr(),
                dst.len(),
            )
        }
    }
    fn ctx(&mut self) -> *mut blst_pairing {
        self.v.as_mut_ptr() as *mut blst_pairing
    }
    fn const_ctx(&self) -> *const blst_pairing {
        self.v.as_ptr() as *const blst_pairing
    }

    pub fn aggregate(
        &mut self,
        pk: &dyn Any,
        pk_validate: bool,
        sig: &dyn Any,
        sig_groupcheck: bool,
        msg: &[u8],
        aug: &[u8],
    ) -> BLST_ERROR {
        if pk.is::<blst_p1_affine>() {
            unsafe {
                blst_pairing_chk_n_aggr_pk_in_g1(
                    self.ctx(),
                    match pk.downcast_ref::<blst_p1_affine>() {
                        Some(pk) => pk,
                        None => ptr::null(),
                    },
                    pk_validate,
                    match sig.downcast_ref::<blst_p2_affine>() {
                        Some(sig) => sig,
                        None => ptr::null(),
                    },
                    sig_groupcheck,
                    msg.as_ptr(),
                    msg.len(),
                    aug.as_ptr(),
                    aug.len(),
                )
            }
        } else if pk.is::<blst_p2_affine>() {
            unsafe {
                blst_pairing_chk_n_aggr_pk_in_g2(
                    self.ctx(),
                    match pk.downcast_ref::<blst_p2_affine>() {
                        Some(pk) => pk,
                        None => ptr::null(),
                    },
                    pk_validate,
                    match sig.downcast_ref::<blst_p1_affine>() {
                        Some(sig) => sig,
                        None => ptr::null(),
                    },
                    sig_groupcheck,
                    msg.as_ptr(),
                    msg.len(),
                    aug.as_ptr(),
                    aug.len(),
                )
            }
        } else {
            panic!("whaaaa?")
        }
    }

    pub fn mul_n_aggregate(
        &mut self,
        pk: &dyn Any,
        pk_validate: bool,
        sig: &dyn Any,
        sig_groupcheck: bool,
        scalar: &[u8],
        nbits: usize,
        msg: &[u8],
        aug: &[u8],
    ) -> BLST_ERROR {
        if pk.is::<blst_p1_affine>() {
            unsafe {
                blst_pairing_chk_n_mul_n_aggr_pk_in_g1(
                    self.ctx(),
                    match pk.downcast_ref::<blst_p1_affine>() {
                        Some(pk) => pk,
                        None => ptr::null(),
                    },
                    pk_validate,
                    match sig.downcast_ref::<blst_p2_affine>() {
                        Some(sig) => sig,
                        None => ptr::null(),
                    },
                    sig_groupcheck,
                    scalar.as_ptr(),
                    nbits,
                    msg.as_ptr(),
                    msg.len(),
                    aug.as_ptr(),
                    aug.len(),
                )
            }
        } else if pk.is::<blst_p2_affine>() {
            unsafe {
                blst_pairing_chk_n_mul_n_aggr_pk_in_g2(
                    self.ctx(),
                    match pk.downcast_ref::<blst_p2_affine>() {
                        Some(pk) => pk,
                        None => ptr::null(),
                    },
                    pk_validate,
                    match sig.downcast_ref::<blst_p1_affine>() {
                        Some(sig) => sig,
                        None => ptr::null(),
                    },
                    sig_groupcheck,
                    scalar.as_ptr(),
                    nbits,
                    msg.as_ptr(),
                    msg.len(),
                    aug.as_ptr(),
                    aug.len(),
                )
            }
        } else {
            panic!("whaaaa?")
        }
    }

    pub fn aggregated(gtsig: &mut blst_fp12, sig: &dyn Any) {
        if sig.is::<blst_p1_affine>() {
            unsafe {
                blst_aggregated_in_g1(
                    gtsig,
                    sig.downcast_ref::<blst_p1_affine>().unwrap(),
                )
            }
        } else if sig.is::<blst_p2_affine>() {
            unsafe {
                blst_aggregated_in_g2(
                    gtsig,
                    sig.downcast_ref::<blst_p2_affine>().unwrap(),
                )
            }
        } else {
            panic!("whaaaa?")
        }
    }

    pub fn commit(&mut self) {
        unsafe { blst_pairing_commit(self.ctx()) }
    }

    pub fn merge(&mut self, ctx1: &Self) -> BLST_ERROR {
        unsafe { blst_pairing_merge(self.ctx(), ctx1.const_ctx()) }
    }

    pub fn finalverify(&self, gtsig: Option<&blst_fp12>) -> bool {
        unsafe {
            blst_pairing_finalverify(
                self.const_ctx(),
                match gtsig {
                    Some(gtsig) => gtsig,
                    None => ptr::null(),
                },
            )
        }
    }

    pub fn raw_aggregate(&mut self, q: &blst_p2_affine, p: &blst_p1_affine) {
        unsafe { blst_pairing_raw_aggregate(self.ctx(), q, p) }
    }

    pub fn as_fp12(&mut self) -> blst_fp12 {
        unsafe { *blst_pairing_as_fp12(self.ctx()) }
    }
}

pub fn uniq(msgs: &[&[u8]]) -> bool {
    let n_elems = msgs.len();

    if n_elems == 1 {
        return true;
    } else if n_elems == 2 {
        return msgs[0] != msgs[1];
    }

    let mut v: Vec<u64> = vec![0; unsafe { blst_uniq_sizeof(n_elems) } / 8];
    let ctx = v.as_mut_ptr() as *mut blst_uniq;

    unsafe { blst_uniq_init(ctx) };

    for msg in msgs.iter() {
        if !unsafe { blst_uniq_test(ctx, msg.as_ptr(), msg.len()) } {
            return false;
        }
    }

    true
}

pub fn print_bytes(bytes: &[u8], name: &str) {
    print!("{} ", name);
    for b in bytes.iter() {
        print!("{:02x}", b);
    }
    println!();
}

macro_rules! sig_variant_impl {
    (
        $name:expr,
        $pk:ty,
        $pk_aff:ty,
        $sig:ty,
        $sig_aff:ty,
        $sk_to_pk:ident,
        $hash_or_encode:expr,
        $hash_or_encode_to:ident,
        $sign:ident,
        $pk_eq:ident,
        $sig_eq:ident,
        $verify:ident,
        $pk_in_group:ident,
        $pk_to_aff:ident,
        $pk_from_aff:ident,
        $pk_ser:ident,
        $pk_comp:ident,
        $pk_deser:ident,
        $pk_uncomp:ident,
        $pk_comp_size:expr,
        $pk_ser_size:expr,
        $sig_in_group:ident,
        $sig_to_aff:ident,
        $sig_from_aff:ident,
        $sig_ser:ident,
        $sig_comp:ident,
        $sig_deser:ident,
        $sig_uncomp:ident,
        $sig_comp_size:expr,
        $sig_ser_size:expr,
        $pk_add_or_dbl:ident,
        $pk_add_or_dbl_aff:ident,
        $sig_add_or_dbl:ident,
        $sig_add_or_dbl_aff:ident,
        $pk_is_inf:ident,
        $sig_is_inf:ident,
        $sig_aggr_in_group:ident,
    ) => {
        /// Secret Key
        #[derive(Default, Debug, Clone, Zeroize)]
        #[zeroize(drop)]
        pub struct SecretKey {
            value: blst_scalar,
        }

        impl SecretKey {
            /// Deterministically generate a secret key from key material
            pub fn key_gen(
                ikm: &[u8],
                key_info: &[u8],
            ) -> Result<Self, BLST_ERROR> {
                if ikm.len() < 32 {
                    return Err(BLST_ERROR::BLST_BAD_ENCODING);
                }
                let mut sk = SecretKey::default();
                unsafe {
                    blst_keygen(
                        &mut sk.value,
                        ikm.as_ptr(),
                        ikm.len(),
                        key_info.as_ptr(),
                        key_info.len(),
                    );
                }
                Ok(sk)
            }

            // sk_to_pk
            pub fn sk_to_pk(&self) -> PublicKey {
                // TODO - would the user like the serialized/compressed pk as well?
                let mut pk_aff = PublicKey::default();
                //let mut pk_ser = [0u8; $pk_ser_size];

                unsafe {
                    $sk_to_pk(
                        //pk_ser.as_mut_ptr(),
                        ptr::null_mut(),
                        &mut pk_aff.point,
                        &self.value,
                    );
                }
                pk_aff
            }

            // Sign
            pub fn sign(
                &self,
                msg: &[u8],
                dst: &[u8],
                aug: &[u8],
            ) -> Signature {
                // TODO - would the user like the serialized/compressed sig as well?
                let mut q = <$sig>::default();
                let mut sig_aff = <$sig_aff>::default();
                //let mut sig_ser = [0u8; $sig_ser_size];
                unsafe {
                    $hash_or_encode_to(
                        &mut q,
                        msg.as_ptr(),
                        msg.len(),
                        dst.as_ptr(),
                        dst.len(),
                        aug.as_ptr(),
                        aug.len(),
                    );
                    $sign(ptr::null_mut(), &mut sig_aff, &q, &self.value);
                }
                Signature { point: sig_aff }
            }

            // TODO - formally speaking application is entitled to have
            // ultimate control over secret key storage, which means that
            // corresponding serialization/deserialization subroutines
            // should accept reference to where to store the result, as
            // opposite to returning one.

            // serialize
            pub fn serialize(&self) -> [u8; 32] {
                let mut sk_out = [0; 32];
                unsafe {
                    blst_bendian_from_scalar(sk_out.as_mut_ptr(), &self.value);
                }
                sk_out
            }

            // deserialize
            pub fn deserialize(sk_in: &[u8]) -> Result<Self, BLST_ERROR> {
                let mut sk = blst_scalar::default();
                if sk_in.len() != 32 {
                    return Err(BLST_ERROR::BLST_BAD_ENCODING);
                }
                unsafe {
                    blst_scalar_from_bendian(&mut sk, sk_in.as_ptr());
                    if !blst_sk_check(&sk) {
                        return Err(BLST_ERROR::BLST_BAD_ENCODING);
                    }
                }
                Ok(Self { value: sk })
            }

            pub fn to_bytes(&self) -> [u8; 32] {
                SecretKey::serialize(&self)
            }

            pub fn from_bytes(sk_in: &[u8]) -> Result<Self, BLST_ERROR> {
                SecretKey::deserialize(sk_in)
            }
        }

        #[derive(Default, Debug, Clone, Copy)]
        pub struct PublicKey {
            point: $pk_aff,
        }

        impl PublicKey {
            // Core operations

            // key_validate
            pub fn validate(&self) -> Result<(), BLST_ERROR> {
                unsafe {
                    if $pk_is_inf(&self.point) {
                        return Err(BLST_ERROR::BLST_PK_IS_INFINITY);
                    }
                    if !$pk_in_group(&self.point) {
                        return Err(BLST_ERROR::BLST_POINT_NOT_IN_GROUP);
                    }
                }
                Ok(())
            }

            pub fn key_validate(key: &[u8]) -> Result<Self, BLST_ERROR> {
                let pk = PublicKey::from_bytes(key)?;
                pk.validate()?;
                Ok(pk)
            }

            pub fn from_aggregate(agg_pk: &AggregatePublicKey) -> Self {
                let mut pk_aff = <$pk_aff>::default();
                unsafe {
                    $pk_to_aff(&mut pk_aff, &agg_pk.point);
                }
                Self { point: pk_aff }
            }

            // Serdes

            pub fn compress(&self) -> [u8; $pk_comp_size] {
                let mut pk_comp = [0u8; $pk_comp_size];
                unsafe {
                    $pk_comp(pk_comp.as_mut_ptr(), &self.point);
                }
                pk_comp
            }

            pub fn serialize(&self) -> [u8; $pk_ser_size] {
                let mut pk_out = [0u8; $pk_ser_size];
                unsafe {
                    $pk_ser(pk_out.as_mut_ptr(), &self.point);
                }
                pk_out
            }

            pub fn uncompress(pk_comp: &[u8]) -> Result<Self, BLST_ERROR> {
                if pk_comp.len() == $pk_comp_size && (pk_comp[0] & 0x80) != 0 {
                    let mut pk = <$pk_aff>::default();
                    let err = unsafe { $pk_uncomp(&mut pk, pk_comp.as_ptr()) };
                    if err != BLST_ERROR::BLST_SUCCESS {
                        return Err(err);
                    }
                    Ok(Self { point: pk })
                } else {
                    Err(BLST_ERROR::BLST_BAD_ENCODING)
                }
            }

            pub fn deserialize(pk_in: &[u8]) -> Result<Self, BLST_ERROR> {
                if (pk_in.len() == $pk_ser_size && (pk_in[0] & 0x80) == 0)
                    || (pk_in.len() == $pk_comp_size && (pk_in[0] & 0x80) != 0)
                {
                    let mut pk = <$pk_aff>::default();
                    let err = unsafe { $pk_deser(&mut pk, pk_in.as_ptr()) };
                    if err != BLST_ERROR::BLST_SUCCESS {
                        return Err(err);
                    }
                    Ok(Self { point: pk })
                } else {
                    Err(BLST_ERROR::BLST_BAD_ENCODING)
                }
            }

            pub fn from_bytes(pk_in: &[u8]) -> Result<Self, BLST_ERROR> {
                PublicKey::deserialize(pk_in)
            }

            pub fn to_bytes(&self) -> [u8; $pk_comp_size] {
                self.compress()
            }
        }

        // Trait for equality comparisons which are equivalence relations.
        //
        // This means, that in addition to a == b and a != b being strict
        // inverses, the equality must be reflexive, symmetric and transitive.
        impl Eq for PublicKey {}

        impl PartialEq for PublicKey {
            fn eq(&self, other: &Self) -> bool {
                unsafe { $pk_eq(&self.point, &other.point) }
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub struct AggregatePublicKey {
            point: $pk,
        }

        impl AggregatePublicKey {
            pub fn from_public_key(pk: &PublicKey) -> Self {
                let mut agg_pk = <$pk>::default();
                unsafe {
                    $pk_from_aff(&mut agg_pk, &pk.point);
                }
                Self { point: agg_pk }
            }

            pub fn to_public_key(&self) -> PublicKey {
                let mut pk = <$pk_aff>::default();
                unsafe {
                    $pk_to_aff(&mut pk, &self.point);
                }
                PublicKey { point: pk }
            }

            // Aggregate
            pub fn aggregate(
                pks: &[&PublicKey],
                pks_validate: bool,
            ) -> Result<Self, BLST_ERROR> {
                if pks.len() == 0 {
                    return Err(BLST_ERROR::BLST_AGGR_TYPE_MISMATCH);
                }
                if pks_validate {
                    pks[0].validate()?;
                }
                let mut agg_pk = AggregatePublicKey::from_public_key(pks[0]);
                for s in pks.iter().skip(1) {
                    if pks_validate {
                        s.validate()?;
                    }
                    unsafe {
                        $pk_add_or_dbl_aff(
                            &mut agg_pk.point,
                            &agg_pk.point,
                            &s.point,
                        );
                    }
                }
                Ok(agg_pk)
            }

            pub fn aggregate_serialized(
                pks: &[&[u8]],
                pks_validate: bool,
            ) -> Result<Self, BLST_ERROR> {
                // TODO - threading
                if pks.len() == 0 {
                    return Err(BLST_ERROR::BLST_AGGR_TYPE_MISMATCH);
                }
                let mut pk = if pks_validate {
                    PublicKey::key_validate(pks[0])?
                } else {
                    PublicKey::from_bytes(pks[0])?
                };
                let mut agg_pk = AggregatePublicKey::from_public_key(&pk);
                for s in pks.iter().skip(1) {
                    pk = if pks_validate {
                        PublicKey::key_validate(s)?
                    } else {
                        PublicKey::from_bytes(s)?
                    };
                    unsafe {
                        $pk_add_or_dbl_aff(
                            &mut agg_pk.point,
                            &agg_pk.point,
                            &pk.point,
                        );
                    }
                }
                Ok(agg_pk)
            }

            pub fn add_aggregate(&mut self, agg_pk: &AggregatePublicKey) {
                unsafe {
                    $pk_add_or_dbl(&mut self.point, &self.point, &agg_pk.point);
                }
            }

            pub fn add_public_key(
                &mut self,
                pk: &PublicKey,
                pk_validate: bool,
            ) -> Result<(), BLST_ERROR> {
                if pk_validate {
                    pk.validate()?;
                }
                unsafe {
                    $pk_add_or_dbl_aff(&mut self.point, &self.point, &pk.point);
                }
                Ok(())
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub struct Signature {
            point: $sig_aff,
        }

        impl Signature {
            // sig_infcheck, check for infinity, is a way to avoid going
            // into resource-consuming verification. Passing 'false' is
            // always cryptographically safe, but application might want
            // to guard against obviously bogus individual[!] signatures.
            pub fn validate(
                &self,
                sig_infcheck: bool,
            ) -> Result<(), BLST_ERROR> {
                unsafe {
                    if sig_infcheck && $sig_is_inf(&self.point) {
                        return Err(BLST_ERROR::BLST_PK_IS_INFINITY);
                    }
                    if !$sig_in_group(&self.point) {
                        return Err(BLST_ERROR::BLST_POINT_NOT_IN_GROUP);
                    }
                }
                Ok(())
            }

            pub fn sig_validate(
                sig: &[u8],
                sig_infcheck: bool,
            ) -> Result<Self, BLST_ERROR> {
                let sig = Signature::from_bytes(sig)?;
                sig.validate(sig_infcheck)?;
                Ok(sig)
            }

            pub fn verify(
                &self,
                sig_groupcheck: bool,
                msg: &[u8],
                dst: &[u8],
                aug: &[u8],
                pk: &PublicKey,
                pk_validate: bool,
            ) -> BLST_ERROR {
                let aug_msg = [aug, msg].concat();
                self.aggregate_verify(
                    sig_groupcheck,
                    &[aug_msg.as_slice()],
                    dst,
                    &[pk],
                    pk_validate,
                )
            }

            pub fn aggregate_verify(
                &self,
                sig_groupcheck: bool,
                msgs: &[&[u8]],
                dst: &[u8],
                pks: &[&PublicKey],
                pks_validate: bool,
            ) -> BLST_ERROR {
                let n_elems = pks.len();
                if n_elems == 0 || msgs.len() != n_elems {
                    return BLST_ERROR::BLST_VERIFY_FAIL;
                }

                // TODO - check msg uniqueness?

                let pool = mt::da_pool();
                let (tx, rx) = channel();
                let counter = Arc::new(AtomicUsize::new(0));
                let valid = Arc::new(AtomicBool::new(true));

                let n_workers = core::cmp::min(pool.max_count(), n_elems);
                for _ in 0..n_workers {
                    let tx = tx.clone();
                    let counter = counter.clone();
                    let valid = valid.clone();

                    pool.joined_execute(move || {
                        let mut pairing = Pairing::new($hash_or_encode, dst);

                        while valid.load(Ordering::Relaxed) {
                            let work = counter.fetch_add(1, Ordering::Relaxed);
                            if work >= n_elems {
                                break;
                            }
                            if pairing.aggregate(
                                &pks[work].point,
                                pks_validate,
                                &unsafe { ptr::null::<$sig_aff>().as_ref() },
                                false,
                                &msgs[work],
                                &[],
                            ) != BLST_ERROR::BLST_SUCCESS
                            {
                                valid.store(false, Ordering::Relaxed);
                                break;
                            }
                        }
                        if valid.load(Ordering::Relaxed) {
                            pairing.commit();
                        }
                        tx.send(pairing).expect("disaster");
                    });
                }

                if sig_groupcheck && valid.load(Ordering::Relaxed) {
                    match self.validate(false) {
                        Err(_err) => valid.store(false, Ordering::Relaxed),
                        _ => (),
                    }
                }

                let mut gtsig = blst_fp12::default();
                if valid.load(Ordering::Relaxed) {
                    Pairing::aggregated(&mut gtsig, &self.point);
                }

                let mut acc = rx.recv().unwrap();
                for _ in 1..n_workers {
                    acc.merge(&rx.recv().unwrap());
                }

                if valid.load(Ordering::Relaxed)
                    && acc.finalverify(Some(&gtsig))
                {
                    BLST_ERROR::BLST_SUCCESS
                } else {
                    BLST_ERROR::BLST_VERIFY_FAIL
                }
            }

            // pks are assumed to be verified for proof of possession,
            // which implies that they are already group-checked
            pub fn fast_aggregate_verify(
                &self,
                sig_groupcheck: bool,
                msg: &[u8],
                dst: &[u8],
                pks: &[&PublicKey],
            ) -> BLST_ERROR {
                let agg_pk = match AggregatePublicKey::aggregate(pks, false) {
                    Ok(agg_sig) => agg_sig,
                    Err(err) => return err,
                };
                let pk = agg_pk.to_public_key();
                self.aggregate_verify(
                    sig_groupcheck,
                    &[msg],
                    dst,
                    &[&pk],
                    false,
                )
            }

            pub fn fast_aggregate_verify_pre_aggregated(
                &self,
                sig_groupcheck: bool,
                msg: &[u8],
                dst: &[u8],
                pk: &PublicKey,
            ) -> BLST_ERROR {
                self.aggregate_verify(sig_groupcheck, &[msg], dst, &[pk], false)
            }

            // https://ethresear.ch/t/fast-verification-of-multiple-bls-signatures/5407
            pub fn verify_multiple_aggregate_signatures(
                msgs: &[&[u8]],
                dst: &[u8],
                pks: &[&PublicKey],
                pks_validate: bool,
                sigs: &[&Signature],
                sigs_groupcheck: bool,
                rands: &[blst_scalar],
                rand_bits: usize,
            ) -> BLST_ERROR {
                let n_elems = pks.len();
                if n_elems == 0
                    || msgs.len() != n_elems
                    || sigs.len() != n_elems
                    || rands.len() != n_elems
                {
                    return BLST_ERROR::BLST_VERIFY_FAIL;
                }

                // TODO - check msg uniqueness?

                let pool = mt::da_pool();
                let (tx, rx) = channel();
                let counter = Arc::new(AtomicUsize::new(0));
                let valid = Arc::new(AtomicBool::new(true));

                let n_workers = core::cmp::min(pool.max_count(), n_elems);
                for _ in 0..n_workers {
                    let tx = tx.clone();
                    let counter = counter.clone();
                    let valid = valid.clone();

                    pool.joined_execute(move || {
                        let mut pairing = Pairing::new($hash_or_encode, dst);

                        // TODO - engage multi-point mul-n-add for larger
                        // amount of inputs...
                        while valid.load(Ordering::Relaxed) {
                            let work = counter.fetch_add(1, Ordering::Relaxed);
                            if work >= n_elems {
                                break;
                            }

                            if pairing.mul_n_aggregate(
                                &pks[work].point,
                                pks_validate,
                                &sigs[work].point,
                                sigs_groupcheck,
                                &rands[work].b,
                                rand_bits,
                                msgs[work],
                                &[],
                            ) != BLST_ERROR::BLST_SUCCESS
                            {
                                valid.store(false, Ordering::Relaxed);
                                break;
                            }
                        }
                        if valid.load(Ordering::Relaxed) {
                            pairing.commit();
                        }
                        tx.send(pairing).expect("disaster");
                    });
                }

                let mut acc = rx.recv().unwrap();
                for _ in 1..n_workers {
                    acc.merge(&rx.recv().unwrap());
                }

                if valid.load(Ordering::Relaxed) && acc.finalverify(None) {
                    BLST_ERROR::BLST_SUCCESS
                } else {
                    BLST_ERROR::BLST_VERIFY_FAIL
                }
            }

            pub fn from_aggregate(agg_sig: &AggregateSignature) -> Self {
                let mut sig_aff = <$sig_aff>::default();
                unsafe {
                    $sig_to_aff(&mut sig_aff, &agg_sig.point);
                }
                Self { point: sig_aff }
            }

            pub fn compress(&self) -> [u8; $sig_comp_size] {
                let mut sig_comp = [0; $sig_comp_size];
                unsafe {
                    $sig_comp(sig_comp.as_mut_ptr(), &self.point);
                }
                sig_comp
            }

            pub fn serialize(&self) -> [u8; $sig_ser_size] {
                let mut sig_out = [0; $sig_ser_size];
                unsafe {
                    $sig_ser(sig_out.as_mut_ptr(), &self.point);
                }
                sig_out
            }

            pub fn uncompress(sig_comp: &[u8]) -> Result<Self, BLST_ERROR> {
                if sig_comp.len() == $sig_comp_size && (sig_comp[0] & 0x80) != 0
                {
                    let mut sig = <$sig_aff>::default();
                    let err =
                        unsafe { $sig_uncomp(&mut sig, sig_comp.as_ptr()) };
                    if err != BLST_ERROR::BLST_SUCCESS {
                        return Err(err);
                    }
                    Ok(Self { point: sig })
                } else {
                    Err(BLST_ERROR::BLST_BAD_ENCODING)
                }
            }

            pub fn deserialize(sig_in: &[u8]) -> Result<Self, BLST_ERROR> {
                if (sig_in.len() == $sig_ser_size && (sig_in[0] & 0x80) == 0)
                    || (sig_in.len() == $sig_comp_size
                        && (sig_in[0] & 0x80) != 0)
                {
                    let mut sig = <$sig_aff>::default();
                    let err = unsafe { $sig_deser(&mut sig, sig_in.as_ptr()) };
                    if err != BLST_ERROR::BLST_SUCCESS {
                        return Err(err);
                    }
                    Ok(Self { point: sig })
                } else {
                    Err(BLST_ERROR::BLST_BAD_ENCODING)
                }
            }

            pub fn from_bytes(sig_in: &[u8]) -> Result<Self, BLST_ERROR> {
                Signature::deserialize(sig_in)
            }

            pub fn to_bytes(&self) -> [u8; $sig_comp_size] {
                self.compress()
            }

            pub fn subgroup_check(&self) -> bool {
                unsafe { $sig_in_group(&self.point) }
            }
        }

        // Trait for equality comparisons which are equivalence relations.
        //
        // This means, that in addition to a == b and a != b being strict
        // inverses, the equality must be reflexive, symmetric and transitive.
        impl Eq for Signature {}

        impl PartialEq for Signature {
            fn eq(&self, other: &Self) -> bool {
                unsafe { $sig_eq(&self.point, &other.point) }
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub struct AggregateSignature {
            point: $sig,
        }

        impl AggregateSignature {
            pub fn validate(&self) -> Result<(), BLST_ERROR> {
                unsafe {
                    if !$sig_aggr_in_group(&self.point) {
                        return Err(BLST_ERROR::BLST_POINT_NOT_IN_GROUP);
                    }
                }
                Ok(())
            }

            pub fn from_signature(sig: &Signature) -> Self {
                let mut agg_sig = <$sig>::default();
                unsafe {
                    $sig_from_aff(&mut agg_sig, &sig.point);
                }
                Self { point: agg_sig }
            }

            pub fn to_signature(&self) -> Signature {
                let mut sig = <$sig_aff>::default();
                unsafe {
                    $sig_to_aff(&mut sig, &self.point);
                }
                Signature { point: sig }
            }

            // Aggregate
            pub fn aggregate(
                sigs: &[&Signature],
                sigs_groupcheck: bool,
            ) -> Result<Self, BLST_ERROR> {
                if sigs.len() == 0 {
                    return Err(BLST_ERROR::BLST_AGGR_TYPE_MISMATCH);
                }
                if sigs_groupcheck {
                    // We can't actually judge if input is individual or
                    // aggregated signature, so we can't enforce infinitiy
                    // check.
                    sigs[0].validate(false)?;
                }
                let mut agg_sig = AggregateSignature::from_signature(sigs[0]);
                for s in sigs.iter().skip(1) {
                    if sigs_groupcheck {
                        s.validate(false)?;
                    }
                    unsafe {
                        $sig_add_or_dbl_aff(
                            &mut agg_sig.point,
                            &agg_sig.point,
                            &s.point,
                        );
                    }
                }
                Ok(agg_sig)
            }

            pub fn aggregate_serialized(
                sigs: &[&[u8]],
                sigs_groupcheck: bool,
            ) -> Result<Self, BLST_ERROR> {
                // TODO - threading
                if sigs.len() == 0 {
                    return Err(BLST_ERROR::BLST_AGGR_TYPE_MISMATCH);
                }
                let mut sig = if sigs_groupcheck {
                    Signature::sig_validate(sigs[0], false)?
                } else {
                    Signature::from_bytes(sigs[0])?
                };
                let mut agg_sig = AggregateSignature::from_signature(&sig);
                for s in sigs.iter().skip(1) {
                    sig = if sigs_groupcheck {
                        Signature::sig_validate(s, false)?
                    } else {
                        Signature::from_bytes(s)?
                    };
                    unsafe {
                        $sig_add_or_dbl_aff(
                            &mut agg_sig.point,
                            &agg_sig.point,
                            &sig.point,
                        );
                    }
                }
                Ok(agg_sig)
            }

            pub fn add_aggregate(&mut self, agg_sig: &AggregateSignature) {
                unsafe {
                    $sig_add_or_dbl(
                        &mut self.point,
                        &self.point,
                        &agg_sig.point,
                    );
                }
            }

            pub fn add_signature(
                &mut self,
                sig: &Signature,
                sig_groupcheck: bool,
            ) -> Result<(), BLST_ERROR> {
                if sig_groupcheck {
                    sig.validate(false)?;
                }
                unsafe {
                    $sig_add_or_dbl_aff(
                        &mut self.point,
                        &self.point,
                        &sig.point,
                    );
                }
                Ok(())
            }

            pub fn subgroup_check(&self) -> bool {
                unsafe { $sig_aggr_in_group(&self.point) }
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use rand::{RngCore, SeedableRng};
            use rand_chacha::ChaCha20Rng;

            // Testing only - do not use for production
            pub fn gen_random_key(
                rng: &mut rand_chacha::ChaCha20Rng,
            ) -> SecretKey {
                let mut ikm = [0u8; 32];
                rng.fill_bytes(&mut ikm);

                let mut sk = <blst_scalar>::default();
                unsafe {
                    blst_keygen(&mut sk, ikm.as_ptr(), 32, ptr::null(), 0);
                }
                SecretKey { value: sk }
            }

            #[test]
            fn test_sign() {
                let ikm: [u8; 32] = [
                    0x93, 0xad, 0x7e, 0x65, 0xde, 0xad, 0x05, 0x2a, 0x08, 0x3a,
                    0x91, 0x0c, 0x8b, 0x72, 0x85, 0x91, 0x46, 0x4c, 0xca, 0x56,
                    0x60, 0x5b, 0xb0, 0x56, 0xed, 0xfe, 0x2b, 0x60, 0xa6, 0x3c,
                    0x48, 0x99,
                ];

                let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
                let pk = sk.sk_to_pk();

                let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";
                let msg = b"hello foo";
                let sig = sk.sign(msg, dst, &[]);

                let err = sig.verify(true, msg, dst, &[], &pk, true);
                assert_eq!(err, BLST_ERROR::BLST_SUCCESS);
            }

            #[test]
            fn test_aggregate() {
                let num_msgs = 10;
                let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

                let seed = [0u8; 32];
                let mut rng = ChaCha20Rng::from_seed(seed);

                let sks: Vec<_> =
                    (0..num_msgs).map(|_| gen_random_key(&mut rng)).collect();
                let pks =
                    sks.iter().map(|sk| sk.sk_to_pk()).collect::<Vec<_>>();
                let pks_refs: Vec<&PublicKey> =
                    pks.iter().map(|pk| pk).collect();
                let pks_rev: Vec<&PublicKey> =
                    pks.iter().rev().map(|pk| pk).collect();

                let pk_comp = pks[0].compress();
                let pk_uncomp = PublicKey::uncompress(&pk_comp);
                assert_eq!(pk_uncomp.is_ok(), true);

                let mut msgs: Vec<Vec<u8>> = vec![vec![]; num_msgs];
                for i in 0..num_msgs {
                    let msg_len = (rng.next_u64() & 0x3F) + 1;
                    msgs[i] = vec![0u8; msg_len as usize];
                    rng.fill_bytes(&mut msgs[i]);
                }

                let msgs_refs: Vec<&[u8]> =
                    msgs.iter().map(|m| m.as_slice()).collect();

                let sigs = sks
                    .iter()
                    .zip(msgs.iter())
                    .map(|(sk, m)| (sk.sign(m, dst, &[])))
                    .collect::<Vec<Signature>>();

                let mut errs = sigs
                    .iter()
                    .zip(msgs.iter())
                    .zip(pks.iter())
                    .map(|((s, m), pk)| (s.verify(true, m, dst, &[], pk, true)))
                    .collect::<Vec<BLST_ERROR>>();
                assert_eq!(errs, vec![BLST_ERROR::BLST_SUCCESS; num_msgs]);

                // Swap message/public key pairs to create bad signature
                errs = sigs
                    .iter()
                    .zip(msgs.iter())
                    .zip(pks.iter().rev())
                    .map(|((s, m), pk)| (s.verify(true, m, dst, &[], pk, true)))
                    .collect::<Vec<BLST_ERROR>>();
                assert_ne!(errs, vec![BLST_ERROR::BLST_SUCCESS; num_msgs]);

                let sig_refs =
                    sigs.iter().map(|s| s).collect::<Vec<&Signature>>();
                let agg = match AggregateSignature::aggregate(&sig_refs, true) {
                    Ok(agg) => agg,
                    Err(err) => panic!("aggregate failure: {:?}", err),
                };

                let agg_sig = agg.to_signature();
                let mut result = agg_sig
                    .aggregate_verify(false, &msgs_refs, dst, &pks_refs, false);
                assert_eq!(result, BLST_ERROR::BLST_SUCCESS);

                // Swap message/public key pairs to create bad signature
                result = agg_sig
                    .aggregate_verify(false, &msgs_refs, dst, &pks_rev, false);
                assert_ne!(result, BLST_ERROR::BLST_SUCCESS);
            }

            #[test]
            fn test_multiple_agg_sigs() {
                let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
                let num_pks_per_sig = 10;
                let num_sigs = 10;

                let seed = [0u8; 32];
                let mut rng = ChaCha20Rng::from_seed(seed);

                let mut msgs: Vec<Vec<u8>> = vec![vec![]; num_sigs];
                let mut sigs: Vec<Signature> = Vec::with_capacity(num_sigs);
                let mut pks: Vec<PublicKey> = Vec::with_capacity(num_sigs);
                let mut rands: Vec<blst_scalar> = Vec::with_capacity(num_sigs);
                for i in 0..num_sigs {
                    // Create public keys
                    let sks_i: Vec<_> = (0..num_pks_per_sig)
                        .map(|_| gen_random_key(&mut rng))
                        .collect();

                    let pks_i = sks_i
                        .iter()
                        .map(|sk| sk.sk_to_pk())
                        .collect::<Vec<_>>();
                    let pks_refs_i: Vec<&PublicKey> =
                        pks_i.iter().map(|pk| pk).collect();

                    // Create random message for pks to all sign
                    let msg_len = (rng.next_u64() & 0x3F) + 1;
                    msgs[i] = vec![0u8; msg_len as usize];
                    rng.fill_bytes(&mut msgs[i]);

                    // Generate signature for each key pair
                    let sigs_i = sks_i
                        .iter()
                        .map(|sk| sk.sign(&msgs[i], dst, &[]))
                        .collect::<Vec<Signature>>();

                    // Test each current single signature
                    let errs = sigs_i
                        .iter()
                        .zip(pks_i.iter())
                        .map(|(s, pk)| {
                            (s.verify(true, &msgs[i], dst, &[], pk, true))
                        })
                        .collect::<Vec<BLST_ERROR>>();
                    assert_eq!(
                        errs,
                        vec![BLST_ERROR::BLST_SUCCESS; num_pks_per_sig]
                    );

                    let sig_refs_i =
                        sigs_i.iter().map(|s| s).collect::<Vec<&Signature>>();
                    let agg_i =
                        match AggregateSignature::aggregate(&sig_refs_i, false)
                        {
                            Ok(agg_i) => agg_i,
                            Err(err) => panic!("aggregate failure: {:?}", err),
                        };

                    // Test current aggregate signature
                    sigs.push(agg_i.to_signature());
                    let mut result = sigs[i].fast_aggregate_verify(
                        false,
                        &msgs[i],
                        dst,
                        &pks_refs_i,
                    );
                    assert_eq!(result, BLST_ERROR::BLST_SUCCESS);

                    // negative test
                    if i != 0 {
                        result = sigs[i - 1].fast_aggregate_verify(
                            false,
                            &msgs[i],
                            dst,
                            &pks_refs_i,
                        );
                        assert_ne!(result, BLST_ERROR::BLST_SUCCESS);
                    }

                    // aggregate public keys and push into vec
                    let agg_pk_i =
                        match AggregatePublicKey::aggregate(&pks_refs_i, false)
                        {
                            Ok(agg_pk_i) => agg_pk_i,
                            Err(err) => panic!("aggregate failure: {:?}", err),
                        };
                    pks.push(agg_pk_i.to_public_key());

                    // Test current aggregate signature with aggregated pks
                    result = sigs[i].fast_aggregate_verify_pre_aggregated(
                        false, &msgs[i], dst, &pks[i],
                    );
                    assert_eq!(result, BLST_ERROR::BLST_SUCCESS);

                    // negative test
                    if i != 0 {
                        result = sigs[i - 1]
                            .fast_aggregate_verify_pre_aggregated(
                                false, &msgs[i], dst, &pks[i],
                            );
                        assert_ne!(result, BLST_ERROR::BLST_SUCCESS);
                    }

                    // create random values
                    let mut vals = [0u64; 4];
                    vals[0] = rng.next_u64();
                    while vals[0] == 0 {
                        // Reject zero as it is used for multiplication.
                        vals[0] = rng.next_u64();
                    }
                    let mut rand_i = MaybeUninit::<blst_scalar>::uninit();
                    unsafe {
                        blst_scalar_from_uint64(
                            rand_i.as_mut_ptr(),
                            vals.as_ptr(),
                        );
                        rands.push(rand_i.assume_init());
                    }
                }

                let msgs_refs: Vec<&[u8]> =
                    msgs.iter().map(|m| m.as_slice()).collect();
                let sig_refs =
                    sigs.iter().map(|s| s).collect::<Vec<&Signature>>();
                let pks_refs: Vec<&PublicKey> =
                    pks.iter().map(|pk| pk).collect();

                let msgs_rev: Vec<&[u8]> =
                    msgs.iter().rev().map(|m| m.as_slice()).collect();
                let sig_rev =
                    sigs.iter().rev().map(|s| s).collect::<Vec<&Signature>>();
                let pks_rev: Vec<&PublicKey> =
                    pks.iter().rev().map(|pk| pk).collect();

                let mut result =
                    Signature::verify_multiple_aggregate_signatures(
                        &msgs_refs, dst, &pks_refs, false, &sig_refs, true,
                        &rands, 64,
                    );
                assert_eq!(result, BLST_ERROR::BLST_SUCCESS);

                // negative tests (use reverse msgs, pks, and sigs)
                result = Signature::verify_multiple_aggregate_signatures(
                    &msgs_rev, dst, &pks_refs, false, &sig_refs, true, &rands,
                    64,
                );
                assert_ne!(result, BLST_ERROR::BLST_SUCCESS);

                result = Signature::verify_multiple_aggregate_signatures(
                    &msgs_refs, dst, &pks_rev, false, &sig_refs, true, &rands,
                    64,
                );
                assert_ne!(result, BLST_ERROR::BLST_SUCCESS);

                result = Signature::verify_multiple_aggregate_signatures(
                    &msgs_refs, dst, &pks_refs, false, &sig_rev, true, &rands,
                    64,
                );
                assert_ne!(result, BLST_ERROR::BLST_SUCCESS);
            }

            #[test]
            fn test_serialization() {
                let seed = [0u8; 32];
                let mut rng = ChaCha20Rng::from_seed(seed);

                let sk = gen_random_key(&mut rng);
                let sk2 = gen_random_key(&mut rng);

                let pk = sk.sk_to_pk();
                let pk_comp = pk.compress();
                let pk_ser = pk.serialize();

                let pk_uncomp = PublicKey::uncompress(&pk_comp);
                assert_eq!(pk_uncomp.is_ok(), true);
                assert_eq!(pk_uncomp.unwrap(), pk);

                let pk_deser = PublicKey::deserialize(&pk_ser);
                assert_eq!(pk_deser.is_ok(), true);
                assert_eq!(pk_deser.unwrap(), pk);

                let pk2 = sk2.sk_to_pk();
                let pk_comp2 = pk2.compress();
                let pk_ser2 = pk2.serialize();

                let pk_uncomp2 = PublicKey::uncompress(&pk_comp2);
                assert_eq!(pk_uncomp2.is_ok(), true);
                assert_eq!(pk_uncomp2.unwrap(), pk2);

                let pk_deser2 = PublicKey::deserialize(&pk_ser2);
                assert_eq!(pk_deser2.is_ok(), true);
                assert_eq!(pk_deser2.unwrap(), pk2);

                assert_ne!(pk, pk2);
                assert_ne!(pk_uncomp.unwrap(), pk2);
                assert_ne!(pk_deser.unwrap(), pk2);
                assert_ne!(pk_uncomp2.unwrap(), pk);
                assert_ne!(pk_deser2.unwrap(), pk);
            }
        }
    };
}

pub mod min_pk {
    use super::*;

    sig_variant_impl!(
        "MinPk",
        blst_p1,
        blst_p1_affine,
        blst_p2,
        blst_p2_affine,
        blst_sk_to_pk2_in_g1,
        true,
        blst_hash_to_g2,
        blst_sign_pk2_in_g1,
        blst_p1_affine_is_equal,
        blst_p2_affine_is_equal,
        blst_core_verify_pk_in_g1,
        blst_p1_affine_in_g1,
        blst_p1_to_affine,
        blst_p1_from_affine,
        blst_p1_affine_serialize,
        blst_p1_affine_compress,
        blst_p1_deserialize,
        blst_p1_uncompress,
        48,
        96,
        blst_p2_affine_in_g2,
        blst_p2_to_affine,
        blst_p2_from_affine,
        blst_p2_affine_serialize,
        blst_p2_affine_compress,
        blst_p2_deserialize,
        blst_p2_uncompress,
        96,
        192,
        blst_p1_add_or_double,
        blst_p1_add_or_double_affine,
        blst_p2_add_or_double,
        blst_p2_add_or_double_affine,
        blst_p1_affine_is_inf,
        blst_p2_affine_is_inf,
        blst_p2_in_g2,
    );
}

pub mod min_sig {
    use super::*;

    sig_variant_impl!(
        "MinSig",
        blst_p2,
        blst_p2_affine,
        blst_p1,
        blst_p1_affine,
        blst_sk_to_pk2_in_g2,
        true,
        blst_hash_to_g1,
        blst_sign_pk2_in_g2,
        blst_p2_affine_is_equal,
        blst_p1_affine_is_equal,
        blst_core_verify_pk_in_g2,
        blst_p2_affine_in_g2,
        blst_p2_to_affine,
        blst_p2_from_affine,
        blst_p2_affine_serialize,
        blst_p2_affine_compress,
        blst_p2_deserialize,
        blst_p2_uncompress,
        96,
        192,
        blst_p1_affine_in_g1,
        blst_p1_to_affine,
        blst_p1_from_affine,
        blst_p1_affine_serialize,
        blst_p1_affine_compress,
        blst_p1_deserialize,
        blst_p1_uncompress,
        48,
        96,
        blst_p2_add_or_double,
        blst_p2_add_or_double_affine,
        blst_p1_add_or_double,
        blst_p1_add_or_double_affine,
        blst_p2_affine_is_inf,
        blst_p1_affine_is_inf,
        blst_p1_in_g1,
    );
}

struct tile {
    x: usize,
    dx: usize,
    y: usize,
    dy: usize,
}

// Minimalist core::cell::Cell stand-in, but with Sync marker, which
// makes it possible to pass it to multiple threads. It works, because
// *here* each Cell is written only once and by just one thread.
#[repr(transparent)]
struct Cell<T: ?Sized> {
    value: T,
}
unsafe impl<T: ?Sized + Sync> Sync for Cell<T> {}
impl<T> Cell<T> {
    pub fn as_ptr(&self) -> *mut T {
        &self.value as *const T as *mut T
    }
}

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
    ) => {
        pub struct $points {
            points: Vec<$point_affine>,
        }

        impl $points {
            pub fn from(points: &[$point]) -> Self {
                let npoints = points.len();
                let mut ret = Self {
                    points: Vec::with_capacity(npoints),
                };
                unsafe { ret.points.set_len(npoints) };

                let pool = mt::da_pool();
                let ncpus = pool.max_count();
                if ncpus < 2 || npoints < 768 {
                    let p: [*const $point; 2] = [&points[0], ptr::null()];
                    unsafe { $to_affines(&mut ret.points[0], &p[0], npoints) };
                    return ret;
                }

                let mut nslices = (npoints + 511) / 512;
                nslices = core::cmp::min(nslices, ncpus);
                let wg = Arc::new((Barrier::new(2), AtomicUsize::new(nslices)));

                let (mut delta, mut rem) =
                    (npoints / nslices + 1, Wrapping(npoints % nslices));
                let mut x = 0usize;
                while x < npoints {
                    let out = &mut ret.points[x];
                    let inp = &points[x];

                    delta -= (rem == Wrapping(0)) as usize;
                    rem -= Wrapping(1);
                    x += delta;

                    let wg = wg.clone();
                    pool.joined_execute(move || {
                        let p: [*const $point; 2] = [inp, ptr::null()];
                        unsafe { $to_affines(out, &p[0], delta) };
                        if wg.1.fetch_sub(1, Ordering::AcqRel) == 1 {
                            wg.0.wait();
                        }
                    });
                }
                wg.0.wait();

                ret
            }

            pub fn mult(&self, scalars: &[u8], nbits: usize) -> $point {
                let npoints = self.points.len();
                let nbytes = (nbits + 7) / 8;

                if scalars.len() < nbytes * npoints {
                    panic!("scalars length mismatch");
                }

                let pool = mt::da_pool();
                let ncpus = pool.max_count();
                if ncpus < 2 || npoints < 32 {
                    let p: [*const $point_affine; 2] =
                        [&self.points[0], ptr::null()];
                    let s: [*const u8; 2] = [&scalars[0], ptr::null()];

                    unsafe {
                        let mut scratch: Vec<u64> =
                            Vec::with_capacity($scratch_sizeof(npoints) / 8);
                        scratch.set_len(scratch.capacity());
                        let mut ret = <$point>::default();
                        $multi_scalar_mult(
                            &mut ret,
                            &p[0],
                            npoints,
                            &s[0],
                            nbits,
                            &mut scratch[0],
                        );
                        return ret;
                    }
                }

                let (nx, ny, window) =
                    breakdown(nbits, pippenger_window_size(npoints), ncpus);

                // |grid[]| holds "coordinates" and place for result
                let mut grid: Vec<(tile, Cell<$point>)> =
                    Vec::with_capacity(nx * ny);
                unsafe { grid.set_len(grid.capacity()) };
                let dx = npoints / nx;
                let mut y = window * (ny - 1);
                let mut total = 0usize;

                while total < nx {
                    grid[total].0.x = total * dx;
                    grid[total].0.dx = dx;
                    grid[total].0.y = y;
                    grid[total].0.dy = nbits - y;
                    total += 1;
                }
                grid[total - 1].0.dx = npoints - grid[total - 1].0.x;
                while y != 0 {
                    y -= window;
                    for i in 0..nx {
                        grid[total].0.x = grid[i].0.x;
                        grid[total].0.dx = grid[i].0.dx;
                        grid[total].0.y = y;
                        grid[total].0.dy = window;
                        total += 1;
                    }
                }
                let grid = &grid[..];

                let points = &self.points[..];
                let sz = unsafe { $scratch_sizeof(0) / 8 };

                let mut row_sync: Vec<AtomicUsize> = Vec::with_capacity(ny);
                row_sync.resize_with(ny, Default::default);
                let row_sync = Arc::new(row_sync);
                let counter = Arc::new(AtomicUsize::new(0));
                let (tx, rx) = channel();
                let n_workers = core::cmp::min(ncpus, total);
                for _ in 0..n_workers {
                    let tx = tx.clone();
                    let counter = counter.clone();
                    let row_sync = row_sync.clone();

                    pool.joined_execute(move || {
                        let mut scratch = vec![0u64; sz << (window - 1)];
                        let mut p: [*const $point_affine; 2] =
                            [ptr::null(), ptr::null()];
                        let mut s: [*const u8; 2] = [ptr::null(), ptr::null()];

                        loop {
                            let work = counter.fetch_add(1, Ordering::Relaxed);
                            if work >= total {
                                break;
                            }
                            let x = grid[work].0.x;
                            let y = grid[work].0.y;

                            p[0] = &points[x];
                            s[0] = &scalars[x * nbytes];
                            unsafe {
                                $tile_mult(
                                    grid[work].1.as_ptr(),
                                    &p[0],
                                    grid[work].0.dx,
                                    &s[0],
                                    nbits,
                                    &mut scratch[0],
                                    y,
                                    window,
                                );
                            }
                            if row_sync[y / window]
                                .fetch_add(1, Ordering::AcqRel)
                                == nx - 1
                            {
                                tx.send(y).expect("disaster");
                            }
                        }
                    });
                }

                let mut ret = <$point>::default();
                let mut rows = vec![false; ny];
                let mut row = 0usize;
                for _ in 0..ny {
                    let mut y = rx.recv().unwrap();
                    rows[y / window] = true;
                    while grid[row].0.y == y {
                        while row < total && grid[row].0.y == y {
                            unsafe {
                                $add_or_double(
                                    &mut ret,
                                    &ret,
                                    grid[row].1.as_ptr(),
                                );
                            }
                            row += 1;
                        }
                        if y == 0 {
                            break;
                        }
                        for _ in 0..window {
                            unsafe { $double(&mut ret, &ret) };
                        }
                        y -= window;
                        if !rows[y / window] {
                            break;
                        }
                    }
                }
                ret
            }
        }

        #[cfg(test)]
        mod $test_mod {
            use super::*;
            use rand::{RngCore, SeedableRng};
            use rand_chacha::ChaCha20Rng;

            #[test]
            fn test_mult() {
                const npoints: usize = 2000;
                const nbits: usize = 160;
                const nbytes: usize = (nbits + 7) / 8;

                let mut scalars = [0u8; nbytes * npoints];
                ChaCha20Rng::from_seed([0u8; 32]).fill_bytes(&mut scalars);

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
                }

                let points = $points::from(&points);
                assert_eq!(naive, points.mult(&scalars, nbits));
            }
        }
    };
}

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
    p1_multi_scalar,
    blst_p1_generator,
    blst_p1_mult,
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
    p2_multi_scalar,
    blst_p2_generator,
    blst_p2_mult,
);

fn num_bits(l: usize) -> usize {
    8 * core::mem::size_of_val(&l) - l.leading_zeros() as usize
}

fn breakdown(
    nbits: usize,
    window: usize,
    ncpus: usize,
) -> (usize, usize, usize) {
    let mut nx: usize;
    let mut wnd: usize;

    if nbits > window * ncpus {
        nx = 1;
        wnd = window - num_bits(ncpus / 4);
    } else {
        nx = 2;
        wnd = window - 2;
        while (nbits / wnd + 1) * nx < ncpus {
            nx += 1;
            wnd = window - num_bits(3 * nx / 2);
        }
        nx -= 1;
        wnd = window - num_bits(3 * nx / 2);
    }
    let ny = nbits / wnd + 1;
    wnd = nbits / ny + 1;

    (nx, ny, wnd)
}

fn pippenger_window_size(npoints: usize) -> usize {
    let wbits = num_bits(npoints);

    if wbits > 13 {
        return wbits - 4;
    }
    if wbits > 5 {
        return wbits - 3;
    }
    2
}
