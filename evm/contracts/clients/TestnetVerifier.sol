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
        vk.alfa1 = Pairing.G1Point(uint256(10900365700008785951386810059031907651998862503081677518760135848615814781151), uint256(2813918487984701514495866538640757245795310074506986809086187064257311477759));
        vk.beta2 = Pairing.G2Point([uint256(18016272642940762206675806612642011201735472249051322975835744218682902713062), uint256(16357750055644777342678346173266324254774518645338614913789308235366025659928)], [uint256(19758911916407078708266271128615419628883042320273723741836641386408409501327), uint256(3554513121429906029657855924332897856861387724805388158790066103361201204210)]);
        vk.gamma2 = Pairing.G2Point([uint256(19512875184572867569120416814748692776010528469274793083343283869434864471759), uint256(2549029521023100058468087326457927716968672157213288863707354374956257266809)], [uint256(160761816387853926196115454931163650061973435705108488809338003258641051841), uint256(14034778810220884153712266420860367207624938118479081815653668110709504182639)]);
        vk.delta2 = Pairing.G2Point([uint256(10617559592538378308874786090228720967765022744707492676272766520035077403025), uint256(12652367968539648179486762187587112566335954262614105271143741065670049337363)], [uint256(5377251285991785595312700760925386441208533509670674647478120822710314216997), uint256(12369114061254266277902937238404080678011257369662724502280673834418158554593)]);
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

        vk_x.X = uint256(15306751257115996922046245789904024812171305466642942480064397212755507058688); // vk.K[0].X
        vk_x.Y = uint256(12759999967270786331387453373858589735971137181671080110315892782079676131874); // vk.K[0].Y
        add_input[0] = vk_x.X;
        add_input[1] = vk_x.Y;
        add_input[2] = input[7];
        add_input[3] = input[8];
        Pairing.plus_raw(add_input, vk_x);
        mul_input[0] = uint256(3548223831724705000195252499885090339557210575683668213682733289524309515359); // vk.K[1].X
        mul_input[1] = uint256(21632634098450235872830583689357432925039094162224165558266192563872100625705); // vk.K[1].Y
        mul_input[2] = input[0];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[1] * input[0]
        mul_input[0] = uint256(4321119025430213864295153592563322246832740792995914245200478999233805861628); // vk.K[2].X
        mul_input[1] = uint256(18616911277202834760914976241332791800643185228033392418467404146733793044730); // vk.K[2].Y
        mul_input[2] = input[1];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[2] * input[1]
        mul_input[0] = uint256(15685223153963246445919442544425267373966467610106703623226227387118544549432); // vk.K[3].X
        mul_input[1] = uint256(11473518119258233867048477098089798495777464828572892625319391999598481585842); // vk.K[3].Y
        mul_input[2] = input[2];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[3] * input[2]
        mul_input[0] = uint256(432736116405447307808714121564973089819517564416720461274423091613737786793); // vk.K[4].X
        mul_input[1] = uint256(19517500262453478353076874678858196315854073460273448312742058224018278386413); // vk.K[4].Y
        mul_input[2] = input[3];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[4] * input[3]
        mul_input[0] = uint256(4379712019656232179050299831603961143239683484019111611172386826702435579618); // vk.K[5].X
        mul_input[1] = uint256(6937816401058425690911323681153605066038715139288351049476647311903574154552); // vk.K[5].Y
        mul_input[2] = input[4];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[5] * input[4]
        mul_input[0] = uint256(21763089232083212206036601850568492720324890876918101295522159373088664991468); // vk.K[6].X
        mul_input[1] = uint256(20840260120803223232278551010977437167156113810525213523555749327205351100725); // vk.K[6].Y
        mul_input[2] = input[5];
        accumulate(mul_input, q, add_input, vk_x); // vk_x += vk.K[6] * input[5]
        mul_input[0] = uint256(12684646261113662286697672937698669711405584900450108554594447254498940598914); // vk.K[7].X
        mul_input[1] = uint256(2446400168549826690903655782917719879593427261438382262680944092055169957518); // vk.K[7].Y
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
