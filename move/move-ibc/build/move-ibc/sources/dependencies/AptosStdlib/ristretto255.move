/// This module contains functions for Ristretto255 curve arithmetic, assuming addition as the group operation.
///
/// The order of the Ristretto255 elliptic curve group is $\ell = 2^252 + 27742317777372353535851937790883648493$, same
/// as the order of the prime-order subgroup of Curve25519.
///
/// This module provides two structs for encoding Ristretto elliptic curves to the developer:
///
///  - First, a 32-byte-sized CompressedRistretto struct, which is used to persist points in storage.
///
///  - Second, a larger, in-memory, RistrettoPoint struct, which is decompressable from a CompressedRistretto struct. This
/// larger struct can be used for fast arithmetic operations (additions, multiplications, etc.). The results can be saved
/// back into storage by compressing RistrettoPoint structs back to CompressedRistretto structs.
///
/// This module also provides a Scalar struct for persisting scalars in storage and doing fast arithmetic on them.
///
/// One invariant maintained by this module is that all CompressedRistretto structs store a canonically-encoded point,
/// which can always be decompressed into a valid point on the curve as a RistrettoPoint struct. Unfortunately, due to
/// limitations in our underlying curve25519-dalek elliptic curve library, this decompression will unnecessarily verify
/// the validity of the point and thus slightly decrease performance.
///
/// Similarly, all Scalar structs store a canonically-encoded scalar, which can always be safely operated on using
/// arithmetic operations.
///
/// In the future, we might support additional features:
///
/// * For scalars:
///    - batch_invert()
///
///  * For points:
///    - double()
///      + The challenge is that curve25519-dalek does NOT export double for Ristretto points (nor for Edwards)
///
///    - double_and_compress_batch()
///
///    - fixed-base, variable-time via optional_mixed_multiscalar_mul() in VartimePrecomputedMultiscalarMul
///      + This would require a storage-friendly RistrettoBasepointTable and an in-memory variant of it too
///      + Similar to the CompressedRistretto and RistrettoPoint structs in this module
///      + The challenge is that curve25519-dalek's RistrettoBasepointTable is not serializable

module aptos_std::ristretto255 {
    use std::features;
    use std::option::Option;

    #[test_only]
    use std::option;

    //
    // Constants
    //

    /// The order of the Ristretto255 group and its scalar field, in little-endian.
    const ORDER_ELL: vector<u8> = x"edd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010";

    /// `ORDER_ELL` - 1: i.e., the "largest", reduced scalar in the field
    const L_MINUS_ONE: vector<u8> = x"ecd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010";

    /// The maximum size in bytes of a canonically-encoded Scalar is 32 bytes.
    const MAX_SCALAR_NUM_BYTES: u64 = 32u64;

    /// The maximum size in bits of a canonically-encoded Scalar is 256 bits.
    const MAX_SCALAR_NUM_BITS: u64 = 256u64;

    /// The maximum size in bytes of a canonically-encoded Ristretto255 point is 32 bytes.
    const MAX_POINT_NUM_BYTES: u64 = 32u64;

    /// The basepoint (generator) of the Ristretto255 group
    const BASE_POINT: vector<u8> = x"e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d76";

    /// The hash of the basepoint of the Ristretto255 group using SHA3_512
    const HASH_BASE_POINT: vector<u8> = x"8c9240b456a9e6dc65c377a1048d745f94a08cdb7f44cbcd7b46f34048871134";

    //
    // Reasons for error codes
    //

    /// The number of scalars does not match the number of points.
    const E_DIFFERENT_NUM_POINTS_AND_SCALARS: u64 = 1;
    /// Expected more than zero points as input.
    const E_ZERO_POINTS: u64 = 2;
    /// Expected more than zero scalars as input.
    const E_ZERO_SCALARS: u64 = 3;
    /// Too many points have been created in the current transaction execution.
    const E_TOO_MANY_POINTS_CREATED: u64 = 4;
    /// The native function has not been deployed yet.
    const E_NATIVE_FUN_NOT_AVAILABLE: u64 = 5;

    //
    // Scalar and point structs
    //

    /// This struct represents a scalar as a little-endian byte encoding of an integer in $\mathbb{Z}_\ell$, which is
    /// stored in `data`. Here, \ell denotes the order of the scalar field (and the underlying elliptic curve group).
    struct Scalar has copy, store, drop {
        data: vector<u8>
    }

    /// This struct represents a serialized point on the Ristretto255 curve, in 32 bytes.
    /// This struct can be decompressed from storage into an in-memory RistrettoPoint, on which fast curve arithmetic
    /// can be performed.
    struct CompressedRistretto has copy, store, drop {
        data: vector<u8>
    }

    /// This struct represents an in-memory Ristretto255 point and supports fast curve arithmetic.
    ///
    /// An important invariant: There will never be two RistrettoPoint's constructed with the same handle. One can have
    /// immutable references to the same RistrettoPoint, of course.
    struct RistrettoPoint has drop {
        handle: u64
    }

    //
    // Functions for arithmetic on points
    //

    /// Returns the identity point as a CompressedRistretto.
    public fun point_identity_compressed(): CompressedRistretto {
        CompressedRistretto {
            data: x"0000000000000000000000000000000000000000000000000000000000000000"
        }
    }

    /// Returns the identity point as a CompressedRistretto.
    public fun point_identity(): RistrettoPoint {
        RistrettoPoint {
            handle: point_identity_internal()
        }
    }

    /// Returns the basepoint (generator) of the Ristretto255 group as a compressed point
    public fun basepoint_compressed(): CompressedRistretto {
        CompressedRistretto {
            data: BASE_POINT
        }
    }

    /// Returns the hash-to-point result of serializing the basepoint of the Ristretto255 group.
    /// For use as the random value basepoint in Pedersen commitments
    public fun hash_to_point_base(): RistrettoPoint {
        let comp_res = CompressedRistretto { data: HASH_BASE_POINT };
        point_decompress(&comp_res)
    }

    /// Returns the basepoint (generator) of the Ristretto255 group
    public fun basepoint(): RistrettoPoint {
        let (handle, _) = point_decompress_internal(BASE_POINT);

        RistrettoPoint {
            handle
        }
    }

    /// Multiplies the basepoint (generator) of the Ristretto255 group by a scalar and returns the result.
    /// This call is much faster than `point_mul(&basepoint(), &some_scalar)` because of precomputation tables.
    public fun basepoint_mul(a: &Scalar): RistrettoPoint {
        RistrettoPoint {
            handle: basepoint_mul_internal(a.data)
        }
    }

    /// Creates a new CompressedRistretto point from a sequence of 32 bytes. If those bytes do not represent a valid
    /// point, returns None.
    public fun new_compressed_point_from_bytes(bytes: vector<u8>): Option<CompressedRistretto> {
        if (point_is_canonical_internal(bytes)) {
            std::option::some(CompressedRistretto {
                data: bytes
            })
        } else {
            std::option::none<CompressedRistretto>()
        }
    }

    /// Creates a new RistrettoPoint from a sequence of 32 bytes. If those bytes do not represent a valid point,
    /// returns None.
    public fun new_point_from_bytes(bytes: vector<u8>): Option<RistrettoPoint> {
        let (handle, is_canonical) = point_decompress_internal(bytes);
        if (is_canonical) {
            std::option::some(RistrettoPoint { handle })
        } else {
            std::option::none<RistrettoPoint>()
        }
    }

    /// Given a compressed ristretto point `point`, returns the byte representation of that point
    public fun compressed_point_to_bytes(point: CompressedRistretto): vector<u8> {
        point.data
    }

    /// DEPRECATED: Use the more clearly-named `new_point_from_sha2_512`
    ///
    /// Hashes the input to a uniformly-at-random RistrettoPoint via SHA512.
    public fun new_point_from_sha512(sha2_512_input: vector<u8>): RistrettoPoint {
        new_point_from_sha2_512(sha2_512_input)
    }

    /// Hashes the input to a uniformly-at-random RistrettoPoint via SHA2-512.
    public fun new_point_from_sha2_512(sha2_512_input: vector<u8>): RistrettoPoint {
        RistrettoPoint {
            handle: new_point_from_sha512_internal(sha2_512_input)
        }
    }

    /// Samples a uniformly-at-random RistrettoPoint given a sequence of 64 uniformly-at-random bytes. This function
    /// can be used to build a collision-resistant hash function that maps 64-byte messages to RistrettoPoint's.
    public fun new_point_from_64_uniform_bytes(bytes: vector<u8>): Option<RistrettoPoint> {
        if (std::vector::length(&bytes) == 64) {
            std::option::some(RistrettoPoint {
                handle: new_point_from_64_uniform_bytes_internal(bytes)
            })
        } else {
            std::option::none<RistrettoPoint>()
        }
    }

    /// Decompresses a CompressedRistretto from storage into a RistrettoPoint which can be used for fast arithmetic.
    public fun point_decompress(point: &CompressedRistretto): RistrettoPoint {
        // NOTE: Our CompressedRistretto invariant assures us that every CompressedRistretto in storage is a valid
        // RistrettoPoint
        let (handle, _) = point_decompress_internal(point.data);
        RistrettoPoint { handle }
    }

    /// Clones a RistrettoPoint.
    public fun point_clone(point: &RistrettoPoint): RistrettoPoint {
        if(!features::bulletproofs_enabled()) {
            abort(std::error::invalid_state(E_NATIVE_FUN_NOT_AVAILABLE))
        };

        RistrettoPoint {
            handle: point_clone_internal(point.handle)
        }
    }

    /// Compresses a RistrettoPoint to a CompressedRistretto which can be put in storage.
    public fun point_compress(point: &RistrettoPoint): CompressedRistretto {
        CompressedRistretto {
            data: point_compress_internal(point)
        }
    }

    /// Returns the sequence of bytes representin this Ristretto point.
    /// To convert a RistrettoPoint 'p' to bytes, first compress it via `c = point_compress(&p)`, and then call this
    /// function on `c`.
    public fun point_to_bytes(point: &CompressedRistretto): vector<u8> {
        point.data
    }

    /// Returns a * point.
    public fun point_mul(point: &RistrettoPoint, a: &Scalar): RistrettoPoint {
        RistrettoPoint {
            handle: point_mul_internal(point, a.data, false)
        }
    }

    /// Sets a *= point and returns 'a'.
    public fun point_mul_assign(point: &mut RistrettoPoint, a: &Scalar): &mut RistrettoPoint {
        point_mul_internal(point, a.data, true);
        point
    }

    /// Returns (a * a_base + b * base_point), where base_point is the Ristretto basepoint encoded in `BASE_POINT`.
    public fun basepoint_double_mul(a: &Scalar, a_base: &RistrettoPoint, b: &Scalar): RistrettoPoint {
        RistrettoPoint {
            handle: basepoint_double_mul_internal(a.data, a_base, b.data)
        }
    }

    /// Returns a + b
    public fun point_add(a: &RistrettoPoint, b: &RistrettoPoint): RistrettoPoint {
        RistrettoPoint {
            handle: point_add_internal(a, b, false)
        }
    }

    /// Sets a += b and returns 'a'.
    public fun point_add_assign(a: &mut RistrettoPoint, b: &RistrettoPoint): &mut RistrettoPoint {
        point_add_internal(a, b, true);
        a
    }

    /// Returns a - b
    public fun point_sub(a: &RistrettoPoint, b: &RistrettoPoint): RistrettoPoint {
        RistrettoPoint {
            handle: point_sub_internal(a, b, false)
        }
    }

    /// Sets a -= b and returns 'a'.
    public fun point_sub_assign(a: &mut RistrettoPoint, b: &RistrettoPoint): &mut RistrettoPoint {
        point_sub_internal(a, b, true);
        a
    }

    /// Returns -a
    public fun point_neg(a: &RistrettoPoint): RistrettoPoint {
        RistrettoPoint {
            handle: point_neg_internal(a, false)
        }
    }

    /// Sets a = -a, and returns 'a'.
    public fun point_neg_assign(a: &mut RistrettoPoint): &mut RistrettoPoint {
        point_neg_internal(a, true);
        a
    }

    /// Returns true if the two RistrettoPoints are the same points on the elliptic curve.
    native public fun point_equals(g: &RistrettoPoint, h: &RistrettoPoint): bool;

    /// Computes a double-scalar multiplication, returning a_1 p_1 + a_2 p_2
    /// This function is much faster than computing each a_i p_i using `point_mul` and adding up the results using `point_add`.
    public fun double_scalar_mul(scalar1: &Scalar, point1: &RistrettoPoint, scalar2: &Scalar, point2: &RistrettoPoint): RistrettoPoint {
        if(!features::bulletproofs_enabled()) {
            abort(std::error::invalid_state(E_NATIVE_FUN_NOT_AVAILABLE))
        };

        RistrettoPoint {
            handle: double_scalar_mul_internal(point1.handle, point2.handle, scalar1.data, scalar2.data)
        }
    }

    /// Computes a multi-scalar multiplication, returning a_1 p_1 + a_2 p_2 + ... + a_n p_n.
    /// This function is much faster than computing each a_i p_i using `point_mul` and adding up the results using `point_add`.
    public fun multi_scalar_mul(points: &vector<RistrettoPoint>, scalars: &vector<Scalar>): RistrettoPoint {
        assert!(!std::vector::is_empty(points), std::error::invalid_argument(E_ZERO_POINTS));
        assert!(!std::vector::is_empty(scalars), std::error::invalid_argument(E_ZERO_SCALARS));
        assert!(std::vector::length(points) == std::vector::length(scalars), std::error::invalid_argument(E_DIFFERENT_NUM_POINTS_AND_SCALARS));

        RistrettoPoint {
            handle: multi_scalar_mul_internal<RistrettoPoint, Scalar>(points, scalars)
        }
    }

    //
    // Functions for arithmetic on Scalars
    //

    /// Given a sequence of 32 bytes, checks if they canonically-encode a Scalar and return it.
    /// Otherwise, returns None.
    public fun new_scalar_from_bytes(bytes: vector<u8>): Option<Scalar> {
        if (scalar_is_canonical_internal(bytes)) {
            std::option::some(Scalar {
                data: bytes
            })
        } else {
            std::option::none<Scalar>()
        }
    }

    /// DEPRECATED: Use the more clearly-named `new_scalar_from_sha2_512`
    ///
    /// Hashes the input to a uniformly-at-random Scalar via SHA2-512
    public fun new_scalar_from_sha512(sha2_512_input: vector<u8>): Scalar {
        new_scalar_from_sha2_512(sha2_512_input)
    }

    /// Hashes the input to a uniformly-at-random Scalar via SHA2-512
    public fun new_scalar_from_sha2_512(sha2_512_input: vector<u8>): Scalar {
        Scalar {
            data: scalar_from_sha512_internal(sha2_512_input)
        }
    }

    /// Creates a Scalar from an u8.
    public fun new_scalar_from_u8(byte: u8): Scalar {
        let s = scalar_zero();
        let byte_zero = std::vector::borrow_mut(&mut s.data, 0);
        *byte_zero = byte;

        s
    }

    /// Creates a Scalar from an u32.
    public fun new_scalar_from_u32(four_bytes: u32): Scalar {
        Scalar {
            data: scalar_from_u64_internal((four_bytes as u64))
        }
    }

    /// Creates a Scalar from an u64.
    public fun new_scalar_from_u64(eight_bytes: u64): Scalar {
        Scalar {
            data: scalar_from_u64_internal(eight_bytes)
        }
    }

    /// Creates a Scalar from an u128.
    public fun new_scalar_from_u128(sixteen_bytes: u128): Scalar {
        Scalar {
            data: scalar_from_u128_internal(sixteen_bytes)
        }
    }

    /// Creates a Scalar from 32 bytes by reducing the little-endian-encoded number in those bytes modulo $\ell$.
    public fun new_scalar_reduced_from_32_bytes(bytes: vector<u8>): Option<Scalar> {
        if (std::vector::length(&bytes) == 32) {
            std::option::some(Scalar {
                data: scalar_reduced_from_32_bytes_internal(bytes)
            })
        } else {
            std::option::none()
        }
    }

    /// Samples a scalar uniformly-at-random given 64 uniform-at-random bytes as input by reducing the little-endian-encoded number
    /// in those bytes modulo $\ell$.
    public fun new_scalar_uniform_from_64_bytes(bytes: vector<u8>): Option<Scalar> {
        if (std::vector::length(&bytes) == 64) {
            std::option::some(Scalar {
                data: scalar_uniform_from_64_bytes_internal(bytes)
            })
        } else {
            std::option::none()
        }
    }

    /// Returns 0 as a Scalar.
    public fun scalar_zero(): Scalar {
        Scalar {
            data: x"0000000000000000000000000000000000000000000000000000000000000000"
        }
    }

    /// Returns true if the given Scalar equals 0.
    public fun scalar_is_zero(s: &Scalar): bool {
        s.data == x"0000000000000000000000000000000000000000000000000000000000000000"
    }

    /// Returns 1 as a Scalar.
    public fun scalar_one(): Scalar {
        Scalar {
            data: x"0100000000000000000000000000000000000000000000000000000000000000"
        }
    }

    /// Returns true if the given Scalar equals 1.
    public fun scalar_is_one(s: &Scalar): bool {
        s.data == x"0100000000000000000000000000000000000000000000000000000000000000"
    }

    /// Returns true if the two scalars are equal.
    public fun scalar_equals(lhs: &Scalar, rhs: &Scalar): bool {
        lhs.data == rhs.data
    }

    /// Returns the inverse s^{-1} mod \ell of a scalar s.
    /// Returns None if s is zero.
    public fun scalar_invert(s: &Scalar): Option<Scalar> {
        if (scalar_is_zero(s)) {
            std::option::none<Scalar>()
        } else {
            std::option::some(Scalar {
                data: scalar_invert_internal(s.data)
            })
        }
    }

    /// Returns the product of the two scalars.
    public fun scalar_mul(a: &Scalar, b: &Scalar): Scalar {
        Scalar {
            data: scalar_mul_internal(a.data, b.data)
        }
    }

    /// Computes the product of 'a' and 'b' and assigns the result to 'a'.
    /// Returns 'a'.
    public fun scalar_mul_assign(a: &mut Scalar, b: &Scalar): &mut Scalar {
        a.data = scalar_mul(a, b).data;
        a
    }

    /// Returns the sum of the two scalars.
    public fun scalar_add(a: &Scalar, b: &Scalar): Scalar {
        Scalar {
            data: scalar_add_internal(a.data, b.data)
        }
    }

    /// Computes the sum of 'a' and 'b' and assigns the result to 'a'
    /// Returns 'a'.
    public fun scalar_add_assign(a: &mut Scalar, b: &Scalar): &mut Scalar {
        a.data = scalar_add(a, b).data;
        a
    }

    /// Returns the difference of the two scalars.
    public fun scalar_sub(a: &Scalar, b: &Scalar): Scalar {
        Scalar {
            data: scalar_sub_internal(a.data, b.data)
        }
    }

    /// Subtracts 'b' from 'a' and assigns the result to 'a'.
    /// Returns 'a'.
    public fun scalar_sub_assign(a: &mut Scalar, b: &Scalar): &mut Scalar {
        a.data = scalar_sub(a, b).data;
        a
    }

    /// Returns the negation of 'a': i.e., $(0 - a) \mod \ell$.
    public fun scalar_neg(a: &Scalar): Scalar {
        Scalar {
            data: scalar_neg_internal(a.data)
        }
    }

    /// Replaces 'a' by its negation.
    ///  Returns 'a'.
    public fun scalar_neg_assign(a: &mut Scalar): &mut Scalar {
        a.data = scalar_neg(a).data;
        a
    }

    /// Returns the byte-representation of the scalar.
    public fun scalar_to_bytes(s: &Scalar): vector<u8> {
        s.data
    }

    //
    // Only used internally for implementing CompressedRistretto and RistrettoPoint
    //

    // NOTE: This was supposed to be more clearly named with *_sha2_512_*.
    native fun new_point_from_sha512_internal(sha2_512_input: vector<u8>): u64;

    native fun new_point_from_64_uniform_bytes_internal(bytes: vector<u8>): u64;

    native fun point_is_canonical_internal(bytes: vector<u8>): bool;

    native fun point_identity_internal(): u64;

    native fun point_decompress_internal(maybe_non_canonical_bytes: vector<u8>): (u64, bool);

    native fun point_clone_internal(point_handle: u64): u64;
    native fun point_compress_internal(point: &RistrettoPoint): vector<u8>;

    native fun point_mul_internal(point: &RistrettoPoint, a: vector<u8>, in_place: bool): u64;

    native fun basepoint_mul_internal(a: vector<u8>): u64;

    native fun basepoint_double_mul_internal(a: vector<u8>, some_point: &RistrettoPoint, b: vector<u8>): u64;

    native fun point_add_internal(a: &RistrettoPoint, b: &RistrettoPoint, in_place: bool): u64;

    native fun point_sub_internal(a: &RistrettoPoint, b: &RistrettoPoint, in_place: bool): u64;

    native fun point_neg_internal(a: &RistrettoPoint, in_place: bool): u64;

    native fun double_scalar_mul_internal(point1: u64, point2: u64, scalar1: vector<u8>, scalar2: vector<u8>): u64;

    /// The generic arguments are needed to deal with some Move VM peculiarities which prevent us from borrowing the
    /// points (or scalars) inside a &vector in Rust.
    ///
    /// WARNING: This function can only be called with P = RistrettoPoint and S = Scalar.
    native fun multi_scalar_mul_internal<P, S>(points: &vector<P>, scalars: &vector<S>): u64;

    //
    // Only used internally for implementing Scalar.
    //

    native fun scalar_is_canonical_internal(s: vector<u8>): bool;

    native fun scalar_from_u64_internal(num: u64): vector<u8>;

    native fun scalar_from_u128_internal(num: u128): vector<u8>;

    native fun scalar_reduced_from_32_bytes_internal(bytes: vector<u8>): vector<u8>;

    native fun scalar_uniform_from_64_bytes_internal(bytes: vector<u8>): vector<u8>;

    native fun scalar_invert_internal(bytes: vector<u8>): vector<u8>;

    // NOTE: This was supposed to be more clearly named with *_sha2_512_*.
    native fun scalar_from_sha512_internal(sha2_512_input: vector<u8>): vector<u8>;

    native fun scalar_mul_internal(a_bytes: vector<u8>, b_bytes: vector<u8>): vector<u8>;

    native fun scalar_add_internal(a_bytes: vector<u8>, b_bytes: vector<u8>): vector<u8>;

    native fun scalar_sub_internal(a_bytes: vector<u8>, b_bytes: vector<u8>): vector<u8>;

    native fun scalar_neg_internal(a_bytes: vector<u8>): vector<u8>;

    #[test_only]
    native fun random_scalar_internal(): vector<u8>;

    //
    // Test-only functions
    //

    #[test_only]
    public fun random_scalar(): Scalar {
        Scalar {
            data: random_scalar_internal()
        }
    }

    #[test_only]
    public fun random_point(): RistrettoPoint {
        let s = random_scalar();

        basepoint_mul(&s)
    }

    //
    // Testing constants
    //

    // The scalar 2
    #[test_only]
    const TWO_SCALAR: vector<u8> = x"0200000000000000000000000000000000000000000000000000000000000000";

    // Non-canonical scalar: the order \ell of the group + 1
    #[test_only]
    const L_PLUS_ONE: vector<u8> = x"eed3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010";

    // Non-canonical scalar: the order \ell of the group + 2
    #[test_only]
    const L_PLUS_TWO: vector<u8> = x"efd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010";

    // Some random scalar denoted by X
    #[test_only]
    const X_SCALAR: vector<u8> = x"4e5ab4345d4708845913b4641bc27d5252a585101bcc4244d449f4a879d9f204";

    // X^{-1} = 1/X = 6859937278830797291664592131120606308688036382723378951768035303146619657244
    // 0x1CDC17FCE0E9A5BBD9247E56BB016347BBBA31EDD5A9BB96D50BCD7A3F962A0F
    #[test_only]
    const X_INV_SCALAR: vector<u8> = x"1cdc17fce0e9a5bbd9247e56bb016347bbba31edd5a9bb96d50bcd7a3f962a0f";

    // Some random scalar Y = 2592331292931086675770238855846338635550719849568364935475441891787804997264
    #[test_only]
    const Y_SCALAR: vector<u8> = x"907633fe1c4b66a4a28d2dd7678386c353d0de5455d4fc9de8ef7ac31f35bb05";

    // X * Y = 5690045403673944803228348699031245560686958845067437804563560795922180092780
    #[test_only]
    const X_TIMES_Y_SCALAR: vector<u8> = x"6c3374a1894f62210aaa2fe186a6f92ce0aa75c2779581c295fc08179a73940c";

    // X + 2^256 * X \mod \ell
    #[test_only]
    const REDUCED_X_PLUS_2_TO_256_TIMES_X_SCALAR: vector<u8> = x"d89ab38bd279024745639ed817ad3f64cc005b32db9939f91c521fc564a5c008";

    // sage: l = 2^252 + 27742317777372353535851937790883648493
    // sage: big = 2^256 - 1
    // sage: repr((big % l).digits(256))
    #[test_only]
    const REDUCED_2_256_MINUS_1_SCALAR: vector<u8> = x"1c95988d7431ecd670cf7d73f45befc6feffffffffffffffffffffffffffff0f";

    #[test_only]
    const NON_CANONICAL_ALL_ONES: vector<u8> = x"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";

    #[test_only]
    const A_SCALAR: vector<u8> = x"1a0e978a90f6622d3747023f8ad8264da758aa1b88e040d1589e7b7f2376ef09";

    // Generated in curve25519-dalek via:
    // ```
    //     let mut hasher = sha2::Sha512::default();
    //     hasher.update(b"bello!");
    //     let s = Scalar::from_hash(hasher);
    //     println!("scalar: {:x?}", s.to_bytes());
    // ```
    #[test_only]
    const B_SCALAR: vector<u8> = x"dbfd97afd38a06f0138d0527efb28ead5b7109b486465913bf3aa472a8ed4e0d";

    #[test_only]
    const A_TIMES_B_SCALAR: vector<u8> = x"2ab50e383d7c210f74d5387330735f18315112d10dfb98fcce1e2620c0c01402";

    #[test_only]
    const A_PLUS_B_SCALAR: vector<u8> = x"083839dd491e57c5743710c39a91d6e502cab3cf0e279ae417d91ff2cb633e07";

    #[test_only]
    /// A_SCALAR * BASE_POINT, computed by modifying a test in curve25519-dalek in src/edwards.rs to do:
    /// ```
    ///     let comp = RistrettoPoint(A_TIMES_BASEPOINT.decompress().unwrap()).compress();
    ///     println!("hex: {:x?}", comp.to_bytes());
    /// ```
    const A_TIMES_BASE_POINT: vector<u8> = x"96d52d9262ee1e1aae79fbaee8c1d9068b0d01bf9a4579e618090c3d1088ae10";

    #[test_only]
    const A_POINT: vector<u8> = x"e87feda199d72b83de4f5b2d45d34805c57019c6c59c42cb70ee3d19aa996f75";
    #[test_only]
    const B_POINT: vector<u8> = x"fa0b3624b081c62f364d0b2839dcc76d7c3ab0e27e31beb2b9ed766575f28e76";
    #[test_only]
    const A_PLUS_B_POINT: vector<u8> = x"70cf3753475b9ff33e2f84413ed6b5052073bccc0a0a81789d3e5675dc258056";

    //    const NON_CANONICAL_LARGEST_ED25519_S: vector<u8> = x"f8ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f";
    //    const CANONICAL_LARGEST_ED25519_S_PLUS_ONE: vector<u8> = x"7e344775474a7f9723b63a8be92ae76dffffffffffffffffffffffffffffff0f";
    //    const CANONICAL_LARGEST_ED25519_S_MINUS_ONE: vector<u8> = x"7c344775474a7f9723b63a8be92ae76dffffffffffffffffffffffffffffff0f";

    //
    // Tests
    //

    #[test]
    fun test_point_decompression() {
        let compressed = new_compressed_point_from_bytes(A_POINT);
        assert!(std::option::is_some(&compressed), 1);

        let point = new_point_from_bytes(A_POINT);
        assert!(std::option::is_some(&point), 1);

        let point = std::option::extract(&mut point);
        let compressed = std::option::extract(&mut compressed);
        let same_point = point_decompress(&compressed);

        assert!(point_equals(&point, &same_point), 1);
    }

    #[test]
    fun test_point_equals() {
        let g = basepoint();
        let same_g = std::option::extract(&mut new_point_from_bytes(BASE_POINT));
        let ag = std::option::extract(&mut new_point_from_bytes(A_TIMES_BASE_POINT));

        assert!(point_equals(&g, &same_g), 1);
        assert!(!point_equals(&g, &ag), 1);
    }

    #[test]
    fun test_point_mul() {
        // fetch g
        let g = basepoint();
        // fetch a
        let a = std::option::extract(&mut new_scalar_from_bytes(A_SCALAR));
        // fetch expected a*g
        let ag = std::option::extract(&mut new_point_from_bytes(A_TIMES_BASE_POINT));

        // compute a*g
        let p = point_mul(&g, &a);

        // sanity-check the handles
        assert!(g.handle == 0, 1);
        assert!(ag.handle == 1, 1);
        assert!(p.handle == 2, 1);

        assert!(!point_equals(&g, &ag), 1);     // make sure input g remains unmodifed
        assert!(point_equals(&p, &ag), 1);   // make sure output a*g is correct
    }

    #[test]
    fun test_point_mul_assign() {
        let g = basepoint();
        assert!(g.handle == 0, 1);

        let a = std::option::extract(&mut new_scalar_from_bytes(A_SCALAR));

        let ag = std::option::extract(&mut new_point_from_bytes(A_TIMES_BASE_POINT));
        assert!(ag.handle == 1, 1);
        assert!(!point_equals(&g, &ag), 1);

        {
            // NOTE: new_g is just a mutable reference to g
            let upd_g = point_mul_assign(&mut g, &a);

            // in a mul_assign the returned &mut RistrettoPoint reference should have the same handle as 'g'
            assert!(upd_g.handle == 0, 1);

            assert!(point_equals(upd_g, &ag), 1);
        };

        assert!(point_equals(&g, &ag), 1);
    }

    #[test]
    fun test_point_add() {
        // fetch a
        let a = std::option::extract(&mut new_point_from_bytes(A_POINT));

        // fetch b
        let b = std::option::extract(&mut new_point_from_bytes(B_POINT));

        // fetch expected a + b
        let a_plus_b = std::option::extract(&mut new_point_from_bytes(A_PLUS_B_POINT));

        // compute a*g
        let result = point_add(&a, &b);

        assert!(!point_equals(&a, &b), 1);

        // sanity-check the handles
        assert!(a.handle == 0, 1);
        assert!(b.handle == 1, 1);
        assert!(a_plus_b.handle == 2, 1);
        assert!(result.handle == 3, 1);

        assert!(!point_equals(&a, &result), 1);     // make sure input a remains unmodifed
        assert!(!point_equals(&b, &result), 1);     // make sure input b remains unmodifed
        assert!(point_equals(&a_plus_b, &result), 1);   // make sure output a+b is correct
    }

    #[test]
    fun test_point_add_assign_0_0() {
        test_point_add_assign_internal(0, 0);
    }

    #[test]
    fun test_point_add_assign_1_0() {
        test_point_add_assign_internal(1, 0);
    }

    #[test]
    fun test_point_add_assign_0_1() {
        test_point_add_assign_internal(0, 1);
    }

    #[test]
    fun test_point_add_assign_3_7() {
        test_point_add_assign_internal(3, 7);
    }

    #[test_only]
    fun test_point_add_assign_internal(before_a_gap: u64, before_b_gap: u64) {
        // create extra RistrettoPoints here, so as to generate different PointStore layouts inside the native Rust implementation
        let c = before_a_gap;
        while (c > 0) {
            let _ignore = std::option::extract(&mut new_point_from_bytes(BASE_POINT));

            c = c - 1;
        };

        // fetch a
        let a = std::option::extract(&mut new_point_from_bytes(A_POINT));

        // create extra RistrettoPoints here, so as to generate different PointStore layouts inside the native Rust implementation
        let c = before_b_gap;
        while (c > 0) {
            let _ignore = std::option::extract(&mut new_point_from_bytes(BASE_POINT));

            c = c - 1;
        };
        // fetch b
        let b = std::option::extract(&mut new_point_from_bytes(B_POINT));

        let a_plus_b = std::option::extract(&mut new_point_from_bytes(A_PLUS_B_POINT));

        // sanity-check the handles
        assert!(a.handle == before_a_gap, 1);
        assert!(b.handle == 1 + before_a_gap + before_b_gap, 1);
        assert!(a_plus_b.handle == 2 + before_a_gap + before_b_gap, 1);

        assert!(!point_equals(&a, &b), 1);
        assert!(!point_equals(&a, &a_plus_b), 1);

        {
            // NOTE: new_h is just a mutable reference to g
            let upd_a = point_add_assign(&mut a, &b);

            // in a add_assign the returned &mut RistrettoPoint reference should have the same handle as 'a'
            assert!(upd_a.handle == before_a_gap, 1);

            assert!(point_equals(upd_a, &a_plus_b), 1);
        };

        assert!(point_equals(&a, &a_plus_b), 1);
    }

    #[test]
    fun test_point_sub() {
        // fetch a
        let a = std::option::extract(&mut new_point_from_bytes(A_POINT));

        // fetch b
        let b = std::option::extract(&mut new_point_from_bytes(B_POINT));

        // fetch expected a + b
        let a_plus_b = std::option::extract(&mut new_point_from_bytes(A_PLUS_B_POINT));

        // compute a*g
        let result = point_sub(&a_plus_b, &b);

        assert!(!point_equals(&a, &b), 1);

        // sanity-check the handles
        assert!(a.handle == 0, 1);
        assert!(b.handle == 1, 1);
        assert!(a_plus_b.handle == 2, 1);
        assert!(result.handle == 3, 1);

        assert!(!point_equals(&a_plus_b, &result), 1);     // make sure input a_plus_b remains unmodifed
        assert!(!point_equals(&b, &result), 1);     // make sure input b remains unmodifed
        assert!(point_equals(&a, &result), 1);   // make sure output 'a+b-b' is correct
    }

    #[test]
    fun test_point_neg() {
        let a = std::option::extract(&mut new_point_from_bytes(A_POINT));

        let neg_a = point_neg(&a);

        assert!(a.handle != neg_a.handle, 1);
        assert!(!point_equals(&a, &neg_a), 1);
        assert!(!point_equals(&point_add(&point_identity(), &a), &neg_a), 1);
        assert!(point_equals(&point_add(&a, &neg_a), &point_identity()), 1);

        let handle = a.handle;
        let neg_a_ref = point_neg_assign(&mut a);
        assert!(handle == neg_a_ref.handle, 1);
        assert!(point_equals(neg_a_ref, &neg_a), 1);
    }

    #[test]
    fun test_basepoint_mul() {
        let a = Scalar { data: A_SCALAR };
        let basepoint = basepoint();
        let expected = point_mul(&basepoint, &a);
        assert!(point_equals(&expected, &basepoint_mul(&a)), 1);
    }

    #[test(fx = @std)]
    fun test_basepoint_double_mul(fx: signer) {
        features::change_feature_flags_for_testing(&fx, vector[ features::get_bulletproofs_feature() ], vector[]);

        let expected = option::extract(&mut new_point_from_bytes(x"be5d615d8b8f996723cdc6e1895b8b6d312cc75d1ffb0259873b99396a38c05a"));

        let a = Scalar { data: A_SCALAR };
        let a_point = option::extract(&mut new_point_from_bytes(A_POINT));
        let b = Scalar { data: B_SCALAR };
        let actual = basepoint_double_mul(&a, &a_point, &b);

        assert!(point_equals(&expected, &actual), 1);

        let expected = double_scalar_mul(&a, &a_point, &b, &basepoint());
        assert!(point_equals(&expected, &actual), 1);
    }

    #[test]
    #[expected_failure]
    fun test_multi_scalar_mul_aborts_empty_scalars() {
        multi_scalar_mul(&vector[ basepoint() ], &vector[]);
    }

    #[test]
    #[expected_failure]
    fun test_multi_scalar_mul_aborts_empty_points() {
        multi_scalar_mul(&vector[ ], &vector[ Scalar { data: A_SCALAR } ]);
    }

    #[test]
    #[expected_failure]
    fun test_multi_scalar_mul_aborts_empty_all() {
        multi_scalar_mul(&vector[ ], &vector[ ]);
    }

    #[test]
    #[expected_failure]
    fun test_multi_scalar_mul_aborts_different_sizes() {
        multi_scalar_mul(&vector[ basepoint() ], &vector[ Scalar { data: A_SCALAR }, Scalar { data: B_SCALAR }  ]);
    }

    #[test]
    fun test_multi_scalar_mul_single() {
        // Test single exp
        let points = vector[
            basepoint(),
        ];

        let scalars = vector[
            Scalar { data: A_SCALAR },
        ];

        let result = multi_scalar_mul(&points, &scalars);
        let expected = std::option::extract(&mut new_point_from_bytes(A_TIMES_BASE_POINT));

        assert!(point_equals(&result, &expected), 1);
    }

    #[test]
    fun test_multi_scalar_mul_double() {
        // Test double exp
        let points = vector[
            basepoint(),
            basepoint(),
        ];

        let scalars = vector[
            Scalar { data: A_SCALAR },
            Scalar { data: B_SCALAR },
        ];

        let result = multi_scalar_mul(&points, &scalars);
        let expected = basepoint_double_mul(
            std::vector::borrow(&scalars, 0),
            &basepoint(),
            std::vector::borrow(&scalars, 1));

        assert!(point_equals(&result, &expected), 1);
    }

    #[test]
    fun test_multi_scalar_mul_many() {
        let scalars = vector[
            new_scalar_from_sha2_512(b"1"),
            new_scalar_from_sha2_512(b"2"),
            new_scalar_from_sha2_512(b"3"),
            new_scalar_from_sha2_512(b"4"),
            new_scalar_from_sha2_512(b"5"),
        ];

        let points = vector[
            new_point_from_sha2_512(b"1"),
            new_point_from_sha2_512(b"2"),
            new_point_from_sha2_512(b"3"),
            new_point_from_sha2_512(b"4"),
            new_point_from_sha2_512(b"5"),
        ];

        let expected = std::option::extract(&mut new_point_from_bytes(x"c4a98fbe6bd0f315a0c150858aec8508be397443093e955ef982e299c1318928"));
        let result = multi_scalar_mul(&points, &scalars);

        assert!(point_equals(&expected, &result), 1);
    }

    #[test]
    fun test_new_point_from_sha2_512() {
        let msg = b"To really appreciate architecture, you may even need to commit a murder";
        let expected = option::extract(&mut new_point_from_bytes(x"baaa91eb43e5e2f12ffc96347e14bc458fdb1772b2232b08977ee61ea9f84e31"));

        assert!(point_equals(&expected, &new_point_from_sha2_512(msg)), 1);
    }

    #[test]
    fun test_new_point_from_64_uniform_bytes() {
        let bytes_64 = x"baaa91eb43e5e2f12ffc96347e14bc458fdb1772b2232b08977ee61ea9f84e31e87feda199d72b83de4f5b2d45d34805c57019c6c59c42cb70ee3d19aa996f75";
        let expected = option::extract(&mut new_point_from_bytes(x"4a8e429f906478654232d7ae180ad60854754944ac67f38e20d8fa79e4b7d71e"));

        let point = option::extract(&mut new_point_from_64_uniform_bytes(bytes_64));
        assert!(point_equals(&expected, &point), 1);
    }

    #[test]
    fun test_scalar_basic_viability() {
        // Test conversion from u8
        let two = Scalar { data: TWO_SCALAR };
        assert!(scalar_equals(&new_scalar_from_u8(2u8), &two), 1);

        // Test conversion from u64
        assert!(scalar_equals(&new_scalar_from_u64(2u64), &two), 1);

        // Test conversion from u128
        assert!(scalar_equals(&new_scalar_from_u128(2u128), &two), 1);

        // Test (0 - 1) % order = order - 1
        assert!(scalar_equals(&scalar_sub(&scalar_zero(), &scalar_one()), &Scalar { data: L_MINUS_ONE }), 1);
    }

    #[test]
    /// Tests deserializing a Scalar from a sequence of canonical bytes
    fun test_scalar_from_canonical_bytes() {
        // Too few bytes
        assert!(std::option::is_none(&new_scalar_from_bytes(x"00")), 1);

        // 32 zero bytes are canonical
        assert!(std::option::is_some(&new_scalar_from_bytes(x"0000000000000000000000000000000000000000000000000000000000000000")), 1);

        // Non-canonical because unreduced
        assert!(std::option::is_none(&new_scalar_from_bytes(x"1010101010101010101010101010101010101010101010101010101010101010")), 1);

        // Canonical because \ell - 1
        assert!(std::option::is_some(&new_scalar_from_bytes(L_MINUS_ONE)), 1);

        // Non-canonical because \ell
        assert!(std::option::is_none(&new_scalar_from_bytes(ORDER_ELL)), 1);

        // Non-canonical because \ell+1
        assert!(std::option::is_none(&new_scalar_from_bytes(L_PLUS_ONE)), 1);

        // Non-canonical because \ell+2
        assert!(std::option::is_none(&new_scalar_from_bytes(L_PLUS_TWO)), 1);

        // Non-canonical because high bit is set
        let non_canonical_highbit = vector[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128];
        let non_canonical_highbit_hex = x"0000000000000000000000000000000000000000000000000000000000000080";
        assert!(non_canonical_highbit == non_canonical_highbit_hex, 1);
        assert!(std::option::is_none(&new_scalar_from_bytes(non_canonical_highbit)), 1);
    }

    #[test]
    fun test_scalar_zero() {
        // 0 == 0
        assert!(scalar_is_zero(&scalar_zero()), 1);
        assert!(scalar_is_zero(&new_scalar_from_u8(0u8)), 1);

        // 0 != 1
        assert!(scalar_is_zero(&scalar_one()) == false, 1);

        // Pick a random scalar by hashing from some "random" bytes
        let s = new_scalar_from_sha2_512(x"deadbeef");

        // Technically, there is a negligible probability (i.e., 1/2^\ell) that the hashed s is zero or one
        assert!(scalar_is_zero(&s) == false, 1);
        assert!(scalar_is_one(&s) == false, 1);

        // Multiply 0 with a random scalar and make sure you get zero
        assert!(scalar_is_zero(&scalar_mul(&scalar_zero(), &s)), 1);
        assert!(scalar_is_zero(&scalar_mul(&s, &scalar_zero())), 1);
    }

    #[test]
    fun test_scalar_one() {
        // 1 == 1
        assert!(scalar_is_one(&scalar_one()), 1);
        assert!(scalar_is_one(&new_scalar_from_u8(1u8)), 1);

        // 1 != 0
        assert!(scalar_is_one(&scalar_zero()) == false, 1);

        // Pick a random scalar by hashing from some "random" bytes
        let s = new_scalar_from_sha2_512(x"deadbeef");
        let inv = scalar_invert(&s);

        // Technically, there is a negligible probability (i.e., 1/2^\ell) that s was zero and the call above returned None
        assert!(std::option::is_some(&inv), 1);

        let inv = std::option::extract(&mut inv);

        // Multiply s with s^{-1} and make sure you get one
        assert!(scalar_is_one(&scalar_mul(&s, &inv)), 1);
        assert!(scalar_is_one(&scalar_mul(&inv, &s)), 1);
    }

    #[test]
    fun test_scalar_from_sha2_512() {
        // Test a specific message hashes correctly to the field
        let str: vector<u8> = vector[];
        std::vector::append(&mut str, b"To really appreciate architecture, you may even need to commit a murder.");
        std::vector::append(&mut str, b"While the programs used for The Manhattan Transcripts are of the most extreme");
        std::vector::append(&mut str, b"nature, they also parallel the most common formula plot: the archetype of");
        std::vector::append(&mut str, b"murder. Other phantasms were occasionally used to underline the fact that");
        std::vector::append(&mut str, b"perhaps all architecture, rather than being about functional standards, is");
        std::vector::append(&mut str, b"about love and death.");

        let s = new_scalar_from_sha2_512(str);

        let expected: vector<u8> = vector[
            21, 88, 208, 252, 63, 122, 210, 152,
            154, 38, 15, 23, 16, 167, 80, 150,
            192, 221, 77, 226, 62, 25, 224, 148,
            239, 48, 176, 10, 185, 69, 168, 11
        ];

        assert!(s.data == expected, 1)
    }

    #[test]
    fun test_scalar_invert() {
        // Cannot invert zero
        assert!(std::option::is_none(&scalar_invert(&scalar_zero())), 1);

        // One's inverse is one
        let one = scalar_invert(&scalar_one());
        assert!(std::option::is_some(&one), 1);

        let one = std::option::extract(&mut one);
        assert!(scalar_is_one(&one), 1);

        // Test a random point X's inverse is correct
        let x = Scalar { data: X_SCALAR };
        let xinv = scalar_invert(&x);
        assert!(std::option::is_some(&xinv), 1);

        let xinv = std::option::extract(&mut xinv);
        let xinv_expected = Scalar { data: X_INV_SCALAR };

        assert!(scalar_equals(&xinv, &xinv_expected), 1)
    }

    #[test]
    fun test_scalar_neg() {
        // -(-X) == X
        let x = Scalar { data: X_SCALAR };

        let x_neg = scalar_neg(&x);
        let x_neg_neg = scalar_neg(&x_neg);

        assert!(scalar_equals(&x, &x_neg_neg), 1);
    }

    #[test]
    fun test_scalar_neg_assign() {
        let x = Scalar { data: X_SCALAR };
        let x_copy = x;

        scalar_neg_assign(&mut x);
        assert!(!scalar_equals(&x, &x_copy), 1);
        scalar_neg_assign(&mut x);
        assert!(scalar_equals(&x, &x_copy), 1);

        assert!(scalar_equals(scalar_neg_assign(scalar_neg_assign(&mut x)), &x_copy), 1);
    }

    #[test]
    fun test_scalar_mul() {
        // X * 1 == X
        let x = Scalar { data: X_SCALAR };
        assert!(scalar_equals(&x, &scalar_mul(&x, &scalar_one())), 1);

        // Test multiplication of two random scalars
        let y = Scalar { data: Y_SCALAR };
        let x_times_y = Scalar { data: X_TIMES_Y_SCALAR };
        assert!(scalar_equals(&scalar_mul(&x, &y), &x_times_y), 1);

        // A * B
        assert!(scalar_equals(&scalar_mul(&Scalar { data: A_SCALAR }, &Scalar { data: B_SCALAR }), &Scalar { data: A_TIMES_B_SCALAR }), 1);
    }

    #[test]
    fun test_scalar_mul_assign() {
        let x = Scalar { data: X_SCALAR };
        let y = Scalar { data: Y_SCALAR };
        let x_times_y = Scalar { data: X_TIMES_Y_SCALAR };

        scalar_mul_assign(&mut x, &y);

        assert!(scalar_equals(&x, &x_times_y), 1);
    }

    #[test]
    fun test_scalar_add() {
        // Addition reduces: \ell-1 + 1 = \ell = 0
        let ell_minus_one = Scalar { data: L_MINUS_ONE };
        assert!(scalar_is_zero(&scalar_add(&ell_minus_one, &scalar_one())), 1);

        // 1 + 1 = 2
        let two = Scalar { data: TWO_SCALAR };
        assert!(scalar_equals(&scalar_add(&scalar_one(), &scalar_one()), &two), 1);

        // A + B
        assert!(scalar_equals(&scalar_add(&Scalar { data: A_SCALAR }, &Scalar { data: B_SCALAR }), &Scalar { data: A_PLUS_B_SCALAR }), 1);
    }

    #[test]
    fun test_scalar_sub() {
        // Subtraction reduces: 0 - 1 = \ell - 1
        let ell_minus_one = Scalar { data: L_MINUS_ONE };
        assert!(scalar_equals(&scalar_sub(&scalar_zero(), &scalar_one()), &ell_minus_one), 1);

        // 2 - 1 = 1
        let two = Scalar { data: TWO_SCALAR };
        assert!(scalar_is_one(&scalar_sub(&two, &scalar_one())), 1);

        // 1 - 2 = -1 = \ell - 1
        let ell_minus_one = Scalar { data: L_MINUS_ONE };
        assert!(scalar_equals(&scalar_sub(&scalar_one(), &two), &ell_minus_one), 1);
    }

    #[test]
    fun test_scalar_reduced_from_32_bytes() {
        // \ell + 2 = 0 + 2 = 2 (modulo \ell)
        let s = std::option::extract(&mut new_scalar_reduced_from_32_bytes(L_PLUS_TWO));
        let two = Scalar { data: TWO_SCALAR };
        assert!(scalar_equals(&s, &two), 1);

        // Reducing the all 1's bit vector yields $(2^256 - 1) \mod \ell$
        let biggest = std::option::extract(&mut new_scalar_reduced_from_32_bytes(NON_CANONICAL_ALL_ONES));
        assert!(scalar_equals(&biggest, &Scalar { data: REDUCED_2_256_MINUS_1_SCALAR }), 1);
    }

    #[test]
    fun test_scalar_from_64_uniform_bytes() {
        // Test X + 2^256 * X reduces correctly
        let x_plus_2_to_256_times_x: vector<u8> = vector[];

        std::vector::append(&mut x_plus_2_to_256_times_x, X_SCALAR);
        std::vector::append(&mut x_plus_2_to_256_times_x, X_SCALAR);

        let reduced = std::option::extract(&mut new_scalar_uniform_from_64_bytes(x_plus_2_to_256_times_x));
        let expected = Scalar { data: REDUCED_X_PLUS_2_TO_256_TIMES_X_SCALAR };
        assert!(scalar_equals(&reduced, &expected), 1)
    }

    #[test]
    fun test_scalar_to_bytes() {
        // zero is canonical
        assert!(scalar_is_canonical_internal(scalar_zero().data), 1);

        // ...but if we maul it and set the high bit to 1, it is non-canonical
        let non_can = scalar_zero();
        let last_byte = std::vector::borrow_mut(&mut non_can.data, 31);
        *last_byte = 128;
        assert!(!scalar_is_canonical_internal(non_can.data), 1);

        // This test makes sure scalar_to_bytes does not return a mutable reference to a scalar's bits
        let non_can = scalar_zero();
        let bytes = scalar_to_bytes(&scalar_zero());
        let last_byte = std::vector::borrow_mut(&mut bytes, 31);
        *last_byte = 128;
        assert!(scalar_is_canonical_internal(non_can.data), 1);
        assert!(scalar_equals(&non_can, &scalar_zero()), 1);
    }

    #[test]
    fun test_num_points_within_limit() {
        let limit = 10000;
        let i = 0;
        while (i < limit) {
            point_identity();
            i = i + 1;
        }
    }

    #[test]
    #[expected_failure(abort_code=0x090004, location=Self)]
    fun test_num_points_limit_exceeded() {
        let limit = 10001;
        let i = 0;
        while (i < limit) {
            point_identity();
            i = i + 1;
        }
    }
}
