pragma solidity ^0.8.18;

import "../lib/Pairing.sol";
import "../core/IZKVerifier.sol";

contract DevnetVerifier is IZKVerifier {
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
                9974399132350238449672423145167802132344597176432790937987673566759904354712
            ),
            uint256(
                10396217607362300103655122228113983820745493114140883199476303464408811706471
            )
        );
        vk.beta2 = Pairing.G2Point(
            [
                uint256(
                    20043334460449324572644561653520106968487299991365945714189067590923833559557
                ),
                uint256(
                    3782843380964690766572041754552260909078546283792951053210110465664576118592
                )
            ],
            [
                uint256(
                    4546441854933490265510538123407299251387870046105247930781926195493537303978
                ),
                uint256(
                    19728170969753285624598425791670262520539566544285475380089632156164753610432
                )
            ]
        );
        vk.gamma2 = Pairing.G2Point(
            [
                uint256(
                    15890984819252760833184574925585572560291816058221856734884092043888365097798
                ),
                uint256(
                    13558421301005029939663494790802233493306340917537858200716018199215933051901
                )
            ],
            [
                uint256(
                    8951430351447595274973237553867518771312837295026859105316664000150429223102
                ),
                uint256(
                    9774001800913153454819154173343364291874345033268265728436390595601923216347
                )
            ]
        );
        vk.delta2 = Pairing.G2Point(
            [
                uint256(
                    12986197120328725341178217804701057807111123287171378211441714126957192190146
                ),
                uint256(
                    6358811827968308311530932341706580062890352807488954904621603172031504605990
                )
            ],
            [
                uint256(
                    11008158088064515525514471643307844090914721261889690007083299787176620957920
                ),
                uint256(
                    1693569334322206029064251989727378784021135124521538321367277282710375215047
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
            19536632576810938663755749582603546087222638703180042738513910916616519682978
        ); // vk.K[0].X
        vk_x.Y = uint256(
            6698373882991209028988452302062701111718632869665993606888642342207051892975
        ); // vk.K[0].Y
        add_input[0] = vk_x.X;
        add_input[1] = vk_x.Y;
        add_input[2] = input[7];
        add_input[3] = input[8];
        Pairing.plus_raw(add_input, vk_x);
        mul_input[0] = uint256(
            2175229953105907030386086995813356912474746827628735806700482499160750847843
        ); // vk.K[1].X
        mul_input[1] = uint256(
            19823529752927772060409556160428145736017998454909758668189685129708026335065
        ); // vk.K[1].Y
        mul_input[2] = input[0];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[1] * input[0]
        mul_input[0] = uint256(
            219999636782629863970338640713754993296807671982705311132408472476488701731
        ); // vk.K[2].X
        mul_input[1] = uint256(
            11582191598571666113262523487623760501658738560317219321241346601375876165826
        ); // vk.K[2].Y
        mul_input[2] = input[1];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[2] * input[1]
        mul_input[0] = uint256(
            914394202216898966177299917746741778977940677187377639141420924936000943248
        ); // vk.K[3].X
        mul_input[1] = uint256(
            8726710514357051704626909121942479242019757832647898014481949563241929367905
        ); // vk.K[3].Y
        mul_input[2] = input[2];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[3] * input[2]
        mul_input[0] = uint256(
            410530762185814800540583115824275203642834613850491151197240739569603959187
        ); // vk.K[4].X
        mul_input[1] = uint256(
            5236570818789858673799951129197614899816105385688852546833382795763613513196
        ); // vk.K[4].Y
        mul_input[2] = input[3];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[4] * input[3]
        mul_input[0] = uint256(
            13915041915789362048532482320640272960446035437675260680928350524425298814782
        ); // vk.K[5].X
        mul_input[1] = uint256(
            4402873937379531482689066168118493057889537848402358898771477872149907606547
        ); // vk.K[5].Y
        mul_input[2] = input[4];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[5] * input[4]
        mul_input[0] = uint256(
            13581186194999365488187594952848464234662365346750156291065587645871146629135
        ); // vk.K[6].X
        mul_input[1] = uint256(
            3315129916730707978366419309543430205662621536944423844626436472087928543555
        ); // vk.K[6].Y
        mul_input[2] = input[5];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[6] * input[5]
        mul_input[0] = uint256(
            13424886217355741741339780135239743700963066884441662888102595067007931455321
        ); // vk.K[7].X
        mul_input[1] = uint256(
            5117715757204980109056335794282910227777795377124703228976341447232892223753
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
