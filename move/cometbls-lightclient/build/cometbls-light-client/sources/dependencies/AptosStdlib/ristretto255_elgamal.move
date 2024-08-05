/// This module implements an ElGamal encryption API, over the Ristretto255 curve, that can be used with the
/// Bulletproofs module.
///
/// An ElGamal *ciphertext* is an encryption of a value `v` under a basepoint `G` and public key `Y = sk * G`, where `sk`
/// is the corresponding secret key, is `(v * G + r * Y, r * G)`, for a random scalar `r`.
///
/// Note that we place the value `v` "in the exponent" of `G` so that ciphertexts are additively homomorphic: i.e., so
/// that `Enc_Y(v, r) + Enc_Y(v', r') = Enc_Y(v + v', r + r')` where `v, v'` are plaintext messages, `Y` is a public key and `r, r'`
/// are the randomness of the ciphertexts.

module aptos_std::ristretto255_elgamal {
    use aptos_std::ristretto255::{Self, RistrettoPoint, Scalar, CompressedRistretto, point_compress};
    use std::option::Option;
    use std::vector;

    //
    // Structs
    //

    /// An ElGamal ciphertext.
    struct Ciphertext has drop {
        left: RistrettoPoint,   // v * G + r * Y
        right: RistrettoPoint,  // r * G
    }

    /// A compressed ElGamal ciphertext.
    struct CompressedCiphertext has store, copy, drop {
        left: CompressedRistretto,
        right: CompressedRistretto,
    }

    /// An ElGamal public key.
    struct CompressedPubkey has store, copy, drop {
        point: CompressedRistretto,
    }

    //
    // Public functions
    //

    /// Creates a new public key from a serialized Ristretto255 point.
    public fun new_pubkey_from_bytes(bytes: vector<u8>): Option<CompressedPubkey> {
        let point = ristretto255::new_compressed_point_from_bytes(bytes);
        if (std::option::is_some(&mut point)) {
            let pk = CompressedPubkey {
                point: std::option::extract(&mut point)
            };
            std::option::some(pk)
        } else {
            std::option::none<CompressedPubkey>()
        }
    }

    /// Given an ElGamal public key `pubkey`, returns the byte representation of that public key.
    public fun pubkey_to_bytes(pubkey: &CompressedPubkey): vector<u8> {
        ristretto255::compressed_point_to_bytes(pubkey.point)
    }

    /// Given a public key `pubkey`, returns the underlying `RistrettoPoint` representing that key.
    public fun pubkey_to_point(pubkey: &CompressedPubkey): RistrettoPoint {
        ristretto255::point_decompress(&pubkey.point)
    }

    /// Given a public key, returns the underlying `CompressedRistretto` point representing that key.
    public fun pubkey_to_compressed_point(pubkey: &CompressedPubkey): CompressedRistretto {
        pubkey.point
    }

    /// Creates a new ciphertext from two serialized Ristretto255 points: the first 32 bytes store `r * G` while the
    /// next 32 bytes store `v * G + r * Y`, where `Y` is the public key.
    public fun new_ciphertext_from_bytes(bytes: vector<u8>): Option<Ciphertext> {
        if(vector::length(&bytes) != 64) {
            return std::option::none<Ciphertext>()
        };

        let bytes_right = vector::trim(&mut bytes, 32);

        let left_point = ristretto255::new_point_from_bytes(bytes);
        let right_point = ristretto255::new_point_from_bytes(bytes_right);

        if (std::option::is_some<RistrettoPoint>(&mut left_point) && std::option::is_some<RistrettoPoint>(&mut right_point)) {
            std::option::some<Ciphertext>(Ciphertext {
                left: std::option::extract<RistrettoPoint>(&mut left_point),
                right: std::option::extract<RistrettoPoint>(&mut right_point)
            })
        } else {
            std::option::none<Ciphertext>()
        }
    }

    /// Creates a new ciphertext `(val * G + 0 * Y, 0 * G) = (val * G, 0 * G)` where `G` is the Ristretto255 basepoint
    /// and the randomness is set to zero.
    public fun new_ciphertext_no_randomness(val: &Scalar): Ciphertext {
        Ciphertext {
            left: ristretto255::basepoint_mul(val),
            right: ristretto255::point_identity(),
        }
    }

    /// Moves a pair of Ristretto points into an ElGamal ciphertext.
    public fun ciphertext_from_points(left: RistrettoPoint, right: RistrettoPoint): Ciphertext {
        Ciphertext {
            left,
            right,
        }
    }

    /// Moves a pair of `CompressedRistretto` points into an ElGamal ciphertext.
    public fun ciphertext_from_compressed_points(left: CompressedRistretto, right: CompressedRistretto): CompressedCiphertext {
        CompressedCiphertext {
            left,
            right,
        }
    }

    /// Given a ciphertext `ct`, serializes that ciphertext into bytes.
    public fun ciphertext_to_bytes(ct: &Ciphertext): vector<u8> {
        let bytes_left = ristretto255::point_to_bytes(&ristretto255::point_compress(&ct.left));
        let bytes_right = ristretto255::point_to_bytes(&ristretto255::point_compress(&ct.right));
        let bytes = vector::empty<u8>();
        vector::append<u8>(&mut bytes, bytes_left);
        vector::append<u8>(&mut bytes, bytes_right);
        bytes
    }

    /// Moves the ciphertext into a pair of `RistrettoPoint`'s.
    public fun ciphertext_into_points(c: Ciphertext): (RistrettoPoint, RistrettoPoint) {
        let Ciphertext { left, right } = c;
        (left, right)
    }

    /// Returns the pair of `RistrettoPoint`'s representing the ciphertext.
    public fun ciphertext_as_points(c: &Ciphertext): (&RistrettoPoint, &RistrettoPoint) {
        (&c.left, &c.right)
    }

    /// Creates a new compressed ciphertext from a decompressed ciphertext.
    public fun compress_ciphertext(ct: &Ciphertext): CompressedCiphertext {
        CompressedCiphertext {
            left: point_compress(&ct.left),
            right: point_compress(&ct.right),
        }
    }

    /// Creates a new decompressed ciphertext from a compressed ciphertext.
    public fun decompress_ciphertext(ct: &CompressedCiphertext): Ciphertext {
        Ciphertext {
            left: ristretto255::point_decompress(&ct.left),
            right: ristretto255::point_decompress(&ct.right),
        }
    }

    /// Homomorphically combines two ciphertexts `lhs` and `rhs` as `lhs + rhs`.
    /// Useful for re-randomizing the ciphertext or updating the committed value.
    public fun ciphertext_add(lhs: &Ciphertext, rhs: &Ciphertext): Ciphertext {
        Ciphertext {
            left: ristretto255::point_add(&lhs.left, &rhs.left),
            right: ristretto255::point_add(&lhs.right, &rhs.right),
        }
    }

    /// Like `ciphertext_add` but assigns `lhs = lhs + rhs`.
    public fun ciphertext_add_assign(lhs: &mut Ciphertext, rhs: &Ciphertext) {
        ristretto255::point_add_assign(&mut lhs.left, &rhs.left);
        ristretto255::point_add_assign(&mut lhs.right, &rhs.right);
    }

    /// Homomorphically combines two ciphertexts `lhs` and `rhs` as `lhs - rhs`.
    /// Useful for re-randomizing the ciphertext or updating the committed value.
    public fun ciphertext_sub(lhs: &Ciphertext, rhs: &Ciphertext): Ciphertext {
        Ciphertext {
            left: ristretto255::point_sub(&lhs.left, &rhs.left),
            right: ristretto255::point_sub(&lhs.right, &rhs.right),
        }
    }

    /// Like `ciphertext_add` but assigns `lhs = lhs - rhs`.
    public fun ciphertext_sub_assign(lhs: &mut Ciphertext, rhs: &Ciphertext) {
        ristretto255::point_sub_assign(&mut lhs.left, &rhs.left);
        ristretto255::point_sub_assign(&mut lhs.right, &rhs.right);
    }

    /// Creates a copy of this ciphertext.
    public fun ciphertext_clone(c: &Ciphertext): Ciphertext {
        Ciphertext {
            left: ristretto255::point_clone(&c.left),
            right: ristretto255::point_clone(&c.right),
        }
    }

    /// Returns true if the two ciphertexts are identical: i.e., same value and same randomness.
    public fun ciphertext_equals(lhs: &Ciphertext, rhs: &Ciphertext): bool {
        ristretto255::point_equals(&lhs.left, &rhs.left) &&
        ristretto255::point_equals(&lhs.right, &rhs.right)
    }

    /// Returns the `RistrettoPoint` in the ciphertext which contains the encrypted value in the exponent.
    public fun get_value_component(ct: &Ciphertext): &RistrettoPoint {
        &ct.left
    }

    //
    // Test-only functions
    //

    #[test_only]
    /// Given an ElGamal secret key `sk`, returns the corresponding ElGamal public key as `sk * G`.
    public fun pubkey_from_secret_key(sk: &Scalar): CompressedPubkey {
        let point = ristretto255::basepoint_mul(sk);
        CompressedPubkey {
            point: point_compress(&point)
        }
    }

    #[test_only]
    /// Returns a ciphertext (v * point + r * pubkey, r * point) where `point` is *any* Ristretto255 point,
    /// `pubkey` is the public key and `r` is the randomness.
    public fun new_ciphertext(v: &Scalar, point: &RistrettoPoint, r: &Scalar, pubkey: &CompressedPubkey): Ciphertext {
        Ciphertext {
            left: ristretto255::double_scalar_mul(v, point, r, &pubkey_to_point(pubkey)),
            right: ristretto255::point_mul(point, r),
        }
    }

    #[test_only]
    /// Returns a ciphertext (v * basepoint + r * pubkey, r * basepoint) where `basepoint` is the Ristretto255 basepoint
    /// `pubkey` is the public key and `r` is the randomness.
    public fun new_ciphertext_with_basepoint(v: &Scalar, r: &Scalar, pubkey: &CompressedPubkey): Ciphertext {
        Ciphertext {
            left: ristretto255::basepoint_double_mul(r, &pubkey_to_point(pubkey), v),
            right: ristretto255::basepoint_mul(r),
        }
    }
}
