pragma solidity ^0.8.23;

import "../lib/Pairing.sol";
import "../core/IZKVerifier.sol";

contract TestVerifier is IZKVerifier {
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
                1125671039621810089428171450188893727232365974023656248475833836938972484667
            ),
            uint256(
                16708059979227448575127783600120361097122367964683038521677601550114485714490
            )
        );
        vk.beta2 = Pairing.G2Point(
            [
                uint256(
                    16470019197094627162299087301831442275638218677130007222670205426156957100865
                ),
                uint256(
                    14958059389656804321793202880891013104349254812250653654254419540045874479613
                )
            ],
            [
                uint256(
                    6462132794450663952942967898368307576456675700257099429655713611570640619298
                ),
                uint256(
                    17123404116076159959762469925340895612001057027173989919807096071077171764514
                )
            ]
        );
        vk.gamma2 = Pairing.G2Point(
            [
                uint256(
                    11418548283192814327818020566212000543664079199800580120911544819799726125734
                ),
                uint256(
                    5920641909552990144642456894725912706474505958697724624280513022734799073628
                )
            ],
            [
                uint256(
                    13424292823323948119718142680493017498112842428096491321197851435006803504886
                ),
                uint256(
                    9535504282195516941649015444320702490439814259887213402473830583601855984569
                )
            ]
        );
        vk.delta2 = Pairing.G2Point(
            [
                uint256(
                    11704745517448317616837684982261126400037964268545711184519502786602539467064
                ),
                uint256(
                    16888938809769781383314095342387355965362215790775205037422449393153218364207
                )
            ],
            [
                uint256(
                    8414763178397780322774566660669074914723220818750198492373048213419596194001
                ),
                uint256(
                    13109301182837003690701475443653586480186945433257156209129736993852895513512
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
            18767338542014411857313993782381800397387924826542899835474681706360337468968
        ); // vk.K[0].X
        vk_x.Y = uint256(
            17085667352205594924878160128155941314151724412939678808952186650026637982496
        ); // vk.K[0].Y
        add_input[0] = vk_x.X;
        add_input[1] = vk_x.Y;
        add_input[2] = input[7];
        add_input[3] = input[8];
        Pairing.plus_raw(add_input, vk_x);
        mul_input[0] = uint256(
            12504772314995318667085598268489910500204762055609759048236479468946268434715
        ); // vk.K[1].X
        mul_input[1] = uint256(
            8983729810354454237350709663955361486255804349684341343883945534227372772583
        ); // vk.K[1].Y
        mul_input[2] = input[0];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[1] * input[0]
        mul_input[0] = uint256(
            21771893660210241160978984895410580596702195089317438720917623130541287562951
        ); // vk.K[2].X
        mul_input[1] = uint256(
            6826939490233329137937827410099630746030273673600789410359384510477424990528
        ); // vk.K[2].Y
        mul_input[2] = input[1];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[2] * input[1]
        mul_input[0] = uint256(
            13483374748428281059028219899599319192039698231059011030408926432855884913059
        ); // vk.K[3].X
        mul_input[1] = uint256(
            1580546223836162786043487702113163279370733460000671658613748752775709896738
        ); // vk.K[3].Y
        mul_input[2] = input[2];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[3] * input[2]
        mul_input[0] = uint256(
            878844219607870183522904288494135794814180324863533284370581898731325763878
        ); // vk.K[4].X
        mul_input[1] = uint256(
            5029967920102374989452979629721980248435538706167040492456898113056724473757
        ); // vk.K[4].Y
        mul_input[2] = input[3];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[4] * input[3]
        mul_input[0] = uint256(
            19178552108232008347712559118598665047168074310079689947325291874527681136632
        ); // vk.K[5].X
        mul_input[1] = uint256(
            4887450214492350443087393893730872155113124252746183476199372947155640792254
        ); // vk.K[5].Y
        mul_input[2] = input[4];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[5] * input[4]
        mul_input[0] = uint256(
            4357893561187570586884283989859896948424540980085231995327372677863129685190
        ); // vk.K[6].X
        mul_input[1] = uint256(
            17366381213742021981454676402842182355603824163159232749914711635677740694004
        ); // vk.K[6].Y
        mul_input[2] = input[5];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[6] * input[5]
        mul_input[0] = uint256(
            11447136153403079013535596580526837954163849580215646916316817462954879467374
        ); // vk.K[7].X
        mul_input[1] = uint256(
            4022142721279929532160588989277562498040471874620641926810806863247508170238
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
