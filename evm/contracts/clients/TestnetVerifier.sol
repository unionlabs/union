pragma solidity ^0.8.18;

import "../lib/Pairing.sol";
import "../core/IZKVerifier.sol";

contract TestnetVerifier is IZKVerifier {
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
                20753981458927019466153957860079451018375214958180376032006284689982307844411
            ),
            uint256(
                261636491223027205180432940735188094436038293184967748243333673075073746864
            )
        );
        vk.beta2 = Pairing.G2Point(
            [
                uint256(
                    10563447260258244808699345930646260318261620629910432245923526023647363839236
                ),
                uint256(
                    19270738939845887318945688464283609624585720278349806041035065846792049510491
                )
            ],
            [
                uint256(
                    11793853856849629867577505902700628470533246086792466424213128519027929757018
                ),
                uint256(
                    1322453826053782595273719607689289695120385081456884844264101134215819859871
                )
            ]
        );
        vk.gamma2 = Pairing.G2Point(
            [
                uint256(
                    19931424113432822023217057010667879702979483157964757004460626017857620755514
                ),
                uint256(
                    2792729266857068653949369525947554095694842350164236264962558171073873011974
                )
            ],
            [
                uint256(
                    14706781277918543243688950001598636061029098535531493881388770302311088655737
                ),
                uint256(
                    17960830411148709723596035740448974203115210090243776330918372770245382903032
                )
            ]
        );
        vk.delta2 = Pairing.G2Point(
            [
                uint256(
                    18656115990271853213718698921968483532502617065587907702052387941392528297502
                ),
                uint256(
                    8773592451832452750137905766660715233239952719665611366502133436018075508465
                )
            ],
            [
                uint256(
                    9900476032394396276441706766306096779319209485925101745811561104098524924314
                ),
                uint256(
                    9188856336940275467328217160560015874791897996316114494200248773202188965513
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
            13485607458858639393083217751822021073477314065439275958784369588803104875697
        ); // vk.K[0].X
        vk_x.Y = uint256(
            19173401129976349636933481679236726317672872385524740687782014892700976539059
        ); // vk.K[0].Y
        add_input[0] = vk_x.X;
        add_input[1] = vk_x.Y;
        add_input[2] = input[7];
        add_input[3] = input[8];
        Pairing.plus_raw(add_input, vk_x);
        mul_input[0] = uint256(
            11578577938670609924008404196369521603517734357400088139112663646360364932213
        ); // vk.K[1].X
        mul_input[1] = uint256(
            21472633237099071287902960283439505035560710589336211087457038893958560687803
        ); // vk.K[1].Y
        mul_input[2] = input[0];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[1] * input[0]
        mul_input[0] = uint256(
            15062363730263544725717289489645889364818980936849585029159038041588591546105
        ); // vk.K[2].X
        mul_input[1] = uint256(
            19613647661392802334737483413257137106686678479418553884459140865133338818852
        ); // vk.K[2].Y
        mul_input[2] = input[1];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[2] * input[1]
        mul_input[0] = uint256(
            19682435837491946482111014757051337274820603244398249508281438223136512964528
        ); // vk.K[3].X
        mul_input[1] = uint256(
            20451998810488963270149097552026518558236632110067945193669893418403248061141
        ); // vk.K[3].Y
        mul_input[2] = input[2];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[3] * input[2]
        mul_input[0] = uint256(
            11466360020405181787693495082746131320756872039302595083123843448670671029396
        ); // vk.K[4].X
        mul_input[1] = uint256(
            6721718818755755220018851945285618991387801443435740374833010927371580204742
        ); // vk.K[4].Y
        mul_input[2] = input[3];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[4] * input[3]
        mul_input[0] = uint256(
            14991268137626951298220572592587738788010381335156619232523135312026141853200
        ); // vk.K[5].X
        mul_input[1] = uint256(
            75917828916766303907441220796556013259925635731782654212877519051137487179
        ); // vk.K[5].Y
        mul_input[2] = input[4];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[5] * input[4]
        mul_input[0] = uint256(
            16879398223138455672098119871230571429183587949894880831582180865431699929008
        ); // vk.K[6].X
        mul_input[1] = uint256(
            5748924997849784482983345963170266794001059838899347161323159413599425655344
        ); // vk.K[6].Y
        mul_input[2] = input[5];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[6] * input[5]
        mul_input[0] = uint256(
            12033810803556168471847152511630143784777986063528352102996189036466861876708
        ); // vk.K[7].X
        mul_input[1] = uint256(
            4403881910866240119593230582978956204148439659493046343382118555286509593316
        ); // vk.K[7].Y
        mul_input[2] = input[6];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[7] * input[6]

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
