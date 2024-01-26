pragma solidity ^0.8.23;

import "../core/IZKVerifierV2.sol";

/// @title Groth16 verifier template.
/// @author Remco Bloemen
/// @notice Supports verifying Groth16 proofs. Proofs can be in uncompressed
/// (256 bytes) and compressed (128 bytes) format.
contract Verifier is IZKVerifierV2 {
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

    // Groth16 alpha point in G1
    uint256 constant ALPHA_X =
        210400609118751867867594962339236416900807823190943555300977451252876367251;
    uint256 constant ALPHA_Y =
        15114917314049487003769383074865116286536524761334960573881383182006812098667;

    // Groth16 beta point in G2 in powers of i
    uint256 constant BETA_NEG_X_0 =
        4025695320685928294502537638656612753817559258692794664435345502849231300067;
    uint256 constant BETA_NEG_X_1 =
        18548846535245326130909941053625664549629964661821753718960511623672350665970;
    uint256 constant BETA_NEG_Y_0 =
        11041812570379749260952002555243362169050281371699289540491470062146377949858;
    uint256 constant BETA_NEG_Y_1 =
        7086530552084536928760407590619509472686325136379940088298434085503010610917;

    // Groth16 gamma point in G2 in powers of i
    uint256 constant GAMMA_NEG_X_0 =
        3871507673786634538856304899308535382710007469487017238115999457851260261753;
    uint256 constant GAMMA_NEG_X_1 =
        9263528541268382918393290127320839732843244267184360590957560203659557939807;
    uint256 constant GAMMA_NEG_Y_0 =
        460085126711753980079882291129874808860955437963028782772312595071958830910;
    uint256 constant GAMMA_NEG_Y_1 =
        5659507275301128335480015236667146972531485943427570808675852670599024875145;

    // Groth16 delta point in G2 in powers of i
    uint256 constant DELTA_NEG_X_0 =
        10511954649625640946194391595271440757788300118705902871380699323889590717070;
    uint256 constant DELTA_NEG_X_1 =
        3624887307974581652668788593721111206379446020443390411004753466558440004576;
    uint256 constant DELTA_NEG_Y_0 =
        14620721906662022476476821247254018294642326056513995738211968660558997332653;
    uint256 constant DELTA_NEG_Y_1 =
        8017039972559091515318513919400394779640332862574074418135272025093207644503;

    // Constant and public input points
    uint256 constant CONSTANT_X =
        7548860451745230603118483440705740915442110531593946049093378206268978489889;
    uint256 constant CONSTANT_Y =
        8642676949380445344876359442033848518648809724845273937857877647136985821580;
    uint256 constant PUB_0_X =
        3761607900566975305271041912404145777243969522402194441451368484814115763984;
    uint256 constant PUB_0_Y =
        5236577135937549420013214983430521575368103283652548157969360454835627955674;
    uint256 constant PUB_1_X =
        6551227099151518169715147953582656472887860242160769774314802712412742643231;
    uint256 constant PUB_1_Y =
        1819741224363792162320411058820080302711126963872887137191002940993034590657;
    uint256 constant PUB_2_X =
        14381003547507154363999629345246039179209989264380273368866915523962951751353;
    uint256 constant PUB_2_Y =
        11723454477524607914925823310578875758974497210070578907830734479226804006505;
    uint256 constant PUB_3_X =
        9428637111213429505715745409259434820588419778432030319817634906362966081466;
    uint256 constant PUB_3_Y =
        20385940124935492750321915374844475607796921194332338238981177318927052319236;

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
        uint256[4] calldata input
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
            mstore(g, calldataload(proofCommitment))
            mstore(add(g, 0x20), calldataload(add(proofCommitment, 32)))
            success := and(
                success,
                staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40)
            )

            mstore(g, PUB_0_X)
            mstore(add(g, 0x20), PUB_0_Y)
            s := calldataload(input)
            mstore(add(g, 0x40), s)
            success := and(success, lt(s, R))
            success := and(
                success,
                staticcall(gas(), PRECOMPILE_MUL, g, 0x60, g, 0x40)
            )
            success := and(
                success,
                staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40)
            )
            mstore(g, PUB_1_X)
            mstore(add(g, 0x20), PUB_1_Y)
            s := calldataload(add(input, 32))
            mstore(add(g, 0x40), s)
            success := and(success, lt(s, R))
            success := and(
                success,
                staticcall(gas(), PRECOMPILE_MUL, g, 0x60, g, 0x40)
            )
            success := and(
                success,
                staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40)
            )
            mstore(g, PUB_2_X)
            mstore(add(g, 0x20), PUB_2_Y)
            s := calldataload(add(input, 64))
            mstore(add(g, 0x40), s)
            success := and(success, lt(s, R))
            success := and(
                success,
                staticcall(gas(), PRECOMPILE_MUL, g, 0x60, g, 0x40)
            )
            success := and(
                success,
                staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40)
            )
            mstore(g, PUB_3_X)
            mstore(add(g, 0x20), PUB_3_Y)
            s := calldataload(add(input, 96))
            mstore(add(g, 0x40), s)
            success := and(success, lt(s, R))
            success := and(
                success,
                staticcall(gas(), PRECOMPILE_MUL, g, 0x60, g, 0x40)
            )
            success := and(
                success,
                staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40)
            )
            x := mload(f)
            y := mload(add(f, 0x20))
        }
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
        uint256[4] calldata input
    ) public view returns (bool) {
        (bool success, uint256 x, uint256 y) = publicInputMSM(
            proofCommitment,
            input
        );
        if (!success) {
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
