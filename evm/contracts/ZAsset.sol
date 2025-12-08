pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/token/ERC20/ERC20Upgradeable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

import "solady/utils/LibBytes.sol";
import "solady/utils/SignatureCheckerLib.sol";

import "./UnionversalToken.sol";

contract ZAsset is UnionversalToken {
    using LibBytes for *;
    using SafeERC20 for *;

    error OnlyKnownLightClients();
    error NotWrapped();
    error TooManyLightClients();
    error ValueExceedScalarField();
    error ValueAfterWithdrawExceedsScalarField();
    error NullifierExceedScalarField();
    error AttestedMessageExceedsScalarField();
    error InvalidAttestationSig();
    error InvalidAttestation();

    event Redeemed(
        uint256 indexed nullifier,
        uint256 indexed redeemAmount,
        address indexed beneficiary
    );

    event Attested(
        bytes32 indexed attestedMessage, uint256 indexed redeemAmount
    );

    struct LightClientSource {
        uint32 clientId;
        uint64 height;
    }

    struct LightClientChoice {
        bytes32 stateRoot;
        bytes32 tokenAddressKey;
        bytes32 balanceSlot;
    }

    bytes32 internal constant ZAsset_STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("union.storage.zasset")) - 1)
    ) & ~bytes32(uint256(0xff));

    struct Counterparty {
        // keccak256(address)
        bytes32 tokenAddressKey;
        // slot of the balance mapping for the void companion
        bytes32 balanceSlot;
    }

    struct ZAssetStorage {
        // attestor signing deposit addresses
        address attestor;
        // union ibc handler
        address ibcHandler;
        // local chain id
        uint256 chainId;
        // whether this token is a wrapper
        address underlying;
        // light client counterparties
        mapping(uint32 => Counterparty) counterparties;
        // record used nullifiers
        mapping(uint256 => uint256) nullifierBalance;
        // light client consensus state state root index
        mapping(uint32 => uint256) stateRootIndex;
    }

    function _getZAssetStorage()
        private
        pure
        returns (ZAssetStorage storage $)
    {
        bytes32 slot = ZAsset_STORAGE_SLOT;
        assembly {
            $.slot := slot
        }
    }

    constructor() {
        _disableInitializers();
    }

    function initialize(
        address _authority,
        address _zkgm,
        string calldata _name,
        string calldata _symbol,
        uint8 _decimals,
        bytes calldata _salt,
        address _attestor,
        address _ibcHandler,
        uint256 _chainId,
        address _underlying
    ) external initializer {
        __UnionversalToken_init(
            _authority, _zkgm, _name, _symbol, _decimals, _salt
        );
        ZAssetStorage storage $ = _getZAssetStorage();
        $.attestor = _attestor;
        $.ibcHandler = _ibcHandler;
        $.chainId = _chainId;
        $.underlying = _underlying;
    }

    function underlying() external view returns (address) {
        return _getZAssetStorage().underlying;
    }

    function chainId() external view returns (uint256) {
        return _getZAssetStorage().chainId;
    }

    function ibcHandler() external view returns (address) {
        return _getZAssetStorage().ibcHandler;
    }

    function attestor() external view returns (address) {
        return _getZAssetStorage().attestor;
    }

    function nullifierBalance(
        uint256 nullifier
    ) external view returns (uint256) {
        return _getZAssetStorage().nullifierBalance[nullifier];
    }

    function counterparty(
        uint32 clientId
    ) external view returns (Counterparty memory) {
        return _getZAssetStorage().counterparties[clientId];
    }

    function stateRootIndex(
        uint32 clientId
    ) external view returns (uint256) {
        return _getZAssetStorage().stateRootIndex[clientId];
    }

    function setConfidentialCounterparty(
        uint32 clientId,
        Counterparty calldata counterpary
    ) external restricted {
        ZAssetStorage storage $ = _getZAssetStorage();
        $.counterparties[clientId] = counterpary;
    }

    function setStateRootIndex(
        uint32 clientId,
        uint256 stateRootIndex
    ) external restricted {
        ZAssetStorage storage $ = _getZAssetStorage();
        $.stateRootIndex[clientId] = stateRootIndex;
    }

    function setAttestor(
        address attestor
    ) external restricted {
        ZAssetStorage storage $ = _getZAssetStorage();
        $.attestor = attestor;
    }

    function setChainId(
        uint256 chainId
    ) external restricted {
        ZAssetStorage storage $ = _getZAssetStorage();
        $.chainId = chainId;
    }

    function deposit(
        uint256 amount
    ) external {
        ZAssetStorage storage $ = _getZAssetStorage();
        if ($.underlying == address(0)) revert NotWrapped();
        IERC20($.underlying).safeTransferFrom(msg.sender, address(this), amount);
        _mint(msg.sender, amount);
    }

    function withdraw(
        uint256 amount
    ) external {
        ZAssetStorage storage $ = _getZAssetStorage();
        if ($.underlying == address(0)) revert NotWrapped();
        _burn(msg.sender, amount);
        IERC20($.underlying).transfer(msg.sender, amount);
    }

    function redeem(
        uint256[8] calldata proof,
        uint256[2] calldata commitments,
        uint256[2] calldata commitmentPok,
        LightClientSource[] calldata lightClients,
        uint256 nullifier,
        uint256 redeemAmount,
        address beneficiary,
        bytes32 attestedMessage,
        bytes calldata signature,
        bool unwrap
    ) external {
        if (lightClients.length > 4) revert TooManyLightClients();

        if (redeemAmount >= Verifier.R) {
            revert ValueExceedScalarField();
        }
        if (nullifier >= Verifier.R) {
            revert NullifierExceedScalarField();
        }
        if (uint256(attestedMessage) >= Verifier.R) {
            revert InvalidAttestation();
        }

        ZAssetStorage storage $ = _getZAssetStorage();

        // Update redeemed amount
        uint256 alreadyRedeemed = $.nullifierBalance[nullifier];
        if (alreadyRedeemed + redeemAmount >= Verifier.R) {
            revert ValueAfterWithdrawExceedsScalarField();
        }
        $.nullifierBalance[nullifier] += redeemAmount;

        // Verify attestor knowledge of the deposit/beneficiary link
        if (
            !SignatureCheckerLib.isValidSignatureNowCalldata(
                $.attestor, attestedMessage, signature
            )
        ) {
            revert InvalidAttestationSig();
        }

        // Extract possible sources
        LightClientChoice[4] memory lightClientChoices;
        for (uint256 i = 0; i < lightClients.length; i++) {
            LightClientSource calldata lightClient = lightClients[i];
            Counterparty storage counterparty =
                $.counterparties[lightClient.clientId];
            if (counterparty.tokenAddressKey == 0) {
                revert OnlyKnownLightClients();
            }
            bytes memory consensusState = IBCStore($.ibcHandler).getClient(
                lightClient.clientId
            ).getConsensusState(lightClient.clientId, lightClient.height);
            uint256 stateRootIndex = $.stateRootIndex[lightClient.clientId];
            bytes32 stateRoot = bytes32(consensusState.load(stateRootIndex));
            lightClientChoices[i] = LightClientChoice({
                stateRoot: stateRoot,
                tokenAddressKey: counterparty.tokenAddressKey,
                balanceSlot: counterparty.balanceSlot
            });
        }

        bytes32 inputsHash = keccak256(
            abi.encode(
                lightClientChoices,
                $.chainId,
                nullifier,
                redeemAmount,
                alreadyRedeemed,
                attestedMessage,
                beneficiary
            )
        ) & 0x00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;

        Verifier.verifyProof(
            proof, commitments, commitmentPok, [uint256(inputsHash)]
        );

        if (unwrap) {
            if ($.underlying == address(0)) {
                revert NotWrapped();
            }
            IERC20($.underlying).transfer(beneficiary, redeemAmount);
        } else {
            _mint(beneficiary, redeemAmount);
        }

        emit Redeemed(nullifier, redeemAmount, beneficiary);
        emit Attested(attestedMessage, redeemAmount);
    }
}

library Verifier {
    /// Some of the provided public input values are larger than the field modulus.
    /// @dev Public input elements are not automatically reduced, as this is can be
    /// a dangerous source of bugs.
    error PublicInputNotInField();

    /// The proof is invalid.
    /// @dev This can mean that provided Groth16 proof points are not on their
    /// curves, that pairing equation fails, or that the proof is not for the
    /// provided public input.
    error ProofInvalid();
    /// The commitment is invalid
    /// @dev This can mean that provided commitment points and/or proof of knowledge are not on their
    /// curves, that pairing equation fails, or that the commitment and/or proof of knowledge is not for the
    /// commitment key.
    error CommitmentInvalid();

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
        13202483887328734126525590560085624603268286383806781884716633747216672803896;
    uint256 constant ALPHA_Y =
        6587421134278433303989377515847718410012254970047333769165368294623929290339;

    // Groth16 beta point in G2 in powers of i
    uint256 constant BETA_NEG_X_0 =
        866288018242003154707410892674449629299404125988100847464773759419884528066;
    uint256 constant BETA_NEG_X_1 =
        13860785509609054463888651830518763108312288079980232200317249776290320576354;
    uint256 constant BETA_NEG_Y_0 =
        18092465630743135592178379140519629504347470106733770722812436542703045760141;
    uint256 constant BETA_NEG_Y_1 =
        327900345219319871652298789819465438580042831730043914345518889777515776107;

    // Groth16 gamma point in G2 in powers of i
    uint256 constant GAMMA_NEG_X_0 =
        11803215649783261923543435784880078933483945853863920771927486681083708467901;
    uint256 constant GAMMA_NEG_X_1 =
        15878185339252663012094065904857230429271153936857158549385167426659490182043;
    uint256 constant GAMMA_NEG_Y_0 =
        20247768036040097220925771195393013811313169738680383538742713997183969614967;
    uint256 constant GAMMA_NEG_Y_1 =
        9812335414647851689057289973803028902716928566554585015688557428575022090091;

    // Groth16 delta point in G2 in powers of i
    uint256 constant DELTA_NEG_X_0 =
        20451582412018207445833149506633854876144305682030967641147734144853809705782;
    uint256 constant DELTA_NEG_X_1 =
        5643477926117307928074825903088934975717136499720222789748159601283939482907;
    uint256 constant DELTA_NEG_Y_0 =
        18332576448576294314814331792152666161870761141367550139946743704308094900718;
    uint256 constant DELTA_NEG_Y_1 =
        1982200718146918211189827387060510950070127760499800659677209490449909542909;
    // Pedersen G point in G2 in powers of i
    uint256 constant PEDERSEN_G_X_0 =
        6691419515711157061320969865162358929404362925149433333404482183545186267106;
    uint256 constant PEDERSEN_G_X_1 =
        11666670002105975664971602996279283767426913573524509281544747532266288198182;
    uint256 constant PEDERSEN_G_Y_0 =
        2260086901092160285824157262860931384308144043500383014022335550233120693699;
    uint256 constant PEDERSEN_G_Y_1 =
        18040097862264300097069968752839293844802911825893051757192190193006761517059;

    // Pedersen GSigmaNeg point in G2 in powers of i
    uint256 constant PEDERSEN_GSIGMANEG_X_0 =
        17850740643238677992982747511807197625498241750155050330347056605798300123143;
    uint256 constant PEDERSEN_GSIGMANEG_X_1 =
        9880660552047654913666937149550701202764900521269038957797089428305019896764;
    uint256 constant PEDERSEN_GSIGMANEG_Y_0 =
        11342724258164142708626195491070959227123093219143551211816797949253856465170;
    uint256 constant PEDERSEN_GSIGMANEG_Y_1 =
        9341137331456583367229859851951398896507158756136152923818193164619302932906;

    // Constant and public input points
    uint256 constant CONSTANT_X =
        7160379222790257133298946541755281136846462776993632204684268847183192198731;
    uint256 constant CONSTANT_Y =
        4447449304759157122472761917727283028717271970069317539033611286683212540206;
    uint256 constant PUB_0_X =
        19832584598508590085682423552565224918522496432893008118680499760726741937956;
    uint256 constant PUB_0_Y =
        13565865150087339303908589492763998508569450069274896255815290227262024115011;
    uint256 constant PUB_1_X =
        20125105247966879286719626607440841367951359914669910663783154476666481658021;
    uint256 constant PUB_1_Y =
        11191926410018716901640408432248409317534560628646286068251887762666798636261;

    /// Negation in Fp.
    /// @notice Returns a number x such that a + x = 0 in Fp.
    /// @notice The input does not need to be reduced.
    /// @param a the base
    /// @return x the result
    function negate(
        uint256 a
    ) internal pure returns (uint256 x) {
        unchecked {
            x = (P - (a % P)) % P; // Modulo is cheaper than branching
        }
    }

    /// Exponentiation in Fp.
    /// @notice Returns a number x such that a ^ e = x in Fp.
    /// @notice The input does not need to be reduced.
    /// @param a the base
    /// @param e the exponent
    /// @return x the result
    function exp(uint256 a, uint256 e) internal view returns (uint256 x) {
        bool success;
        assembly ("memory-safe") {
            let f := mload(0x40)
            mstore(f, 0x20)
            mstore(add(f, 0x20), 0x20)
            mstore(add(f, 0x40), 0x20)
            mstore(add(f, 0x60), a)
            mstore(add(f, 0x80), e)
            mstore(add(f, 0xa0), P)
            success := staticcall(gas(), PRECOMPILE_MODEXP, f, 0xc0, f, 0x20)
            x := mload(f)
        }
        if (!success) {
            // Exponentiation failed.
            // Should not happen.
            revert ProofInvalid();
        }
    }

    /// Invertsion in Fp.
    /// @notice Returns a number x such that a * x = 1 in Fp.
    /// @notice The input does not need to be reduced.
    /// @notice Reverts with ProofInvalid() if the inverse does not exist
    /// @param a the input
    /// @return x the solution
    function invert_Fp(
        uint256 a
    ) internal view returns (uint256 x) {
        x = exp(a, EXP_INVERSE_FP);
        if (mulmod(a, x, P) != 1) {
            // Inverse does not exist.
            // Can only happen during G2 point decompression.
            revert ProofInvalid();
        }
    }

    /// Square root in Fp.
    /// @notice Returns a number x such that x * x = a in Fp.
    /// @notice Will revert with InvalidProof() if the input is not a square
    /// or not reduced.
    /// @param a the square
    /// @return x the solution
    function sqrt_Fp(
        uint256 a
    ) internal view returns (uint256 x) {
        x = exp(a, EXP_SQRT_FP);
        if (mulmod(x, x, P) != a) {
            // Square root does not exist or a is not reduced.
            // Happens when G1 point is not on curve.
            revert ProofInvalid();
        }
    }

    /// Square test in Fp.
    /// @notice Returns whether a number x exists such that x * x = a in Fp.
    /// @notice Will revert with InvalidProof() if the input is not a square
    /// or not reduced.
    /// @param a the square
    /// @return x the solution
    function isSquare_Fp(
        uint256 a
    ) internal view returns (bool) {
        uint256 x = exp(a, EXP_SQRT_FP);
        return mulmod(x, x, P) == a;
    }

    /// Square root in Fp2.
    /// @notice Fp2 is the complex extension Fp[i]/(i^2 + 1). The input is
    /// a0 + a1 ⋅ i and the result is x0 + x1 ⋅ i.
    /// @notice Will revert with InvalidProof() if
    ///   * the input is not a square,
    ///   * the hint is incorrect, or
    ///   * the input coefficients are not reduced.
    /// @param a0 The real part of the input.
    /// @param a1 The imaginary part of the input.
    /// @param hint A hint which of two possible signs to pick in the equation.
    /// @return x0 The real part of the square root.
    /// @return x1 The imaginary part of the square root.
    function sqrt_Fp2(
        uint256 a0,
        uint256 a1,
        bool hint
    ) internal view returns (uint256 x0, uint256 x1) {
        // If this square root reverts there is no solution in Fp2.
        uint256 d = sqrt_Fp(addmod(mulmod(a0, a0, P), mulmod(a1, a1, P), P));
        if (hint) {
            d = negate(d);
        }
        // If this square root reverts there is no solution in Fp2.
        x0 = sqrt_Fp(mulmod(addmod(a0, d, P), FRACTION_1_2_FP, P));
        x1 = mulmod(a1, invert_Fp(mulmod(x0, 2, P)), P);

        // Check result to make sure we found a root.
        // Note: this also fails if a0 or a1 is not reduced.
        if (
            a0 != addmod(mulmod(x0, x0, P), negate(mulmod(x1, x1, P)), P)
                || a1 != mulmod(2, mulmod(x0, x1, P), P)
        ) {
            revert ProofInvalid();
        }
    }

    /// Compress a G1 point.
    /// @notice Reverts with InvalidProof if the coordinates are not reduced
    /// or if the point is not on the curve.
    /// @notice The point at infinity is encoded as (0,0) and compressed to 0.
    /// @param x The X coordinate in Fp.
    /// @param y The Y coordinate in Fp.
    /// @return c The compresed point (x with one signal bit).
    function compress_g1(
        uint256 x,
        uint256 y
    ) internal view returns (uint256 c) {
        if (x >= P || y >= P) {
            // G1 point not in field.
            revert ProofInvalid();
        }
        if (x == 0 && y == 0) {
            // Point at infinity
            return 0;
        }

        // Note: sqrt_Fp reverts if there is no solution, i.e. the x coordinate is invalid.
        uint256 y_pos = sqrt_Fp(addmod(mulmod(mulmod(x, x, P), x, P), 3, P));
        if (y == y_pos) {
            return (x << 1) | 0;
        } else if (y == negate(y_pos)) {
            return (x << 1) | 1;
        } else {
            // G1 point not on curve.
            revert ProofInvalid();
        }
    }

    /// Decompress a G1 point.
    /// @notice Reverts with InvalidProof if the input does not represent a valid point.
    /// @notice The point at infinity is encoded as (0,0) and compressed to 0.
    /// @param c The compresed point (x with one signal bit).
    /// @return x The X coordinate in Fp.
    /// @return y The Y coordinate in Fp.
    function decompress_g1(
        uint256 c
    ) internal view returns (uint256 x, uint256 y) {
        // Note that X = 0 is not on the curve since 0³ + 3 = 3 is not a square.
        // so we can use it to represent the point at infinity.
        if (c == 0) {
            // Point at infinity as encoded in EIP196 and EIP197.
            return (0, 0);
        }
        bool negate_point = c & 1 == 1;
        x = c >> 1;
        if (x >= P) {
            // G1 x coordinate not in field.
            revert ProofInvalid();
        }

        // Note: (x³ + 3) is irreducible in Fp, so it can not be zero and therefore
        //       y can not be zero.
        // Note: sqrt_Fp reverts if there is no solution, i.e. the point is not on the curve.
        y = sqrt_Fp(addmod(mulmod(mulmod(x, x, P), x, P), 3, P));
        if (negate_point) {
            y = negate(y);
        }
    }

    /// Compress a G2 point.
    /// @notice Reverts with InvalidProof if the coefficients are not reduced
    /// or if the point is not on the curve.
    /// @notice The G2 curve is defined over the complex extension Fp[i]/(i^2 + 1)
    /// with coordinates (x0 + x1 ⋅ i, y0 + y1 ⋅ i).
    /// @notice The point at infinity is encoded as (0,0,0,0) and compressed to (0,0).
    /// @param x0 The real part of the X coordinate.
    /// @param x1 The imaginary poart of the X coordinate.
    /// @param y0 The real part of the Y coordinate.
    /// @param y1 The imaginary part of the Y coordinate.
    /// @return c0 The first half of the compresed point (x0 with two signal bits).
    /// @return c1 The second half of the compressed point (x1 unmodified).
    function compress_g2(
        uint256 x0,
        uint256 x1,
        uint256 y0,
        uint256 y1
    ) internal view returns (uint256 c0, uint256 c1) {
        if (x0 >= P || x1 >= P || y0 >= P || y1 >= P) {
            // G2 point not in field.
            revert ProofInvalid();
        }
        if ((x0 | x1 | y0 | y1) == 0) {
            // Point at infinity
            return (0, 0);
        }

        // Compute y^2
        // Note: shadowing variables and scoping to avoid stack-to-deep.
        uint256 y0_pos;
        uint256 y1_pos;
        {
            uint256 n3ab = mulmod(mulmod(x0, x1, P), P - 3, P);
            uint256 a_3 = mulmod(mulmod(x0, x0, P), x0, P);
            uint256 b_3 = mulmod(mulmod(x1, x1, P), x1, P);
            y0_pos = addmod(
                FRACTION_27_82_FP, addmod(a_3, mulmod(n3ab, x1, P), P), P
            );
            y1_pos = negate(
                addmod(FRACTION_3_82_FP, addmod(b_3, mulmod(n3ab, x0, P), P), P)
            );
        }

        // Determine hint bit
        // If this sqrt fails the x coordinate is not on the curve.
        bool hint;
        {
            uint256 d = sqrt_Fp(
                addmod(mulmod(y0_pos, y0_pos, P), mulmod(y1_pos, y1_pos, P), P)
            );
            hint =
                !isSquare_Fp(mulmod(addmod(y0_pos, d, P), FRACTION_1_2_FP, P));
        }

        // Recover y
        (y0_pos, y1_pos) = sqrt_Fp2(y0_pos, y1_pos, hint);
        if (y0 == y0_pos && y1 == y1_pos) {
            c0 = (x0 << 2) | (hint ? 2 : 0) | 0;
            c1 = x1;
        } else if (y0 == negate(y0_pos) && y1 == negate(y1_pos)) {
            c0 = (x0 << 2) | (hint ? 2 : 0) | 1;
            c1 = x1;
        } else {
            // G1 point not on curve.
            revert ProofInvalid();
        }
    }

    /// Decompress a G2 point.
    /// @notice Reverts with InvalidProof if the input does not represent a valid point.
    /// @notice The G2 curve is defined over the complex extension Fp[i]/(i^2 + 1)
    /// with coordinates (x0 + x1 ⋅ i, y0 + y1 ⋅ i).
    /// @notice The point at infinity is encoded as (0,0,0,0) and compressed to (0,0).
    /// @param c0 The first half of the compresed point (x0 with two signal bits).
    /// @param c1 The second half of the compressed point (x1 unmodified).
    /// @return x0 The real part of the X coordinate.
    /// @return x1 The imaginary poart of the X coordinate.
    /// @return y0 The real part of the Y coordinate.
    /// @return y1 The imaginary part of the Y coordinate.
    function decompress_g2(
        uint256 c0,
        uint256 c1
    ) internal view returns (uint256 x0, uint256 x1, uint256 y0, uint256 y1) {
        // Note that X = (0, 0) is not on the curve since 0³ + 3/(9 + i) is not a square.
        // so we can use it to represent the point at infinity.
        if (c0 == 0 && c1 == 0) {
            // Point at infinity as encoded in EIP197.
            return (0, 0, 0, 0);
        }
        bool negate_point = c0 & 1 == 1;
        bool hint = c0 & 2 == 2;
        x0 = c0 >> 2;
        x1 = c1;
        if (x0 >= P || x1 >= P) {
            // G2 x0 or x1 coefficient not in field.
            revert ProofInvalid();
        }

        uint256 n3ab = mulmod(mulmod(x0, x1, P), P - 3, P);
        uint256 a_3 = mulmod(mulmod(x0, x0, P), x0, P);
        uint256 b_3 = mulmod(mulmod(x1, x1, P), x1, P);

        y0 = addmod(FRACTION_27_82_FP, addmod(a_3, mulmod(n3ab, x1, P), P), P);
        y1 = negate(
            addmod(FRACTION_3_82_FP, addmod(b_3, mulmod(n3ab, x0, P), P), P)
        );

        // Note: sqrt_Fp2 reverts if there is no solution, i.e. the point is not on the curve.
        // Note: (X³ + 3/(9 + i)) is irreducible in Fp2, so y can not be zero.
        //       But y0 or y1 may still independently be zero.
        (y0, y1) = sqrt_Fp2(y0, y1, hint);
        if (negate_point) {
            y0 = negate(y0);
            y1 = negate(y1);
        }
    }

    /// Compute the public input linear combination.
    /// @notice Reverts with PublicInputNotInField if the input is not in the field.
    /// @notice Computes the multi-scalar-multiplication of the public input
    /// elements and the verification key including the constant term.
    /// @param input The public inputs. These are elements of the scalar field Fr.
    /// @param publicCommitments public inputs generated from pedersen commitments.
    /// @param commitments The Pedersen commitments from the proof.
    /// @return x The X coordinate of the resulting G1 point.
    /// @return y The Y coordinate of the resulting G1 point.
    function publicInputMSM(
        uint256[1] memory input,
        uint256[1] memory publicCommitments,
        uint256[2] memory commitments
    ) internal view returns (uint256 x, uint256 y) {
        // Note: The ECMUL precompile does not reject unreduced values, so we check this.
        // Note: Unrolling this loop does not cost much extra in code-size, the bulk of the
        //       code-size is in the PUB_ constants.
        // ECMUL has input (x, y, scalar) and output (x', y').
        // ECADD has input (x1, y1, x2, y2) and output (x', y').
        // We reduce commitments(if any) with constants as the first point argument to ECADD.
        // We call them such that ecmul output is already in the second point
        // argument to ECADD so we can have a tight loop.
        bool success = true;
        assembly ("memory-safe") {
            let f := mload(0x40)
            let g := add(f, 0x40)
            let s
            mstore(f, CONSTANT_X)
            mstore(add(f, 0x20), CONSTANT_Y)
            mstore(g, mload(commitments))
            mstore(add(g, 0x20), mload(add(commitments, 0x20)))
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
            s := mload(publicCommitments)
            mstore(add(g, 0x40), s)
            success := and(success, lt(s, R))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_MUL, g, 0x60, g, 0x40))
            success :=
                and(success, staticcall(gas(), PRECOMPILE_ADD, f, 0x80, f, 0x40))

            x := mload(f)
            y := mload(add(f, 0x20))
        }
        if (!success) {
            // Either Public input not in field, or verification key invalid.
            // We assume the contract is correctly generated, so the verification key is valid.
            revert PublicInputNotInField();
        }
    }

    /// Compress a proof.
    /// @notice Will revert with InvalidProof if the curve points are invalid,
    /// but does not verify the proof itself.
    /// @param proof The uncompressed Groth16 proof. Elements are in the same order as for
    /// verifyProof. I.e. Groth16 points (A, B, C) encoded as in EIP-197.
    /// @param commitments Pedersen commitments from the proof.
    /// @param commitmentPok proof of knowledge for the Pedersen commitments.
    /// @return compressed The compressed proof. Elements are in the same order as for
    /// verifyCompressedProof. I.e. points (A, B, C) in compressed format.
    /// @return compressedCommitments compressed Pedersen commitments from the proof.
    /// @return compressedCommitmentPok compressed proof of knowledge for the Pedersen commitments.
    function compressProof(
        uint256[8] calldata proof,
        uint256[2] calldata commitments,
        uint256[2] calldata commitmentPok
    )
        internal
        view
        returns (
            uint256[4] memory compressed,
            uint256[1] memory compressedCommitments,
            uint256 compressedCommitmentPok
        )
    {
        compressed[0] = compress_g1(proof[0], proof[1]);
        (compressed[2], compressed[1]) =
            compress_g2(proof[3], proof[2], proof[5], proof[4]);
        compressed[3] = compress_g1(proof[6], proof[7]);
        compressedCommitments[0] = compress_g1(commitments[0], commitments[1]);
        compressedCommitmentPok =
            compress_g1(commitmentPok[0], commitmentPok[1]);
    }

    /// Verify an uncompressed Groth16 proof.
    /// @notice Reverts with InvalidProof if the proof is invalid or
    /// with PublicInputNotInField the public input is not reduced.
    /// @notice There is no return value. If the function does not revert, the
    /// proof was successfully verified.
    /// @param proof the points (A, B, C) in EIP-197 format matching the output
    /// of compressProof.
    /// @param commitments the Pedersen commitments from the proof.
    /// @param commitmentPok the proof of knowledge for the Pedersen commitments.
    /// @param input the public input field elements in the scalar field Fr.
    /// Elements must be reduced.
    function verifyProof(
        uint256[8] calldata proof,
        uint256[2] calldata commitments,
        uint256[2] calldata commitmentPok,
        uint256[1] memory input
    ) internal view {
        // HashToField
        uint256[1] memory publicCommitments;
        uint256[] memory publicAndCommitmentCommitted;

        publicCommitments[0] = uint256(
            keccak256(
                abi.encodePacked(
                    commitments[0], commitments[1], publicAndCommitmentCommitted
                )
            )
        ) % R;

        // Verify pedersen commitments
        bool success;
        assembly ("memory-safe") {
            let f := mload(0x40)

            calldatacopy(f, commitments, 0x40) // Copy Commitments
            mstore(add(f, 0x40), PEDERSEN_GSIGMANEG_X_1)
            mstore(add(f, 0x60), PEDERSEN_GSIGMANEG_X_0)
            mstore(add(f, 0x80), PEDERSEN_GSIGMANEG_Y_1)
            mstore(add(f, 0xa0), PEDERSEN_GSIGMANEG_Y_0)
            calldatacopy(add(f, 0xc0), commitmentPok, 0x40)
            mstore(add(f, 0x100), PEDERSEN_G_X_1)
            mstore(add(f, 0x120), PEDERSEN_G_X_0)
            mstore(add(f, 0x140), PEDERSEN_G_Y_1)
            mstore(add(f, 0x160), PEDERSEN_G_Y_0)

            success := staticcall(gas(), PRECOMPILE_VERIFY, f, 0x180, f, 0x20)
            success := and(success, mload(f))
        }
        if (!success) {
            revert CommitmentInvalid();
        }

        (uint256 x, uint256 y) =
            publicInputMSM(input, publicCommitments, commitments);

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
        if (!success) {
            // Either proof or verification key invalid.
            // We assume the contract is correctly generated, so the verification key is valid.
            revert ProofInvalid();
        }
    }
}
