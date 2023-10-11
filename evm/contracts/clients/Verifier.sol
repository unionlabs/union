pragma solidity ^0.8.21;

import "../lib/Pairing.sol";
import "../core/IZKVerifierV2.sol";

contract Verifier is IZKVerifierV2 {
    using Pairing for *;

    uint256 constant SNARK_SCALAR_FIELD =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;
    uint256 constant PRIME_Q =
        21888242871839275222246405745257275088696311157297823662689037894645226208583;

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
        vk.alfa1 = Pairing.G1Point(
            uint256(
                2550450311668052934365492757435655272086248044926623610031063320730829667555
            ),
            uint256(
                8703042035341333540858177383145184333521890832996178936720364709928918567324
            )
        );
        vk.beta2 = Pairing.G2Point(
            [
                uint256(
                    12087544808749353461394614734663969995991213332932322707094948493323523587075
                ),
                uint256(
                    5609784734408442342743459772838249525064711794655914581473900530365072178092
                )
            ],
            [
                uint256(
                    3620271307123320035058604702546780432892661890675938601781290078906891602600
                ),
                uint256(
                    12561749020549188140474766670178579480395789909983438907997458237645471245637
                )
            ]
        );
        vk.gamma2 = Pairing.G2Point(
            [
                uint256(
                    16152858558628938416519886954198273278207481765134586011504238844337112405492
                ),
                uint256(
                    1666473556802178727950421359638955559724204816316988428054967258707482978937
                )
            ],
            [
                uint256(
                    11671729608220716829286816261707767434127200387281583853687528523414420405187
                ),
                uint256(
                    11529896311426611528742842264352476302384772641871656355304253747472884088839
                )
            ]
        );
        vk.delta2 = Pairing.G2Point(
            [
                uint256(
                    1978394675920979727010360476867770152431151269630391960275282752505331298298
                ),
                uint256(
                    8057837855735530867774111909354816419959310377834316597048249385367299183616
                )
            ],
            [
                uint256(
                    617817208407359897149661494204930461401582985150453668270265136103096748375
                ),
                uint256(
                    20112109139090099709565952821220817370470570950311800907839011307723047316048
                )
            ]
        );
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
        uint256[5] calldata input,
        uint256[2] memory proofCommitment
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
            require(
                input[i] < SNARK_SCALAR_FIELD,
                "verifier-gte-snark-scalar-field"
            );
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

        vk_x.X = uint256(
            19784694582259872339546543192603616109827470778459686418308124157049352412391
        ); // vk.K[0].X
        vk_x.Y = uint256(
            8050675694081171924276577880525281636790024000613356369940397902103623997045
        ); // vk.K[0].Y
        add_input[0] = vk_x.X;
        add_input[1] = vk_x.Y;
        add_input[2] = proofCommitment[0];
        add_input[3] = proofCommitment[1];
        Pairing.plus_raw(add_input, vk_x);
        mul_input[0] = uint256(
            11971204246816059209275294976507473955184279486240207887069942049166626280793
        ); // vk.K[1].X
        mul_input[1] = uint256(
            17504727181738195812829690872641417282627885783178145006033783186285339904875
        ); // vk.K[1].Y
        mul_input[2] = input[0];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[1] * input[0]
        mul_input[0] = uint256(
            11657740610894456804295347994135540407849394439286467178511726083546573935190
        ); // vk.K[2].X
        mul_input[1] = uint256(
            15322460840551705023577477711103730789391402917721093431052634068176417470830
        ); // vk.K[2].Y
        mul_input[2] = input[1];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[2] * input[1]
        mul_input[0] = uint256(
            18128705705780345472226931423901518026915070087276737749651398231700428372595
        ); // vk.K[3].X
        mul_input[1] = uint256(
            3028226418589796602201694314311238770080143896157409106468825846675506179552
        ); // vk.K[3].Y
        mul_input[2] = input[2];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[3] * input[2]
        mul_input[0] = uint256(
            4197945774141365264567395738908678390136055541810363021270211823598396886106
        ); // vk.K[4].X
        mul_input[1] = uint256(
            784961274727179063355557729271941546761163202680786306491941279774542819927
        ); // vk.K[4].Y
        mul_input[2] = input[3];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[4] * input[3]
        mul_input[0] = uint256(
            8766677265255536993942124635649738274909387559882289319459250062942891332208
        ); // vk.K[5].X
        mul_input[1] = uint256(
            4058621307841112497722744752034571789854603453063040480920996807162994795066
        ); // vk.K[5].Y
        mul_input[2] = input[4];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[5] * input[4]

        return
            Pairing.pairing(
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
