package plonk

const solidityTemplate = `
// Warning this code was contributed into gnark here: 
// https://github.com/ConsenSys/gnark/pull/358
// 
// It has not been audited and is provided as-is, we make no guarantees or warranties to its safety and reliability. 
// 
// According to https://eprint.iacr.org/archive/2019/953/1585767119.pdf
pragma solidity ^0.8.0;
pragma experimental ABIEncoderV2;

library PairingsBn254 {
    uint256 constant q_mod = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
    uint256 constant r_mod = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    uint256 constant bn254_b_coeff = 3;

    struct G1Point {
        uint256 X;
        uint256 Y;
    }

    struct Fr {
        uint256 value;
    }

    function new_fr(uint256 fr) internal pure returns (Fr memory) {
        require(fr < r_mod);
        return Fr({value: fr});
    }

    function copy(Fr memory self) internal pure returns (Fr memory n) {
        n.value = self.value;
    }

    function assign(Fr memory self, Fr memory other) internal pure {
        self.value = other.value;
    }

    function inverse(Fr memory fr) internal view returns (Fr memory) {
        require(fr.value != 0);
        return pow(fr, r_mod-2);
    }

    function add_assign(Fr memory self, Fr memory other) internal pure {
        self.value = addmod(self.value, other.value, r_mod);
    }

    function sub_assign(Fr memory self, Fr memory other) internal pure {
        self.value = addmod(self.value, r_mod - other.value, r_mod);
    }

    function mul_assign(Fr memory self, Fr memory other) internal pure {
        self.value = mulmod(self.value, other.value, r_mod);
    }

    function pow(Fr memory self, uint256 power) internal view returns (Fr memory) {
        uint256[6] memory input = [32, 32, 32, self.value, power, r_mod];
        uint256[1] memory result;
        bool success;
        assembly {
            success := staticcall(gas(), 0x05, input, 0xc0, result, 0x20)
        }
        require(success);
        return Fr({value: result[0]});
    }

    // Encoding of field elements is: X[0] * z + X[1]
    struct G2Point {
        uint[2] X;
        uint[2] Y;
    }

    function P1() internal pure returns (G1Point memory) {
        return G1Point(1, 2);
    }

    function new_g1(uint256 x, uint256 y) internal pure returns (G1Point memory) {
        return G1Point(x, y);
    }

    function new_g1_checked(uint256 x, uint256 y) internal pure returns (G1Point memory) {
        if (x == 0 && y == 0) {
            // point of infinity is (0,0)
            return G1Point(x, y);
        }

        // check encoding
        require(x < q_mod);
        require(y < q_mod);
        // check on curve
        uint256 lhs = mulmod(y, y, q_mod); // y^2
        uint256 rhs = mulmod(x, x, q_mod); // x^2
        rhs = mulmod(rhs, x, q_mod); // x^3
        rhs = addmod(rhs, bn254_b_coeff, q_mod); // x^3 + b
        require(lhs == rhs);

        return G1Point(x, y);
    }

    function new_g2(uint256[2] memory x, uint256[2] memory y) internal pure returns (G2Point memory) {
        return G2Point(x, y);
    }

    function copy_g1(G1Point memory self) internal pure returns (G1Point memory result) {
        result.X = self.X;
        result.Y = self.Y;
    }

    function P2() internal pure returns (G2Point memory) {
        // for some reason ethereum expects to have c1*v + c0 form

        return G2Point(
            [0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
            0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed],
            [0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b,
            0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa]
        );
    }

    function negate(G1Point memory self) internal pure {
        // The prime q in the base field F_q for G1
        if (self.Y == 0) {
            require(self.X == 0);
            return;
        }

        self.Y = q_mod - self.Y;
    }

    function point_add(G1Point memory p1, G1Point memory p2)
    internal view returns (G1Point memory r)
    {
        point_add_into_dest(p1, p2, r);
        return r;
    }

    function point_add_assign(G1Point memory p1, G1Point memory p2)
    internal view
    {
        point_add_into_dest(p1, p2, p1);
    }

    function point_add_into_dest(G1Point memory p1, G1Point memory p2, G1Point memory dest)
    internal view
    {
        if (p2.X == 0 && p2.Y == 0) {
            // we add zero, nothing happens
            dest.X = p1.X;
            dest.Y = p1.Y;
            return;
        } else if (p1.X == 0 && p1.Y == 0) {
            // we add into zero, and we add non-zero point
            dest.X = p2.X;
            dest.Y = p2.Y;
            return;
        } else {
            uint256[4] memory input;

            input[0] = p1.X;
            input[1] = p1.Y;
            input[2] = p2.X;
            input[3] = p2.Y;

            bool success = false;
            assembly {
                success := staticcall(gas(), 6, input, 0x80, dest, 0x40)
            }
            require(success);
        }
    }

    function point_sub_assign(G1Point memory p1, G1Point memory p2)
    internal view
    {
        point_sub_into_dest(p1, p2, p1);
    }

    function point_sub_into_dest(G1Point memory p1, G1Point memory p2, G1Point memory dest)
    internal view
    {
        if (p2.X == 0 && p2.Y == 0) {
            // we subtracted zero, nothing happens
            dest.X = p1.X;
            dest.Y = p1.Y;
            return;
        } else if (p1.X == 0 && p1.Y == 0) {
            // we subtract from zero, and we subtract non-zero point
            dest.X = p2.X;
            dest.Y = q_mod - p2.Y;
            return;
        } else {
            uint256[4] memory input;

            input[0] = p1.X;
            input[1] = p1.Y;
            input[2] = p2.X;
            input[3] = q_mod - p2.Y;

            bool success = false;
            assembly {
                success := staticcall(gas(), 6, input, 0x80, dest, 0x40)
            }
            require(success);
        }
    }

    function point_mul(G1Point memory p, Fr memory s)
    internal view returns (G1Point memory r)
    {
        point_mul_into_dest(p, s, r);
        return r;
    }

    function point_mul_assign(G1Point memory p, Fr memory s)
    internal view
    {
        point_mul_into_dest(p, s, p);
    }

    function point_mul_into_dest(G1Point memory p, Fr memory s, G1Point memory dest)
    internal view
    {
        uint[3] memory input;
        input[0] = p.X;
        input[1] = p.Y;
        input[2] = s.value;
        bool success;
        assembly {
            success := staticcall(gas(), 7, input, 0x60, dest, 0x40)
        }
        require(success);
    }

    function pairing(G1Point[] memory p1, G2Point[] memory p2)
    internal view returns (bool)
    {
        require(p1.length == p2.length);
        uint elements = p1.length;
        uint inputSize = elements * 6;
        uint[] memory input = new uint[](inputSize);
        for (uint i = 0; i < elements; i++)
        {
            input[i * 6 + 0] = p1[i].X;
            input[i * 6 + 1] = p1[i].Y;
            input[i * 6 + 2] = p2[i].X[0];
            input[i * 6 + 3] = p2[i].X[1];
            input[i * 6 + 4] = p2[i].Y[0];
            input[i * 6 + 5] = p2[i].Y[1];
        }
        uint[1] memory out;
        bool success;
        assembly {
            success := staticcall(gas(), 8, add(input, 0x20), mul(inputSize, 0x20), out, 0x20)
        }
        require(success);
        return out[0] != 0;
    }

    /// Convenience method for a pairing check for two pairs.
    function pairingProd2(G1Point memory a1, G2Point memory a2, G1Point memory b1, G2Point memory b2)
    internal view returns (bool)
    {
        G1Point[] memory p1 = new G1Point[](2);
        G2Point[] memory p2 = new G2Point[](2);
        p1[0] = a1;
        p1[1] = b1;
        p2[0] = a2;
        p2[1] = b2;
        return pairing(p1, p2);
    }
}

library TranscriptLibrary {
    uint32 constant DST_0 = 0;
    uint32 constant DST_1 = 1;
    uint32 constant DST_CHALLENGE = 2;

    struct Transcript {
        bytes32 previous_randomness;
        bytes bindings;
        string name;
        uint32 challenge_counter;
    }

    function new_transcript() internal pure returns (Transcript memory t) {
        t.challenge_counter = 0;
    }

    function set_challenge_name(Transcript memory self, string memory name) internal pure {
        self.name = name;
    }

    function update_with_u256(Transcript memory self, uint256 value) internal pure {
        self.bindings = abi.encodePacked(self.bindings, value);
    }

    function update_with_fr(Transcript memory self, PairingsBn254.Fr memory value) internal pure {
        self.bindings = abi.encodePacked(self.bindings, value.value);
    }

    function update_with_g1(Transcript memory self, PairingsBn254.G1Point memory p) internal pure {
        self.bindings = abi.encodePacked(self.bindings, p.X, p.Y);
    }

    function get_encode(Transcript memory self) internal pure returns(bytes memory query) {
        if (self.challenge_counter != 0) {
            query = abi.encodePacked(self.name, self.previous_randomness, self.bindings);
        } else {
            query = abi.encodePacked(self.name, self.bindings);
        }
        return query;
    }
    function get_challenge(Transcript memory self) internal pure returns(PairingsBn254.Fr memory challenge) {
        bytes32 query;
        if (self.challenge_counter != 0) {
            query = sha256(abi.encodePacked(self.name, self.previous_randomness, self.bindings));
        } else {
            query = sha256(abi.encodePacked(self.name, self.bindings));
        }
        self.challenge_counter += 1;
        self.previous_randomness = query;
        challenge = PairingsBn254.Fr({value: uint256(query) % PairingsBn254.r_mod});
        self.bindings = "";
    }
}

contract PlonkVerifier {
    using PairingsBn254 for PairingsBn254.G1Point;
    using PairingsBn254 for PairingsBn254.G2Point;
    using PairingsBn254 for PairingsBn254.Fr;

    using TranscriptLibrary for TranscriptLibrary.Transcript;

    uint256 constant STATE_WIDTH = 3;

    struct VerificationKey {
        uint256 domain_size;
        uint256 num_inputs;
        PairingsBn254.Fr omega;                                     // w
        PairingsBn254.G1Point[STATE_WIDTH+2] selector_commitments;  // STATE_WIDTH for witness + multiplication + constant
        PairingsBn254.G1Point[STATE_WIDTH] permutation_commitments; // [Sσ1(x)],[Sσ2(x)],[Sσ3(x)]
        PairingsBn254.Fr[STATE_WIDTH-1] permutation_non_residues;   // k1, k2
        PairingsBn254.G2Point g2_x;
    }

    struct Proof {
        uint256[] input_values;
        PairingsBn254.G1Point[STATE_WIDTH] wire_commitments;  // [a(x)]/[b(x)]/[c(x)]
        PairingsBn254.G1Point grand_product_commitment;      // [z(x)]
        PairingsBn254.G1Point[STATE_WIDTH] quotient_poly_commitments;  // [t_lo]/[t_mid]/[t_hi]
        PairingsBn254.Fr[STATE_WIDTH] wire_values_at_zeta;   // a(zeta)/b(zeta)/c(zeta)
        PairingsBn254.Fr grand_product_at_zeta_omega;        // z(w*zeta)
        PairingsBn254.Fr quotient_polynomial_at_zeta;        // t(zeta)
        PairingsBn254.Fr linearization_polynomial_at_zeta;   // r(zeta)
        PairingsBn254.Fr[STATE_WIDTH-1] permutation_polynomials_at_zeta;  // Sσ1(zeta),Sσ2(zeta)

        PairingsBn254.G1Point opening_at_zeta_proof;            // [Wzeta]
        PairingsBn254.G1Point opening_at_zeta_omega_proof;      // [Wzeta*omega]
    }

    struct PartialVerifierState {
        PairingsBn254.Fr alpha;
        PairingsBn254.Fr beta;
        PairingsBn254.Fr gamma;
        PairingsBn254.Fr v;
        PairingsBn254.Fr u;
        PairingsBn254.Fr zeta;
        PairingsBn254.Fr[] cached_lagrange_evals;

        PairingsBn254.G1Point cached_fold_quotient_ploy_commitments;
    }

    function verify_initial(
		PartialVerifierState memory state,
        Proof memory proof,
        VerificationKey memory vk) internal view returns (bool) {

        require(proof.input_values.length == vk.num_inputs, "not match");
        require(vk.num_inputs >= 1, "inv input");
        
        TranscriptLibrary.Transcript memory t = TranscriptLibrary.new_transcript();
        t.set_challenge_name("gamma");
        for (uint256 i = 0; i < vk.permutation_commitments.length; i++) {
            t.update_with_g1(vk.permutation_commitments[i]);
        }
        // this is gnark order: Ql, Qr, Qm, Qo, Qk
        //
        t.update_with_g1(vk.selector_commitments[0]);
        t.update_with_g1(vk.selector_commitments[1]);
        t.update_with_g1(vk.selector_commitments[3]);
        t.update_with_g1(vk.selector_commitments[2]);
        t.update_with_g1(vk.selector_commitments[4]);

        for (uint256 i = 0; i < proof.input_values.length; i++) {
            t.update_with_u256(proof.input_values[i]);
        }
        state.gamma = t.get_challenge();

        t.set_challenge_name("beta");
        state.beta = t.get_challenge();

        t.set_challenge_name("alpha");
        t.update_with_g1(proof.grand_product_commitment);
        state.alpha = t.get_challenge();

        t.set_challenge_name("zeta");
        for (uint256 i = 0; i < proof.quotient_poly_commitments.length; i++) {
            t.update_with_g1(proof.quotient_poly_commitments[i]);
        }
        state.zeta = t.get_challenge();

        uint256[] memory lagrange_poly_numbers = new uint256[](vk.num_inputs);
        for (uint256 i = 0; i < lagrange_poly_numbers.length; i++) {
            lagrange_poly_numbers[i] = i;
        }
        state.cached_lagrange_evals = batch_evaluate_lagrange_poly_out_of_domain(
            lagrange_poly_numbers,
            vk.domain_size,
            vk.omega, state.zeta
        );

        bool valid = verify_quotient_poly_eval_at_zeta(state, proof, vk);
        return valid;
    }

    function verify_commitments(
        PartialVerifierState memory state,
        Proof memory proof,
        VerificationKey memory vk
    ) internal view returns (bool) {
        PairingsBn254.G1Point memory d = reconstruct_d(state, proof, vk);

        PairingsBn254.G1Point memory tmp_g1 = PairingsBn254.P1();

        PairingsBn254.Fr memory aggregation_challenge = PairingsBn254.new_fr(1);

        PairingsBn254.G1Point memory commitment_aggregation = PairingsBn254.copy_g1(state.cached_fold_quotient_ploy_commitments);
        PairingsBn254.Fr memory tmp_fr = PairingsBn254.new_fr(1);

        aggregation_challenge.mul_assign(state.v);
        commitment_aggregation.point_add_assign(d);

        for (uint i = 0; i < proof.wire_commitments.length; i++) {
            aggregation_challenge.mul_assign(state.v);
            tmp_g1 = proof.wire_commitments[i].point_mul(aggregation_challenge);
            commitment_aggregation.point_add_assign(tmp_g1);
        }

        for (uint i = 0; i < vk.permutation_commitments.length - 1; i++) {
            aggregation_challenge.mul_assign(state.v);
            tmp_g1 = vk.permutation_commitments[i].point_mul(aggregation_challenge);
            commitment_aggregation.point_add_assign(tmp_g1);
        }

        // collect opening values
        aggregation_challenge = PairingsBn254.new_fr(1);

        PairingsBn254.Fr memory aggregated_value = PairingsBn254.copy(proof.quotient_polynomial_at_zeta);

        aggregation_challenge.mul_assign(state.v);

        tmp_fr.assign(proof.linearization_polynomial_at_zeta);
        tmp_fr.mul_assign(aggregation_challenge);
        aggregated_value.add_assign(tmp_fr);

        for (uint i = 0; i < proof.wire_values_at_zeta.length; i++) {
            aggregation_challenge.mul_assign(state.v);

            tmp_fr.assign(proof.wire_values_at_zeta[i]);
            tmp_fr.mul_assign(aggregation_challenge);
            aggregated_value.add_assign(tmp_fr);
        }

        for (uint i = 0; i < proof.permutation_polynomials_at_zeta.length; i++) {
            aggregation_challenge.mul_assign(state.v);

            tmp_fr.assign(proof.permutation_polynomials_at_zeta[i]);
            tmp_fr.mul_assign(aggregation_challenge);
            aggregated_value.add_assign(tmp_fr);
        }
        tmp_fr.assign(proof.grand_product_at_zeta_omega);
        tmp_fr.mul_assign(state.u);
        aggregated_value.add_assign(tmp_fr);

        commitment_aggregation.point_sub_assign(PairingsBn254.P1().point_mul(aggregated_value));

        PairingsBn254.G1Point memory pair_with_generator = commitment_aggregation;
        pair_with_generator.point_add_assign(proof.opening_at_zeta_proof.point_mul(state.zeta));

        tmp_fr.assign(state.zeta);
        tmp_fr.mul_assign(vk.omega);
        tmp_fr.mul_assign(state.u);
        pair_with_generator.point_add_assign(proof.opening_at_zeta_omega_proof.point_mul(tmp_fr));

        PairingsBn254.G1Point memory pair_with_x = proof.opening_at_zeta_omega_proof.point_mul(state.u);
        pair_with_x.point_add_assign(proof.opening_at_zeta_proof);
        pair_with_x.negate();

        return PairingsBn254.pairingProd2(pair_with_generator, PairingsBn254.P2(), pair_with_x, vk.g2_x);
    }

    function reconstruct_d(
        PartialVerifierState memory state,
        Proof memory proof,
        VerificationKey memory vk
    ) internal view returns (PairingsBn254.G1Point memory res) {
        res = PairingsBn254.copy_g1(vk.selector_commitments[STATE_WIDTH + 1]);

        PairingsBn254.G1Point memory tmp_g1 = PairingsBn254.P1();
        PairingsBn254.Fr memory tmp_fr = PairingsBn254.new_fr(0);

        // addition gates
        for (uint256 i = 0; i < STATE_WIDTH; i++) {
            tmp_g1 = vk.selector_commitments[i].point_mul(proof.wire_values_at_zeta[i]);
            res.point_add_assign(tmp_g1);
        }

        // multiplication gate
        tmp_fr.assign(proof.wire_values_at_zeta[0]);
        tmp_fr.mul_assign(proof.wire_values_at_zeta[1]);
        tmp_g1 = vk.selector_commitments[STATE_WIDTH].point_mul(tmp_fr);
        res.point_add_assign(tmp_g1);

        // z * non_res * beta + gamma + a
        PairingsBn254.Fr memory grand_product_part_at_z = PairingsBn254.copy(state.zeta);
        grand_product_part_at_z.mul_assign(state.beta);
        grand_product_part_at_z.add_assign(proof.wire_values_at_zeta[0]);
        grand_product_part_at_z.add_assign(state.gamma);
        for (uint256 i = 0; i < vk.permutation_non_residues.length; i++) {
            tmp_fr.assign(state.zeta);
            tmp_fr.mul_assign(vk.permutation_non_residues[i]);
            tmp_fr.mul_assign(state.beta);
            tmp_fr.add_assign(state.gamma);
            tmp_fr.add_assign(proof.wire_values_at_zeta[i+1]);

            grand_product_part_at_z.mul_assign(tmp_fr);
        }

        grand_product_part_at_z.mul_assign(state.alpha);

        tmp_fr.assign(state.cached_lagrange_evals[0]);
        tmp_fr.mul_assign(state.alpha);
        tmp_fr.mul_assign(state.alpha);
        // NOTICE
        grand_product_part_at_z.sub_assign(tmp_fr);
        PairingsBn254.Fr memory last_permutation_part_at_z = PairingsBn254.new_fr(1);
        for (uint256 i = 0; i < proof.permutation_polynomials_at_zeta.length; i++) {
            tmp_fr.assign(state.beta);
            tmp_fr.mul_assign(proof.permutation_polynomials_at_zeta[i]);
            tmp_fr.add_assign(state.gamma);
            tmp_fr.add_assign(proof.wire_values_at_zeta[i]);

            last_permutation_part_at_z.mul_assign(tmp_fr);
        }

        last_permutation_part_at_z.mul_assign(state.beta);
        last_permutation_part_at_z.mul_assign(proof.grand_product_at_zeta_omega);
        last_permutation_part_at_z.mul_assign(state.alpha);

        // gnark implementation: add third part and sub second second part
        // plonk paper implementation: add second part and sub third part
        /*
        tmp_g1 = proof.grand_product_commitment.point_mul(grand_product_part_at_z);
        tmp_g1.point_sub_assign(vk.permutation_commitments[STATE_WIDTH - 1].point_mul(last_permutation_part_at_z));
        */
        // add to the linearization

        tmp_g1 = vk.permutation_commitments[STATE_WIDTH - 1].point_mul(last_permutation_part_at_z);
        tmp_g1.point_sub_assign(proof.grand_product_commitment.point_mul(grand_product_part_at_z));
        res.point_add_assign(tmp_g1);

        generate_uv_challenge(state, proof, vk, res);

        res.point_mul_assign(state.v);
        res.point_add_assign(proof.grand_product_commitment.point_mul(state.u));
    }

    // gnark v generation process:
    // sha256(zeta, proof.quotient_poly_commitments, linearizedPolynomialDigest, proof.wire_commitments, vk.permutation_commitments[0..1], )
    // NOTICE: gnark use "gamma" name for v, it's not reasonable
    // NOTICE: gnark use zeta^(n+2) which is a bit different with plonk paper
    // generate_v_challenge();
    function generate_uv_challenge(
        PartialVerifierState memory state,
        Proof memory proof,
        VerificationKey memory vk,
        PairingsBn254.G1Point memory linearization_point) view internal {
        TranscriptLibrary.Transcript memory transcript = TranscriptLibrary.new_transcript();
        transcript.set_challenge_name("gamma");
        transcript.update_with_fr(state.zeta);
        PairingsBn254.Fr memory zeta_plus_two = PairingsBn254.copy(state.zeta);
        PairingsBn254.Fr memory n_plus_two = PairingsBn254.new_fr(vk.domain_size);
        n_plus_two.add_assign(PairingsBn254.new_fr(2));
        zeta_plus_two = zeta_plus_two.pow(n_plus_two.value);
        state.cached_fold_quotient_ploy_commitments = PairingsBn254.copy_g1(proof.quotient_poly_commitments[STATE_WIDTH-1]);
        for (uint256 i = 0; i < STATE_WIDTH - 1; i++) {
            state.cached_fold_quotient_ploy_commitments.point_mul_assign(zeta_plus_two);
            state.cached_fold_quotient_ploy_commitments.point_add_assign(proof.quotient_poly_commitments[STATE_WIDTH - 2 - i]);
        }
        transcript.update_with_g1(state.cached_fold_quotient_ploy_commitments);
        transcript.update_with_g1(linearization_point);

        for (uint256 i = 0; i < proof.wire_commitments.length; i++) {
            transcript.update_with_g1(proof.wire_commitments[i]);
        }
        for (uint256 i = 0; i < vk.permutation_commitments.length - 1; i++) {
            transcript.update_with_g1(vk.permutation_commitments[i]);
        }
        state.v = transcript.get_challenge();
        // gnark use local randomness to generate u
        // we use opening_at_zeta_proof and opening_at_zeta_omega_proof
        transcript.set_challenge_name("u");
        transcript.update_with_g1(proof.opening_at_zeta_proof);
        transcript.update_with_g1(proof.opening_at_zeta_omega_proof);
        state.u = transcript.get_challenge();
    }

    function batch_evaluate_lagrange_poly_out_of_domain(
        uint256[] memory poly_nums,
        uint256 domain_size,
        PairingsBn254.Fr memory omega,
        PairingsBn254.Fr memory at
    ) internal view returns (PairingsBn254.Fr[] memory res) {
        PairingsBn254.Fr memory one = PairingsBn254.new_fr(1);
        PairingsBn254.Fr memory tmp_1 = PairingsBn254.new_fr(0);
        PairingsBn254.Fr memory tmp_2 = PairingsBn254.new_fr(domain_size);
        PairingsBn254.Fr memory vanishing_at_zeta = at.pow(domain_size);
        vanishing_at_zeta.sub_assign(one);
        // we can not have random point z be in domain
        require(vanishing_at_zeta.value != 0);
        PairingsBn254.Fr[] memory nums = new PairingsBn254.Fr[](poly_nums.length);
        PairingsBn254.Fr[] memory dens = new PairingsBn254.Fr[](poly_nums.length);
        // numerators in a form omega^i * (z^n - 1)
        // denoms in a form (z - omega^i) * N
        for (uint i = 0; i < poly_nums.length; i++) {
            tmp_1 = omega.pow(poly_nums[i]); // power of omega
            nums[i].assign(vanishing_at_zeta);
            nums[i].mul_assign(tmp_1);

            dens[i].assign(at); // (X - omega^i) * N
            dens[i].sub_assign(tmp_1);
            dens[i].mul_assign(tmp_2); // mul by domain size
        }

        PairingsBn254.Fr[] memory partial_products = new PairingsBn254.Fr[](poly_nums.length);
        partial_products[0].assign(PairingsBn254.new_fr(1));
        for (uint i = 1; i < dens.length; i++) {
            partial_products[i].assign(dens[i-1]);
            partial_products[i].mul_assign(partial_products[i-1]);
        }

        tmp_2.assign(partial_products[partial_products.length - 1]);
        tmp_2.mul_assign(dens[dens.length - 1]);
        tmp_2 = tmp_2.inverse(); // tmp_2 contains a^-1 * b^-1 (with! the last one)

        for (uint i = dens.length; i > 0; i--) {
            tmp_1.assign(tmp_2); // all inversed
            tmp_1.mul_assign(partial_products[i-1]); // clear lowest terms
            tmp_2.mul_assign(dens[i-1]);
            dens[i-1].assign(tmp_1);
        }

        for (uint i = 0; i < nums.length; i++) {
            nums[i].mul_assign(dens[i]);
        }

        return nums;
    }

    // plonk paper verify process step8: Compute quotient polynomial evaluation
    function verify_quotient_poly_eval_at_zeta(
        PartialVerifierState memory state,
        Proof memory proof,
        VerificationKey memory vk
    ) internal view returns (bool) {
        PairingsBn254.Fr memory lhs = evaluate_vanishing(vk.domain_size, state.zeta);
        require(lhs.value != 0); // we can not check a polynomial relationship if point z is in the domain
        lhs.mul_assign(proof.quotient_polynomial_at_zeta);

        PairingsBn254.Fr memory quotient_challenge = PairingsBn254.new_fr(1);
        PairingsBn254.Fr memory rhs = PairingsBn254.copy(proof.linearization_polynomial_at_zeta);

        // public inputs
        PairingsBn254.Fr memory tmp = PairingsBn254.new_fr(0);
        for (uint256 i = 0; i < proof.input_values.length; i++) {
            tmp.assign(state.cached_lagrange_evals[i]);
            tmp.mul_assign(PairingsBn254.new_fr(proof.input_values[i]));
            rhs.add_assign(tmp);
        }

        quotient_challenge.mul_assign(state.alpha);

        PairingsBn254.Fr memory z_part = PairingsBn254.copy(proof.grand_product_at_zeta_omega);
        for (uint256 i = 0; i < proof.permutation_polynomials_at_zeta.length; i++) {
            tmp.assign(proof.permutation_polynomials_at_zeta[i]);
            tmp.mul_assign(state.beta);
            tmp.add_assign(state.gamma);
            tmp.add_assign(proof.wire_values_at_zeta[i]);

            z_part.mul_assign(tmp);
        }

        tmp.assign(state.gamma);
        // we need a wire value of the last polynomial in enumeration
        tmp.add_assign(proof.wire_values_at_zeta[STATE_WIDTH - 1]);

        z_part.mul_assign(tmp);
        z_part.mul_assign(quotient_challenge);

        // NOTICE: this is different with plonk paper
        // plonk paper should be: rhs.sub_assign(z_part);
        rhs.add_assign(z_part);

        quotient_challenge.mul_assign(state.alpha);

        tmp.assign(state.cached_lagrange_evals[0]);
        tmp.mul_assign(quotient_challenge);

        rhs.sub_assign(tmp);

        return lhs.value == rhs.value;
    }

    function evaluate_vanishing(
        uint256 domain_size,
        PairingsBn254.Fr memory at
    ) internal view returns (PairingsBn254.Fr memory res) {
        res = at.pow(domain_size);
        res.sub_assign(PairingsBn254.new_fr(1));
    }

	// This verifier is for a PLONK with a state width 3
    // and main gate equation
    // q_a(X) * a(X) + 
    // q_b(X) * b(X) + 
    // q_c(X) * c(X) +
    // q_m(X) * a(X) * b(X) + 
    // q_constants(X)+
    // where q_{}(X) are selectors a, b, c - state (witness) polynomials
    
    function verify(Proof memory proof, VerificationKey memory vk) internal view returns (bool) {
        PartialVerifierState memory state;
        
        bool valid = verify_initial(state, proof, vk);
        
        if (valid == false) {
            return false;
        }
        
        valid = verify_commitments(state, proof, vk);
        
        return valid;
    }
}

contract KeyedPlonkVerifier is PlonkVerifier {
    uint256 constant SERIALIZED_PROOF_LENGTH = 26;
	using PairingsBn254 for PairingsBn254.Fr;
    function get_verification_key() internal pure returns(VerificationKey memory vk) {
        vk.domain_size = {{.Size}};
        vk.num_inputs = {{.NbPublicVariables}};
        vk.omega = PairingsBn254.new_fr(uint256({{.Generator.String}}));
        vk.selector_commitments[0] = PairingsBn254.new_g1(
        	uint256({{.Ql.X.String}}),
        	uint256({{.Ql.Y.String}})
        );
        vk.selector_commitments[1] = PairingsBn254.new_g1(
			uint256({{.Qr.X.String}}),
			uint256({{.Qr.Y.String}})
        );
        vk.selector_commitments[2] = PairingsBn254.new_g1(
			uint256({{.Qo.X.String}}),
			uint256({{.Qo.Y.String}})
        );
        vk.selector_commitments[3] = PairingsBn254.new_g1(
			uint256({{.Qm.X.String}}),
			uint256({{.Qm.Y.String}})
        );
        vk.selector_commitments[4] = PairingsBn254.new_g1(
			uint256({{.Qk.X.String}}),
			uint256({{.Qk.Y.String}})
        );

        vk.permutation_commitments[0] = PairingsBn254.new_g1(
        	uint256({{(index .S 0).X.String}}),
			uint256({{(index .S 0).Y.String}})
        );
        vk.permutation_commitments[1] = PairingsBn254.new_g1(
			uint256({{(index .S 1).X.String}}),
			uint256({{(index .S 1).Y.String}})
        );
        vk.permutation_commitments[2] = PairingsBn254.new_g1(
			uint256({{(index .S 2).X.String}}),
			uint256({{(index .S 2).Y.String}})
        );

        vk.permutation_non_residues[0] = PairingsBn254.new_fr(
        	uint256({{.CosetShift.String}})
        );
        vk.permutation_non_residues[1] = PairingsBn254.copy(
			vk.permutation_non_residues[0]
        );
		vk.permutation_non_residues[1].mul_assign(vk.permutation_non_residues[0]);

        vk.g2_x = PairingsBn254.new_g2(
			[uint256({{(index .KZGSRS.G2 1).X.A1.String}}),
			uint256({{(index .KZGSRS.G2 1).X.A0.String}})],
			[uint256({{(index .KZGSRS.G2 1).Y.A1.String}}),
			uint256({{(index .KZGSRS.G2 1).Y.A0.String}})]
        );
    }


    function deserialize_proof(
        uint256[] memory public_inputs,
        uint256[] memory serialized_proof
    ) internal pure returns(Proof memory proof) {
        require(serialized_proof.length == SERIALIZED_PROOF_LENGTH);
        proof.input_values = new uint256[](public_inputs.length);
        for (uint256 i = 0; i < public_inputs.length; i++) {
            proof.input_values[i] = public_inputs[i];
        }

        uint256 j = 0;
        for (uint256 i = 0; i < STATE_WIDTH; i++) {
            proof.wire_commitments[i] = PairingsBn254.new_g1_checked(
                serialized_proof[j],
                serialized_proof[j+1]
            );

            j += 2;
        }

        proof.grand_product_commitment = PairingsBn254.new_g1_checked(
            serialized_proof[j],
            serialized_proof[j+1]
        );
        j += 2;

        for (uint256 i = 0; i < STATE_WIDTH; i++) {
            proof.quotient_poly_commitments[i] = PairingsBn254.new_g1_checked(
                serialized_proof[j],
                serialized_proof[j+1]
            );

            j += 2;
        }

        for (uint256 i = 0; i < STATE_WIDTH; i++) {
            proof.wire_values_at_zeta[i] = PairingsBn254.new_fr(
                serialized_proof[j]
            );

            j += 1;
        }

        proof.grand_product_at_zeta_omega = PairingsBn254.new_fr(
            serialized_proof[j]
        );

        j += 1;

        proof.quotient_polynomial_at_zeta = PairingsBn254.new_fr(
            serialized_proof[j]
        );

        j += 1;

        proof.linearization_polynomial_at_zeta = PairingsBn254.new_fr(
            serialized_proof[j]
        );

        j += 1;

        for (uint256 i = 0; i < proof.permutation_polynomials_at_zeta.length; i++) {
            proof.permutation_polynomials_at_zeta[i] = PairingsBn254.new_fr(
                serialized_proof[j]
            );

            j += 1;
        }

        proof.opening_at_zeta_proof = PairingsBn254.new_g1_checked(
            serialized_proof[j],
            serialized_proof[j+1]
        );
        j += 2;

        proof.opening_at_zeta_omega_proof = PairingsBn254.new_g1_checked(
            serialized_proof[j],
            serialized_proof[j+1]
        );
    }

    function verify_serialized_proof(
        uint256[] memory public_inputs,
        uint256[] memory serialized_proof
    ) public view returns (bool) {
        VerificationKey memory vk = get_verification_key();
        require(vk.num_inputs == public_inputs.length);
        Proof memory proof = deserialize_proof(public_inputs, serialized_proof);
        bool valid = verify(proof, vk);
        return valid;
    }
}
`
