/// This module provides generic structs/functions for operations of algebraic structures (e.g. fields and groups),
/// which can be used to build generic cryptographic schemes atop.
/// E.g., a Groth16 ZK proof verifier can be built to work over any pairing supported in this module.
///
/// In general, every structure implements basic operations like (de)serialization, equality check, random sampling.
///
/// A group may also implement the following operations. (Additive group notation is assumed.)
/// - `order()` for getting the group order.
/// - `zero()` for getting the group identity.
/// - `one()` for getting the group generator (if exists).
/// - `neg()` for group element inversion.
/// - `add()` for group operation (i.e., a group addition).
/// - `sub()` for group element subtraction.
/// - `double()` for efficient doubling.
/// - `scalar_mul()` for group scalar multiplication.
/// - `multi_scalar_mul()` for efficient group multi-scalar multiplication.
/// - `hash_to()` for hash-to-group.
///
/// A field may also implement the following operations.
/// - `zero()` for getting the field additive identity.
/// - `one()` for getting the field multiplicative identity.
/// - `add()` for field addition.
/// - `sub()` for field subtraction.
/// - `mul()` for field multiplication.
/// - `div()` for field division.
/// - `neg()` for field negation.
/// - `inv()` for field inversion.
/// - `sqr()` for efficient field element squaring.
/// - `from_u64()` for quick conversion from u64 to field element.
///
/// For 3 groups that admit a bilinear map, `pairing()` and `multi_pairing()` may be implemented.
///
/// For a subset/superset relationship between 2 structures, `upcast()` and `downcast()` may be implemented.
/// E.g., in BLS12-381 pairing, since `Gt` is a subset of `Fq12`,
/// `upcast<Gt, Fq12>()` and `downcast<Fq12, Gt>()` will be supported.
///
/// See `*_algebra.move` for currently implemented algebraic structures.
module aptos_std::crypto_algebra {
    use std::option::{Option, some, none};
    use std::features;

    const E_NOT_IMPLEMENTED: u64 = 1;
    const E_NON_EQUAL_LENGTHS: u64 = 2;
    const E_TOO_MUCH_MEMORY_USED: u64 = 3;

    /// This struct represents an element of a structure `S`.
    struct Element<phantom S> has copy, drop {
        handle: u64
    }

    //
    // Public functions begin.
    //

    /// Check if `x == y` for elements `x` and `y` of a structure `S`.
    public fun eq<S>(x: &Element<S>, y: &Element<S>): bool {
        abort_unless_cryptography_algebra_natives_enabled();
        eq_internal<S>(x.handle, y.handle)
    }

    /// Convert a u64 to an element of a structure `S`.
    public fun from_u64<S>(value: u64): Element<S> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<S> {
            handle: from_u64_internal<S>(value)
        }
    }

    /// Return the additive identity of field `S`, or the identity of group `S`.
    public fun zero<S>(): Element<S> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<S> {
            handle: zero_internal<S>()
        }
    }

    /// Return the multiplicative identity of field `S`, or a fixed generator of group `S`.
    public fun one<S>(): Element<S> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<S> {
            handle: one_internal<S>()
        }
    }

    /// Compute `-x` for an element `x` of a structure `S`.
    public fun neg<S>(x: &Element<S>): Element<S> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<S> {
            handle: neg_internal<S>(x.handle)
        }
    }

    /// Compute `x + y` for elements `x` and `y` of structure `S`.
    public fun add<S>(x: &Element<S>, y: &Element<S>): Element<S> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<S> {
            handle: add_internal<S>(x.handle, y.handle)
        }
    }

    /// Compute `x - y` for elements `x` and `y` of a structure `S`.
    public fun sub<S>(x: &Element<S>, y: &Element<S>): Element<S> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<S> {
            handle: sub_internal<S>(x.handle, y.handle)
        }
    }

    /// Compute `x * y` for elements `x` and `y` of a structure `S`.
    public fun mul<S>(x: &Element<S>, y: &Element<S>): Element<S> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<S> {
            handle: mul_internal<S>(x.handle, y.handle)
        }
    }

    /// Try computing `x / y` for elements `x` and `y` of a structure `S`.
    /// Return none if `y` does not have a multiplicative inverse in the structure `S`
    /// (e.g., when `S` is a field, and `y` is zero).
    public fun div<S>(x: &Element<S>, y: &Element<S>): Option<Element<S>> {
        abort_unless_cryptography_algebra_natives_enabled();
        let (succ, handle) = div_internal<S>(x.handle, y.handle);
        if (succ) {
            some(Element<S> { handle })
        } else {
            none()
        }
    }

    /// Compute `x^2` for an element `x` of a structure `S`. Faster and cheaper than `mul(x, x)`.
    public fun sqr<S>(x: &Element<S>): Element<S> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<S> {
            handle: sqr_internal<S>(x.handle)
        }
    }

    /// Try computing `x^(-1)` for an element `x` of a structure `S`.
    /// Return none if `x` does not have a multiplicative inverse in the structure `S`
    /// (e.g., when `S` is a field, and `x` is zero).
    public fun inv<S>(x: &Element<S>): Option<Element<S>> {
        abort_unless_cryptography_algebra_natives_enabled();
        let (succeeded, handle) = inv_internal<S>(x.handle);
        if (succeeded) {
            let scalar = Element<S> { handle };
            some(scalar)
        } else {
            none()
        }
    }

    /// Compute `2*P` for an element `P` of a structure `S`. Faster and cheaper than `add(P, P)`.
    public fun double<S>(element_p: &Element<S>): Element<S> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<S> {
            handle: double_internal<S>(element_p.handle)
        }
    }

    /// Compute `k[0]*P[0]+...+k[n-1]*P[n-1]`, where
    /// `P[]` are `n` elements of group `G` represented by parameter `elements`, and
    /// `k[]` are `n` elements of the scalarfield `S` of group `G` represented by parameter `scalars`.
    ///
    /// Abort with code `std::error::invalid_argument(E_NON_EQUAL_LENGTHS)` if the sizes of `elements` and `scalars` do not match.
    public fun multi_scalar_mul<G, S>(elements: &vector<Element<G>>, scalars: &vector<Element<S>>): Element<G> {
        let element_handles = handles_from_elements(elements);
        let scalar_handles = handles_from_elements(scalars);
        Element<G> {
            handle: multi_scalar_mul_internal<G, S>(element_handles, scalar_handles)
        }
    }

    /// Compute `k*P`, where `P` is an element of a group `G` and `k` is an element of the scalar field `S` associated to the group `G`.
    public fun scalar_mul<G, S>(element_p: &Element<G>, scalar_k: &Element<S>): Element<G> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<G> {
            handle: scalar_mul_internal<G, S>(element_p.handle, scalar_k.handle)
        }
    }

    /// Efficiently compute `e(P[0],Q[0])+...+e(P[n-1],Q[n-1])`,
    /// where `e: (G1,G2) -> (Gt)` is the pairing function from groups `(G1,G2)` to group `Gt`,
    /// `P[]` are `n` elements of group `G1` represented by parameter `g1_elements`, and
    /// `Q[]` are `n` elements of group `G2` represented by parameter `g2_elements`.
    ///
    /// Abort with code `std::error::invalid_argument(E_NON_EQUAL_LENGTHS)` if the sizes of `g1_elements` and `g2_elements` do not match.
    ///
    /// NOTE: we are viewing the target group `Gt` of the pairing as an additive group,
    /// rather than a multiplicative one (which is typically the case).
    public fun multi_pairing<G1,G2,Gt>(g1_elements: &vector<Element<G1>>, g2_elements: &vector<Element<G2>>): Element<Gt> {
        abort_unless_cryptography_algebra_natives_enabled();
        let g1_handles = handles_from_elements(g1_elements);
        let g2_handles = handles_from_elements(g2_elements);
        Element<Gt> {
            handle: multi_pairing_internal<G1,G2,Gt>(g1_handles, g2_handles)
        }
    }

    /// Compute the pairing function (a.k.a., bilinear map) on a `G1` element and a `G2` element.
    /// Return an element in the target group `Gt`.
    public fun pairing<G1,G2,Gt>(element_1: &Element<G1>, element_2: &Element<G2>): Element<Gt> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<Gt> {
            handle: pairing_internal<G1,G2,Gt>(element_1.handle, element_2.handle)
        }
    }

    /// Try deserializing a byte array to an element of an algebraic structure `S` using a given serialization format `F`.
    /// Return none if the deserialization failed.
    public fun deserialize<S, F>(bytes: &vector<u8>): Option<Element<S>> {
        abort_unless_cryptography_algebra_natives_enabled();
        let (succeeded, handle) = deserialize_internal<S, F>(bytes);
        if (succeeded) {
            some(Element<S> { handle })
        } else {
            none()
        }
    }

    /// Serialize an element of an algebraic structure `S` to a byte array using a given serialization format `F`.
    public fun serialize<S, F>(element: &Element<S>): vector<u8> {
        abort_unless_cryptography_algebra_natives_enabled();
        serialize_internal<S, F>(element.handle)
    }

    /// Get the order of structure `S`, a big integer little-endian encoded as a byte array.
    public fun order<S>(): vector<u8> {
        abort_unless_cryptography_algebra_natives_enabled();
        order_internal<S>()
    }

    /// Cast an element of a structure `S` to a parent structure `L`.
    public fun upcast<S,L>(element: &Element<S>): Element<L> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<L> {
            handle: upcast_internal<S,L>(element.handle)
        }
    }

    /// Try casting an element `x` of a structure `L` to a sub-structure `S`.
    /// Return none if `x` is not a member of `S`.
    ///
    /// NOTE: Membership check in `S` is performed inside, which can be expensive, depending on the structures `L` and `S`.
    public fun downcast<L,S>(element_x: &Element<L>): Option<Element<S>> {
        abort_unless_cryptography_algebra_natives_enabled();
        let (succ, new_handle) = downcast_internal<L,S>(element_x.handle);
        if (succ) {
            some(Element<S> { handle: new_handle })
        } else {
            none()
        }
    }

    /// Hash an arbitrary-length byte array `msg` into structure `S` with a domain separation tag `dst`
    /// using the given hash-to-structure suite `H`.
    ///
    /// NOTE: some hashing methods do not accept a `dst` and will abort if a non-empty one is provided.
    public fun hash_to<S, H>(dst: &vector<u8>, msg: &vector<u8>): Element<S> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element {
            handle: hash_to_internal<S, H>(dst, msg)
        }
    }

    #[test_only]
    /// Generate a random element of an algebraic structure `S`.
    public fun rand_insecure<S>(): Element<S> {
        abort_unless_cryptography_algebra_natives_enabled();
        Element<S> {
            handle: rand_insecure_internal<S>()
        }
    }

    //
    // (Public functions end here.)
    // Private functions begin.
    //

    fun abort_unless_cryptography_algebra_natives_enabled() {
        if (features::cryptography_algebra_enabled()) return;
        abort(std::error::not_implemented(0))
    }

    #[test_only]
    public fun enable_cryptography_algebra_natives(fx: &signer) {
        std::features::change_feature_flags_for_testing(fx, vector[std::features::get_cryptography_algebra_natives_feature()], vector[]);
    }

    fun handles_from_elements<S>(elements: &vector<Element<S>>): vector<u64> {
        let num_elements = std::vector::length(elements);
        let element_handles = std::vector::empty();
        let i = 0;
        while ({
            spec {
                invariant len(element_handles) == i;
                invariant forall k in 0..i: element_handles[k] == elements[k].handle;
            };
            i < num_elements
        }) {
            std::vector::push_back(&mut element_handles, std::vector::borrow(elements, i).handle);
            i = i + 1;
        };
        element_handles
    }

    //
    // (Private functions end here.)
    // Native functions begin.
    //

    native fun add_internal<S>(handle_1: u64, handle_2: u64): u64;
    native fun deserialize_internal<S, F>(bytes: &vector<u8>): (bool, u64);
    native fun div_internal<F>(handle_1: u64, handle_2: u64): (bool, u64);
    native fun double_internal<G>(element_handle: u64): u64;
    native fun downcast_internal<L,S>(handle: u64): (bool, u64);
    native fun from_u64_internal<S>(value: u64): u64;
    native fun eq_internal<S>(handle_1: u64, handle_2: u64): bool;
    native fun hash_to_internal<S, H>(dst: &vector<u8>, bytes: &vector<u8>): u64;
    native fun inv_internal<F>(handle: u64): (bool, u64);
    #[test_only]
    native fun rand_insecure_internal<S>(): u64;
    native fun mul_internal<F>(handle_1: u64, handle_2: u64): u64;
    native fun multi_pairing_internal<G1,G2,Gt>(g1_handles: vector<u64>, g2_handles: vector<u64>): u64;
    native fun multi_scalar_mul_internal<G, S>(element_handles: vector<u64>, scalar_handles: vector<u64>): u64;
    native fun neg_internal<F>(handle: u64): u64;
    native fun one_internal<S>(): u64;
    native fun order_internal<G>(): vector<u8>;
    native fun pairing_internal<G1,G2,Gt>(g1_handle: u64, g2_handle: u64): u64;
    native fun scalar_mul_internal<G, S>(element_handle: u64, scalar_handle: u64): u64;
    native fun serialize_internal<S, F>(handle: u64): vector<u8>;
    native fun sqr_internal<G>(handle: u64): u64;
    native fun sub_internal<G>(handle_1: u64, handle_2: u64): u64;
    native fun upcast_internal<S,L>(handle: u64): u64;
    native fun zero_internal<S>(): u64;

    //
    // (Native functions end here.)
    // Tests begin.
    //

    #[test_only]
    struct MysteriousGroup {}

    #[test(fx = @std)]
    #[expected_failure(abort_code = 0x0c0001, location = Self)]
    fun test_generic_operation_should_abort_with_unsupported_structures(fx: signer) {
        enable_cryptography_algebra_natives(&fx);
        let _ = order<MysteriousGroup>();
    }
    // Tests end.
}
