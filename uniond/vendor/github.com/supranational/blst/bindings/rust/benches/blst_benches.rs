// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use blst::*;

// Benchmark min_pk
use blst::min_pk::*;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

struct BenchData {
    sk: SecretKey,
    pk: PublicKey,
    msg: Vec<u8>,
    dst: Vec<u8>,
    sig: Signature,
}

fn gen_bench_data(rng: &mut rand_chacha::ChaCha20Rng) -> BenchData {
    let msg_len = (rng.next_u64() & 0x3F) + 1;
    let mut msg = vec![0u8; msg_len as usize];
    rng.fill_bytes(&mut msg);

    gen_bench_data_for_msg(rng, &msg)
}

fn gen_bench_data_for_msg(
    rng: &mut rand_chacha::ChaCha20Rng,
    msg: &Vec<u8>,
) -> BenchData {
    let mut ikm = [0u8; 32];
    rng.fill_bytes(&mut ikm);

    let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
    let pk = sk.sk_to_pk();
    let dst = "BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_"
        .as_bytes()
        .to_owned();

    let sig = sk.sign(&msg, &dst, &[]);

    let bd = BenchData {
        sk,
        pk,
        dst,
        msg: msg.clone(),
        sig,
    };
    bd
}

fn bench_verify_multi_aggregate(c: &mut Criterion) {
    let mut group = c.benchmark_group("verify_multi_aggregate");

    let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
    let mut ikm = [0u8; 32];

    let seed = [0u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);

    let num_sigs = vec![8, 16, 32, 64, 128];
    let pks_per_sig = 3;

    for n in num_sigs.iter() {
        let mut msgs: Vec<Vec<u8>> = vec![vec![]; *n];
        let mut sigs: Vec<Signature> = Vec::with_capacity(*n);
        let mut pks: Vec<PublicKey> = Vec::with_capacity(*n);
        let mut rands: Vec<blst_scalar> = Vec::with_capacity(*n);

        for i in 0..*n {
            // Create public keys
            rng.fill_bytes(&mut ikm);
            let sks_i: Vec<_> = (0..pks_per_sig)
                .map(|_| {
                    ikm[0] += 1;
                    SecretKey::key_gen(&ikm, &[]).unwrap()
                })
                .collect();
            let pks_i =
                sks_i.iter().map(|sk| sk.sk_to_pk()).collect::<Vec<_>>();
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

            // Aggregate signature
            let sig_refs_i =
                sigs_i.iter().map(|s| s).collect::<Vec<&Signature>>();
            let agg_i = match AggregateSignature::aggregate(&sig_refs_i, false)
            {
                Ok(agg_i) => agg_i,
                Err(err) => panic!("aggregate failure: {:?}", err),
            };
            sigs.push(agg_i.to_signature());

            // aggregate public keys and push into vec
            let agg_pk_i =
                match AggregatePublicKey::aggregate(&pks_refs_i, false) {
                    Ok(agg_pk_i) => agg_pk_i,
                    Err(err) => panic!("aggregate failure: {:?}", err),
                };
            pks.push(agg_pk_i.to_public_key());

            // create random values
            let mut vals = [0u64; 4];
            vals[0] = rng.next_u64();
            let mut rand_i = std::mem::MaybeUninit::<blst_scalar>::uninit();
            unsafe {
                blst_scalar_from_uint64(rand_i.as_mut_ptr(), vals.as_ptr());
                rands.push(rand_i.assume_init());
            }
        }

        let msgs_refs: Vec<&[u8]> = msgs.iter().map(|m| m.as_slice()).collect();
        let sig_refs = sigs.iter().map(|s| s).collect::<Vec<&Signature>>();
        let pks_refs: Vec<&PublicKey> = pks.iter().map(|pk| pk).collect();

        let agg_ver = (sig_refs, pks_refs, msgs_refs, dst, rands);

        group.bench_with_input(
            BenchmarkId::new("verify_multi_aggregate", n),
            &agg_ver,
            |b, (s, p, m, d, r)| {
                b.iter(|| {
                    let result =
                        Signature::verify_multiple_aggregate_signatures(
                            &m, *d, &p, false, &s, false, &r, 64,
                        );
                    assert_eq!(result, BLST_ERROR::BLST_SUCCESS);
                });
            },
        );
    }

    group.finish();
}

fn bench_fast_aggregate_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_aggregate_verify");

    let seed = [0u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);

    let msg_len = (rng.next_u64() & 0x3F) + 1;
    let mut msg = vec![0u8; msg_len as usize];
    rng.fill_bytes(&mut msg);

    let sizes = vec![8, 16, 32, 64, 128];

    let bds: Vec<_> = (0..sizes[sizes.len() - 1])
        .map(|_| gen_bench_data_for_msg(&mut rng, &msg))
        .collect();

    for size in sizes.iter() {
        let pks_refs = bds
            .iter()
            .take(*size)
            .map(|s| &s.pk)
            .collect::<Vec<&PublicKey>>();

        let sig_refs = bds
            .iter()
            .take(*size)
            .map(|s| &s.sig)
            .collect::<Vec<&Signature>>();

        let agg = match AggregateSignature::aggregate(&sig_refs, false) {
            Ok(agg) => agg,
            Err(err) => panic!("aggregate failure: {:?}", err),
        };
        let agg_sig = agg.to_signature();

        let agg_pks = match AggregatePublicKey::aggregate(&pks_refs, false) {
            Ok(agg_pks) => agg_pks,
            Err(err) => panic!("aggregate failure: {:?}", err),
        };
        let agg_pk = agg_pks.to_public_key();

        let agg_ver = (agg_sig, pks_refs, &bds[0].msg, &bds[0].dst);
        let agg_pre_ver = (agg_sig, agg_pk, &bds[0].msg, &bds[0].dst);

        group.bench_with_input(
            BenchmarkId::new("fast_aggregate_verify", size),
            &agg_ver,
            |b, (a, p, m, d)| {
                b.iter(|| {
                    let result = a.fast_aggregate_verify(true, &m, &d, &p);
                    assert_eq!(result, BLST_ERROR::BLST_SUCCESS);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("fast_aggregate_verify_preagg", size),
            &agg_pre_ver,
            |b, (a, p, m, d)| {
                b.iter(|| {
                    let result = a
                        .fast_aggregate_verify_pre_aggregated(true, &m, &d, &p);
                    assert_eq!(result, BLST_ERROR::BLST_SUCCESS);
                });
            },
        );
    }

    group.finish();
}

fn bench_aggregate_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_verify");

    let seed = [0u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);

    let sizes = vec![8, 16, 32, 64, 128];
    // [10, 50, 100, 300, 1000, 4000];

    let bds: Vec<_> = (0..sizes[sizes.len() - 1])
        .map(|_| gen_bench_data(&mut rng))
        .collect();

    for size in sizes.iter() {
        let msgs_refs = bds
            .iter()
            .take(*size)
            .map(|s| s.msg.as_slice())
            .collect::<Vec<&[u8]>>();

        let pks_refs = bds
            .iter()
            .take(*size)
            .map(|s| &s.pk)
            .collect::<Vec<&PublicKey>>();

        let sig_refs = bds
            .iter()
            .take(*size)
            .map(|s| &s.sig)
            .collect::<Vec<&Signature>>();

        let agg = match AggregateSignature::aggregate(&sig_refs, false) {
            Ok(agg) => agg,
            Err(err) => panic!("aggregate failure: {:?}", err),
        };
        let agg_sig = agg.to_signature();
        let agg_ver = (agg_sig, pks_refs, msgs_refs, &bds[0].dst);

        group.bench_with_input(
            BenchmarkId::new("aggregate_verify", size),
            &agg_ver,
            |b, (a, p, m, d)| {
                b.iter(|| {
                    let result = a.aggregate_verify(true, &m, &d, &p, false);
                    assert_eq!(result, BLST_ERROR::BLST_SUCCESS);
                });
            },
        );
    }

    group.finish();
}

fn bench_aggregate(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate");

    let seed = [0u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);

    let sizes: [usize; 6] = [10, 50, 100, 300, 1000, 4000];

    let bds: Vec<_> = (0..4000).map(|_| gen_bench_data(&mut rng)).collect();

    for size in sizes.iter() {
        let sig_refs = bds
            .iter()
            .take(*size)
            .map(|s| &s.sig)
            .collect::<Vec<&Signature>>();

        group.bench_with_input(
            BenchmarkId::new("aggregate_signature", size),
            &sig_refs,
            |b, s| {
                b.iter(|| AggregateSignature::aggregate(&s, false));
            },
        );

        let pks_refs = bds
            .iter()
            .take(*size)
            .map(|s| &s.pk)
            .collect::<Vec<&PublicKey>>();

        group.bench_with_input(
            BenchmarkId::new("aggregate_public_key", size),
            &pks_refs,
            |b, p| {
                b.iter(|| AggregatePublicKey::aggregate(&p, false));
            },
        );
    }

    group.finish();
}

fn bench_single_message(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_message");

    let seed = [0u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);
    let bd = gen_bench_data(&mut rng);

    group.bench_function("sign", |b| {
        b.iter(|| bd.sk.sign(&bd.msg, &bd.dst, &[]))
    });

    group.bench_function("verify", |b| {
        b.iter(|| bd.sig.verify(true, &bd.msg, &bd.dst, &[], &bd.pk, false))
    });

    group.finish();
}

fn bench_serdes(c: &mut Criterion) {
    let mut group = c.benchmark_group("serdes");

    let seed = [0u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);
    let bd = gen_bench_data(&mut rng);

    let sk = bd.sk;
    let sk_ser = sk.serialize();

    let pk = bd.pk;
    let pk_comp = pk.compress();
    let pk_ser = pk.serialize();

    let sig = bd.sig;
    let sig_comp = sig.compress();
    let sig_ser = sig.serialize();

    let mut pk_jac = std::mem::MaybeUninit::<blst_p1>::uninit();
    let mut sig_jac = std::mem::MaybeUninit::<blst_p2>::uninit();

    let mut p1_comp = [0; 48];
    let mut p2_comp = [0; 96];
    let mut p1_ser = [0; 96];
    let mut p2_ser = [0; 192];

    unsafe {
        let mut junk = [0u8; 32];
        rng.fill_bytes(&mut junk);
        blst_encode_to_g1(
            pk_jac.as_mut_ptr(),
            junk.as_ptr(),
            junk.len(),
            "junk".as_ptr(),
            4,
            std::ptr::null(),
            0,
        );
        blst_encode_to_g2(
            sig_jac.as_mut_ptr(),
            junk.as_ptr(),
            junk.len(),
            "junk".as_ptr(),
            4,
            std::ptr::null(),
            0,
        );
    }

    group.bench_function("secret_key_serialize", |b| b.iter(|| sk.serialize()));

    group.bench_function("secret_key_deserialize", |b| {
        b.iter(|| SecretKey::deserialize(&sk_ser));
    });

    group.bench_function("public_key_serialize", |b| b.iter(|| pk.serialize()));

    group.bench_function("public_key_compress", |b| b.iter(|| pk.compress()));

    group.bench_function("public_key_uncompress", |b| {
        b.iter(|| PublicKey::uncompress(&pk_comp))
    });

    group.bench_function("public_key_deserialize", |b| {
        b.iter(|| PublicKey::deserialize(&pk_ser));
    });

    group.bench_function("signature_serialize", |b| b.iter(|| sig.serialize()));

    group.bench_function("signature_compress", |b| b.iter(|| sig.compress()));

    group.bench_function("signature_uncompress", |b| {
        b.iter(|| Signature::uncompress(&sig_comp))
    });

    group.bench_function("signature_deserialize", |b| {
        b.iter(|| Signature::deserialize(&sig_ser))
    });

    group.bench_function("p1_serialize", |b| {
        b.iter(|| unsafe {
            blst_p1_serialize(p1_ser.as_mut_ptr(), pk_jac.as_ptr())
        })
    });

    group.bench_function("p1_compress", |b| {
        b.iter(|| unsafe {
            blst_p1_compress(p1_comp.as_mut_ptr(), pk_jac.as_ptr())
        })
    });

    group.bench_function("p2_serialize", |b| {
        b.iter(|| unsafe {
            blst_p2_serialize(p2_ser.as_mut_ptr(), sig_jac.as_ptr())
        })
    });

    group.bench_function("p2_compress", |b| {
        b.iter(|| unsafe {
            blst_p2_compress(p2_comp.as_mut_ptr(), sig_jac.as_ptr())
        })
    });

    group.finish();
}

fn bench_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("keys");
    let ikm: [u8; 32] = [
        0x93, 0xad, 0x7e, 0x65, 0xde, 0xad, 0x05, 0x2a, 0x08, 0x3a, 0x91, 0x0c,
        0x8b, 0x72, 0x85, 0x91, 0x46, 0x4c, 0xca, 0x56, 0x60, 0x5b, 0xb0, 0x56,
        0xed, 0xfe, 0x2b, 0x60, 0xa6, 0x3c, 0x48, 0x99,
    ];
    let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
    let pk = sk.sk_to_pk();
    let pk_comp = pk.compress();

    group.bench_function("key_gen", |b| {
        b.iter(|| SecretKey::key_gen(&ikm, &[]))
    });

    group.bench_function("sk_to_pk", |b| {
        b.iter(|| sk.sk_to_pk());
    });

    group.bench_function("key_validate", |b| {
        b.iter(|| PublicKey::key_validate(&pk_comp));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_verify_multi_aggregate,
    bench_fast_aggregate_verify,
    bench_aggregate_verify,
    bench_aggregate,
    bench_single_message,
    bench_serdes,
    bench_keys
);
criterion_main!(benches);
