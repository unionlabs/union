pragma solidity ^0.8.18;

import "../lib/Pairing.sol";
import "../core/IZKVerifier.sol";

contract TestnetVerifier is IZKVerifier {

    using Pairing for *;

    uint256 constant SNARK_SCALAR_FIELD = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    uint256 constant PRIME_Q = 21888242871839275222246405745257275088696311157297823662689037894645226208583;

    struct VerifyingKey {
        Pairing.G1Point alfa1;
        Pairing.G2Point beta2;
        Pairing.G2Point gamma2;
        Pairing.G2Point delta2;
        // []G1Point IC (K in gnark) appears directly in verifyProof
    }

    struct Proof {
        Pairing.G1Point A;
        Pairing.G2Point B;
        Pairing.G1Point C;
    }

    function verifyingKey() internal pure returns (VerifyingKey memory vk) {
        vk.alfa1 = Pairing.G1Point(uint256(5835940158026092252572603517965378162095920253204704423035997036003370201014), uint256(16449855594989795714415573492407523904056159085613857684826358922070007311996));
        vk.beta2 = Pairing.G2Point([uint256(19132236660847243801398814411430747806453429822256621338672944060353867887056), uint256(2266554991113955616918887317322988455486202000110289182110209614228355564026)], [uint256(18082505845361896632368017548190605712874150972684307775447464089621573411497), uint256(6241851243760010177877712852219703052033735740050931065325140922388365843939)]);
        vk.gamma2 = Pairing.G2Point([uint256(2532428574076657711525901954590634715230332558178406116433605001728655642282), uint256(18920289600768724504261538295747488937624940272558269708660913320171103552035)], [uint256(11890950509256353511009703805440629333771576420699064404657743985938553845735), uint256(3999042369684028678785104627187824783143178387758140735779793408530719707160)]);
        vk.delta2 = Pairing.G2Point([uint256(15070739233107706877915368520458135538587039829312653680651946318487523731818), uint256(3214862025832785621449398727946442309886635296506547580212810952773081386032)], [uint256(7776209404554264758510604510342259327608300862458319948874239088080052899701), uint256(13049271793021017843301284444457022664211874821129497233932282350669741702763)]);
    }


    // accumulate scalarMul(mul_input) into q
    // that is computes sets q = (mul_input[0:2] * mul_input[3]) + q
    function accumulate(
        uint256[3] memory mul_input,
        Pairing.G1Point memory p,
        uint256[4] memory buffer,
        Pairing.G1Point memory q
    ) internal view {
        // computes p = mul_input[0:2] * mul_input[3]
        Pairing.scalar_mul_raw(mul_input, p);

        // point addition inputs
        buffer[0] = q.X;
        buffer[1] = q.Y;
        buffer[2] = p.X;
        buffer[3] = p.Y;

        // q = p + q
        Pairing.plus_raw(buffer, q);
    }

    /*
     * @returns Whether the proof is valid given the hardcoded verifying key
     *          above and the public inputs
     */
    function verifyProof(
        uint256[2] memory a,
        uint256[2][2] memory b,
        uint256[2] memory c,
        uint256[9] calldata input
    ) public view returns (bool r) {

        Proof memory proof;
        proof.A = Pairing.G1Point(a[0], a[1]);
        proof.B = Pairing.G2Point([b[0][0], b[0][1]], [b[1][0], b[1][1]]);
        proof.C = Pairing.G1Point(c[0], c[1]);

        // Make sure that proof.A, B, and C are each less than the prime q
        require(proof.A.X < PRIME_Q, "verifier-aX-gte-prime-q");
        require(proof.A.Y < PRIME_Q, "verifier-aY-gte-prime-q");

        require(proof.B.X[0] < PRIME_Q, "verifier-bX0-gte-prime-q");
        require(proof.B.Y[0] < PRIME_Q, "verifier-bY0-gte-prime-q");

        require(proof.B.X[1] < PRIME_Q, "verifier-bX1-gte-prime-q");
        require(proof.B.Y[1] < PRIME_Q, "verifier-bY1-gte-prime-q");

        require(proof.C.X < PRIME_Q, "verifier-cX-gte-prime-q");
        require(proof.C.Y < PRIME_Q, "verifier-cY-gte-prime-q");

        // Make sure that every input is less than the snark scalar field
        for (uint256 i = 0; i < input.length; i++) {
            require(input[i] < SNARK_SCALAR_FIELD,"verifier-gte-snark-scalar-field");
        }

        VerifyingKey memory vk = verifyingKey();

        // Compute the linear combination vk_x
        Pairing.G1Point memory vk_x = Pairing.G1Point(0, 0);

        // Buffer reused for addition p1 + p2 to avoid memory allocations
        // [0:2] -> p1.X, p1.Y ; [2:4] -> p2.X, p2.Y
        uint256[4] memory add_input;

        // Buffer reused for multiplication p1 * s
        // [0:2] -> p1.X, p1.Y ; [3] -> s
        uint256[3] memory mul_input;

        // temporary point to avoid extra allocations in accumulate
        Pairing.G1Point memory q = Pairing.G1Point(0, 0);

        vk_x.X = uint256(12268064097101875737604099380317138142384195991010023286086437035019893800855); // vk.K[0].X
        vk_x.Y = uint256(4230429577015320654122981675328732661587150508056545953627024879417437905652); // vk.K[0].Y
        add_input[0] = vk_x.X;
        add_input[1] = vk_x.Y;
        add_input[2] = input[7];
        add_input[3] = input[8];
        Pairing.plus_raw(add_input, vk_x);
        mul_input[0] = uint256(1944011650582036417224824057085729541534925604351430685937715493252860437033); // vk.K[1].X
        mul_input[1] = uint256(17980669863758152324772991319186044760724551897940525141234748278938748523664); // vk.K[1].Y
        mul_input[2] = input[0];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[1] * input[0]
        mul_input[0] = uint256(12462823491887262508942870795006050444148690329933152347077998120850046664332); // vk.K[2].X
        mul_input[1] = uint256(21001971559375891843542700020915179711645252614110654091339543102924879125778); // vk.K[2].Y
        mul_input[2] = input[1];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[2] * input[1]
        mul_input[0] = uint256(4676717730808650177128481229894526479855836761044423008983822751731651883658); // vk.K[3].X
        mul_input[1] = uint256(2842267579735072137836465030680298258987874184873859162539127909056845222770); // vk.K[3].Y
        mul_input[2] = input[2];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[3] * input[2]
        mul_input[0] = uint256(19694349305035834412875047546183290607723392591043471921589273404565619064468); // vk.K[4].X
        mul_input[1] = uint256(19865849274783742598171237697573391273876383738955067557978847663418647822207); // vk.K[4].Y
        mul_input[2] = input[3];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[4] * input[3]
        mul_input[0] = uint256(12319603886879721748905854075405652966880596708911179444995314720242824029426); // vk.K[5].X
        mul_input[1] = uint256(13697978547564806829653988299630480159718372624477588607025086088375602026471); // vk.K[5].Y
        mul_input[2] = input[4];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[5] * input[4]
        mul_input[0] = uint256(10123178217510811215964071605496997763800877437074826304254411838461407598978); // vk.K[6].X
        mul_input[1] = uint256(20389054878598126562226064045825510373852840545088878376413087325126667103749); // vk.K[6].Y
        mul_input[2] = input[5];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[6] * input[5]
        mul_input[0] = uint256(18511248445418417983555248447772711615276820474119821788814529126279504330766); // vk.K[7].X
        mul_input[1] = uint256(5009251946215151482475294618671424092610000001038116426803741056043795163607); // vk.K[7].Y
        mul_input[2] = input[6];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[7] * input[6]

        return Pairing.pairing(
            Pairing.negate(proof.A),
            proof.B,
            vk.alfa1,
            vk.beta2,
            vk_x,
            vk.gamma2,
            proof.C,
            vk.delta2
        );
    }
}
