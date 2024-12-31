pragma solidity ^0.8.27;

/// @title Groth16 verifier template.
/// @author Remco Bloemen
/// @notice Supports verifying Groth16 proofs. Proofs can be in uncompressed
/// (256 bytes) and compressed (128 bytes) format.
library CometblsZKVerifier {
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
    uint256 constant P =
        0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47;
    uint256 constant R =
        0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001;

    // Extension field Fp2 = Fp[i] / (i² + 1)
    // Note: This is the complex extension field of Fp with i² = -1.
    //       Values in Fp2 are represented as a pair of Fp elements (a₀, a₁) as a₀ + a₁⋅i.
    // Note: The order of Fp2 elements is *opposite* that of the pairing contract, which
    //       expects Fp2 elements in order (a₁, a₀). This is also the order in which
    //       Fp2 elements are encoded in the public interface as this became convention.

    // Constants in Fp
    uint256 constant FRACTION_1_2_FP =
        0x183227397098d014dc2822db40c0ac2ecbc0b548b438e5469e10460b6c3e7ea4;
    uint256 constant FRACTION_27_82_FP =
        0x2b149d40ceb8aaae81be18991be06ac3b5b4c5e559dbefa33267e6dc24a138e5;
    uint256 constant FRACTION_3_82_FP =
        0x2fcd3ac2a640a154eb23960892a85a68f031ca0c8344b23a577dcf1052b9e775;

    // Exponents for inversions and square roots mod P
    uint256 constant EXP_INVERSE_FP =
        0x30644E72E131A029B85045B68181585D97816A916871CA8D3C208C16D87CFD45; // P - 2
    uint256 constant EXP_SQRT_FP =
        0xC19139CB84C680A6E14116DA060561765E05AA45A1C72A34F082305B61F3F52; // (P + 1) / 4;

    // Verifying key
    uint256 constant ALPHA_X =
        0x245229d9b076b3c0e8a4d70bde8c1cccffa08a9fae7557b165b3b0dbd653e2c7;
    uint256 constant ALPHA_Y =
        0x253ec85988dbb84e46e94b5efa3373b47a000b4ac6c86b2d4b798d274a182302;
    uint256 constant BETA_NEG_X_0 =
        0x2424bcc1f60a5472685fd50705b2809626e170120acaf441e133a2bd5e61d244;
    uint256 constant BETA_NEG_X_1 =
        0x07090a82e8fabbd39299be24705b92cf208ee8b3487f6f2b39ff27978a29a1db;
    uint256 constant BETA_NEG_Y_0 =
        0x04ddc8d30d5c438ca34091c5d2c6ded571382cba2b3c4fdc4222df2938b4e51e;
    uint256 constant BETA_NEG_Y_1 =
        0x25833b15e156ae01f2741f4f4120ddb466c52eb83a959f79eb99b23caa7fbf1d;
    uint256 constant GAMMA_NEG_X_0 =
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed;
    uint256 constant GAMMA_NEG_X_1 =
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2;
    uint256 constant GAMMA_NEG_Y_0 =
        0x1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d;
    uint256 constant GAMMA_NEG_Y_1 =
        0x275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec;
    uint256 constant DELTA_NEG_X_0 =
        0x02aca5d2a73f8d34e4b26eee3932365e6526c8d5e2f3347d679c2cb1867104dc;
    uint256 constant DELTA_NEG_X_1 =
        0x07b8dbefa90bde075a26318e5066db729155514e3c06b888d4e03c56d82c97e6;
    uint256 constant DELTA_NEG_Y_0 =
        0x1696ccafaefe49a5d8bad8e79630e19b25e5392a203aff0042d0216f254806f5;
    uint256 constant DELTA_NEG_Y_1 =
        0x2edb19cbb2b6ad0c98fdd7d1845500c26e497dc35e4cdc1cb02cc65dc4ba1bf2;
    uint256 constant CONSTANT_X =
        0x2f5d8a3817f21d3e453573c90c3cc47b7ff235fad7bdfbd59bbd6ae5d153273e;
    uint256 constant CONSTANT_Y =
        0x147fa22142b1fd86ce75fc87230a0feac8765d02938784dcfc828d17d7e7c432;
    uint256 constant PUB_0_X =
        0x2a81b98e1c997bd01a20893a08a46c6804493e838c1a0ff6c8c069ef5ab66b9a;
    uint256 constant PUB_0_Y =
        0x276938ada8075cec20d4d6a1f157ec94cc7ba6207c98576e98c1ad9d6378fb6f;
    uint256 constant PUB_1_X =
        0x179496ce140df89ce35c5ee7fb496efdffda5e5d3b95ff9116e2e5df96b36ab7;
    uint256 constant PUB_1_Y =
        0x0326e7d44688ce5903676b7d646e46a5938c8e5fd8cd54e4d5aa3300649f3cfc;
    uint256 constant PEDERSEN_G_X_0 =
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed;
    uint256 constant PEDERSEN_G_X_1 =
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2;
    uint256 constant PEDERSEN_G_Y_0 =
        0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa;
    uint256 constant PEDERSEN_G_Y_1 =
        0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b;
    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_X_0 =
        0x02aca5d2a73f8d34e4b26eee3932365e6526c8d5e2f3347d679c2cb1867104dc;
    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_X_1 =
        0x07b8dbefa90bde075a26318e5066db729155514e3c06b888d4e03c56d82c97e6;
    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_Y_0 =
        0x1696ccafaefe49a5d8bad8e79630e19b25e5392a203aff0042d0216f254806f5;
    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_Y_1 =
        0x2edb19cbb2b6ad0c98fdd7d1845500c26e497dc35e4cdc1cb02cc65dc4ba1bf2;

    /// Compute the public input linear combination.
    /// @notice Reverts with PublicInputNotInField if the input is not in the field.
    /// @notice Computes the multi-scalar-multiplication of the public input
    /// elements and the verification key including the constant term.
    /// @param input The public inputs. These are elements of the scalar field Fr.
    /// @return success the result of the msm.
    /// @return x The X coordinate of the resulting G1 point.
    /// @return y The Y coordinate of the resulting G1 point.
    function publicInputMSM(
        uint256[2] calldata proofCommitment,
        uint256[2] memory input
    ) internal view returns (bool success, uint256 x, uint256 y) {
        // Note: The ECMUL precompile does not reject unreduced values, so we check this.
        // Note: Unrolling this loop does not cost much extra in code-size, the bulk of the
        //       code-size is in the PUB_ constants.
        // ECMUL has input (x, y, scalar) and output (x', y').
        // ECADD has input (x1, y1, x2, y2) and output (x', y').
        // We call them such that ecmul output is already in the second point
        // argument to ECADD so we can have a tight loop.
        success = true;
        assembly ("memory-safe") {
            let f := mload(0x40)
            let g := add(f, 0x40)
            let s
            mstore(f, CONSTANT_X)
            mstore(add(f, 0x20), CONSTANT_Y)

            // Add the proof commitment
            calldatacopy(g, proofCommitment, 0x40)
            success :=
                and(success, staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40))

            mstore(g, PUB_0_X)
            mstore(add(g, 0x20), PUB_0_Y)
            s := mload(input)
            mstore(add(g, 0x40), s)
            success := and(success, lt(s, R))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_MUL, g, 0x60, g, 0x40))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40))
            mstore(g, PUB_1_X)
            mstore(add(g, 0x20), PUB_1_Y)
            s := mload(add(input, 32))
            mstore(add(g, 0x40), s)
            success := and(success, lt(s, R))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_MUL, g, 0x60, g, 0x40))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40))
            x := mload(f)
            y := mload(add(f, 0x20))
        }
    }

    function verifyProofCommitmentPOK(
        uint256[2] calldata proofCommitment,
        uint256[2] calldata proofCommitmentPOK
    ) internal view returns (bool) {
        bool success = true;
        assembly ("memory-safe") {
            let f := mload(0x40)
            calldatacopy(f, proofCommitment, 0x40)
            mstore(add(f, 0x40), PEDERSEN_G_X_1)
            mstore(add(f, 0x60), PEDERSEN_G_X_0)
            mstore(add(f, 0x80), PEDERSEN_G_Y_1)
            mstore(add(f, 0xA0), PEDERSEN_G_Y_0)
            calldatacopy(add(f, 0xC0), proofCommitmentPOK, 0x40)
            mstore(add(f, 0x100), PEDERSEN_G_ROOT_SIGMA_NEG_X_1)
            mstore(add(f, 0x120), PEDERSEN_G_ROOT_SIGMA_NEG_X_0)
            mstore(add(f, 0x140), PEDERSEN_G_ROOT_SIGMA_NEG_Y_1)
            mstore(add(f, 0x160), PEDERSEN_G_ROOT_SIGMA_NEG_Y_0)
            success := staticcall(gas(), PRECOMPILE_VERIFY, f, 0x180, f, 0x20)
            success := and(success, mload(f))
        }
        return success;
    }

    /// Verify an uncompressed Groth16 proof.
    /// @notice Reverts with InvalidProof if the proof is invalid or
    /// with PublicInputNotInField the public input is not reduced.
    /// @notice There is no return value. If the function does not revert, the
    /// proof was successfully verified.
    /// @param proof the points (A, B, C) in EIP-197 format matching the output
    /// of compressProof.
    /// @param input the public input field elements in the scalar field Fr.
    /// Elements must be reduced.
    function verifyProof(
        uint256[8] calldata proof,
        uint256[2] calldata proofCommitment,
        uint256[2] calldata proofCommitmentPOK,
        uint256[2] memory input
    ) internal view returns (bool) {
        (bool success, uint256 x, uint256 y) =
            publicInputMSM(proofCommitment, input);
        if (!success) {
            return false;
        }

        if (!verifyProofCommitmentPOK(proofCommitment, proofCommitmentPOK)) {
            return false;
        }

        // Note: The precompile expects the F2 coefficients in big-endian order.
        // Note: The pairing precompile rejects unreduced values, so we won't check that here.
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
        return success;
    }
}
