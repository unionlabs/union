/// This module defines marker types, constants and test cases for working with BN254 curves using the generic API defined in `algebra.move`.
/// BN254 was sampled as part of the [\[BCTV14\]](https://eprint.iacr.org/2013/879.pdf) paper .
/// The name denotes that it is a Barreto-Naehrig curve of embedding degree 12, defined over a 254-bit (prime) field.
/// The scalar field is highly 2-adic which supports subgroups of roots of unity of size <= 2^28.
/// (as (21888242871839275222246405745257275088548364400416034343698204186575808495617 - 1) mod 2^28 = 0)
///
/// This curve is also implemented in [libff](https://github.com/scipr-lab/libff/tree/master/libff/algebra/curves/alt_bn128) under the name `bn128`.
/// It is the same as the `bn254` curve used in Ethereum (eg: [go-ethereum](https://github.com/ethereum/go-ethereum/tree/master/crypto/bn254/cloudflare)).
///
/// #CAUTION
/// **This curve does not satisfy the 128-bit security level anymore.**
///
/// Its current security is estimated at 128-bits (see "Updating Key Size Estimations for Pairings"; by Barbulescu, Razvan and Duquesne, Sylvain; in Journal of Cryptology; 2019; https://doi.org/10.1007/s00145-018-9280-5)
///
///
/// Curve information:
/// * Base field: q =
///   21888242871839275222246405745257275088696311157297823662689037894645226208583
/// * Scalar field: r =
///   21888242871839275222246405745257275088548364400416034343698204186575808495617
/// * valuation(q - 1, 2) = 1
/// * valuation(r - 1, 2) = 28
/// * G1 curve equation: y^2 = x^3 + 3
/// * G2 curve equation: y^2 = x^3 + B, where
///    * B = 3/(u+9) where Fq2 is represented as Fq\[u\]/(u^2+1) =
///      Fq2(19485874751759354771024239261021720505790618469301721065564631296452457478373,
///      266929791119991161246907387137283842545076965332900288569378510910307636690)
///
///
/// Currently-supported BN254 structures include `Fq12`, `Fr`, `Fq`, `Fq2`, `G1`, `G2` and `Gt`,
/// along with their widely-used serialization formats,
/// the pairing between `G1`, `G2` and `Gt`.
///
/// Other unimplemented BN254 structures and serialization formats are also listed here,
/// as they help define some of the currently supported structures.
/// Their implementation may also be added in the future.
///
/// `Fq2`: The finite field $F_{q^2}$ that can be used as the base field of $G_2$
/// which is an extension field of `Fq`, constructed as $F_{q^2}=F_{q}[u]/(u^2+1)$.
///
/// `FormatFq2LscLsb`: A serialization scheme for `Fq2` elements,
/// where an element $(c_0+c_1\cdot u)$ is represented by a byte array `b[]` of size N=64,
/// which is a concatenation of its coefficients serialized, with the least significant coefficient (LSC) coming first.
/// - `b[0..32]` is $c_0$ serialized using `FormatFqLscLsb`.
/// - `b[32..64]` is $c_1$ serialized using `FormatFqLscLsb`.
///
/// `Fq6`: the finite field $F_{q^6}$ used in BN254 curves,
/// which is an extension field of `Fq2`, constructed as $F_{q^6}=F_{q^2}[v]/(v^3-u-9)$.
///
/// `FormatFq6LscLsb`: a serialization scheme for `Fq6` elements,
/// where an element in the form $(c_0+c_1\cdot v+c_2\cdot v^2)$ is represented by a byte array `b[]` of size 192,
/// which is a concatenation of its coefficients serialized, with the least significant coefficient (LSC) coming first:
/// - `b[0..64]` is $c_0$ serialized using `FormatFq2LscLsb`.
/// - `b[64..128]` is $c_1$ serialized using `FormatFq2LscLsb`.
/// - `b[128..192]` is $c_2$ serialized using `FormatFq2LscLsb`.
///
/// `G1Full`: a group constructed by the points on the BN254 curve $E(F_q): y^2=x^3+3$ and the point at infinity,
/// under the elliptic curve point addition.
/// It contains the prime-order subgroup $G_1$ used in pairing.
///
/// `G2Full`: a group constructed by the points on a curve $E'(F_{q^2}): y^2=x^3+3/(u+9)$ and the point at infinity,
/// under the elliptic curve point addition.
/// It contains the prime-order subgroup $G_2$ used in pairing.
module std::bn254_algebra {
    //
    // Marker types + serialization formats begin.
    //

    /// The finite field $F_r$ that can be used as the scalar fields
    /// associated with the groups $G_1$, $G_2$, $G_t$ in BN254-based pairing.
    struct Fr {}

    /// A serialization format for `Fr` elements,
    /// where an element is represented by a byte array `b[]` of size 32 with the least significant byte (LSB) coming first.
    ///
    /// NOTE: other implementation(s) using this format: ark-bn254-0.4.0.
    struct FormatFrLsb {}

    /// A serialization scheme for `Fr` elements,
    /// where an element is represented by a byte array `b[]` of size 32 with the most significant byte (MSB) coming first.
    ///
    /// NOTE: other implementation(s) using this format: ark-bn254-0.4.0.
    struct FormatFrMsb {}

    /// The finite field $F_q$ that can be used as the base field of $G_1$
    struct Fq {}

    /// A serialization format for `Fq` elements,
    /// where an element is represented by a byte array `b[]` of size 32 with the least significant byte (LSB) coming first.
    ///
    /// NOTE: other implementation(s) using this format: ark-bn254-0.4.0.
    struct FormatFqLsb {}

    /// A serialization scheme for `Fq` elements,
    /// where an element is represented by a byte array `b[]` of size 32 with the most significant byte (MSB) coming first.
    ///
    /// NOTE: other implementation(s) using this format: ark-bn254-0.4.0.
    struct FormatFqMsb {}

    /// The finite field $F_{q^12}$ used in BN254 curves,
    /// which is an extension field of `Fq6` (defined in the module documentation), constructed as $F_{q^12}=F_{q^6}[w]/(w^2-v)$.
    /// The field can downcast to `Gt` if it's an element of the multiplicative subgroup `Gt` of `Fq12`
    /// with a prime order $r$ = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001.
    struct Fq12 {}
    /// A serialization scheme for `Fq12` elements,
    /// where an element $(c_0+c_1\cdot w)$ is represented by a byte array `b[]` of size 384,
    /// which is a concatenation of its coefficients serialized, with the least significant coefficient (LSC) coming first.
    /// - `b[0..192]` is $c_0$ serialized using `FormatFq6LscLsb` (defined in the module documentation).
    /// - `b[192..384]` is $c_1$ serialized using `FormatFq6LscLsb`.
    ///
    /// NOTE: other implementation(s) using this format: ark-bn254-0.4.0.
    struct FormatFq12LscLsb {}

    /// The group $G_1$ in BN254-based pairing $G_1 \times G_2 \rightarrow G_t$.
    /// It is a subgroup of `G1Full` (defined in the module documentation) with a prime order $r$
    /// equal to 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001.
    /// (so `Fr` is the associated scalar field).
    struct G1 {}

    /// A serialization scheme for `G1` elements derived from arkworks.rs.
    ///
    /// Below is the serialization procedure that takes a `G1` element `p` and outputs a byte array of size N=64.
    /// 1. Let `(x,y)` be the coordinates of `p` if `p` is on the curve, or `(0,0)` otherwise.
    /// 1. Serialize `x` and `y` into `b_x[]` and `b_y[]` respectively using `FormatFqLsb` (defined in the module documentation).
    /// 1. Concatenate `b_x[]` and `b_y[]` into `b[]`.
    /// 1. If `p` is the point at infinity, set the infinity bit: `b[N-1]: = b[N-1] | 0b0100_0000`.
    /// 1. If `y > -y`, set the lexicographical bit:  `b[N-1]: = b[N-1] | 0b1000_0000`.
    /// 1. Return `b[]`.
    ///
    /// Below is the deserialization procedure that takes a byte array `b[]` and outputs either a `G1` element or none.
    /// 1. If the size of `b[]` is not N, return none.
    /// 1. Compute the infinity flag as `b[N-1] & 0b0100_0000 != 0`.
    /// 1. If the infinity flag is set, return the point at infinity.
    /// 1. Deserialize `[b[0], b[1], ..., b[N/2-1]]` to `x` using `FormatFqLsb`. If `x` is none, return none.
    /// 1. Deserialize `[b[N/2], ..., b[N] & 0b0011_1111]` to `y` using `FormatFqLsb`. If `y` is none, return none.
    /// 1. Check if `(x,y)` is on curve `E`. If not, return none.
    /// 1. Check if `(x,y)` is in the subgroup of order `r`. If not, return none.
    /// 1. Return `(x,y)`.
    ///
    /// NOTE: other implementation(s) using this format: ark-bn254-0.4.0.
    struct FormatG1Uncompr {}

    /// A serialization scheme for `G1` elements derived from arkworks.rs
    ///
    /// Below is the serialization procedure that takes a `G1` element `p` and outputs a byte array of size N=32.
    /// 1. Let `(x,y)` be the coordinates of `p` if `p` is on the curve, or `(0,0)` otherwise.
    /// 1. Serialize `x` into `b[]` using `FormatFqLsb` (defined in the module documentation).
    /// 1. If `p` is the point at infinity, set the infinity bit: `b[N-1]: = b[N-1] | 0b0100_0000`.
    /// 1. If `y > -y`, set the lexicographical flag: `b[N-1] := b[N-1] | 0x1000_0000`.
    /// 1. Return `b[]`.
    ///
    /// Below is the deserialization procedure that takes a byte array `b[]` and outputs either a `G1` element or none.
    /// 1. If the size of `b[]` is not N, return none.
    /// 1. Compute the infinity flag as `b[N-1] & 0b0100_0000 != 0`.
    /// 1. If the infinity flag is set, return the point at infinity.
    /// 1. Compute the lexicographical flag as `b[N-1] & 0b1000_0000 != 0`.
    /// 1. Deserialize `[b[0], b[1], ..., b[N/2-1] & 0b0011_1111]` to `x` using `FormatFqLsb`. If `x` is none, return none.
    /// 1. Solve the curve equation with `x` for `y`. If no such `y` exists, return none.
    /// 1. Let `y'` be `max(y,-y)` if the lexicographical flag is set, or `min(y,-y)` otherwise.
    /// 1. Check if `(x,y')` is in the subgroup of order `r`. If not, return none.
    /// 1. Return `(x,y')`.
    ///
    /// NOTE: other implementation(s) using this format: ark-bn254-0.4.0.
    struct FormatG1Compr {}

    /// The group $G_2$ in BN254-based pairing $G_1 \times G_2 \rightarrow G_t$.
    /// It is a subgroup of `G2Full` (defined in the module documentation) with a prime order $r$ equal to
    /// 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001.
    /// (so `Fr` is the scalar field).
    struct G2 {}

    /// A serialization scheme for `G2` elements derived from arkworks.rs.
    ///
    /// Below is the serialization procedure that takes a `G2` element `p` and outputs a byte array of size N=128.
    /// 1. Let `(x,y)` be the coordinates of `p` if `p` is on the curve, or `(0,0)` otherwise.
    /// 1. Serialize `x` and `y` into `b_x[]` and `b_y[]` respectively using `FormatFq2LscLsb` (defined in the module documentation).
    /// 1. Concatenate `b_x[]` and `b_y[]` into `b[]`.
    /// 1. If `p` is the point at infinity, set the infinity bit: `b[N-1]: = b[N-1] | 0b0100_0000`.
    /// 1. If `y > -y`, set the lexicographical bit:  `b[N-1]: = b[N-1] | 0b1000_0000`.
    /// 1. Return `b[]`.
    ///
    /// Below is the deserialization procedure that takes a byte array `b[]` and outputs either a `G1` element or none.
    /// 1. If the size of `b[]` is not N, return none.
    /// 1. Compute the infinity flag as `b[N-1] & 0b0100_0000 != 0`.
    /// 1. If the infinity flag is set, return the point at infinity.
    /// 1. Deserialize `[b[0], b[1], ..., b[N/2-1]]` to `x` using `FormatFq2LscLsb`. If `x` is none, return none.
    /// 1. Deserialize `[b[N/2], ..., b[N] & 0b0011_1111]` to `y` using `FormatFq2LscLsb`. If `y` is none, return none.
    /// 1. Check if `(x,y)` is on curve `E`. If not, return none.
    /// 1. Check if `(x,y)` is in the subgroup of order `r`. If not, return none.
    /// 1. Return `(x,y)`.
    ///
    /// NOTE: other implementation(s) using this format: ark-bn254-0.4.0.
    struct FormatG2Uncompr {}

    /// A serialization scheme for `G1` elements derived from arkworks.rs
    ///
    /// Below is the serialization procedure that takes a `G1` element `p` and outputs a byte array of size N=64.
    /// 1. Let `(x,y)` be the coordinates of `p` if `p` is on the curve, or `(0,0)` otherwise.
    /// 1. Serialize `x` into `b[]` using `FormatFq2LscLsb` (defined in the module documentation).
    /// 1. If `p` is the point at infinity, set the infinity bit: `b[N-1]: = b[N-1] | 0b0100_0000`.
    /// 1. If `y > -y`, set the lexicographical flag: `b[N-1] := b[N-1] | 0x1000_0000`.
    /// 1. Return `b[]`.
    ///
    /// Below is the deserialization procedure that takes a byte array `b[]` and outputs either a `G1` element or none.
    /// 1. If the size of `b[]` is not N, return none.
    /// 1. Compute the infinity flag as `b[N-1] & 0b0100_0000 != 0`.
    /// 1. If the infinity flag is set, return the point at infinity.
    /// 1. Compute the lexicographical flag as `b[N-1] & 0b1000_0000 != 0`.
    /// 1. Deserialize `[b[0], b[1], ..., b[N/2-1] & 0b0011_1111]` to `x` using `FormatFq2LscLsb`. If `x` is none, return none.
    /// 1. Solve the curve equation with `x` for `y`. If no such `y` exists, return none.
    /// 1. Let `y'` be `max(y,-y)` if the lexicographical flag is set, or `min(y,-y)` otherwise.
    /// 1. Check if `(x,y')` is in the subgroup of order `r`. If not, return none.
    /// 1. Return `(x,y')`.
    ///
    /// NOTE: other implementation(s) using this format: ark-bn254-0.4.0.
    struct FormatG2Compr {}

    /// The group $G_t$ in BN254-based pairing $G_1 \times G_2 \rightarrow G_t$.
    /// It is a multiplicative subgroup of `Fq12`, so it  can upcast to `Fq12`.
    /// with a prime order $r$ equal to 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001.
    /// (so `Fr` is the scalar field).
    /// The identity of `Gt` is 1.
    struct Gt {}

    /// A serialization scheme for `Gt` elements.
    ///
    /// To serialize, it treats a `Gt` element `p` as an `Fq12` element and serialize it using `FormatFq12LscLsb`.
    ///
    /// To deserialize, it uses `FormatFq12LscLsb` to try deserializing to an `Fq12` element then test the membership in `Gt`.
    ///
    /// NOTE: other implementation(s) using this format: ark-bn254-0.4.0.
    struct FormatGt {}

    // Tests begin.

    #[test_only]
    fun rand_vector<S>(num: u64): vector<Element<S>> {
        let elements = vector[];
        while (num > 0) {
            std::vector::push_back(&mut elements, rand_insecure<S>());
            num = num - 1;
        };
        elements
    }


    #[test_only]
    const FQ12_VAL_0_SERIALIZED: vector<u8> = x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const FQ12_VAL_1_SERIALIZED: vector<u8> = x"010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const FQ12_VAL_7_SERIALIZED: vector<u8> = x"070000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const FQ12_VAL_7_NEG_SERIALIZED: vector<u8> = x"40fd7cd8168c203c8dca7168916a81975d588181b64550b829a031e1724e643000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const Q12_SERIALIZED: vector<u8> = x"21f186cad2e2d4c1dbaf8a066b0ebf41f734e3f859b1c523a6c1f4d457413fdbe3cd44add090135d3ae519acc30ee3bdb6bfac6573b767e975b18a77d53cdcddebf3672c74da9d1409d51b2b2db7ff000d59e3aa7cf09220159f925c86b65459ca6558c4eaa703bf45d85030ff85cc6a879c7e2c4034f7045faf20e4d3dcfffac5eb6634c3e7b939b69b2be70bdf6b9a4680297839b4e3a48cd746bd4d0ea82749ffb7e71bd9b3fb10aa684d71e6adab1250b1d8604d91b51c76c256a50b60ddba2f52b6cc853ac926c6ea86d09d400b2f2330e5c8e92e38905ba50a50c9e11cd979c284bf1327ccdc051a6da1a4a7eac5cec16757a27a1a2311bedd108a9b21ac0814269e7523a5dd3a1f5f4767ffe504a6cb3994fb0ec98d5cd5da00b9cb1188a85f2aa871ecb8a0f9d64141f1ccd2699c138e0ef9ac4d8d6a692b29db0f38b60eb08426ab46109fbab9a5221bb44dd338aafebcc4e6c10dd933597f3ff44ba41d04e82871447f3a759cfa9397c22c0c77f13618dfb65adc8aacf008";


    #[test(fx = @std)]
    fun test_fq12(fx: signer) {
        enable_cryptography_algebra_natives(&fx);

        // Constants.
        assert!(Q12_SERIALIZED == order<Fq12>(), 1);

        // Serialization/deserialization.
        let val_0 = zero<Fq12>();
        let val_1 = one<Fq12>();
        assert!(FQ12_VAL_0_SERIALIZED == serialize<Fq12, FormatFq12LscLsb>(&val_0), 1);
        assert!(FQ12_VAL_1_SERIALIZED == serialize<Fq12, FormatFq12LscLsb>(&val_1), 1);
        let val_7 = from_u64<Fq12>(7);
        let val_7_another = std::option::extract(&mut deserialize<Fq12, FormatFq12LscLsb>(&FQ12_VAL_7_SERIALIZED));
        assert!(eq(&val_7, &val_7_another), 1);
        assert!(FQ12_VAL_7_SERIALIZED == serialize<Fq12, FormatFq12LscLsb>(&val_7), 1);
        assert!(std::option::is_none(&deserialize<Fq12, FormatFq12LscLsb>(&x"ffff")), 1);

        // Negation.
        let val_minus_7 = neg(&val_7);
        assert!(FQ12_VAL_7_NEG_SERIALIZED == serialize<Fq12, FormatFq12LscLsb>(&val_minus_7), 1);

        // Addition.
        let val_9 = from_u64<Fq12>(9);
        let val_2 = from_u64<Fq12>(2);
        assert!(eq(&val_2, &add(&val_minus_7, &val_9)), 1);

        // Subtraction.
        assert!(eq(&val_9, &sub(&val_2, &val_minus_7)), 1);

        // Multiplication.
        let val_63 = from_u64<Fq12>(63);
        assert!(eq(&val_63, &mul(&val_7, &val_9)), 1);

        // division.
        let val_0 = from_u64<Fq12>(0);
        assert!(eq(&val_7, &std::option::extract(&mut div(&val_63, &val_9))), 1);
        assert!(std::option::is_none(&div(&val_63, &val_0)), 1);

        // Inversion.
        assert!(eq(&val_minus_7, &neg(&val_7)), 1);
        assert!(std::option::is_none(&inv(&val_0)), 1);

        // Squaring.
        let val_x = rand_insecure<Fq12>();
        assert!(eq(&mul(&val_x, &val_x), &sqr(&val_x)), 1);

        // Downcasting.
        assert!(eq(&zero<Gt>(), &std::option::extract(&mut downcast<Fq12, Gt>(&val_1))), 1);
        // upcasting
        assert!(eq(&val_1, &upcast<Gt, Fq12>(&zero<Gt>())), 1);
    }

    #[test_only]
    const R_SERIALIZED: vector<u8> = x"010000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430";
    #[test_only]
    const G1_INF_SERIALIZED_COMP: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000040";
    #[test_only]
    const G1_INF_SERIALIZED_UNCOMP: vector<u8> = x"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040";
    #[test_only]
    const G1_GENERATOR_SERIALIZED_COMP: vector<u8> = x"0100000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const G1_GENERATOR_SERIALIZED_UNCOMP: vector<u8> = x"01000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const G1_GENERATOR_MUL_BY_7_SERIALIZED_COMP: vector<u8> = x"78e0ffab866b3a9876bd01b8ecc66fcb86936277f425539a758dbbd32e2b0717";
    #[test_only]
    const G1_GENERATOR_MUL_BY_7_SERIALIZED_UNCOMP: vector<u8> = x"78e0ffab866b3a9876bd01b8ecc66fcb86936277f425539a758dbbd32e2b07179eafd4607f9f80771bf4185df03bfead7a3719fa4bb57b0152dd30d16cda8a16";
    #[test_only]
    const G1_GENERATOR_MUL_BY_7_NEG_SERIALIZED_COMP: vector<u8> = x"78e0ffab866b3a9876bd01b8ecc66fcb86936277f425539a758dbbd32e2b0797";
    #[test_only]
    const G1_GENERATOR_MUL_BY_7_NEG_SERIALIZED_UNCOMP: vector<u8> = x"78e0ffab866b3a9876bd01b8ecc66fcb86936277f425539a758dbbd32e2b0717a94da87797ec9fc471d6580ba12e83e9e22068876a90d4b6d7c200100674d999";

    #[test(fx = @std)]
    fun test_g1affine(fx: signer) {
        enable_cryptography_algebra_natives(&fx);

        // Constants.
        assert!(R_SERIALIZED == order<G1>(), 1);
        let point_at_infinity = zero<G1>();
        let generator = one<G1>();

        // Serialization/deserialization.
        assert!(G1_GENERATOR_SERIALIZED_UNCOMP == serialize<G1, FormatG1Uncompr>(&generator), 1);
        assert!(G1_GENERATOR_SERIALIZED_COMP == serialize<G1, FormatG1Compr>(&generator), 1);
        let generator_from_comp = std::option::extract(&mut deserialize<G1, FormatG1Compr>(&G1_GENERATOR_SERIALIZED_COMP));
        let generator_from_uncomp = std::option::extract(&mut deserialize<G1, FormatG1Uncompr>(&G1_GENERATOR_SERIALIZED_UNCOMP));
        assert!(eq(&generator, &generator_from_comp), 1);
        assert!(eq(&generator, &generator_from_uncomp), 1);

        // Deserialization should fail if given a byte array of correct size but the value is not a member.
        assert!(std::option::is_none(&deserialize<Fq12, FormatFq12LscLsb>(&x"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")), 1);

        // Deserialization should fail if given a byte array of wrong size.
        assert!(std::option::is_none(&deserialize<Fq12, FormatFq12LscLsb>(&x"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")), 1);

        assert!(
            G1_INF_SERIALIZED_UNCOMP == serialize<G1, FormatG1Uncompr>(&point_at_infinity), 1);
        assert!(G1_INF_SERIALIZED_COMP == serialize<G1, FormatG1Compr>(&point_at_infinity), 1);
        let inf_from_uncomp = std::option::extract(&mut deserialize<G1, FormatG1Uncompr>(&G1_INF_SERIALIZED_UNCOMP
        ));
        let inf_from_comp = std::option::extract(&mut deserialize<G1, FormatG1Compr>(&G1_INF_SERIALIZED_COMP
        ));
        assert!(eq(&point_at_infinity, &inf_from_comp), 1);
        assert!(eq(&point_at_infinity, &inf_from_uncomp), 1);

        let point_7g_from_uncomp = std::option::extract(&mut deserialize<G1, FormatG1Uncompr>(&G1_GENERATOR_MUL_BY_7_SERIALIZED_UNCOMP
        ));
        let point_7g_from_comp = std::option::extract(&mut deserialize<G1, FormatG1Compr>(&G1_GENERATOR_MUL_BY_7_SERIALIZED_COMP
        ));
        assert!(eq(&point_7g_from_comp, &point_7g_from_uncomp), 1);

        // Deserialization should fail if given a point on the curve but off its prime-order subgroup, e.g., `(0,2)`.
        assert!(std::option::is_none(&deserialize<G1, FormatG1Uncompr>(&x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002")), 1);
        assert!(std::option::is_none(&deserialize<G1, FormatG1Compr>(&x"800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")), 1);

        // Deserialization should fail if given a valid point in (Fq,Fq) but not on the curve.
        assert!(std::option::is_none(&deserialize<G1, FormatG1Uncompr>(&x"8959e137e0719bf872abb08411010f437a8955bd42f5ba20fca64361af58ce188b1adb96ef229698bb7860b79e24ba12000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")), 1);

        // Deserialization should fail if given an invalid point (x not in Fq).
        assert!(std::option::is_none(&deserialize<G1, FormatG1Uncompr>(&x"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa76e9853b35f5c9b2002d9e5833fd8f9ab4cd3934a4722a06f6055bfca720c91629811e2ecae7f0cf301b6d07898a90f")), 1);
        assert!(std::option::is_none(&deserialize<G1, FormatG1Compr>(&x"9fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")), 1);

        // Deserialization should fail if given a byte array of wrong size.
        assert!(std::option::is_none(&deserialize<G1, FormatG1Uncompr>(&x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ab")), 1);
        assert!(std::option::is_none(&deserialize<G1, FormatG1Compr>(&x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ab")), 1);

        // Scalar multiplication.
        let scalar_7 = from_u64<Fr>(7);
        let point_7g_calc = scalar_mul(&generator, &scalar_7);
        assert!(eq(&point_7g_calc, &point_7g_from_comp), 1);
        assert!(G1_GENERATOR_MUL_BY_7_SERIALIZED_UNCOMP == serialize<G1, FormatG1Uncompr>(&point_7g_calc), 1);
        assert!(G1_GENERATOR_MUL_BY_7_SERIALIZED_COMP == serialize<G1, FormatG1Compr>( &point_7g_calc), 1);

        // Multi-scalar multiplication.
        let num_entries = 1;
        while (num_entries < 10) {
            let scalars = rand_vector<Fr>(num_entries);
            let elements = rand_vector<G1>(num_entries);

            let expected = zero<G1>();
            let i = 0;
            while (i < num_entries) {
                let element = std::vector::borrow(&elements, i);
                let scalar = std::vector::borrow(&scalars, i);
                expected = add(&expected, &scalar_mul(element, scalar));
                i = i + 1;
            };

            let actual = multi_scalar_mul(&elements, &scalars);
            assert!(eq(&expected, &actual), 1);

            num_entries = num_entries + 1;
        };

        // Doubling.
        let scalar_2 = from_u64<Fr>(2);
        let point_2g = scalar_mul(&generator, &scalar_2);
        let point_double_g = double(&generator);
        assert!(eq(&point_2g, &point_double_g), 1);

        // Negation.
        let point_minus_7g_calc = neg(&point_7g_calc);
        assert!(G1_GENERATOR_MUL_BY_7_NEG_SERIALIZED_COMP == serialize<G1, FormatG1Compr>(&point_minus_7g_calc), 1);
        assert!(G1_GENERATOR_MUL_BY_7_NEG_SERIALIZED_UNCOMP == serialize<G1, FormatG1Uncompr>(&point_minus_7g_calc), 1);

        // Addition.
        let scalar_9 = from_u64<Fr>(9);
        let point_9g = scalar_mul(&generator, &scalar_9);
        let point_2g = scalar_mul(&generator, &scalar_2);
        let point_2g_calc = add(&point_minus_7g_calc, &point_9g);
        assert!(eq(&point_2g, &point_2g_calc), 1);

        // Subtraction.
        assert!(eq(&point_9g, &sub(&point_2g, &point_minus_7g_calc)), 1);
    }

    #[test_only]
    const G2_INF_SERIALIZED_COMP: vector<u8> = x"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040";
    #[test_only]
    const G2_INF_SERIALIZED_UNCOMP: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040";
    #[test_only]
    const G2_GENERATOR_SERIALIZED_COMP: vector<u8> = x"edf692d95cbdde46ddda5ef7d422436779445c5e66006a42761e1f12efde0018c212f3aeb785e49712e7a9353349aaf1255dfb31b7bf60723a480d9293938e19";
    #[test_only]
    const G2_GENERATOR_SERIALIZED_UNCOMP: vector<u8> = x"edf692d95cbdde46ddda5ef7d422436779445c5e66006a42761e1f12efde0018c212f3aeb785e49712e7a9353349aaf1255dfb31b7bf60723a480d9293938e19aa7dfa6601cce64c7bd3430c69e7d1e38f40cb8d8071ab4aeb6d8cdba55ec8125b9722d1dcdaac55f38eb37033314bbc95330c69ad999eec75f05f58d0890609";
    #[test_only]
    const G2_GENERATOR_MUL_BY_7_SERIALIZED_UNCOMP: vector<u8> = x"08b328aa2a1490c3892ae375ba53a257162f1cde012e70edf8fc27435ddc4b2255243646bade3e596dee466e51d40fbe631e55841e085d6ae2bd9a5a01ba03293f23144105e8212ed8df28ca0e8031d47b7a7de372b3ccee1750262af5ff921dd8e03503be1eedbaadf7e6c4a1be3670d14a46da5fafee7adbdeb2a6cdb7c803";
    #[test_only]
    const G2_GENERATOR_MUL_BY_7_SERIALIZED_COMP: vector<u8> = x"08b328aa2a1490c3892ae375ba53a257162f1cde012e70edf8fc27435ddc4b2255243646bade3e596dee466e51d40fbe631e55841e085d6ae2bd9a5a01ba0329";
    #[test_only]
    const G2_GENERATOR_MUL_BY_7_NEG_SERIALIZED_UNCOMP: vector<u8> = x"08b328aa2a1490c3892ae375ba53a257162f1cde012e70edf8fc27435ddc4b2255243646bade3e596dee466e51d40fbe631e55841e085d6ae2bd9a5a01ba032908da689711a4fe0db5ea489e82ea4fc3e1dd039e439283c911500bb77d4ed1126f1c47d5586d3381dfd28aa3efab4a278c0d3ba75696613d4ec17e3aa5969bac";
    #[test_only]
    const G2_GENERATOR_MUL_BY_7_NEG_SERIALIZED_COMP: vector<u8> = x"08b328aa2a1490c3892ae375ba53a257162f1cde012e70edf8fc27435ddc4b2255243646bade3e596dee466e51d40fbe631e55841e085d6ae2bd9a5a01ba03a9";

    #[test(fx = @std)]
    fun test_g2affine(fx: signer) {
        enable_cryptography_algebra_natives(&fx);

        // Special constants.
        assert!(R_SERIALIZED == order<G2>(), 1);
        let point_at_infinity = zero<G2>();
        let generator = one<G2>();

        // Serialization/deserialization.
        assert!(G2_GENERATOR_SERIALIZED_COMP == serialize<G2, FormatG2Compr>(&generator), 1);
        assert!(G2_GENERATOR_SERIALIZED_UNCOMP == serialize<G2, FormatG2Uncompr>(&generator), 1);
        let generator_from_uncomp = std::option::extract(&mut deserialize<G2, FormatG2Uncompr>(&G2_GENERATOR_SERIALIZED_UNCOMP
        ));
        let generator_from_comp = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&G2_GENERATOR_SERIALIZED_COMP
        ));
        assert!(eq(&generator, &generator_from_comp), 1);
        assert!(eq(&generator, &generator_from_uncomp), 1);
        assert!(G2_INF_SERIALIZED_UNCOMP == serialize<G2, FormatG2Uncompr>(&point_at_infinity), 1);
        assert!(G2_INF_SERIALIZED_COMP == serialize<G2, FormatG2Compr>(&point_at_infinity), 1);
        let inf_from_uncomp = std::option::extract(&mut deserialize<G2, FormatG2Uncompr>(&G2_INF_SERIALIZED_UNCOMP));
        let inf_from_comp = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&G2_INF_SERIALIZED_COMP));
        assert!(eq(&point_at_infinity, &inf_from_comp), 1);
        assert!(eq(&point_at_infinity, &inf_from_uncomp), 1);
        let point_7g_from_uncomp = std::option::extract(&mut deserialize<G2, FormatG2Uncompr>(&G2_GENERATOR_MUL_BY_7_SERIALIZED_UNCOMP
        ));
        let point_7g_from_comp = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&G2_GENERATOR_MUL_BY_7_SERIALIZED_COMP
        ));
        assert!(eq(&point_7g_from_comp, &point_7g_from_uncomp), 1);

        // Deserialization should fail if given a point on the curve but not in the prime-order subgroup.
        assert!(std::option::is_none(&deserialize<G1, FormatG1Uncompr>(&x"f037d4ccd5ee751eba1c1fd4c7edbb76d2b04c3a1f3f554827cf37c3acbc2dbb7cdb320a2727c2462d6c55ca1f637707b96eeebc622c1dbe7c56c34f93887c8751b42bd04f29253a82251c192ef27ece373993b663f4360505299c5bd18c890ddd862a6308796bf47e2265073c1f7d81afd69f9497fc1403e2e97a866129b43b672295229c21116d4a99f3e5c2ae720a31f181dbed8a93e15f909c20cf69d11a8879adbbe6890740def19814e6d4ed23fb0dcbd79291655caf48b466ac9cae04")), 1);
        assert!(std::option::is_none(&deserialize<G1, FormatG1Compr>(&x"f037d4ccd5ee751eba1c1fd4c7edbb76d2b04c3a1f3f554827cf37c3acbc2dbb7cdb320a2727c2462d6c55ca1f637707b96eeebc622c1dbe7c56c34f93887c8751b42bd04f29253a82251c192ef27ece373993b663f4360505299c5bd18c890d")), 1);

        // Deserialization should fail if given a valid point in (Fq2,Fq2) but not on the curve.
        assert!(std::option::is_none(&deserialize<G1, FormatG1Uncompr>(&x"f037d4ccd5ee751eba1c1fd4c7edbb76d2b04c3a1f3f554827cf37c3acbc2dbb7cdb320a2727c2462d6c55ca1f637707b96eeebc622c1dbe7c56c34f93887c8751b42bd04f29253a82251c192ef27ece373993b663f4360505299c5bd18c890d000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")), 1);

        // Deserialization should fail if given an invalid point (x not in Fq2).
        assert!(std::option::is_none(&deserialize<G1, FormatG1Uncompr>(&x"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffdd862a6308796bf47e2265073c1f7d81afd69f9497fc1403e2e97a866129b43b672295229c21116d4a99f3e5c2ae720a31f181dbed8a93e15f909c20cf69d11a8879adbbe6890740def19814e6d4ed23fb0dcbd79291655caf48b466ac9cae04")), 1);
        assert!(std::option::is_none(&deserialize<G1, FormatG1Compr>(&x"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")), 1);

        // Deserialization should fail if given a byte array of wrong size.
        assert!(std::option::is_none(&deserialize<G1, FormatG1Uncompr>(&x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ab")), 1);
        assert!(std::option::is_none(&deserialize<G1, FormatG1Compr>(&x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ab")), 1);

        // Scalar multiplication.
        let scalar_7 = from_u64<Fr>(7);
        let point_7g_calc = scalar_mul(&generator, &scalar_7);
        assert!(eq(&point_7g_calc, &point_7g_from_comp), 1);
        assert!(G2_GENERATOR_MUL_BY_7_SERIALIZED_UNCOMP == serialize<G2, FormatG2Uncompr>(&point_7g_calc), 1);
        assert!(G2_GENERATOR_MUL_BY_7_SERIALIZED_COMP == serialize<G2, FormatG2Compr>(&point_7g_calc), 1);

        // Multi-scalar multiplication.
        let num_entries = 1;
        while (num_entries < 10) {
            let scalars = rand_vector<Fr>(num_entries);
            let elements = rand_vector<G2>(num_entries);

            let expected = zero<G2>();
            let i = 0;
            while (i < num_entries) {
                let element = std::vector::borrow(&elements, i);
                let scalar = std::vector::borrow(&scalars, i);
                expected = add(&expected, &scalar_mul(element, scalar));
                i = i + 1;
            };

            let actual = multi_scalar_mul(&elements, &scalars);
            assert!(eq(&expected, &actual), 1);

            num_entries = num_entries + 1;
        };

        // Doubling.
        let scalar_2 = from_u64<Fr>(2);
        let point_2g = scalar_mul(&generator, &scalar_2);
        let point_double_g = double(&generator);
        assert!(eq(&point_2g, &point_double_g), 1);

        // Negation.
        let point_minus_7g_calc = neg(&point_7g_calc);
        assert!(G2_GENERATOR_MUL_BY_7_NEG_SERIALIZED_COMP == serialize<G2, FormatG2Compr>(&point_minus_7g_calc), 1);
        assert!(G2_GENERATOR_MUL_BY_7_NEG_SERIALIZED_UNCOMP == serialize<G2, FormatG2Uncompr>(&point_minus_7g_calc), 1);

        // Addition.
        let scalar_9 = from_u64<Fr>(9);
        let point_9g = scalar_mul(&generator, &scalar_9);
        let point_2g = scalar_mul(&generator, &scalar_2);
        let point_2g_calc = add(&point_minus_7g_calc, &point_9g);
        assert!(eq(&point_2g, &point_2g_calc), 1);

        // Subtraction.
        assert!(eq(&point_9g, &sub(&point_2g, &point_minus_7g_calc)), 1);
    }

    #[test_only]
    const FQ12_ONE_SERIALIZED: vector<u8> = x"010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const GT_GENERATOR_SERIALIZED: vector<u8> = x"950e879d73631f5eb5788589eb5f7ef8d63e0a28de1ba00dfe4ca9ed3f252b264a8afb8eb4349db466ed1809ea4d7c39bdab7938821f1b0a00a295c72c2de002e01dbdfd0254134efcb1ec877395d25f937719b344adb1a58d129be2d6f2a9132b16a16e8ab030b130e69c69bd20b4c45986e6744a98314b5c1a0f50faa90b04dbaf9ef8aeeee3f50be31c210b598f4752f073987f9d35be8f6770d83f2ffc0af0d18dd9d2dbcdf943825acc12a7a9ddca45e629d962c6bd64908c3930a5541cfe2924dcc5580d5cef7a4bfdec90a91b59926f850d4a7923c01a5a5dbf0f5c094a2b9fb9d415820fa6b40c59bb9eade9c953407b0fc11da350a9d872cad6d3142974ca385854afdf5f583c04231adc5957c8914b6b20dc89660ed7c3bbe7c01d972be2d53ecdb27a1bcc16ac610db95aa7d237c8ff55a898cb88645a0e32530b23d7ebf5dafdd79b0f9c2ac4ba07ce18d3d16cf36e47916c4cae5d08d3afa813972c769e8514533e380c9443b3e1ee5c96fa3a0a73f301b626454721527bf900";
    #[test_only]
    const GT_GENERATOR_MUL_BY_7_SERIALIZED: vector<u8> = x"533a587534641b568125fb273eac723c789a347eba9fcfd58d93742b3a0b782fd61bbf6202e04b8a33b6c60150fc62a071cb9ac9749a79031fd0dbb6dd8a1f2bcf1eb450bdf58fd3d124b2e0aaf878d11e96af3051631145a4bf0530b5d19d08bfe2d515530b9059525b2826587f7bf1f146bfd0e91e84411c7722abb7a8c418b20b1660b41e6949beff93b2b36303e74804df3335ab5bd85bfd7959d6fd3101d0bf6f681eb809c9a6c3544db7f81444e5c4fbdd0a31e920616ae08a2ab5f51ebf064c4906c7b29521e8fda3d704830a9a6ef5d455a85ae09216f55fd0e74d0aaf83ad81ba50218f08024910184c9ddab42a28f51912c779556c41c61aba2d075cfc020b61a18a9366c9f71658f00b44369bd86929725cf867a0b8fda694a7134a2790ebf19cbea1f972eedfd51787683f98d80895f630ff0bd513edebd5a217c00e231869178bd41cf47a7c0125379a3926353e5310a578066dfbb974424802b942a8b4f6338d7f9d8b9c4031dc46163a59c58ff503eca69b642398b5a1212b";
    #[test_only]
    const GT_GENERATOR_MUL_BY_7_NEG_SERIALIZED: vector<u8> = x"533a587534641b568125fb273eac723c789a347eba9fcfd58d93742b3a0b782fd61bbf6202e04b8a33b6c60150fc62a071cb9ac9749a79031fd0dbb6dd8a1f2bcf1eb450bdf58fd3d124b2e0aaf878d11e96af3051631145a4bf0530b5d19d08bfe2d515530b9059525b2826587f7bf1f146bfd0e91e84411c7722abb7a8c418b20b1660b41e6949beff93b2b36303e74804df3335ab5bd85bfd7959d6fd3101d0bf6f681eb809c9a6c3544db7f81444e5c4fbdd0a31e920616ae08a2ab5f51e88f6308f10c56da66be273c4b965fe8cc3e98bac609df5d796893c81a26616269879cf565c3bffac84c82858791ee4bca82d598c9c33893ed433f01a58943629eb007acdb5ea95a826017a51397a755327bda8178dd3f3bfc1ff78e3cbb9bc1cfdd5ecec24ef619a93578388bb52fa2e1ec0a878214f1fb91dcb1df48678c11887ee59c0ad74956770d6f6eb8f454afd23324c436335ab3f23333627fe0b1c2e8ebad423205893bcef3ed527608e3a8123ffbbf1c04164118e3b0e49bdac4205";


    #[test(fx = @std)]
    fun test_gt(fx: signer) {
        enable_cryptography_algebra_natives(&fx);

        // Special constants.
        assert!(R_SERIALIZED == order<Gt>(), 1);
        let identity = zero<Gt>();
        let generator = one<Gt>();

        // Serialization/deserialization.
        assert!(GT_GENERATOR_SERIALIZED == serialize<Gt, FormatGt>(&generator), 1);
        let generator_from_deser = std::option::extract(&mut deserialize<Gt, FormatGt>(&GT_GENERATOR_SERIALIZED));
        assert!(eq(&generator, &generator_from_deser), 1);
        assert!(FQ12_ONE_SERIALIZED == serialize<Gt, FormatGt>(&identity), 1);
        let identity_from_deser = std::option::extract(&mut deserialize<Gt, FormatGt>(&FQ12_ONE_SERIALIZED));
        assert!(eq(&identity, &identity_from_deser), 1);
        let element_7g_from_deser = std::option::extract(&mut deserialize<Gt, FormatGt>(&GT_GENERATOR_MUL_BY_7_SERIALIZED
        ));
        assert!(std::option::is_none(&deserialize<Gt, FormatGt>(&x"ffff")), 1);

        // Deserialization should fail if given an element in Fq12 but not in the prime-order subgroup.
        assert!(std::option::is_none(&deserialize<Gt, FormatGt>(&x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")), 1);

        // Deserialization should fail if given a byte array of wrong size.
        assert!(std::option::is_none(&deserialize<Gt, FormatGt>(&x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ab")), 1);

        // Element scalar multiplication.
        let scalar_7 = from_u64<Fr>(7);
        let element_7g_calc = scalar_mul(&generator, &scalar_7);
        assert!(eq(&element_7g_calc, &element_7g_from_deser), 1);
        assert!(GT_GENERATOR_MUL_BY_7_SERIALIZED == serialize<Gt, FormatGt>(&element_7g_calc), 1);

        // Element negation.
        let element_minus_7g_calc = neg(&element_7g_calc);
        assert!(GT_GENERATOR_MUL_BY_7_NEG_SERIALIZED == serialize<Gt, FormatGt>(&element_minus_7g_calc), 1);

        // Element addition.
        let scalar_9 = from_u64<Fr>(9);
        let element_9g = scalar_mul(&generator, &scalar_9);
        let scalar_2 = from_u64<Fr>(2);
        let element_2g = scalar_mul(&generator, &scalar_2);
        let element_2g_calc = add(&element_minus_7g_calc, &element_9g);
        assert!(eq(&element_2g, &element_2g_calc), 1);

        // Subtraction.
        assert!(eq(&element_9g, &sub(&element_2g, &element_minus_7g_calc)), 1);

        // Upcasting to Fq12.
        assert!(eq(&one<Fq12>(), &upcast<Gt, Fq12>(&identity)), 1);
    }

    #[test_only]
    use aptos_std::crypto_algebra::{zero, one, from_u64, eq, deserialize, serialize, neg, add, sub, mul, div, inv, rand_insecure, sqr, order, scalar_mul, multi_scalar_mul, double, upcast, enable_cryptography_algebra_natives, pairing, multi_pairing, downcast, Element};

    #[test_only]
    const FR_VAL_0_SERIALIZED_LSB: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const FR_VAL_1_SERIALIZED_LSB: vector<u8> = x"0100000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const FR_VAL_7_SERIALIZED_LSB: vector<u8> = x"0700000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const FR_VAL_7_SERIALIZED_MSB: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000007";
    #[test_only]
    const FR_VAL_7_NEG_SERIALIZED_LSB: vector<u8> = x"faffffef93f5e1439170b97948e833285d588181b64550b829a031e1724e6430";

    #[test(fx = @std)]
    fun test_fr(fx: signer) {
        enable_cryptography_algebra_natives(&fx);

        // Constants.
        assert!(R_SERIALIZED == order<Fr>(), 1);

        // Serialization/deserialization.
        let val_0 = zero<Fr>();
        let val_1 = one<Fr>();
        assert!(FR_VAL_0_SERIALIZED_LSB == serialize<Fr, FormatFrLsb>(&val_0), 1);
        assert!(FR_VAL_1_SERIALIZED_LSB == serialize<Fr, FormatFrLsb>(&val_1), 1);
        let val_7 = from_u64<Fr>(7);
        let val_7_2nd = std::option::extract(&mut deserialize<Fr, FormatFrLsb>(&FR_VAL_7_SERIALIZED_LSB));
        let val_7_3rd = std::option::extract(&mut deserialize<Fr, FormatFrMsb>(&FR_VAL_7_SERIALIZED_MSB));
        assert!(eq(&val_7, &val_7_2nd), 1);
        assert!(eq(&val_7, &val_7_3rd), 1);
        assert!(FR_VAL_7_SERIALIZED_LSB == serialize<Fr, FormatFrLsb>(&val_7), 1);
        assert!(FR_VAL_7_SERIALIZED_MSB == serialize<Fr, FormatFrMsb>(&val_7), 1);

        // Deserialization should fail if given a byte array of right size but the value is not a member.
        assert!(std::option::is_none(&deserialize<Fr, FormatFrLsb>(&x"01000000fffffffffe5bfeff02a4bd5305d8a10908d83933487d9d2953a7ed73")), 1);
        assert!(std::option::is_none(&deserialize<Fr, FormatFrMsb>(&x"73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001")), 1);

        // Deserialization should fail if given a byte array of wrong size.
        assert!(std::option::is_none(&deserialize<Fr, FormatFrLsb>(&x"01000000fffffffffe5bfeff02a4bd5305d8a10908d83933487d9d2953a7ed7300")), 1);
        assert!(std::option::is_none(&deserialize<Fr, FormatFrMsb>(&x"0073eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001")), 1);
        assert!(std::option::is_none(&deserialize<Fr, FormatFrLsb>(&x"ffff")), 1);
        assert!(std::option::is_none(&deserialize<Fr, FormatFrMsb>(&x"ffff")), 1);

        // Negation.
        let val_minus_7 = neg(&val_7);
        assert!(FR_VAL_7_NEG_SERIALIZED_LSB == serialize<Fr, FormatFrLsb>(&val_minus_7), 1);

        // Addition.
        let val_9 = from_u64<Fr>(9);
        let val_2 = from_u64<Fr>(2);
        assert!(eq(&val_2, &add(&val_minus_7, &val_9)), 1);

        // Subtraction.
        assert!(eq(&val_9, &sub(&val_2, &val_minus_7)), 1);

        // Multiplication.
        let val_63 = from_u64<Fr>(63);
        assert!(eq(&val_63, &mul(&val_7, &val_9)), 1);

        // division.
        let val_0 = from_u64<Fr>(0);
        assert!(eq(&val_7, &std::option::extract(&mut div(&val_63, &val_9))), 1);
        assert!(std::option::is_none(&div(&val_63, &val_0)), 1);

        // Inversion.
        assert!(eq(&val_minus_7, &neg(&val_7)), 1);
        assert!(std::option::is_none(&inv(&val_0)), 1);

        // Squaring.
        let val_x = rand_insecure<Fr>();
        assert!(eq(&mul(&val_x, &val_x), &sqr(&val_x)), 1);
    }

    #[test_only]
    const Q_SERIALIZED: vector<u8> = x"47fd7cd8168c203c8dca7168916a81975d588181b64550b829a031e1724e6430";
    #[test_only]
    const FQ_VAL_0_SERIALIZED_LSB: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const FQ_VAL_1_SERIALIZED_LSB: vector<u8> = x"0100000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const FQ_VAL_7_SERIALIZED_LSB: vector<u8> = x"0700000000000000000000000000000000000000000000000000000000000000";
    #[test_only]
    const FQ_VAL_7_SERIALIZED_MSB: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000007";
    #[test_only]
    const FQ_VAL_7_NEG_SERIALIZED_LSB: vector<u8> = x"40fd7cd8168c203c8dca7168916a81975d588181b64550b829a031e1724e6430";

    #[test(fx = @std)]
    fun test_fq(fx: signer) {
        enable_cryptography_algebra_natives(&fx);

        // Constants.
        assert!(Q_SERIALIZED == order<Fq>(), 1);

        // Serialization/deserialization.
        let val_0 = zero<Fq>();
        let val_1 = one<Fq>();
        assert!(FQ_VAL_0_SERIALIZED_LSB == serialize<Fq, FormatFqLsb>(&val_0), 1);
        assert!(FQ_VAL_1_SERIALIZED_LSB == serialize<Fq, FormatFqLsb>(&val_1), 1);
        let val_7 = from_u64<Fq>(7);
        let val_7_2nd = std::option::extract(&mut deserialize<Fq, FormatFqLsb>(&FQ_VAL_7_SERIALIZED_LSB));
        let val_7_3rd = std::option::extract(&mut deserialize<Fq, FormatFqMsb>(&FQ_VAL_7_SERIALIZED_MSB));
        assert!(eq(&val_7, &val_7_2nd), 1);
        assert!(eq(&val_7, &val_7_3rd), 1);
        assert!(FQ_VAL_7_SERIALIZED_LSB == serialize<Fq, FormatFqLsb>(&val_7), 1);
        assert!(FQ_VAL_7_SERIALIZED_MSB == serialize<Fq, FormatFqMsb>(&val_7), 1);

        // Deserialization should fail if given a byte array of right size but the value is not a member.
        assert!(std::option::is_none(&deserialize<Fq, FormatFqLsb>(&x"47fd7cd8168c203c8dca7168916a81975d588181b64550b829a031e1724e6430")), 1);
        assert!(std::option::is_none(&deserialize<Fq, FormatFqMsb>(&x"30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47")), 1);

        // Deserialization should fail if given a byte array of wrong size.
        assert!(std::option::is_none(&deserialize<Fq, FormatFqLsb>(&x"46fd7cd8168c203c8dca7168916a81975d588181b64550b829a031e1724e643000")), 1);
        assert!(std::option::is_none(&deserialize<Fq, FormatFqMsb>(&x"30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd4600")), 1);
        assert!(std::option::is_none(&deserialize<Fq, FormatFqLsb>(&x"ffff")), 1);
        assert!(std::option::is_none(&deserialize<Fq, FormatFqMsb>(&x"ffff")), 1);

        // Negation.
        let val_minus_7 = neg(&val_7);
        assert!(FQ_VAL_7_NEG_SERIALIZED_LSB == serialize<Fq, FormatFqLsb>(&val_minus_7), 1);

        // Addition.
        let val_9 = from_u64<Fq>(9);
        let val_2 = from_u64<Fq>(2);
        assert!(eq(&val_2, &add(&val_minus_7, &val_9)), 1);

        // Subtraction.
        assert!(eq(&val_9, &sub(&val_2, &val_minus_7)), 1);

        // Multiplication.
        let val_63 = from_u64<Fq>(63);
        assert!(eq(&val_63, &mul(&val_7, &val_9)), 1);

        // division.
        let val_0 = from_u64<Fq>(0);
        assert!(eq(&val_7, &std::option::extract(&mut div(&val_63, &val_9))), 1);
        assert!(std::option::is_none(&div(&val_63, &val_0)), 1);

        // Inversion.
        assert!(eq(&val_minus_7, &neg(&val_7)), 1);
        assert!(std::option::is_none(&inv(&val_0)), 1);

        // Squaring.
        let val_x = rand_insecure<Fq>();
        assert!(eq(&mul(&val_x, &val_x), &sqr(&val_x)), 1);
    }

    #[test(fx = @std)]
    fun test_pairing(fx: signer) {
        enable_cryptography_algebra_natives(&fx);

        // pairing(a*P,b*Q) == (a*b)*pairing(P,Q)
        let element_p = rand_insecure<G1>();
        let element_q = rand_insecure<G2>();
        let a = rand_insecure<Fr>();
        let b = rand_insecure<Fr>();
        let gt_element = pairing<G1, G2,Gt>(&scalar_mul(&element_p, &a), &scalar_mul(&element_q, &b));
        let gt_element_another = scalar_mul(&pairing<G1, G2,Gt>(&element_p, &element_q), &mul(&a, &b));
        assert!(eq(&gt_element, &gt_element_another), 1);
    }

    #[test(fx = @std)]
    fun test_multi_pairing(fx: signer) {
        enable_cryptography_algebra_natives(&fx);

        // Will compute e(a0*P0,b0*Q0)+e(a1*P1,b1*Q1)+e(a2*P2,b2*Q2).
        let a0 = rand_insecure<Fr>();
        let a1 = rand_insecure<Fr>();
        let a2 = rand_insecure<Fr>();
        let element_p0 = rand_insecure<G1>();
        let element_p1 = rand_insecure<G1>();
        let element_p2 = rand_insecure<G1>();
        let p0_a0 = scalar_mul(&element_p0, &a0);
        let p1_a1 = scalar_mul(&element_p1, &a1);
        let p2_a2 = scalar_mul(&element_p2, &a2);
        let b0 = rand_insecure<Fr>();
        let b1 = rand_insecure<Fr>();
        let b2 = rand_insecure<Fr>();
        let element_q0 = rand_insecure<G2>();
        let element_q1 = rand_insecure<G2>();
        let element_q2 = rand_insecure<G2>();
        let q0_b0 = scalar_mul(&element_q0, &b0);
        let q1_b1 = scalar_mul(&element_q1, &b1);
        let q2_b2 = scalar_mul(&element_q2, &b2);

        // Naive method.
        let n0 = pairing<G1, G2,Gt>(&p0_a0, &q0_b0);
        let n1 = pairing<G1, G2,Gt>(&p1_a1, &q1_b1);
        let n2 = pairing<G1, G2,Gt>(&p2_a2, &q2_b2);
        let n = zero<Gt>();
        n = add(&n, &n0);
        n = add(&n, &n1);
        n = add(&n, &n2);

        // Efficient API.
        let m = multi_pairing<G1, G2, Gt>(&vector[p0_a0, p1_a1, p2_a2], &vector[q0_b0, q1_b1, q2_b2]);
        assert!(eq(&n, &m), 1);
    }

    #[test(fx = @std)]
    #[expected_failure(abort_code = 0x010002, location = aptos_std::crypto_algebra)]
    fun test_multi_pairing_should_abort_when_sizes_mismatch(fx: signer) {
        enable_cryptography_algebra_natives(&fx);
        let g1_elements = vector[rand_insecure<G1>()];
        let g2_elements = vector[rand_insecure<G2>(), rand_insecure<G2>()];
        multi_pairing<G1, G2, Gt>(&g1_elements, &g2_elements);
    }

    #[test(fx = @std)]
    #[expected_failure(abort_code = 0x010002, location = aptos_std::crypto_algebra)]
    fun test_multi_scalar_mul_should_abort_when_sizes_mismatch(fx: signer) {
        enable_cryptography_algebra_natives(&fx);
        let elements = vector[rand_insecure<G1>()];
        let scalars = vector[rand_insecure<Fr>(), rand_insecure<Fr>()];
        multi_scalar_mul(&elements, &scalars);
    }

    #[test_only]
    /// The maximum number of `G1` elements that can be created in a transaction,
    /// calculated by the current memory limit (1MB) and the in-mem G1 representation size (96 bytes per element).
    const G1_NUM_MAX: u64 = 1048576 / 96;

    #[test(fx = @std)]
    fun test_memory_limit(fx: signer) {
        enable_cryptography_algebra_natives(&fx);
        let remaining = G1_NUM_MAX;
        while (remaining > 0) {
            zero<G1>();
            remaining = remaining - 1;
        }
    }

    #[test(fx = @std)]
    #[expected_failure(abort_code = 0x090003, location = std::crypto_algebra)]
    fun test_memory_limit_exceeded_with_g1(fx: signer) {
        enable_cryptography_algebra_natives(&fx);
        let remaining = G1_NUM_MAX + 1;
        while (remaining > 0) {
            zero<G1>();
            remaining = remaining - 1;
        }
    }

    //
    // (Tests end here.)
    //

}
