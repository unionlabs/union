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
        843318045813904051851782814226569312224716668519879443037991679093301257400;
    uint256 constant ALPHA_Y =
        15416107080929723745166446798814682078427563000162261911323849488726431649665;

    // Groth16 beta point in G2 in powers of i
    uint256 constant BETA_NEG_X_0 =
        12045932065395624483191137563968354138074439042345149261057471855853342274450;
    uint256 constant BETA_NEG_X_1 =
        12933749417029078157681900380950146806593248644656848159050519236558381927908;
    uint256 constant BETA_NEG_Y_0 =
        9082851089017368762407429600641918693222555188969757112608171999788085541296;
    uint256 constant BETA_NEG_Y_1 =
        7303728976154124279287303283380474877924198486276751589247747025678134560243;

    // Groth16 gamma point in G2 in powers of i
    uint256 constant GAMMA_NEG_X_0 =
        8834401517279732426430709303690144409212027987281176083510451411579725021792;
    uint256 constant GAMMA_NEG_X_1 =
        5046190406338174773452605841724188049492775755571037890518087582412737236350;
    uint256 constant GAMMA_NEG_Y_0 =
        10862086833230676274029914029282650477819381960814725537691816810569857516224;
    uint256 constant GAMMA_NEG_Y_1 =
        17423404563633638824996505607345136333520864045004270305928336097970313608127;

    // Groth16 delta point in G2 in powers of i
    uint256 constant DELTA_NEG_X_0 =
        6081458280594160167006539403251703157253227316242605315149917857620252166561;
    uint256 constant DELTA_NEG_X_1 =
        19972418504378784918799069027339170180958910678041257330604520110151384463379;
    uint256 constant DELTA_NEG_Y_0 =
        19078121358711053922429530973626103352790145458277087968867358323681092483467;
    uint256 constant DELTA_NEG_Y_1 =
        21380583072922005610003682662802526130274043623910469889264187204640234271902;

    // Constant and public input points
    uint256 constant CONSTANT_X =
        11480604022502486320552383084244624890381895090315703237207761641596379587022;
    uint256 constant CONSTANT_Y =
        18194717846527842697597101854227780426448444481406927484453657628859660838012;
    uint256 constant PUB_0_X =
        17151101065438037966418446912782326714022524609709696117185269486796039712846;
    uint256 constant PUB_0_Y =
        20153978099716141223629974435089176387221033706146080785832322758462404157605;
    uint256 constant PUB_1_X =
        4678289054354856791632961819188703697251080339511868428756468986114497080410;
    uint256 constant PUB_1_Y =
        14896883393560813725625670193715860991949118639991676227159535433685188744128;
    uint256 constant PUB_2_X =
        9861092238854536479945771789292920457542225544584991490958628460109410999807;
    uint256 constant PUB_2_Y =
        11463793619454015701756134623742521947898582646550901437275441814334362034435;
    uint256 constant PUB_3_X =
        11893178039948407756040467155600718551687885088832926431141932892836179895466;
    uint256 constant PUB_3_Y =
        9336098954712618302413022925991187147043963777349552077325468595033619593827;

    // Commitment key
    uint256 constant PEDERSEN_G_X_0 =
        0x0DB410C824A5ADBD313D740A430630A107D57410B9BF6FAF5AFCFAAB2B2617FE;
    uint256 constant PEDERSEN_G_X_1 =
        0x1B42C82BA48ECE99B163761C96B995CCC15598BFBA746FC6E9DBAD445DDA796D;
    uint256 constant PEDERSEN_G_Y_0 =
        0x1F34EBAF716B9210C884F8F07F0E08DEB6435B770E5E34B8244497D38840CE2B;
    uint256 constant PEDERSEN_G_Y_1 =
        0x21EA2052643FFDCD11760154036BD266D6A261638E25AFB9917DA8E0C347C9B8;

    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_X_0 =
        0x247B0E5AF7C23D717F5C88E71545A7CD67052C3141DF9EBF8B1FFE7ADB1CBDC1;
    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_X_1 =
        0x0C301FD6EA0C05EF4B9FB346AF88B7BA904A8EB37E87E412B04A002801B429A7;
    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_Y_0 =
        0x161556EA8AE6D6B0B9E74133A53F5F15B2859611C982615E0D7937FD929EB90A;
    uint256 constant PEDERSEN_G_ROOT_SIGMA_NEG_Y_1 =
        0x2C07A459154070A4C140C7766C4034D1AF770F072C1A3C7E5E41B685AB9547A9;

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
            success :=
                and(success, staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40))

            mstore(g, PUB_0_X)
            mstore(add(g, 0x20), PUB_0_Y)
            s := calldataload(input)
            mstore(add(g, 0x40), s)
            success := and(success, lt(s, R))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_MUL, g, 0x60, g, 0x40))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40))
            mstore(g, PUB_1_X)
            mstore(add(g, 0x20), PUB_1_Y)
            s := calldataload(add(input, 32))
            mstore(add(g, 0x40), s)
            success := and(success, lt(s, R))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_MUL, g, 0x60, g, 0x40))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40))
            mstore(g, PUB_2_X)
            mstore(add(g, 0x20), PUB_2_Y)
            s := calldataload(add(input, 64))
            mstore(add(g, 0x40), s)
            success := and(success, lt(s, R))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_MUL, g, 0x60, g, 0x40))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40))
            mstore(g, PUB_3_X)
            mstore(add(g, 0x20), PUB_3_Y)
            s := calldataload(add(input, 96))
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
        uint256[4] calldata input
    ) public view returns (bool) {
        (bool success, uint256 x, uint256 y) =
            publicInputMSM(proofCommitment, input);
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

            // Verify pedersen commitment proof of knowledge
            // Symmetric to https://github.com/Consensys/gnark-crypto/blob/2e4aaaaefdbfdf06515663986ed884fed1b2177e/ecc/bn254/fr/pedersen/pedersen.go#L212-L224
            calldatacopy(add(f, 0x300), proofCommitment, 0x40)
            mstore(add(f, 0x340), PEDERSEN_G_X_0)
            mstore(add(f, 0x360), PEDERSEN_G_X_1)
            mstore(add(f, 0x380), PEDERSEN_G_Y_0)
            mstore(add(f, 0x3A0), PEDERSEN_G_Y_1)
            calldatacopy(add(f, 0x3C0), proofCommitmentPOK, 0x40)
            mstore(add(f, 0x400), PEDERSEN_G_ROOT_SIGMA_NEG_X_0)
            mstore(add(f, 0x420), PEDERSEN_G_ROOT_SIGMA_NEG_X_1)
            mstore(add(f, 0x440), PEDERSEN_G_ROOT_SIGMA_NEG_Y_0)
            mstore(add(f, 0x460), PEDERSEN_G_ROOT_SIGMA_NEG_Y_1)

            // Check pairing equation.
            success := staticcall(gas(), PRECOMPILE_VERIFY, f, 0x480, f, 0x20)
            // Also check returned value (both are either 1 or 0).
            success := and(success, mload(f))
        }
        return success;
    }
}
