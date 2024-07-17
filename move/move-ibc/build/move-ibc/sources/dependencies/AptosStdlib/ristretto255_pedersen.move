/// This module implements a Pedersen commitment API, over the Ristretto255 curve, that can be used with the
/// Bulletproofs module.
///
/// A Pedersen commitment to a value `v` under _commitment key_ `(g, h)` is `v * g + r * h`, for a random scalar `r`.

module aptos_std::ristretto255_pedersen {
    use aptos_std::ristretto255::{Self, RistrettoPoint, Scalar, CompressedRistretto, point_compress};
    use std::option::Option;

    //
    // Constants
    //

    /// The default Pedersen randomness base `h` used in our underlying Bulletproofs library.
    /// This is obtained by hashing the compressed Ristretto255 basepoint using SHA3-512 (not SHA2-512).
    const BULLETPROOF_DEFAULT_PEDERSEN_RAND_BASE : vector<u8> = x"8c9240b456a9e6dc65c377a1048d745f94a08cdb7f44cbcd7b46f34048871134";

    //
    // Structs
    //

    /// A Pedersen commitment to some value with some randomness.
    struct Commitment has drop {
        point: RistrettoPoint,
    }

    //
    // Public functions
    //

    /// Creates a new public key from a serialized Ristretto255 point.
    public fun new_commitment_from_bytes(bytes: vector<u8>): Option<Commitment> {
        let point = ristretto255::new_point_from_bytes(bytes);
        if (std::option::is_some(&mut point)) {
            let comm = Commitment {
                point: std::option::extract(&mut point)
            };
            std::option::some(comm)
        } else {
            std::option::none<Commitment>()
        }
    }

    /// Returns a commitment as a serialized byte array
    public fun commitment_to_bytes(comm: &Commitment): vector<u8> {
        ristretto255::point_to_bytes(&ristretto255::point_compress(&comm.point))
    }

    /// Moves a Ristretto point into a Pedersen commitment.
    public fun commitment_from_point(point: RistrettoPoint): Commitment {
        Commitment {
            point
        }
    }

    /// Deserializes a commitment from a compressed Ristretto point.
    public fun commitment_from_compressed(point: &CompressedRistretto): Commitment {
        Commitment {
            point: ristretto255::point_decompress(point)
        }
    }

    /// Returns a commitment `v * val_base + r * rand_base` where `(val_base, rand_base)` is the commitment key.
    public fun new_commitment(v: &Scalar, val_base: &RistrettoPoint, r: &Scalar, rand_base: &RistrettoPoint): Commitment {
        Commitment {
            point: ristretto255::double_scalar_mul(v, val_base, r, rand_base)
        }
    }

    /// Returns a commitment `v * G + r * rand_base` where `G` is the Ristretto255 basepoint.
    public fun new_commitment_with_basepoint(v: &Scalar, r: &Scalar, rand_base: &RistrettoPoint): Commitment {
        Commitment {
            point: ristretto255::basepoint_double_mul(r, rand_base, v)
        }
    }

    /// Returns a commitment `v * G + r * H` where `G` is the Ristretto255 basepoint and `H` is the default randomness
    /// base used in the Bulletproofs library (i.e., `BULLETPROOF_DEFAULT_PEDERSEN_RAND_BASE`).
    public fun new_commitment_for_bulletproof(v: &Scalar, r: &Scalar): Commitment {
        let rand_base = ristretto255::new_point_from_bytes(BULLETPROOF_DEFAULT_PEDERSEN_RAND_BASE);
        let rand_base = std::option::extract(&mut rand_base);

        Commitment {
            point: ristretto255::basepoint_double_mul(r, &rand_base, v)
        }
    }

    /// Homomorphically combines two commitments `lhs` and `rhs` as `lhs + rhs`.
    /// Useful for re-randomizing the commitment or updating the committed value.
    public fun commitment_add(lhs: &Commitment, rhs: &Commitment): Commitment {
        Commitment {
            point: ristretto255::point_add(&lhs.point, &rhs.point)
        }
    }

    /// Like `commitment_add` but assigns `lhs = lhs + rhs`.
    public fun commitment_add_assign(lhs: &mut Commitment, rhs: &Commitment) {
        ristretto255::point_add_assign(&mut lhs.point, &rhs.point);
    }

    /// Homomorphically combines two commitments `lhs` and `rhs` as `lhs - rhs`.
    /// Useful for re-randomizing the commitment or updating the committed value.
    public fun commitment_sub(lhs: &Commitment, rhs: &Commitment): Commitment {
        Commitment {
            point: ristretto255::point_sub(&lhs.point, &rhs.point)
        }
    }

    /// Like `commitment_add` but assigns `lhs = lhs - rhs`.
    public fun commitment_sub_assign(lhs: &mut Commitment, rhs: &Commitment) {
        ristretto255::point_sub_assign(&mut lhs.point, &rhs.point);
    }

    /// Creates a copy of this commitment.
    public fun commitment_clone(c: &Commitment): Commitment {
        Commitment {
            point: ristretto255::point_clone(&c.point)
        }
    }

    /// Returns true if the two commitments are identical: i.e., same value and same randomness.
    public fun commitment_equals(lhs: &Commitment, rhs: &Commitment): bool {
        ristretto255::point_equals(&lhs.point, &rhs.point)
    }

    /// Returns the underlying elliptic curve point representing the commitment as an in-memory `RistrettoPoint`.
    public fun commitment_as_point(c: &Commitment): &RistrettoPoint {
        &c.point
    }

    /// Returns the Pedersen commitment as a `CompressedRistretto` point.
    public fun commitment_as_compressed_point(c: &Commitment): CompressedRistretto {
        point_compress(&c.point)
    }

    /// Moves the Commitment into a CompressedRistretto point.
    public fun commitment_into_point(c: Commitment): RistrettoPoint {
        let Commitment { point } = c;
        point
    }

    /// Moves the Commitment into a `CompressedRistretto` point.
    public fun commitment_into_compressed_point(c: Commitment): CompressedRistretto {
        point_compress(&c.point)
    }

    /// Returns the randomness base compatible with the Bulletproofs module.
    ///
    /// Recal that a Bulletproof range proof attests, in zero-knowledge, that a value `v` inside a Pedersen commitment
    /// `v * g + r * h` is sufficiently "small" (e.g., is 32-bits wide). Here, `h` is referred to as the
    /// "randomness base" of the commitment scheme.
    ///
    /// Bulletproof has a default choice for `g` and `h` and this function returns the default `h` as used in the
    /// Bulletproofs Move module.
    public fun randomness_base_for_bulletproof(): RistrettoPoint {
        std::option::extract(&mut ristretto255::new_point_from_bytes(BULLETPROOF_DEFAULT_PEDERSEN_RAND_BASE))
    }
}
