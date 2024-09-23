package groth16

import (
	"bytes"

	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
)

// solidityTemplate
// this is an experimental feature and gnark solidity generator as not been thoroughly tested
const solidityTemplate = `
{{- $numPublic := sub (len .Vk.G1.K) 1 }}
{{- $numCommitments := len .Vk.PublicAndCommitmentCommitted }}
{{- $numWitness := sub $numPublic $numCommitments }}
{{- $PublicAndCommitmentCommitted := .Vk.PublicAndCommitmentCommitted }}
// SPDX-License-Identifier: MIT

pragma solidity {{ .Cfg.PragmaVersion }};

/// @title Groth16 verifier template.
/// @author Remco Bloemen
/// @notice Supports verifying Groth16 proofs. Proofs can be in uncompressed
/// (256 bytes) and compressed (128 bytes) format. A view function is provided
/// to compress proofs.
/// @notice See <https://2π.com/23/bn254-compression> for further explanation.
contract Verifier {

    /// Some of the provided public input values are larger than the field modulus.
    /// @dev Public input elements are not automatically reduced, as this is can be
    /// a dangerous source of bugs.
    error PublicInputNotInField();

    /// The proof is invalid.
    /// @dev This can mean that provided Groth16 proof points are not on their
    /// curves, that pairing equation fails, or that the proof is not for the
    /// provided public input.
    error ProofInvalid();

    {{- if gt $numCommitments 0 }}
    /// The commitment is invalid
    /// @dev This can mean that provided commitment points and/or proof of knowledge are not on their
    /// curves, that pairing equation fails, or that the commitment and/or proof of knowledge is not for the
    /// commitment key.
    error CommitmentInvalid();
    {{- end }}

    // Addresses of precompiles
    uint256 constant PRECOMPILE_MODEXP = 0x05;
    uint256 constant PRECOMPILE_ADD = 0x06;
    uint256 constant PRECOMPILE_MUL = 0x07;
    uint256 constant PRECOMPILE_VERIFY = 0x08;

    // Base field Fp order P and scalar field Fr order R.
    // For BN254 these are computed as follows:
    //     t = 4965661367192848881
    //     P = 36⋅t⁴ + 36⋅t³ + 24⋅t² + 6⋅t + 1
    //     R = 36⋅t⁴ + 36⋅t³ + 18⋅t² + 6⋅t + 1
    uint256 constant P = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47;
    uint256 constant R = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001;

    // Extension field Fp2 = Fp[i] / (i² + 1)
    // Note: This is the complex extension field of Fp with i² = -1.
    //       Values in Fp2 are represented as a pair of Fp elements (a₀, a₁) as a₀ + a₁⋅i.
    // Note: The order of Fp2 elements is *opposite* that of the pairing contract, which
    //       expects Fp2 elements in order (a₁, a₀). This is also the order in which
    //       Fp2 elements are encoded in the public interface as this became convention.

    // Constants in Fp
    uint256 constant FRACTION_1_2_FP = 0x183227397098d014dc2822db40c0ac2ecbc0b548b438e5469e10460b6c3e7ea4;
    uint256 constant FRACTION_27_82_FP = 0x2b149d40ceb8aaae81be18991be06ac3b5b4c5e559dbefa33267e6dc24a138e5;
    uint256 constant FRACTION_3_82_FP = 0x2fcd3ac2a640a154eb23960892a85a68f031ca0c8344b23a577dcf1052b9e775;

    // Exponents for inversions and square roots mod P
    uint256 constant EXP_INVERSE_FP = 0x30644E72E131A029B85045B68181585D97816A916871CA8D3C208C16D87CFD45; // P - 2
    uint256 constant EXP_SQRT_FP = 0xC19139CB84C680A6E14116DA060561765E05AA45A1C72A34F082305B61F3F52; // (P + 1) / 4;

    // Groth16 alpha point in G1
    uint256 constant ALPHA_X = {{ (fpstr .Vk.G1.Alpha.X) }};
    uint256 constant ALPHA_Y = {{ (fpstr .Vk.G1.Alpha.Y) }};

    // Groth16 beta point in G2 in powers of i
    uint256 constant BETA_NEG_X_0 = {{ (fpstr .Vk.G2.Beta.X.A0) }};
    uint256 constant BETA_NEG_X_1 = {{ (fpstr .Vk.G2.Beta.X.A1) }};
    uint256 constant BETA_NEG_Y_0 = {{ (fpstr .Vk.G2.Beta.Y.A0) }};
    uint256 constant BETA_NEG_Y_1 = {{ (fpstr .Vk.G2.Beta.Y.A1) }};

    // Groth16 gamma point in G2 in powers of i
    uint256 constant GAMMA_NEG_X_0 = {{ (fpstr .Vk.G2.Gamma.X.A0) }};
    uint256 constant GAMMA_NEG_X_1 = {{ (fpstr .Vk.G2.Gamma.X.A1) }};
    uint256 constant GAMMA_NEG_Y_0 = {{ (fpstr .Vk.G2.Gamma.Y.A0) }};
    uint256 constant GAMMA_NEG_Y_1 = {{ (fpstr .Vk.G2.Gamma.Y.A1) }};

    // Groth16 delta point in G2 in powers of i
    uint256 constant DELTA_NEG_X_0 = {{ (fpstr .Vk.G2.Delta.X.A0) }};
    uint256 constant DELTA_NEG_X_1 = {{ (fpstr .Vk.G2.Delta.X.A1) }};
    uint256 constant DELTA_NEG_Y_0 = {{ (fpstr .Vk.G2.Delta.Y.A0) }};
    uint256 constant DELTA_NEG_Y_1 = {{ (fpstr .Vk.G2.Delta.Y.A1) }};

    {{- if gt $numCommitments 0 }}
    // Pedersen G point in G2 in powers of i
    uint256 constant PEDERSEN_G_X_0 = {{ (fpstr .Vk.CommitmentKey.G.X.A0) }};
    uint256 constant PEDERSEN_G_X_1 = {{ (fpstr .Vk.CommitmentKey.G.X.A1) }};
    uint256 constant PEDERSEN_G_Y_0 = {{ (fpstr .Vk.CommitmentKey.G.Y.A0) }};
    uint256 constant PEDERSEN_G_Y_1 = {{ (fpstr .Vk.CommitmentKey.G.Y.A1) }};

    // Pedersen GRootSigmaNeg point in G2 in powers of i
    uint256 constant PEDERSEN_GROOTSIGMANEG_X_0 = {{ (fpstr .Vk.CommitmentKey.GRootSigmaNeg.X.A0) }};
    uint256 constant PEDERSEN_GROOTSIGMANEG_X_1 = {{ (fpstr .Vk.CommitmentKey.GRootSigmaNeg.X.A1) }};
    uint256 constant PEDERSEN_GROOTSIGMANEG_Y_0 = {{ (fpstr .Vk.CommitmentKey.GRootSigmaNeg.Y.A0) }};
    uint256 constant PEDERSEN_GROOTSIGMANEG_Y_1 = {{ (fpstr .Vk.CommitmentKey.GRootSigmaNeg.Y.A1) }};
    {{- end }}

    // Constant and public input points
    {{- $k0 := index .Vk.G1.K 0}}
    uint256 constant CONSTANT_X = {{ (fpstr $k0.X) }};
    uint256 constant CONSTANT_Y = {{ (fpstr $k0.Y) }};
    {{- range $i, $ki := .Vk.G1.K }}
        {{- if gt $i 0 }}
    uint256 constant PUB_{{sub $i 1}}_X = {{ (fpstr $ki.X) }};
    uint256 constant PUB_{{sub $i 1}}_Y = {{ (fpstr $ki.Y) }};
        {{- end }}
    {{- end }}

    /// Negation in Fp.
    /// @notice Returns a number x such that a + x = 0 in Fp.
    /// @notice The input does not need to be reduced.
    /// @param a the base
    /// @return x the result
    function negate(uint256 a) internal pure returns (uint256 x) {
        unchecked {
            x = (P - (a % P)) % P; // Modulo is cheaper than branching
        }
    }

    /// Exponentiation in Fp.
    /// @notice Returns a number x such that a ^ e = x in Fp.
    /// @notice The input does not need to be reduced.
    /// @param a the base
    /// @param e the exponent
    /// @return x the result
    function exp(uint256 a, uint256 e) internal view returns (uint256 x) {
        bool success;
        assembly ("memory-safe") {
            let f := mload(0x40)
            mstore(f, 0x20)
            mstore(add(f, 0x20), 0x20)
            mstore(add(f, 0x40), 0x20)
            mstore(add(f, 0x60), a)
            mstore(add(f, 0x80), e)
            mstore(add(f, 0xa0), P)
            success := staticcall(gas(), PRECOMPILE_MODEXP, f, 0xc0, f, 0x20)
            x := mload(f)
        }
        if (!success) {
            // Exponentiation failed.
            // Should not happen.
            revert ProofInvalid();
        }
    }

    /// Invertsion in Fp.
    /// @notice Returns a number x such that a * x = 1 in Fp.
    /// @notice The input does not need to be reduced.
    /// @notice Reverts with ProofInvalid() if the inverse does not exist
    /// @param a the input
    /// @return x the solution
    function invert_Fp(uint256 a) internal view returns (uint256 x) {
        x = exp(a, EXP_INVERSE_FP);
        if (mulmod(a, x, P) != 1) {
            // Inverse does not exist.
            // Can only happen during G2 point decompression.
            revert ProofInvalid();
        }
    }

    /// Square root in Fp.
    /// @notice Returns a number x such that x * x = a in Fp.
    /// @notice Will revert with InvalidProof() if the input is not a square
    /// or not reduced.
    /// @param a the square
    /// @return x the solution
    function sqrt_Fp(uint256 a) internal view returns (uint256 x) {
        x = exp(a, EXP_SQRT_FP);
        if (mulmod(x, x, P) != a) {
            // Square root does not exist or a is not reduced.
            // Happens when G1 point is not on curve.
            revert ProofInvalid();
        }
    }

    /// Square test in Fp.
    /// @notice Returns wheter a number x exists such that x * x = a in Fp.
    /// @notice Will revert with InvalidProof() if the input is not a square
    /// or not reduced.
    /// @param a the square
    /// @return x the solution
    function isSquare_Fp(uint256 a) internal view returns (bool) {
        uint256 x = exp(a, EXP_SQRT_FP);
        return mulmod(x, x, P) == a;
    }

    /// Square root in Fp2.
    /// @notice Fp2 is the complex extension Fp[i]/(i^2 + 1). The input is
    /// a0 + a1 ⋅ i and the result is x0 + x1 ⋅ i.
    /// @notice Will revert with InvalidProof() if
    ///   * the input is not a square,
    ///   * the hint is incorrect, or
    ///   * the input coefficents are not reduced.
    /// @param a0 The real part of the input.
    /// @param a1 The imaginary part of the input.
    /// @param hint A hint which of two possible signs to pick in the equation.
    /// @return x0 The real part of the square root.
    /// @return x1 The imaginary part of the square root.
    function sqrt_Fp2(uint256 a0, uint256 a1, bool hint) internal view returns (uint256 x0, uint256 x1) {
        // If this square root reverts there is no solution in Fp2.
        uint256 d = sqrt_Fp(addmod(mulmod(a0, a0, P), mulmod(a1, a1, P), P));
        if (hint) {
            d = negate(d);
        }
        // If this square root reverts there is no solution in Fp2.
        x0 = sqrt_Fp(mulmod(addmod(a0, d, P), FRACTION_1_2_FP, P));
        x1 = mulmod(a1, invert_Fp(mulmod(x0, 2, P)), P);

        // Check result to make sure we found a root.
        // Note: this also fails if a0 or a1 is not reduced.
        if (a0 != addmod(mulmod(x0, x0, P), negate(mulmod(x1, x1, P)), P)
        ||  a1 != mulmod(2, mulmod(x0, x1, P), P)) {
            revert ProofInvalid();
        }
    }

    /// Compress a G1 point.
    /// @notice Reverts with InvalidProof if the coordinates are not reduced
    /// or if the point is not on the curve.
    /// @notice The point at infinity is encoded as (0,0) and compressed to 0.
    /// @param x The X coordinate in Fp.
    /// @param y The Y coordinate in Fp.
    /// @return c The compresed point (x with one signal bit).
    function compress_g1(uint256 x, uint256 y) internal view returns (uint256 c) {
        if (x >= P || y >= P) {
            // G1 point not in field.
            revert ProofInvalid();
        }
        if (x == 0 && y == 0) {
            // Point at infinity
            return 0;
        }

        // Note: sqrt_Fp reverts if there is no solution, i.e. the x coordinate is invalid.
        uint256 y_pos = sqrt_Fp(addmod(mulmod(mulmod(x, x, P), x, P), 3, P));
        if (y == y_pos) {
            return (x << 1) | 0;
        } else if (y == negate(y_pos)) {
            return (x << 1) | 1;
        } else {
            // G1 point not on curve.
            revert ProofInvalid();
        }
    }

    /// Decompress a G1 point.
    /// @notice Reverts with InvalidProof if the input does not represent a valid point.
    /// @notice The point at infinity is encoded as (0,0) and compressed to 0.
    /// @param c The compresed point (x with one signal bit).
    /// @return x The X coordinate in Fp.
    /// @return y The Y coordinate in Fp.
    function decompress_g1(uint256 c) internal view returns (uint256 x, uint256 y) {
        // Note that X = 0 is not on the curve since 0³ + 3 = 3 is not a square.
        // so we can use it to represent the point at infinity.
        if (c == 0) {
            // Point at infinity as encoded in EIP196 and EIP197.
            return (0, 0);
        }
        bool negate_point = c & 1 == 1;
        x = c >> 1;
        if (x >= P) {
            // G1 x coordinate not in field.
            revert ProofInvalid();
        }

        // Note: (x³ + 3) is irreducible in Fp, so it can not be zero and therefore
        //       y can not be zero.
        // Note: sqrt_Fp reverts if there is no solution, i.e. the point is not on the curve.
        y = sqrt_Fp(addmod(mulmod(mulmod(x, x, P), x, P), 3, P));
        if (negate_point) {
            y = negate(y);
        }
    }

    /// Compress a G2 point.
    /// @notice Reverts with InvalidProof if the coefficients are not reduced
    /// or if the point is not on the curve.
    /// @notice The G2 curve is defined over the complex extension Fp[i]/(i^2 + 1)
    /// with coordinates (x0 + x1 ⋅ i, y0 + y1 ⋅ i).
    /// @notice The point at infinity is encoded as (0,0,0,0) and compressed to (0,0).
    /// @param x0 The real part of the X coordinate.
    /// @param x1 The imaginary poart of the X coordinate.
    /// @param y0 The real part of the Y coordinate.
    /// @param y1 The imaginary part of the Y coordinate.
    /// @return c0 The first half of the compresed point (x0 with two signal bits).
    /// @return c1 The second half of the compressed point (x1 unmodified).
    function compress_g2(uint256 x0, uint256 x1, uint256 y0, uint256 y1)
    internal view returns (uint256 c0, uint256 c1) {
        if (x0 >= P || x1 >= P || y0 >= P || y1 >= P) {
            // G2 point not in field.
            revert ProofInvalid();
        }
        if ((x0 | x1 | y0 | y1) == 0) {
            // Point at infinity
            return (0, 0);
        }

        // Compute y^2
        // Note: shadowing variables and scoping to avoid stack-to-deep.
        uint256 y0_pos;
        uint256 y1_pos;
        {
            uint256 n3ab = mulmod(mulmod(x0, x1, P), P-3, P);
            uint256 a_3 = mulmod(mulmod(x0, x0, P), x0, P);
            uint256 b_3 = mulmod(mulmod(x1, x1, P), x1, P);
            y0_pos = addmod(FRACTION_27_82_FP, addmod(a_3, mulmod(n3ab, x1, P), P), P);
            y1_pos = negate(addmod(FRACTION_3_82_FP,  addmod(b_3, mulmod(n3ab, x0, P), P), P));
        }

        // Determine hint bit
        // If this sqrt fails the x coordinate is not on the curve.
        bool hint;
        {
            uint256 d = sqrt_Fp(addmod(mulmod(y0_pos, y0_pos, P), mulmod(y1_pos, y1_pos, P), P));
            hint = !isSquare_Fp(mulmod(addmod(y0_pos, d, P), FRACTION_1_2_FP, P));
        }

        // Recover y
        (y0_pos, y1_pos) = sqrt_Fp2(y0_pos, y1_pos, hint);
        if (y0 == y0_pos && y1 == y1_pos) {
            c0 = (x0 << 2) | (hint ? 2  : 0) | 0;
            c1 = x1;
        } else if (y0 == negate(y0_pos) && y1 == negate(y1_pos)) {
            c0 = (x0 << 2) | (hint ? 2  : 0) | 1;
            c1 = x1;
        } else {
            // G1 point not on curve.
            revert ProofInvalid();
        }
    }

    /// Decompress a G2 point.
    /// @notice Reverts with InvalidProof if the input does not represent a valid point.
    /// @notice The G2 curve is defined over the complex extension Fp[i]/(i^2 + 1)
    /// with coordinates (x0 + x1 ⋅ i, y0 + y1 ⋅ i).
    /// @notice The point at infinity is encoded as (0,0,0,0) and compressed to (0,0).
    /// @param c0 The first half of the compresed point (x0 with two signal bits).
    /// @param c1 The second half of the compressed point (x1 unmodified).
    /// @return x0 The real part of the X coordinate.
    /// @return x1 The imaginary poart of the X coordinate.
    /// @return y0 The real part of the Y coordinate.
    /// @return y1 The imaginary part of the Y coordinate.
    function decompress_g2(uint256 c0, uint256 c1)
    internal view returns (uint256 x0, uint256 x1, uint256 y0, uint256 y1) {
        // Note that X = (0, 0) is not on the curve since 0³ + 3/(9 + i) is not a square.
        // so we can use it to represent the point at infinity.
        if (c0 == 0 && c1 == 0) {
            // Point at infinity as encoded in EIP197.
            return (0, 0, 0, 0);
        }
        bool negate_point = c0 & 1 == 1;
        bool hint = c0 & 2 == 2;
        x0 = c0 >> 2;
        x1 = c1;
        if (x0 >= P || x1 >= P) {
            // G2 x0 or x1 coefficient not in field.
            revert ProofInvalid();
        }

        uint256 n3ab = mulmod(mulmod(x0, x1, P), P-3, P);
        uint256 a_3 = mulmod(mulmod(x0, x0, P), x0, P);
        uint256 b_3 = mulmod(mulmod(x1, x1, P), x1, P);

        y0 = addmod(FRACTION_27_82_FP, addmod(a_3, mulmod(n3ab, x1, P), P), P);
        y1 = negate(addmod(FRACTION_3_82_FP,  addmod(b_3, mulmod(n3ab, x0, P), P), P));

        // Note: sqrt_Fp2 reverts if there is no solution, i.e. the point is not on the curve.
        // Note: (X³ + 3/(9 + i)) is irreducible in Fp2, so y can not be zero.
        //       But y0 or y1 may still independently be zero.
        (y0, y1) = sqrt_Fp2(y0, y1, hint);
        if (negate_point) {
            y0 = negate(y0);
            y1 = negate(y1);
        }
    }

    /// Compute the public input linear combination.
    /// @notice Reverts with PublicInputNotInField if the input is not in the field.
    /// @notice Computes the multi-scalar-multiplication of the public input
    /// elements and the verification key including the constant term.
    /// @param input The public inputs. These are elements of the scalar field Fr.
    {{- if gt $numCommitments 0 }}
    /// @param publicCommitments public inputs generated from pedersen commitments.
    /// @param commitments The Pedersen commitments from the proof.
    {{- end }}
    /// @return x The X coordinate of the resulting G1 point.
    /// @return y The Y coordinate of the resulting G1 point.
    {{- if eq $numCommitments 0 }}
    function publicInputMSM(uint256[{{$numWitness}}] calldata input)
    {{- else }}
    function publicInputMSM(
        uint256[{{$numWitness}}] calldata input,
        uint256[{{$numCommitments}}] memory publicCommitments,
        uint256[{{mul 2 $numCommitments}}] memory commitments
    )
    {{- end }}
    internal view returns (uint256 x, uint256 y) {
        // Note: The ECMUL precompile does not reject unreduced values, so we check this.
        // Note: Unrolling this loop does not cost much extra in code-size, the bulk of the
        //       code-size is in the PUB_ constants.
        // ECMUL has input (x, y, scalar) and output (x', y').
        // ECADD has input (x1, y1, x2, y2) and output (x', y').
        // We reduce commitments(if any) with constants as the first point argument to ECADD.
        // We call them such that ecmul output is already in the second point
        // argument to ECADD so we can have a tight loop.
        bool success = true;
        assembly ("memory-safe") {
            let f := mload(0x40)
            let g := add(f, 0x40)
            let s
            mstore(f, CONSTANT_X)
            mstore(add(f, 0x20), CONSTANT_Y)
            {{- if gt $numCommitments 0 }}
            {{- if eq $numWitness 1 }}
            mstore(g, mload(commitments))
            mstore(add(g, 0x20), mload(add(commitments, 0x20)))
            {{- else }}
            success := and(success,  staticcall(gas(), PRECOMPILE_ADD, commitments, {{mul 0x40 $numCommitments}}, g, 0x40))
            {{- end }}
            success := and(success,  staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40))
            {{- end }}
            {{- range $i := intRange $numPublic }}
            mstore(g, PUB_{{$i}}_X)
            mstore(add(g, 0x20), PUB_{{$i}}_Y)
            {{- if eq $i 0 }}
            s :=  calldataload(input)
            {{- else if lt $i $numWitness }}
            s :=  calldataload(add(input, {{mul $i 0x20}}))
            {{- else if eq $i $numWitness }}
            s := mload(publicCommitments)
            {{- else}}
            s := mload(add(publicCommitments, {{mul 0x20 (sub $i $numWitness)}}))
            {{- end }}
            mstore(add(g, 0x40), s)
            success := and(success, lt(s, R))
            success := and(success, staticcall(gas(), PRECOMPILE_MUL, g, 0x60, g, 0x40))
            success := and(success, staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40))
            {{- end }}

            x := mload(f)
            y := mload(add(f, 0x20))
        }
        if (!success) {
            // Either Public input not in field, or verification key invalid.
            // We assume the contract is correctly generated, so the verification key is valid.
            revert PublicInputNotInField();
        }
    }

    /// Compress a proof.
    /// @notice Will revert with InvalidProof if the curve points are invalid,
    /// but does not verify the proof itself.
    /// @param proof The uncompressed Groth16 proof. Elements are in the same order as for
    /// verifyProof. I.e. Groth16 points (A, B, C) encoded as in EIP-197.
    {{- if gt $numCommitments 0 }}
    /// @param commitments Pedersen commitments from the proof.
    /// @param commitmentPok proof of knowledge for the Pedersen commitments.
    {{- end }}
    /// @return compressed The compressed proof. Elements are in the same order as for
    /// verifyCompressedProof. I.e. points (A, B, C) in compressed format.
    {{- if gt $numCommitments 0 }}
    /// @return compressedCommitments compressed Pedersen commitments from the proof.
    /// @return compressedCommitmentPok compressed proof of knowledge for the Pedersen commitments.
    {{- end }}
    {{- if eq $numCommitments 0 }}
    function compressProof(uint256[8] calldata proof)
    public view returns (uint256[4] memory compressed) {
    {{- else }}
    function compressProof(
        uint256[8] calldata proof,
        uint256[{{mul 2 $numCommitments}}] calldata commitments,
        uint256[2] calldata commitmentPok
    )
    public view returns (
        uint256[4] memory compressed,
        uint256[{{$numCommitments}}] memory compressedCommitments,
        uint256 compressedCommitmentPok
    ) {
    {{- end }}
        compressed[0] = compress_g1(proof[0], proof[1]);
        (compressed[2], compressed[1]) = compress_g2(proof[3], proof[2], proof[5], proof[4]);
        compressed[3] = compress_g1(proof[6], proof[7]);
        {{- if gt $numCommitments 0 }}
        {{- range $i := intRange $numCommitments }}
        compressedCommitments[{{$i}}] = compress_g1(commitments[{{mul 2 $i}}], commitments[{{sum (mul 2 $i) 1}}]);
        {{- end }}
        compressedCommitmentPok = compress_g1(commitmentPok[0], commitmentPok[1]);
        {{- end }}
    }

    /// Verify a Groth16 proof with compressed points.
    /// @notice Reverts with InvalidProof if the proof is invalid or
    /// with PublicInputNotInField the public input is not reduced.
    /// @notice There is no return value. If the function does not revert, the
    /// proof was successfully verified.
    /// @param compressedProof the points (A, B, C) in compressed format
    /// matching the output of compressProof.
    {{- if gt $numCommitments 0 }}
    /// @param compressedCommitments compressed Pedersen commitments from the proof.
    /// @param compressedCommitmentPok compressed proof of knowledge for the Pedersen commitments.
    {{- end }}
    /// @param input the public input field elements in the scalar field Fr.
    /// Elements must be reduced.
    function verifyCompressedProof(
        uint256[4] calldata compressedProof,
        {{- if gt $numCommitments 0}}
        uint256[{{$numCommitments}}] calldata compressedCommitments,
        uint256 compressedCommitmentPok,
        {{- end }}
        uint256[{{$numWitness}}] calldata input
    ) public view {
        {{- if gt $numCommitments 0 }}
        uint256[{{$numCommitments}}] memory publicCommitments;
        uint256[{{mul 2 $numCommitments}}] memory commitments;
        {{- end }}
        uint256[24] memory pairings;

        {{- if gt $numCommitments 0 }}
        {
            {{- if eq $numCommitments 1 }}
            (commitments[0], commitments[1]) = decompress_g1(compressedCommitments[0]);
            {{- else }}
            // TODO: We can fold commitments into a single point for more efficient verification (https://github.com/Consensys/gnark/issues/1095)
            for (uint256 i = 0; i < {{$numCommitments}}; i++) {
                (commitments[2*i], commitments[2*i+1]) = decompress_g1(compressedCommitments[i]);
            }
            {{- end}}
            (uint256 Px, uint256 Py) = decompress_g1(compressedCommitmentPok);

            uint256[] memory publicAndCommitmentCommitted;
            {{- range $i := intRange $numCommitments }}
            {{- $pcIndex := index $PublicAndCommitmentCommitted $i }}
            {{- if gt (len $pcIndex) 0 }}
            publicAndCommitmentCommitted = new uint256[]({{(len $pcIndex)}});
            assembly ("memory-safe") {
                let publicAndCommitmentCommittedOffset := add(publicAndCommitmentCommitted, 0x20)
                {{- $segment_start := index $pcIndex 0 }}
                {{- $segment_end := index $pcIndex 0 }}
                {{- $l := 0 }}
                {{- range $k := intRange (sub (len $pcIndex) 1) }}
                    {{- $next := index $pcIndex (sum $k 1) }}
                    {{- if ne $next (sum $segment_end 1) }}
                calldatacopy(add(publicAndCommitmentCommittedOffset, {{mul $l 0x20}}), add(input, {{mul 0x20 (sub $segment_start 1)}}), {{mul 0x20 (sum 1 (sub $segment_end $segment_start))}})
                        {{- $segment_start = $next }}
                        {{- $l = (sum $k 1) }}
                    {{- end }}
                    {{- $segment_end = $next }}
                {{- end }}
                calldatacopy(add(publicAndCommitmentCommittedOffset, {{mul $l 0x20}}), add(input, {{mul 0x20 (sub $segment_start 1)}}), {{mul 0x20 (sum 1 (sub $segment_end $segment_start))}})
            }
            {{- end }}

            publicCommitments[{{$i}}] = uint256(
                sha256(
                    abi.encodePacked(
                        commitments[{{mul $i 2}}],
                        commitments[{{sum (mul $i 2) 1}}],
                        publicAndCommitmentCommitted
                    )
                )
            ) % R;
            {{- end }}
            // Commitments
            pairings[ 0] = commitments[0];
            pairings[ 1] = commitments[1];
            pairings[ 2] = PEDERSEN_G_X_1;
            pairings[ 3] = PEDERSEN_G_X_0;
            pairings[ 4] = PEDERSEN_G_Y_1;
            pairings[ 5] = PEDERSEN_G_Y_0;
            pairings[ 6] = Px;
            pairings[ 7] = Py;
            pairings[ 8] = PEDERSEN_GROOTSIGMANEG_X_1;
            pairings[ 9] = PEDERSEN_GROOTSIGMANEG_X_0;
            pairings[10] = PEDERSEN_GROOTSIGMANEG_Y_1;
            pairings[11] = PEDERSEN_GROOTSIGMANEG_Y_0;

            // Verify pedersen commitments
            bool success;
            assembly ("memory-safe") {
                let f := mload(0x40)

                success := staticcall(gas(), PRECOMPILE_VERIFY, pairings, 0x180, f, 0x20)
                success := and(success, mload(f))
            }
            if (!success) {
                revert CommitmentInvalid();
            }
        }
        {{- end }}

        {
            (uint256 Ax, uint256 Ay) = decompress_g1(compressedProof[0]);
            (uint256 Bx0, uint256 Bx1, uint256 By0, uint256 By1) = decompress_g2(compressedProof[2], compressedProof[1]);
            (uint256 Cx, uint256 Cy) = decompress_g1(compressedProof[3]);
            {{- if eq $numCommitments 0 }}
            (uint256 Lx, uint256 Ly) = publicInputMSM(input);
            {{- else }}
            (uint256 Lx, uint256 Ly) = publicInputMSM(
                input,
                publicCommitments,
                commitments
            );
            {{- end}}

            // Verify the pairing
            // Note: The precompile expects the F2 coefficients in big-endian order.
            // Note: The pairing precompile rejects unreduced values, so we won't check that here.
            // e(A, B)
            pairings[ 0] = Ax;
            pairings[ 1] = Ay;
            pairings[ 2] = Bx1;
            pairings[ 3] = Bx0;
            pairings[ 4] = By1;
            pairings[ 5] = By0;
            // e(C, -δ)
            pairings[ 6] = Cx;
            pairings[ 7] = Cy;
            pairings[ 8] = DELTA_NEG_X_1;
            pairings[ 9] = DELTA_NEG_X_0;
            pairings[10] = DELTA_NEG_Y_1;
            pairings[11] = DELTA_NEG_Y_0;
            // e(α, -β)
            pairings[12] = ALPHA_X;
            pairings[13] = ALPHA_Y;
            pairings[14] = BETA_NEG_X_1;
            pairings[15] = BETA_NEG_X_0;
            pairings[16] = BETA_NEG_Y_1;
            pairings[17] = BETA_NEG_Y_0;
            // e(L_pub, -γ)
            pairings[18] = Lx;
            pairings[19] = Ly;
            pairings[20] = GAMMA_NEG_X_1;
            pairings[21] = GAMMA_NEG_X_0;
            pairings[22] = GAMMA_NEG_Y_1;
            pairings[23] = GAMMA_NEG_Y_0;

            // Check pairing equation.
            bool success;
            uint256[1] memory output;
            assembly ("memory-safe") {
                success := staticcall(gas(), PRECOMPILE_VERIFY, pairings, 0x300, output, 0x20)
            }
            if (!success || output[0] != 1) {
                // Either proof or verification key invalid.
                // We assume the contract is correctly generated, so the verification key is valid.
                revert ProofInvalid();
            }
        }
    }

    /// Verify an uncompressed Groth16 proof.
    /// @notice Reverts with InvalidProof if the proof is invalid or
    /// with PublicInputNotInField the public input is not reduced.
    /// @notice There is no return value. If the function does not revert, the
    /// proof was successfully verified.
    /// @param proof the points (A, B, C) in EIP-197 format matching the output
    /// of compressProof.
    {{- if gt $numCommitments 0 }}
    /// @param commitments the Pedersen commitments from the proof.
    /// @param commitmentPok the proof of knowledge for the Pedersen commitments.
    {{- end }}
    /// @param input the public input field elements in the scalar field Fr.
    /// Elements must be reduced.
    function verifyProof(
        uint256[8] calldata proof,
        {{- if gt $numCommitments 0}}
        uint256[{{mul 2 $numCommitments}}] calldata commitments,
        uint256[2] calldata commitmentPok,
        {{- end }}
        uint256[{{$numWitness}}] calldata input
    ) public view {
        {{- if eq $numCommitments 0 }}
        (uint256 x, uint256 y) = publicInputMSM(input);
        {{- else }}
        // HashToField
        uint256[{{$numCommitments}}] memory publicCommitments;
        uint256[] memory publicAndCommitmentCommitted;
        {{- range $i := intRange $numCommitments }}
        {{- $pcIndex := index $PublicAndCommitmentCommitted $i }}
        {{- if gt (len $pcIndex) 0 }}
        publicAndCommitmentCommitted = new uint256[]({{(len $pcIndex)}});
        assembly ("memory-safe") {
            let publicAndCommitmentCommittedOffset := add(publicAndCommitmentCommitted, 0x20)
            {{- $segment_start := index $pcIndex 0 }}
            {{- $segment_end := index $pcIndex 0 }}
            {{- $l := 0 }}
            {{- range $k := intRange (sub (len $pcIndex) 1) }}
                {{- $next := index $pcIndex (sum $k 1) }}
                {{- if ne $next (sum $segment_end 1) }}
            calldatacopy(add(publicAndCommitmentCommittedOffset, {{mul $l 0x20}}), add(input, {{mul 0x20 (sub $segment_start 1)}}), {{mul 0x20 (sum 1 (sub $segment_end $segment_start))}})
                    {{- $segment_start = $next }}
                    {{- $l = (sum $k 1) }}
                {{- end }}
                {{- $segment_end = $next }}
            {{- end }}
            calldatacopy(add(publicAndCommitmentCommittedOffset, {{mul $l 0x20}}), add(input, {{mul 0x20 (sub $segment_start 1)}}), {{mul 0x20 (sum 1 (sub $segment_end $segment_start))}})
        }
        {{- end }}

            publicCommitments[{{$i}}] = uint256(
                sha256(
                    abi.encodePacked(
                        commitments[{{mul $i 2}}],
                        commitments[{{sum (mul $i 2) 1}}],
                        publicAndCommitmentCommitted
                    )
                )
            ) % R;
        {{- end }}

        // Verify pedersen commitments
        bool success;
        assembly ("memory-safe") {
            let f := mload(0x40)

            calldatacopy(f, commitments, 0x40) // Copy Commitments
            mstore(add(f, 0x40), PEDERSEN_G_X_1)
            mstore(add(f, 0x60), PEDERSEN_G_X_0)
            mstore(add(f, 0x80), PEDERSEN_G_Y_1)
            mstore(add(f, 0xa0), PEDERSEN_G_Y_0)
            calldatacopy(add(f, 0xc0), commitmentPok, 0x40)
            mstore(add(f, 0x100), PEDERSEN_GROOTSIGMANEG_X_1)
            mstore(add(f, 0x120), PEDERSEN_GROOTSIGMANEG_X_0)
            mstore(add(f, 0x140), PEDERSEN_GROOTSIGMANEG_Y_1)
            mstore(add(f, 0x160), PEDERSEN_GROOTSIGMANEG_Y_0)

            success := staticcall(gas(), PRECOMPILE_VERIFY, f, 0x180, f, 0x20)
            success := and(success, mload(f))
        }
        if (!success) {
            revert CommitmentInvalid();
        }

        (uint256 x, uint256 y) = publicInputMSM(
            input,
            publicCommitments,
            commitments
        );
        {{- end }}

        // Note: The precompile expects the F2 coefficients in big-endian order.
        // Note: The pairing precompile rejects unreduced values, so we won't check that here.

        {{- if eq $numCommitments 0 }}
        bool success;
        {{- end }}
        assembly ("memory-safe") {
            let f := mload(0x40) // Free memory pointer.

            // Copy points (A, B, C) to memory. They are already in correct encoding.
            // This is pairing e(A, B) and G1 of e(C, -δ).
            calldatacopy(f, proof, 0x100)

            // Complete e(C, -δ) and write e(α, -β), e(L_pub, -γ) to memory.
            // OPT: This could be better done using a single codecopy, but
            //      Solidity (unlike standalone Yul) doesn't provide a way to
            //      to do this.
            mstore(add(f, 0x100), DELTA_NEG_X_1)
            mstore(add(f, 0x120), DELTA_NEG_X_0)
            mstore(add(f, 0x140), DELTA_NEG_Y_1)
            mstore(add(f, 0x160), DELTA_NEG_Y_0)
            mstore(add(f, 0x180), ALPHA_X)
            mstore(add(f, 0x1a0), ALPHA_Y)
            mstore(add(f, 0x1c0), BETA_NEG_X_1)
            mstore(add(f, 0x1e0), BETA_NEG_X_0)
            mstore(add(f, 0x200), BETA_NEG_Y_1)
            mstore(add(f, 0x220), BETA_NEG_Y_0)
            mstore(add(f, 0x240), x)
            mstore(add(f, 0x260), y)
            mstore(add(f, 0x280), GAMMA_NEG_X_1)
            mstore(add(f, 0x2a0), GAMMA_NEG_X_0)
            mstore(add(f, 0x2c0), GAMMA_NEG_Y_1)
            mstore(add(f, 0x2e0), GAMMA_NEG_Y_0)

            // Check pairing equation.
            success := staticcall(gas(), PRECOMPILE_VERIFY, f, 0x300, f, 0x20)
            // Also check returned value (both are either 1 or 0).
            success := and(success, mload(f))
        }
        if (!success) {
            // Either proof or verification key invalid.
            // We assume the contract is correctly generated, so the verification key is valid.
            revert ProofInvalid();
        }
    }
}
`

// MarshalSolidity converts a proof to a byte array that can be used in a
// Solidity contract.
func (proof *Proof) MarshalSolidity() []byte {
	var buf bytes.Buffer
	_, err := proof.WriteRawTo(&buf)
	if err != nil {
		panic(err)
	}

	// If there are no commitments, we can return only Ar | Bs | Krs
	if len(proof.Commitments) > 0 {
		return buf.Bytes()
	} else {
		return buf.Bytes()[:8*fr.Bytes]
	}
}
