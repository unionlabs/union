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
 vk.alfa1 = Pairing.G1Point(uint256(19966544675868254802306328237791711448503446711296110747766965560309604939222), uint256(15278058911816016734876175192997742329523519821675904869861297803392240959773));
        vk.beta2 = Pairing.G2Point([uint256(4998596399063779544854550043963573771281946913556092773698463056913197349569), uint256(14509861040404609732416802596326493608766169432154759106673826858945779055644)], [uint256(14786126329877706835538549458636848179143671603550648166295920694642153877650), uint256(3902148141442963311957129005424138885858520872434193296932451116294477681604)]);
        vk.gamma2 = Pairing.G2Point([uint256(13252421860758338307900532496747402173365542255427564556659949426489158686875), uint256(16569480311827748103499682710221497886774425224065712677377398981080993298920)], [uint256(14867133241751749182972755393781181148511450347783947717460713466748489135744), uint256(1276413176030638117434615953784157605759103460347736459526349802855221949102)]);
        vk.delta2 = Pairing.G2Point([uint256(5735642163353556914849649840740012008515238996498073340856698153341057442673), uint256(20824518396164318334510753967280155080841396867102797108929624238157351279735)], [uint256(19193308814364360728936175521920833793198709671912781556234854730621513163479), uint256(10901021076158673456077712764030066598550011375018690695408613051858276166029)]);
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

        vk_x.X = uint256(15042978064282729434097915736008899765423579458018809947050216274164745190132); // vk.K[0].X
        vk_x.Y = uint256(11265907977664336851590564849230170791122547491666730830779601478033278955015); // vk.K[0].Y
        add_input[0] = vk_x.X;
        add_input[1] = vk_x.Y;
        add_input[2] = input[7];
        add_input[3] = input[8];
        Pairing.plus_raw(add_input, vk_x);
        mul_input[0] = uint256(6661950666938820930831946999633959450339223758644464263439041847588260581762); // vk.K[1].X
        mul_input[1] = uint256(7101702247814098532438550109222433738752744616760069986801798463332530581236); // vk.K[1].Y
        mul_input[2] = input[0];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[1] * input[0]
        mul_input[0] = uint256(3361088075993600848803171111574286301457502019958220460469327271701093466494); // vk.K[2].X
        mul_input[1] = uint256(10844376657875979204181760342573049578703892051446675614545345170650433576654); // vk.K[2].Y
        mul_input[2] = input[1];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[2] * input[1]
        mul_input[0] = uint256(9395341113488405219712359767704228346515521645642079227280352555975608594460); // vk.K[3].X
        mul_input[1] = uint256(2521747230977607656643302565380475284728112755389405554628239695354426653691); // vk.K[3].Y
        mul_input[2] = input[2];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[3] * input[2]
        mul_input[0] = uint256(20217714038863009315525049306889764373596080384594829845199559493820034138167); // vk.K[4].X
        mul_input[1] = uint256(20397321140865771997651710267746756873321301366509823011477438523529831945205); // vk.K[4].Y
        mul_input[2] = input[3];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[4] * input[3]
        mul_input[0] = uint256(5583394884721240933791844155552742132928154683330351054455690746059523596635); // vk.K[5].X
        mul_input[1] = uint256(8081533536291741096153503754089712092686355908741959599393060630754250915242); // vk.K[5].Y
        mul_input[2] = input[4];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[5] * input[4]
        mul_input[0] = uint256(8478503024661454146786872982813163632109989670616367415628929026316507640863); // vk.K[6].X
        mul_input[1] = uint256(21223136866394463670639567156405271134413688045613995843555246395152320726067); // vk.K[6].Y
        mul_input[2] = input[5];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[6] * input[5]
        mul_input[0] = uint256(21878165551673409195732572693923698062459971277576933104826988614115205804787); // vk.K[7].X
        mul_input[1] = uint256(13777522905408606176260935041892542383870632009800876845023661981988538491525); // vk.K[7].Y
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
