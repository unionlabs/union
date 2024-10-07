pragma solidity ^0.8.27;

/// @title Groth16 verifier template.
/// @author Remco Bloemen
/// @notice Supports verifying Groth16 proofs. Proofs can be in uncompressed
/// (256 bytes) and compressed (128 bytes) format.
library Verifier {
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
        4252850302693242182654534639730627324742305503909561446344356971523664816281;
    uint256 constant ALPHA_Y =
        3971530409048238023625806606514600982127202826003358538821613170737831313919;

    // Groth16 beta point in G2 in powers of i
    uint256 constant BETA_NEG_X_0 =
        9609903744775525881338738176064678545439912439219033822736570321349357348980;
    uint256 constant BETA_NEG_X_1 =
        11402125448377072234752634956069960846261435348550776006069399216352815312229;
    uint256 constant BETA_NEG_Y_0 =
        18012228678282290194170129154972180638950912669850573130308339510071981008545;
    uint256 constant BETA_NEG_Y_1 =
        15756550515454626729445647420198526257176992371703002957323861385095544414838;

    // Groth16 gamma point in G2 in powers of i
    uint256 constant GAMMA_NEG_X_0 =
        15418804173338388766896385877623893969695670309009587476846726795628238714393;
    uint256 constant GAMMA_NEG_X_1 =
        14882897597913405382982164467298010752166363844685258881581520272046793702095;
    uint256 constant GAMMA_NEG_Y_0 =
        17722217720691050164784298688157009907556422267906762591449788940639280738106;
    uint256 constant GAMMA_NEG_Y_1 =
        21681514378991397271958143575996358636110810782474567203218670880519258244465;

    // Groth16 delta point in G2 in powers of i
    uint256 constant DELTA_NEG_X_0 =
        2636161939055419322743684458857549714230849256995406138405588958157843793131;
    uint256 constant DELTA_NEG_X_1 =
        18711435617866698040659011365354165232283248284733617156044102129651710736892;
    uint256 constant DELTA_NEG_Y_0 =
        2647887006311232967132848950859794223811860619760715975180654346594734512903;
    uint256 constant DELTA_NEG_Y_1 =
        9638871602237154557801043117594638698760262947775166324439744310655148732994;

    // Constant and public input points
    uint256 constant CONSTANT_X =
        17683074019270049519594214298171697666582975915064153618004061598086681825921;
    uint256 constant CONSTANT_Y =
        16826145467743906176166100307225491106961753217491843100452871479833450456070;
    uint256 constant PUB_0_X =
        4999724750322169039879775285047941133298355297928988655266615607529011563466;
    uint256 constant PUB_0_Y =
        8614448667589143428827059805500251818303043966026074735628377626634208993292;
    uint256 constant PUB_1_X =
        1184807858330365651919114999096473332175166887333719856514157833289677967559;
    uint256 constant PUB_1_Y =
        20327610427697660249999185524229068956160879388632193295649998184224119517657;

    // Commitment key
    uint256 constant PEDERSEN_G_X_0 =
        0x257DF6F8132CB0037F7DFDF1A29B04C1FF92BA082EDA513996BA2BFA9FBD1987;
    uint256 constant PEDERSEN_G_X_1 =
        0x13F0D8D8879885CA567EF99298C30C397E6FBA584658F4127713A814C06DE55A;
    uint256 constant PEDERSEN_G_Y_0 =
        0x1660EBCC60C7A3AC560EFCEA5993F528EE13685D3A39694ACD74FE67C80D798A;
    uint256 constant PEDERSEN_G_Y_1 =
        0x15E80642C58DB4DBE0A87F92CE3C65E962F231278353783A691FD64078BA7F34;

    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_X_0 =
        0x2FBFE141A7555CF7E3E86B092660B81CFB68A025AD817E45CEC0B0F2E2CA6368;
    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_X_1 =
        0x02A104DF1C015F2307FA2859627098CDF9FDB521D61D323943343A12304E5BAF;
    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_Y_0 =
        0x27DA3F93ECF3BFD0B3A3354AE2162A6C230C0E539B6D9F82C0826E2B006A5922;
    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_Y_1 =
        0x2C0838551CB9E5CF67DB57DE7E2250BB97807F6687F135A6EB910359BA7BDB8D;

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
            mstore(add(f, 0x40), PEDERSEN_G_X_0)
            mstore(add(f, 0x60), PEDERSEN_G_X_1)
            mstore(add(f, 0x80), PEDERSEN_G_Y_0)
            mstore(add(f, 0xA0), PEDERSEN_G_Y_1)
            calldatacopy(add(f, 0xC0), proofCommitmentPOK, 0x40)
            mstore(add(f, 0x100), PEDERSEN_G_ROOT_SIGMA_NEG_X_0)
            mstore(add(f, 0x120), PEDERSEN_G_ROOT_SIGMA_NEG_X_1)
            mstore(add(f, 0x140), PEDERSEN_G_ROOT_SIGMA_NEG_Y_0)
            mstore(add(f, 0x160), PEDERSEN_G_ROOT_SIGMA_NEG_Y_1)
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
