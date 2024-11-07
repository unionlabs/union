// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// This module provides functionality for generating secure randomness.
module sui::random;

use std::bcs;
use sui::hmac::hmac_sha3_256;
use sui::versioned::{Self, Versioned};

// Sender is not @0x0 the system address.
const ENotSystemAddress: u64 = 0;
const EWrongInnerVersion: u64 = 1;
const EInvalidRandomnessUpdate: u64 = 2;
const EInvalidRange: u64 = 3;
const EInvalidLength: u64 = 4;

const CURRENT_VERSION: u64 = 1;
const RAND_OUTPUT_LEN: u16 = 32;
const U16_MAX: u64 = 0xFFFF;

/// Singleton shared object which stores the global randomness state.
/// The actual state is stored in a versioned inner field.
public struct Random has key {
    id: UID,
    // The inner object must never be accessed outside this module as it could be used for accessing global
    // randomness via deserialization of RandomInner.
    inner: Versioned,
}

public struct RandomInner has store {
    version: u64,
    epoch: u64,
    randomness_round: u64,
    random_bytes: vector<u8>,
}

#[allow(unused_function)]
/// Create and share the Random object. This function is called exactly once, when
/// the Random object is first created.
/// Can only be called by genesis or change_epoch transactions.
fun create(ctx: &mut TxContext) {
    assert!(ctx.sender() == @0x0, ENotSystemAddress);

    let version = CURRENT_VERSION;

    let inner = RandomInner {
        version,
        epoch: ctx.epoch(),
        randomness_round: 0,
        random_bytes: vector[],
    };

    let self = Random {
        id: object::randomness_state(),
        inner: versioned::create(version, inner, ctx),
    };
    transfer::share_object(self);
}

#[test_only]
public fun create_for_testing(ctx: &mut TxContext) {
    create(ctx);
}

fun load_inner_mut(self: &mut Random): &mut RandomInner {
    let version = versioned::version(&self.inner);

    // Replace this with a lazy update function when we add a new version of the inner object.
    assert!(version == CURRENT_VERSION, EWrongInnerVersion);
    let inner: &mut RandomInner = versioned::load_value_mut(&mut self.inner);
    assert!(inner.version == version, EWrongInnerVersion);
    inner
}

fun load_inner(self: &Random): &RandomInner {
    let version = versioned::version(&self.inner);

    // Replace this with a lazy update function when we add a new version of the inner object.
    assert!(version == CURRENT_VERSION, EWrongInnerVersion);
    let inner: &RandomInner = versioned::load_value(&self.inner);
    assert!(inner.version == version, EWrongInnerVersion);
    inner
}

#[allow(unused_function)]
/// Record new randomness. Called when executing the RandomnessStateUpdate system
/// transaction.
fun update_randomness_state(
    self: &mut Random,
    new_round: u64,
    new_bytes: vector<u8>,
    ctx: &TxContext,
) {
    // Validator will make a special system call with sender set as 0x0.
    assert!(ctx.sender() == @0x0, ENotSystemAddress);

    // Randomness should only be incremented.
    let epoch = ctx.epoch();
    let inner = self.load_inner_mut();
    if (inner.randomness_round == 0 && inner.epoch == 0 && inner.random_bytes.is_empty()) {
        // First update should be for round zero.
        assert!(new_round == 0, EInvalidRandomnessUpdate);
    } else {
        // Subsequent updates should either increase epoch or increment randomness_round.
        // Note that epoch may increase by more than 1 if an epoch is completed without
        // randomness ever being generated in that epoch.
        assert!(
            (epoch > inner.epoch && new_round == 0) ||
                    (new_round == inner.randomness_round + 1),
            EInvalidRandomnessUpdate,
        );
    };

    inner.epoch = ctx.epoch();
    inner.randomness_round = new_round;
    inner.random_bytes = new_bytes;
}

#[test_only]
public fun update_randomness_state_for_testing(
    self: &mut Random,
    new_round: u64,
    new_bytes: vector<u8>,
    ctx: &TxContext,
) {
    self.update_randomness_state(new_round, new_bytes, ctx);
}

/// Unique randomness generator, derived from the global randomness.
public struct RandomGenerator has drop {
    seed: vector<u8>,
    counter: u16,
    buffer: vector<u8>,
}

/// Create a generator. Can be used to derive up to MAX_U16 * 32 random bytes.
///
/// Using randomness can be error-prone if you don't observe the subtleties in its correct use, for example, randomness
/// dependent code might be exploitable to attacks that carefully set the gas budget
/// in a way that breaks security. For more information, see:
/// https://docs.sui.io/guides/developer/advanced/randomness-onchain
public fun new_generator(r: &Random, ctx: &mut TxContext): RandomGenerator {
    let inner = load_inner(r);
    let seed = hmac_sha3_256(
        &inner.random_bytes,
        &ctx.fresh_object_address().to_bytes(),
    );
    RandomGenerator { seed, counter: 0, buffer: vector[] }
}

// Get the next block of random bytes.
fun derive_next_block(g: &mut RandomGenerator): vector<u8> {
    g.counter = g.counter + 1;
    hmac_sha3_256(&g.seed, &bcs::to_bytes(&g.counter))
}

// Fill the generator's buffer with 32 random bytes.
fun fill_buffer(g: &mut RandomGenerator) {
    let next_block = derive_next_block(g);
    vector::append(&mut g.buffer, next_block);
}

/// Generate n random bytes.
public fun generate_bytes(g: &mut RandomGenerator, num_of_bytes: u16): vector<u8> {
    let mut result = vector[];
    // Append RAND_OUTPUT_LEN size buffers directly without going through the generator's buffer.
    let mut num_of_blocks = num_of_bytes / RAND_OUTPUT_LEN;
    while (num_of_blocks > 0) {
        vector::append(&mut result, derive_next_block(g));
        num_of_blocks = num_of_blocks - 1;
    };
    // Fill the generator's buffer if needed.
    let num_of_bytes = num_of_bytes as u64;
    if (vector::length(&g.buffer) < (num_of_bytes - vector::length(&result))) {
        fill_buffer(g);
    };
    // Take remaining bytes from the generator's buffer.
    while (vector::length(&result) < num_of_bytes) {
        vector::push_back(&mut result, vector::pop_back(&mut g.buffer));
    };
    result
}

// Helper function that extracts the given number of bytes from the random generator and returns it as u256.
// Assumes that the caller has already checked that num_of_bytes is valid.
// TODO: Replace with a macro when we have support for it.
fun u256_from_bytes(g: &mut RandomGenerator, num_of_bytes: u8): u256 {
    if (vector::length(&g.buffer) < num_of_bytes as u64) {
        fill_buffer(g);
    };
    let mut result: u256 = 0;
    let mut i = 0;
    while (i < num_of_bytes) {
        let byte = vector::pop_back(&mut g.buffer);
        result = (result << 8) + (byte as u256);
        i = i + 1;
    };
    result
}

/// Generate a u256.
public fun generate_u256(g: &mut RandomGenerator): u256 {
    u256_from_bytes(g, 32)
}

/// Generate a u128.
public fun generate_u128(g: &mut RandomGenerator): u128 {
    u256_from_bytes(g, 16) as u128
}

/// Generate a u64.
public fun generate_u64(g: &mut RandomGenerator): u64 {
    u256_from_bytes(g, 8) as u64
}

/// Generate a u32.
public fun generate_u32(g: &mut RandomGenerator): u32 {
    u256_from_bytes(g, 4) as u32
}

/// Generate a u16.
public fun generate_u16(g: &mut RandomGenerator): u16 {
    u256_from_bytes(g, 2) as u16
}

/// Generate a u8.
public fun generate_u8(g: &mut RandomGenerator): u8 {
    u256_from_bytes(g, 1) as u8
}

/// Generate a boolean.
public fun generate_bool(g: &mut RandomGenerator): bool {
    (u256_from_bytes(g, 1) & 1) == 1
}

// Helper function to generate a random u128 in [min, max] using a random number with num_of_bytes bytes.
// Assumes that the caller verified the inputs, and uses num_of_bytes to control the bias (e.g., 8 bytes larger
// than the actual type used by the caller function to limit the bias by 2^{-64}).
// TODO: Replace with a macro when we have support for it.
fun u128_in_range(g: &mut RandomGenerator, min: u128, max: u128, num_of_bytes: u8): u128 {
    assert!(min <= max, EInvalidRange);
    if (min == max) {
        return min
    };
    // Pick a random number in [0, max - min] by generating a random number that is larger than max-min, and taking
    // the modulo of the random number by the range size. Then add the min to the result to get a number in
    // [min, max].
    let range_size = (max - min) as u256 + 1;
    let rand = u256_from_bytes(g, num_of_bytes);
    min + (rand % range_size as u128)
}

/// Generate a random u128 in [min, max] (with a bias of 2^{-64}).
public fun generate_u128_in_range(g: &mut RandomGenerator, min: u128, max: u128): u128 {
    u128_in_range(g, min, max, 24)
}

//// Generate a random u64 in [min, max] (with a bias of 2^{-64}).
public fun generate_u64_in_range(g: &mut RandomGenerator, min: u64, max: u64): u64 {
    u128_in_range(g, min as u128, max as u128, 16) as u64
}

/// Generate a random u32 in [min, max] (with a bias of 2^{-64}).
public fun generate_u32_in_range(g: &mut RandomGenerator, min: u32, max: u32): u32 {
    u128_in_range(g, min as u128, max as u128, 12) as u32
}

/// Generate a random u16 in [min, max] (with a bias of 2^{-64}).
public fun generate_u16_in_range(g: &mut RandomGenerator, min: u16, max: u16): u16 {
    u128_in_range(g, min as u128, max as u128, 10) as u16
}

/// Generate a random u8 in [min, max] (with a bias of 2^{-64}).
public fun generate_u8_in_range(g: &mut RandomGenerator, min: u8, max: u8): u8 {
    u128_in_range(g, min as u128, max as u128, 9) as u8
}

/// Shuffle a vector using the random generator (Fisher–Yates/Knuth shuffle).
public fun shuffle<T>(g: &mut RandomGenerator, v: &mut vector<T>) {
    let n = vector::length(v);
    if (n == 0) {
        return
    };
    assert!(n <= U16_MAX, EInvalidLength);
    let n = n as u16;
    let mut i: u16 = 0;
    let end = n - 1;
    while (i < end) {
        let j = generate_u16_in_range(g, i, end);
        vector::swap(v, i as u64, j as u64);
        i = i + 1;
    };
}

#[test_only]
public fun generator_seed(r: &RandomGenerator): &vector<u8> {
    &r.seed
}

#[test_only]
public fun generator_counter(r: &RandomGenerator): u16 {
    r.counter
}

#[test_only]
public fun generator_buffer(r: &RandomGenerator): &vector<u8> {
    &r.buffer
}

#[test_only]
/// Random generator from a non-deterministic seed.
/// To be used when non-deterministic randomness is needed in tests (e.g., fuzzing).
public fun new_generator_for_testing(): RandomGenerator {
    let seed = generate_rand_seed_for_testing();
    new_generator_from_seed_for_testing(seed)
}

#[test_only]
/// Random generator from a given seed.
public fun new_generator_from_seed_for_testing(seed: vector<u8>): RandomGenerator {
    RandomGenerator { seed, counter: 0, buffer: vector[] }
}

#[test_only]
native fun generate_rand_seed_for_testing(): vector<u8>;
