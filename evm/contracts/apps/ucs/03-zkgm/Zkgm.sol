pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/PausableUpgradeable.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "@openzeppelin/contracts/utils/Address.sol";

import "solady/utils/CREATE3.sol";
import "solady/utils/LibBit.sol";
import "solady/utils/LibString.sol";
import "solady/utils/LibBytes.sol";
import "solady/utils/LibCall.sol";
import "solady/utils/EfficientHashLib.sol";
import "solady/utils/SafeTransferLib.sol";
import "solady/utils/LibTransient.sol";

import "../../Base.sol";
import "../../../core/04-channel/IBCPacket.sol";
import "../../../core/05-port/IIBCModule.sol";
import "../../../core/24-host/IBCCommitment.sol";
import "../../../internal/Versioned.sol";

import "./TokenBucket.sol";
import "./IWETH.sol";
import "./IZkgmable.sol";
import "./IZkgmERC20.sol";
import "./ZkgmERC20.sol";
import "./ZkgmERC721.sol";
import "./IZkgm.sol";
import "./Lib.sol";

// Dummy lib to ensure all types are exported
contract AbiExport {
    function ensureExported(
        ZkgmPacket calldata,
        Instruction calldata,
        Forward calldata,
        Multiplex calldata,
        Batch calldata,
        FungibleAssetOrder calldata,
        Ack calldata,
        BatchAck calldata,
        FungibleAssetOrderAck calldata
    ) public {}
}

function passthrough(
    address impl
) {
    assembly {
        calldatacopy(0, 0, calldatasize())
        let result := delegatecall(gas(), impl, 0, calldatasize(), 0, 0)
        returndatacopy(0, 0, returndatasize())
        switch result
        case 0 { revert(0, returndatasize()) }
        default { return(0, returndatasize()) }
    }
}

abstract contract UCS03ZkgmStore is AccessManagedUpgradeable, IZkgmStore {
    bytes32 internal constant STAKE_NFT_MANAGER_SALT =
        keccak256("union.salt.zkgm.stakeNFTManager");

    string internal constant STAKE_NFT_NAME = "Zkgm Staking Position";
    string internal constant STAKE_NFT_SYMBOL = "ZKGMSP";

    IIBCModulePacket private _deprecated_ibcHandler;
    mapping(bytes32 => IBCPacket) public inFlightPacket;
    mapping(address => uint256) public tokenOrigin;
    mapping(uint32 => mapping(uint256 => mapping(address => uint256))) public
        channelBalance;
    mapping(uint32 => bytes) public channelGovernanceToken;
    mapping(uint256 => ZkgmStake) public stakes;

    function _getGovernanceToken(
        uint32 channelId
    ) internal view returns (ZkgmERC20, bytes memory) {
        bytes storage governanceToken = channelGovernanceToken[channelId];
        if (governanceToken.length == 0) {
            revert ZkgmLib.ErrChannelGovernanceTokenNotSet();
        }
        (address wrappedGovernanceToken,) =
            _predictWrappedTokenMemory(0, channelId, governanceToken);
        return (ZkgmERC20(wrappedGovernanceToken), governanceToken);
    }

    function _predictStakeManagerAddress() internal view returns (ZkgmERC721) {
        return ZkgmERC721(
            CREATE3.predictDeterministicAddress(STAKE_NFT_MANAGER_SALT)
        );
    }

    function predictStakeManagerAddress() public view returns (ZkgmERC721) {
        return _predictStakeManagerAddress();
    }

    function _getStakeNFTManager() internal returns (ZkgmERC721) {
        ZkgmERC721 stakeManager = _predictStakeManagerAddress();
        if (!ZkgmLib.isDeployed(address(stakeManager))) {
            CREATE3.deployDeterministic(
                abi.encodePacked(
                    type(ERC1967Proxy).creationCode,
                    abi.encode(
                        new ZkgmERC721(),
                        abi.encodeCall(
                            ZkgmERC721.initialize,
                            (
                                authority(),
                                address(this),
                                STAKE_NFT_NAME,
                                STAKE_NFT_SYMBOL
                            )
                        )
                    )
                ),
                STAKE_NFT_MANAGER_SALT
            );
        }
        return stakeManager;
    }

    // Increase the outstanding balance of a channel. This ensure that malicious
    // channels can't unescrow/mint more tokens than previously escrowed/burnt.
    function _increaseOutstanding(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        uint256 amount
    ) internal {
        channelBalance[sourceChannelId][path][token] += amount;
    }

    // Decrease the outstanding balance of a (channel, path). If the function is
    // called when receiving funds, hence, to decrease we need to first inverse
    // the path. If we increased the balance for (0, [1, 2, 3]) and funds are
    // sent back over [3, 2, 1], this will only work if the path is the inverse.
    // If the function is called on refund, simplify subtract the refunded
    // amount.
    function _decreaseOutstanding(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        uint256 amount
    ) internal {
        channelBalance[sourceChannelId][path][token] -= amount;
    }

    // Predict a wrapped token address given the path/channel and counterparty
    // address of the token. The computed address is fully deterministic w.r.t
    // to (ucs03Address, path, channel, token).
    function _predictWrappedToken(
        uint256 path,
        uint32 channel,
        bytes calldata token
    ) internal view returns (address, bytes32) {
        bytes32 wrappedTokenSalt =
            EfficientHashLib.hash(abi.encode(path, channel, token));
        address wrappedToken =
            CREATE3.predictDeterministicAddress(wrappedTokenSalt);
        return (wrappedToken, wrappedTokenSalt);
    }

    function _predictWrappedTokenMemory(
        uint256 path,
        uint32 channel,
        bytes memory token
    ) internal view returns (address, bytes32) {
        bytes32 wrappedTokenSalt =
            EfficientHashLib.hash(abi.encode(path, channel, token));
        address wrappedToken =
            CREATE3.predictDeterministicAddress(wrappedTokenSalt);
        return (wrappedToken, wrappedTokenSalt);
    }
}

contract UCS03ZkgmStakeImpl is
    AccessManagedUpgradeable,
    PausableUpgradeable,
    Versioned,
    UCS03ZkgmStore
{
    IIBCModulePacket public immutable IBC_HANDLER;

    constructor(
        IIBCModulePacket _ibcHandler
    ) {
        IBC_HANDLER = _ibcHandler;
    }

    function _stakingSucceeded(
        uint32 channelId,
        Stake calldata _stake
    ) internal {
        ensureStakeIsFromChannel(channelId, _stake.tokenId);
        address beneficiary = address(bytes20(_stake.beneficiary));
        _getStakeNFTManager().transferFrom(
            address(this), beneficiary, _stake.tokenId
        );
        stakes[_stake.tokenId].state = ZkgmStakeState.STAKED;
    }

    function _stakingFailed(uint32 channelId, Stake calldata _stake) internal {
        ensureStakeIsFromChannel(channelId, _stake.tokenId);
        _getStakeNFTManager().burn(_stake.tokenId);
        address sender = address(bytes20(_stake.sender));
        (IZkgmERC20 governanceToken,) = _getGovernanceToken(channelId);
        governanceToken.transfer(sender, _stake.amount);
    }

    function _withdrawSucceeded(
        uint32 channelId,
        WithdrawStake calldata _withdrawStake,
        WithdrawStakeAck calldata _withdrawStakeAck
    ) internal {
        ensureStakeIsFromChannel(channelId, _withdrawStake.tokenId);
        ZkgmStake storage _stake = stakes[_withdrawStake.tokenId];
        _stake.state = ZkgmStakeState.UNSTAKED;
        (IZkgmERC20 governanceToken,) = _getGovernanceToken(_stake.channelId);
        address beneficiary = address(bytes20(_withdrawStake.beneficiary));
        governanceToken.transfer(beneficiary, _stake.amount);
        if (_stake.amount < _withdrawStakeAck.amount) {
            // Mints the reward
            governanceToken.mint(
                beneficiary, _withdrawStakeAck.amount - _stake.amount
            );
        } else if (_stake.amount > _withdrawStakeAck.amount) {
            // Burn if slashing happened and rewards aren't covering it.
            governanceToken.burn(
                beneficiary, _stake.amount - _withdrawStakeAck.amount
            );
        }
        // We do not burn the token so that it's ID can't be reused. It will stay forever locked in this contract.
    }

    function _setUnstaking(uint256 tokenId, uint256 completionTime) internal {
        ZkgmStake storage _stake = stakes[tokenId];
        _stake.state = ZkgmStakeState.UNSTAKING;
        _stake.unstakingCompletion = completionTime;
    }

    function ensureStakeIsFromChannel(
        uint32 channelId,
        uint256 tokenId
    ) internal {
        if (stakes[tokenId].channelId != channelId) {
            revert ZkgmLib.ErrInvalidStakeChannelId();
        }
    }

    function acknowledgeStake(
        IBCPacket calldata ibcPacket,
        Stake calldata _stake,
        bool successful
    ) public whenNotPaused {
        if (successful) {
            _stakingSucceeded(ibcPacket.sourceChannelId, _stake);
        } else {
            _stakingFailed(ibcPacket.sourceChannelId, _stake);
        }
    }

    function acknowledgeUnstake(
        IBCPacket calldata ibcPacket,
        Unstake calldata _unstake,
        bool successful,
        bytes calldata ack
    ) public whenNotPaused {
        ensureStakeIsFromChannel(ibcPacket.sourceChannelId, _unstake.tokenId);
        if (successful) {
            UnstakeAck calldata unstakeAck = ZkgmLib.decodeUnstakeAck(ack);
            _setUnstaking(_unstake.tokenId, unstakeAck.completionTime);
        }
        address sender = address(bytes20(_unstake.sender));
        _getStakeNFTManager().transferFrom(
            address(this), sender, _unstake.tokenId
        );
    }

    function acknowledgeWithdrawStake(
        IBCPacket calldata ibcPacket,
        WithdrawStake calldata _withdrawStake,
        bool successful,
        bytes calldata ack
    ) public whenNotPaused {
        if (successful) {
            WithdrawStakeAck calldata _withdrawStakeAck =
                ZkgmLib.decodeWithdrawStakeAck(ack);
            _withdrawSucceeded(
                ibcPacket.sourceChannelId, _withdrawStake, _withdrawStakeAck
            );
        } else {
            _withdrawStakeFailed(ibcPacket.sourceChannelId, _withdrawStake);
        }
    }

    function _withdrawStakeFailed(
        uint32 channelId,
        WithdrawStake calldata _withdrawStake
    ) internal {
        ensureStakeIsFromChannel(channelId, _withdrawStake.tokenId);
        address sender = address(bytes20(_withdrawStake.sender));
        _getStakeNFTManager().transferFrom(
            address(this), sender, _withdrawStake.tokenId
        );
    }

    function timeoutStake(
        IBCPacket calldata ibcPacket,
        Stake calldata _stake
    ) public whenNotPaused {
        _stakingFailed(ibcPacket.sourceChannelId, _stake);
    }

    function timeoutUnstake(
        IBCPacket calldata ibcPacket,
        Unstake calldata _unstake
    ) public whenNotPaused {
        ensureStakeIsFromChannel(ibcPacket.sourceChannelId, _unstake.tokenId);
        address sender = address(bytes20(_unstake.sender));
        _getStakeNFTManager().transferFrom(
            address(this), sender, _unstake.tokenId
        );
    }

    function timeoutWithdrawStake(
        IBCPacket calldata ibcPacket,
        WithdrawStake calldata _withdrawStake
    ) public whenNotPaused {
        _withdrawStakeFailed(ibcPacket.sourceChannelId, _withdrawStake);
    }
}

contract UCS03ZkgmSendImpl is
    AccessManagedUpgradeable,
    PausableUpgradeable,
    Versioned,
    UCS03ZkgmStore
{
    using ZkgmLib for *;
    using LibString for *;
    using LibBytes for *;
    using SafeERC20 for *;
    using Address for *;
    using LibCall for *;

    IIBCModulePacket public immutable IBC_HANDLER;
    IWETH public immutable WETH;
    bytes32 public immutable NATIVE_TOKEN_NAME_HASH;
    bytes32 public immutable NATIVE_TOKEN_SYMBOL_HASH;
    uint8 public immutable NATIVE_TOKEN_DECIMALS;

    constructor(
        IIBCModulePacket _ibcHandler,
        IWETH _weth,
        string memory _nativeTokenName,
        string memory _nativeTokenSymbol,
        uint8 _nativeTokenDecimals
    ) {
        IBC_HANDLER = _ibcHandler;
        WETH = _weth;
        NATIVE_TOKEN_NAME_HASH = keccak256(bytes(_nativeTokenName));
        NATIVE_TOKEN_SYMBOL_HASH = keccak256(bytes(_nativeTokenSymbol));
        NATIVE_TOKEN_DECIMALS = _nativeTokenDecimals;
    }

    function send(
        uint32 channelId,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt,
        Instruction calldata instruction
    ) public payable whenNotPaused {
        _verifyInternal(channelId, 0, instruction);
        IBC_HANDLER.sendPacket(
            channelId,
            timeoutHeight,
            timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: EfficientHashLib.hash(abi.encodePacked(msg.sender, salt)),
                    path: 0,
                    instruction: instruction
                })
            )
        );
    }

    function _verifyInternal(
        uint32 channelId,
        uint256 path,
        Instruction calldata instruction
    ) internal {
        if (
            instruction.isInst(
                ZkgmLib.OP_FUNGIBLE_ASSET_ORDER, ZkgmLib.INSTR_VERSION_1
            )
        ) {
            FungibleAssetOrder calldata order =
                ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
            _verifyFungibleAssetOrder(channelId, path, order);
        } else if (
            instruction.isInst(ZkgmLib.OP_BATCH, ZkgmLib.INSTR_VERSION_0)
        ) {
            _verifyBatch(
                channelId, path, ZkgmLib.decodeBatch(instruction.operand)
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_FORWARD, ZkgmLib.INSTR_VERSION_0)
        ) {
            _verifyForward(
                channelId, ZkgmLib.decodeForward(instruction.operand)
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_MULTIPLEX, ZkgmLib.INSTR_VERSION_0)
        ) {
            _verifyMultiplex(
                channelId, path, ZkgmLib.decodeMultiplex(instruction.operand)
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_STAKE, ZkgmLib.INSTR_VERSION_0)
        ) {
            _verifyStake(
                channelId, path, ZkgmLib.decodeStake(instruction.operand)
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_UNSTAKE, ZkgmLib.INSTR_VERSION_0)
        ) {
            _verifyUnstake(
                channelId, path, ZkgmLib.decodeUnstake(instruction.operand)
            );
        } else if (
            instruction.isInst(
                ZkgmLib.OP_WITHDRAW_STAKE, ZkgmLib.INSTR_VERSION_0
            )
        ) {
            _verifyWithdrawStake(
                channelId,
                path,
                ZkgmLib.decodeWithdrawStake(instruction.operand)
            );
        } else {
            revert ZkgmLib.ErrUnknownOpcode();
        }
    }

    function _verifyFungibleAssetOrder(
        uint32 channelId,
        uint256 path,
        FungibleAssetOrder calldata order
    ) internal {
        IERC20Metadata baseToken =
            IERC20Metadata(address(bytes20(order.baseToken)));
        if (address(baseToken) != ZkgmLib.NATIVE_TOKEN_ERC_7528_ADDRESS) {
            if (!order.baseTokenName.eq(baseToken.name())) {
                revert ZkgmLib.ErrInvalidAssetName();
            }
            if (!order.baseTokenSymbol.eq(baseToken.symbol())) {
                revert ZkgmLib.ErrInvalidAssetSymbol();
            }
            if (order.baseTokenDecimals != baseToken.decimals()) {
                revert ZkgmLib.ErrInvalidAssetDecimals();
            }
        }
        // The origin is the concatenation of (path, destinationChannelId) where
        // path are the intermediate channels hops, if we send from channel X on
        // A over channel Y on B to channel Z on C, the path would be
        // [(X.destinationChannelId, Y.sourceChannelId)].
        uint256 origin = tokenOrigin[address(baseToken)];
        // Split back the origin as the intermediate path and the destinationChannelId
        (uint256 intermediateChannelPath, uint32 destinationChannelId) =
            ZkgmLib.popChannelFromPath(origin);
        // We compute the wrapped token from the destination to the source. If
        // the base token matches the predicted wrapper, we want to unwrap only
        // if it's being sent back through the same channel/path.
        (address wrappedToken,) = _predictWrappedToken(
            intermediateChannelPath, channelId, order.quoteToken
        );
        bool isInverseIntermediatePath =
            path == ZkgmLib.reverseChannelPath(intermediateChannelPath);
        bool isSendingBackToSameChannel = destinationChannelId == channelId;
        bool isUnwrapping = order.baseToken.eq(abi.encodePacked(wrappedToken));
        // If we take the same path starting from the same channel using the
        // wrapped asset, we unwrap.
        if (
            isInverseIntermediatePath && isSendingBackToSameChannel
                && isUnwrapping
        ) {
            if (order.baseTokenPath != origin) {
                revert ZkgmLib.ErrInvalidAssetOrigin();
            }
            IZkgmERC20(address(baseToken)).burn(msg.sender, order.baseAmount);
        } else {
            if (order.baseTokenPath != 0) {
                revert ZkgmLib.ErrInvalidAssetOrigin();
            }
            _increaseOutstanding(
                channelId, path, address(baseToken), order.baseAmount
            );
            if (
                address(baseToken) == ZkgmLib.NATIVE_TOKEN_ERC_7528_ADDRESS
                    && msg.value >= order.baseAmount
            ) {
                if (
                    keccak256(bytes(order.baseTokenName))
                        != NATIVE_TOKEN_NAME_HASH
                ) {
                    revert ZkgmLib.ErrInvalidAssetName();
                }
                if (
                    keccak256(bytes(order.baseTokenSymbol))
                        != NATIVE_TOKEN_SYMBOL_HASH
                ) {
                    revert ZkgmLib.ErrInvalidAssetSymbol();
                }
                if (order.baseTokenDecimals != NATIVE_TOKEN_DECIMALS) {
                    revert ZkgmLib.ErrInvalidAssetDecimals();
                }
                // Use the deposit as a mechanism to consume the order amount from the msg.value.
                // This avoids issue if multiple native eth orders are present.
                WETH.deposit{value: order.baseAmount}();
            } else {
                baseToken.safeTransferFrom(
                    msg.sender, address(this), order.baseAmount
                );
            }
        }
    }

    function _verifyBatch(
        uint32 channelId,
        uint256 path,
        Batch calldata batch
    ) internal {
        uint256 l = batch.instructions.length;
        for (uint256 i = 0; i < l; i++) {
            if (
                !ZkgmLib.isAllowedBatchInstruction(batch.instructions[i].opcode)
            ) {
                revert ZkgmLib.ErrInvalidBatchInstruction();
            }
            _verifyInternal(channelId, path, batch.instructions[i]);
        }
    }

    function _verifyForward(
        uint32 channelId,
        Forward calldata forward
    ) internal {
        if (!ZkgmLib.isAllowedForwardInstruction(forward.instruction.opcode)) {
            revert ZkgmLib.ErrInvalidForwardInstruction();
        }
        _verifyInternal(channelId, forward.path, forward.instruction);
    }

    function _verifyMultiplex(
        uint32 channelId,
        uint256 path,
        Multiplex calldata multiplex
    ) internal {
        if (!multiplex.sender.eq(abi.encodePacked(msg.sender))) {
            revert ZkgmLib.ErrInvalidMultiplexSender();
        }
    }

    function _mintNFT(
        uint256 tokenId,
        uint32 channelId,
        bytes calldata validator,
        uint256 amount
    ) internal {
        _getStakeNFTManager().mint(tokenId, address(this));
        ZkgmStake storage _stake = stakes[tokenId];
        _stake.state = ZkgmStakeState.STAKING;
        _stake.channelId = channelId;
        _stake.validator = validator;
        _stake.amount = amount;
        _stake.unstakingCompletion = 0;
    }

    function _verifyStake(
        uint32 channelId,
        uint256 path,
        Stake calldata stake
    ) internal {
        if (path != 0) {
            revert ZkgmLib.ErrInstructionCannotBeForwarded();
        }
        // Verify the preimage of the governance token.
        (IZkgmERC20 governanceToken, bytes memory unwrappedGovernanceToken) =
            _getGovernanceToken(channelId);
        if (!stake.governanceToken.eq(unwrappedGovernanceToken)) {
            revert ZkgmLib.ErrInvalidStakeGovernanceToken();
        }
        // Escrow the staked amount.
        governanceToken.transferFrom(msg.sender, address(this), stake.amount);
        // Escrow the NFT until the stake is acknowledged.
        _mintNFT(stake.tokenId, channelId, stake.validator, stake.amount);
    }

    function _canUnstake(
        ZkgmStake storage _stake
    ) internal view returns (bool) {
        return _stake.state == ZkgmStakeState.STAKED;
    }

    function _verifyUnstake(
        uint32 channelId,
        uint256 path,
        Unstake calldata unstake
    ) internal {
        if (path != 0) {
            revert ZkgmLib.ErrInstructionCannotBeForwarded();
        }
        ZkgmStake storage _stake = stakes[unstake.tokenId];
        if (channelId != _stake.channelId) {
            revert ZkgmLib.ErrInvalidStakeChannelId();
        }
        // Verify the preimage of the governance token.
        (IZkgmERC20 governanceToken, bytes memory unwrappedGovernanceToken) =
            _getGovernanceToken(channelId);
        if (!unstake.governanceToken.eq(unwrappedGovernanceToken)) {
            revert ZkgmLib.ErrInvalidStakeGovernanceToken();
        }
        if (!_canUnstake(_stake)) {
            revert ZkgmLib.ErrStakeNotUnstakable();
        }
        if (_stake.amount != unstake.amount) {
            revert ZkgmLib.ErrInvalidStakeAmount();
        }
        if (!_stake.validator.eq(unstake.validator)) {
            revert ZkgmLib.ErrInvalidStakeValidator();
        }
        // Escrow the NFT.
        _getStakeNFTManager().transferFrom(
            msg.sender, address(this), unstake.tokenId
        );
    }

    function _canWithdraw(
        ZkgmStake storage _stake
    ) internal view returns (bool) {
        return _stake.state == ZkgmStakeState.UNSTAKING
            && _stake.unstakingCompletion <= block.timestamp;
    }

    function _verifyWithdrawStake(
        uint32 channelId,
        uint256 path,
        WithdrawStake calldata withdrawStake
    ) internal {
        if (path != 0) {
            revert ZkgmLib.ErrInstructionCannotBeForwarded();
        }
        ZkgmStake storage _stake = stakes[withdrawStake.tokenId];
        if (channelId != _stake.channelId) {
            revert ZkgmLib.ErrInvalidStakeChannelId();
        }
        (IZkgmERC20 governanceToken, bytes memory unwrappedGovernanceToken) =
            _getGovernanceToken(channelId);
        if (!withdrawStake.governanceToken.eq(unwrappedGovernanceToken)) {
            revert ZkgmLib.ErrInvalidStakeGovernanceToken();
        }
        if (!_canWithdraw(_stake)) {
            revert ZkgmLib.ErrStakeNotWithdrawable();
        }
        // Escrow the NFT.
        _getStakeNFTManager().transferFrom(
            msg.sender, address(this), withdrawStake.tokenId
        );
    }
}

contract UCS03Zkgm is
    IBCAppBase,
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    PausableUpgradeable,
    TokenBucket,
    Versioned,
    IZkgm,
    UCS03ZkgmStore
{
    using ZkgmLib for *;
    using LibString for *;
    using LibBytes for *;
    using SafeERC20 for *;
    using Address for *;
    using LibCall for *;

    uint256 public constant EXEC_MIN_GAS = 50_000;

    IIBCModulePacket public immutable IBC_HANDLER;
    IWETH public immutable WETH;
    ZkgmERC20 public immutable ERC20_IMPL;
    bool public immutable RATE_LIMIT_ENABLED;
    address public immutable SEND_IMPL;
    address public immutable STAKE_IMPL;

    constructor(
        IIBCModulePacket _ibcHandler,
        IWETH _weth,
        ZkgmERC20 _erc20Impl,
        bool _rateLimitEnabled,
        UCS03ZkgmSendImpl _sendImpl,
        UCS03ZkgmStakeImpl _stakeImpl
    ) {
        _disableInitializers();
        IBC_HANDLER = _ibcHandler;
        WETH = _weth;
        ERC20_IMPL = _erc20Impl;
        RATE_LIMIT_ENABLED = _rateLimitEnabled;
        SEND_IMPL = address(_sendImpl);
        STAKE_IMPL = address(_stakeImpl);
    }

    function initialize(
        address _authority
    ) public initializer {
        __AccessManaged_init(_authority);
        __UUPSUpgradeable_init();
        __Pausable_init();
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(IBC_HANDLER);
    }

    function registerGovernanceToken(
        uint32 channelId,
        bytes calldata unwrappedGovernanceToken
    ) public {
        if (channelGovernanceToken[channelId].length != 0) {
            revert ZkgmLib.ErrChannelGovernanceTokenAlreadySet();
        }
        channelGovernanceToken[channelId] = unwrappedGovernanceToken;
    }

    function send(
        uint32 channelId,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt,
        Instruction calldata instruction
    ) public payable {
        passthrough(address(SEND_IMPL));
    }

    function onRecvIntentPacket(
        address caller,
        IBCPacket calldata packet,
        address relayer,
        bytes calldata relayerMsg
    ) external virtual override onlyIBC whenNotPaused returns (bytes memory) {
        return _processReceive(caller, packet, relayer, relayerMsg, true);
    }

    function onRecvPacket(
        address caller,
        IBCPacket calldata packet,
        address relayer,
        bytes calldata relayerMsg
    ) external virtual override onlyIBC whenNotPaused returns (bytes memory) {
        return _processReceive(caller, packet, relayer, relayerMsg, false);
    }

    function _processReceive(
        address caller,
        IBCPacket calldata packet,
        address relayer,
        bytes calldata relayerMsg,
        bool intent
    ) internal returns (bytes memory) {
        (bool success, bytes memory returnData) = address(this).call(
            abi.encodeCall(
                this.execute, (caller, packet, relayer, relayerMsg, intent)
            )
        );
        // Avoid gas-starvation trick. Enforce a minimum for griefing relayers.
        // See: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/bd325d56b4c62c9c5c1aff048c37c6bb18ac0290/contracts/metatx/MinimalForwarder.sol#L58-L68
        if (gasleft() <= EXEC_MIN_GAS / 63) {
            assembly {
                invalid()
            }
        }
        if (success) {
            bytes memory acknowledgement = abi.decode(returnData, (bytes));
            // The acknowledgement may be asynchronous (forward/multiplex).
            if (acknowledgement.length == 0) {
                return ZkgmLib.ACK_EMPTY;
            }

            // Special case where we should avoid the packet from being
            // received entirely as it is only fillable by a market maker.
            if (
                EfficientHashLib.hash(acknowledgement)
                    == ZkgmLib.ACK_ERR_ONLYMAKER_HASH
            ) {
                revert ZkgmLib.ErrOnlyMaker();
            }

            return ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_SUCCESS, innerAck: acknowledgement})
            );
        } else {
            return ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
            );
        }
    }

    function execute(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bool intent
    ) public returns (bytes memory) {
        // Only callable through the onRecvPacket endpoint.
        if (msg.sender != address(this)) {
            revert ZkgmLib.ErrUnauthorized();
        }
        ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(ibcPacket.data);
        return _executeInternal(
            caller,
            ibcPacket,
            relayer,
            relayerMsg,
            zkgmPacket.salt,
            zkgmPacket.path,
            zkgmPacket.instruction,
            intent
        );
    }

    function _executeInternal(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        Instruction calldata instruction,
        bool intent
    ) internal returns (bytes memory) {
        if (
            instruction.isInst(
                ZkgmLib.OP_FUNGIBLE_ASSET_ORDER, ZkgmLib.INSTR_VERSION_1
            )
        ) {
            FungibleAssetOrder calldata order =
                ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
            return _executeFungibleAssetOrder(
                caller, ibcPacket, relayer, relayerMsg, path, order, intent
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_BATCH, ZkgmLib.INSTR_VERSION_0)
        ) {
            return _executeBatch(
                caller,
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                path,
                ZkgmLib.decodeBatch(instruction.operand),
                intent
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_FORWARD, ZkgmLib.INSTR_VERSION_0)
        ) {
            return _executeForward(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                path,
                instruction.version,
                ZkgmLib.decodeForward(instruction.operand),
                intent
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_MULTIPLEX, ZkgmLib.INSTR_VERSION_0)
        ) {
            return _executeMultiplex(
                caller,
                ibcPacket,
                relayer,
                relayerMsg,
                path,
                salt,
                ZkgmLib.decodeMultiplex(instruction.operand),
                intent
            );
        } else {
            revert ZkgmLib.ErrUnknownOpcode();
        }
    }

    function _executeBatch(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        Batch calldata batch,
        bool intent
    ) internal returns (bytes memory) {
        uint256 l = batch.instructions.length;
        bytes[] memory acks = new bytes[](l);
        for (uint256 i = 0; i < l; i++) {
            Instruction calldata instruction = batch.instructions[i];
            if (!ZkgmLib.isAllowedBatchInstruction(instruction.opcode)) {
                revert ZkgmLib.ErrInvalidBatchInstruction();
            }
            acks[i] = _executeInternal(
                caller,
                ibcPacket,
                relayer,
                relayerMsg,
                ZkgmLib.deriveBatchSalt(i, salt),
                path,
                instruction,
                intent
            );
            // We should have the guarantee that the acks are non empty because
            // the only instructions allowed in a batch are multiplex and
            // fungibleAssetOrder which returns non-empty acks only.
            if (acks[i].length == 0) {
                revert ZkgmLib.ErrBatchMustBeSync();
            } else if (
                EfficientHashLib.hash(acks[i]) == ZkgmLib.ACK_ERR_ONLYMAKER_HASH
            ) {
                return acks[i];
            }
        }
        return ZkgmLib.encodeBatchAck(BatchAck({acknowledgements: acks}));
    }

    function _executeForward(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        uint8 version,
        Forward calldata forward,
        bool intent
    ) internal returns (bytes memory) {
        if (!ZkgmLib.isAllowedForwardInstruction(forward.instruction.opcode)) {
            revert ZkgmLib.ErrInvalidForwardInstruction();
        }
        // We cannot allow market makers to fill packets containing forward
        // instruction. This would allow them to submit of a proof and fill via the
        // protocol on destination for a fake forward.

        // Instead, they must first fill on destination the orders, awaits finality
        // to settle the forward, then cascade acknowledge.
        if (intent) {
            return ZkgmLib.ACK_ERR_ONLYMAKER;
        }

        (uint256 tailPath, uint32 previousDestinationChannelId) =
            ZkgmLib.dequeueChannelFromPath(forward.path);
        (uint256 continuationPath, uint32 nextSourceChannelId) =
            ZkgmLib.dequeueChannelFromPath(tailPath);
        if (ibcPacket.destinationChannelId != previousDestinationChannelId) {
            revert ZkgmLib.ErrInvalidForwardDestinationChannelId();
        }
        Instruction memory nextInstruction;
        if (continuationPath == 0) {
            // If we are done hopping, the sub-instruction is dispatched for execution.
            nextInstruction = forward.instruction;
        } else {
            // If we are not done, the continuation path is used and the forward is re-executed.
            nextInstruction = Instruction({
                version: version,
                opcode: ZkgmLib.OP_FORWARD,
                operand: ZkgmLib.encodeForward(
                    Forward({
                        path: continuationPath,
                        timeoutHeight: forward.timeoutHeight,
                        timeoutTimestamp: forward.timeoutTimestamp,
                        instruction: forward.instruction
                    })
                )
            });
        }
        IBCPacket memory sentPacket = IBC_HANDLER.sendPacket(
            nextSourceChannelId,
            forward.timeoutHeight,
            forward.timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: ZkgmLib.deriveForwardSalt(salt),
                    path: ZkgmLib.updateChannelPath(
                        ZkgmLib.updateChannelPath(
                            path, previousDestinationChannelId
                        ),
                        nextSourceChannelId
                    ),
                    instruction: nextInstruction
                })
            )
        );
        // Guaranteed to be unique by the above sendPacket
        bytes32 commitmentKey = IBCCommitment.batchPacketsCommitmentKey(
            IBCPacketLib.commitPacket(sentPacket)
        );
        inFlightPacket[commitmentKey] = ibcPacket;
        return ZkgmLib.ACK_EMPTY;
    }

    function _executeMultiplex(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        uint256 path,
        bytes32 salt,
        Multiplex calldata multiplex,
        bool intent
    ) internal returns (bytes memory) {
        address contractAddress = address(bytes20(multiplex.contractAddress));
        if (!multiplex.eureka) {
            if (intent) {
                IZkgmable(contractAddress).onIntentZkgm(
                    caller,
                    path,
                    ibcPacket.sourceChannelId,
                    ibcPacket.destinationChannelId,
                    multiplex.sender,
                    multiplex.contractCalldata,
                    relayer,
                    relayerMsg
                );
            } else {
                IZkgmable(contractAddress).onZkgm(
                    caller,
                    path,
                    ibcPacket.sourceChannelId,
                    ibcPacket.destinationChannelId,
                    multiplex.sender,
                    multiplex.contractCalldata,
                    relayer,
                    relayerMsg
                );
            }
            return abi.encode(ZkgmLib.ACK_SUCCESS);
        } else {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: ZkgmLib.encodeMultiplexCalldata(
                    path, multiplex.sender, multiplex.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            bytes memory acknowledgement;
            if (intent) {
                acknowledgement = IIBCModuleRecv(contractAddress)
                    .onRecvIntentPacket(
                    caller, multiplexIbcPacket, relayer, relayerMsg
                );
            } else {
                acknowledgement = IIBCModuleRecv(contractAddress).onRecvPacket(
                    caller, multiplexIbcPacket, relayer, relayerMsg
                );
            }
            if (acknowledgement.length == 0) {
                revert ZkgmLib.ErrAsyncMultiplexUnsupported();
            }
            return acknowledgement;
        }
    }

    function predictWrappedToken(
        uint256 path,
        uint32 channel,
        bytes calldata token
    ) public view returns (address, bytes32) {
        return _predictWrappedToken(path, channel, token);
    }

    function _protocolFill(
        uint32 channelId,
        uint256 path,
        address wrappedToken,
        address quoteToken,
        address receiver,
        address relayer,
        uint256 baseAmount,
        uint256 quoteAmount,
        bool mint
    ) internal returns (bytes memory) {
        uint256 fee = baseAmount - quoteAmount;
        if (mint) {
            if (quoteAmount > 0) {
                IZkgmERC20(wrappedToken).mint(receiver, quoteAmount);
            }
            if (fee > 0) {
                IZkgmERC20(wrappedToken).mint(relayer, fee);
            }
        } else {
            // If the base token path is being unwrapped, it's going to be non-zero.
            _decreaseOutstanding(
                channelId,
                ZkgmLib.reverseChannelPath(path),
                quoteToken,
                quoteAmount + fee
            );
            if (quoteToken == ZkgmLib.NATIVE_TOKEN_ERC_7528_ADDRESS) {
                if (quoteAmount + fee > 0) {
                    WETH.withdraw(quoteAmount + fee);
                }
                if (quoteAmount > 0) {
                    payable(receiver).sendValue(quoteAmount);
                }

                if (fee > 0) {
                    if (
                        !SafeTransferLib.trySafeTransferETH(
                            relayer,
                            fee,
                            SafeTransferLib.GAS_STIPEND_NO_STORAGE_WRITES
                        )
                    ) {
                        return ZkgmLib.ACK_ERR_ONLYMAKER;
                    }
                }
            } else {
                if (quoteAmount > 0) {
                    IERC20(quoteToken).safeTransfer(receiver, quoteAmount);
                }
                if (fee > 0) {
                    IERC20(quoteToken).safeTransfer(relayer, fee);
                }
            }
        }
        return ZkgmLib.encodeFungibleAssetOrderAck(
            FungibleAssetOrderAck({
                fillType: ZkgmLib.FILL_TYPE_PROTOCOL,
                marketMaker: ZkgmLib.ACK_EMPTY
            })
        );
    }

    function _marketMakerFill(
        address caller,
        bytes calldata relayerMsg,
        address quoteToken,
        address payable receiver,
        uint256 quoteAmount
    ) internal returns (bytes memory) {
        if (quoteAmount != 0) {
            // We want the top level handler in onRecvPacket to know we need to
            // revert for another MM to get a chance to fill. If we revert now
            // the entire packet would be considered to be "failed" and refunded
            // at origin, which we want to avoid.
            // Hence, in case of transfer failure, we yield the ack to notify the onRecvPacket.

            // Special case for gas station where the user is asking for native
            // gas token. The MM has to provide WETH funds that will be
            // unwrapped, avoiding us from having to manage msg.value accross
            // the stack.
            if (quoteToken == ZkgmLib.NATIVE_TOKEN_ERC_7528_ADDRESS) {
                // Transfert to protocol.
                if (
                    !WETH.trySafeTransferFrom(caller, address(this), quoteAmount)
                ) {
                    return ZkgmLib.ACK_ERR_ONLYMAKER;
                }
                // Unwrap and send.
                WETH.withdraw(quoteAmount);
                // We allow this call to fail because in such case the MM was
                // able to provide the funds. A failure ACK will be written and
                // refund will happen.
                receiver.sendValue(quoteAmount);
            } else if (
                !IERC20(quoteToken).trySafeTransferFrom(
                    caller, receiver, quoteAmount
                )
            ) {
                return ZkgmLib.ACK_ERR_ONLYMAKER;
            }
        }
        return ZkgmLib.encodeFungibleAssetOrderAck(
            FungibleAssetOrderAck({
                fillType: ZkgmLib.FILL_TYPE_MARKETMAKER,
                // The relayer has to provide it's maker address using the
                // relayerMsg. This address is specific to the counterparty
                // chain and is where the protocol will pay back the base amount
                // on acknowledgement.
                marketMaker: relayerMsg
            })
        );
    }

    function _deployWrappedToken(
        uint32 channelId,
        uint256 path,
        address wrappedToken,
        bytes32 wrappedTokenSalt,
        string calldata orderBaseTokenSymbol,
        string calldata orderBaseTokenName,
        uint8 orderBaseTokenDecimals
    ) internal {
        if (!ZkgmLib.isDeployed(wrappedToken)) {
            CREATE3.deployDeterministic(
                abi.encodePacked(
                    type(ERC1967Proxy).creationCode,
                    abi.encode(
                        ERC20_IMPL,
                        abi.encodeCall(
                            ZkgmERC20.initialize,
                            (
                                authority(),
                                address(this),
                                orderBaseTokenName,
                                orderBaseTokenSymbol,
                                orderBaseTokenDecimals
                            )
                        )
                    )
                ),
                wrappedTokenSalt
            );
            tokenOrigin[wrappedToken] =
                ZkgmLib.updateChannelPath(path, channelId);
        }
    }

    function _optionalRateLimit(address token, uint256 amount) internal {
        if (RATE_LIMIT_ENABLED) {
            _rateLimit(token, amount);
        }
    }

    function _executeFungibleAssetOrder(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        uint256 path,
        FungibleAssetOrder calldata order,
        bool intent
    ) internal returns (bytes memory) {
        address quoteToken = address(bytes20(order.quoteToken));
        address payable receiver = payable(address(bytes20(order.receiver)));

        // For intent packets, the protocol is not allowed to provide any fund
        // as the packet has not been checked for membership poof. Instead, we
        // know the market maker will be repaid on the source chain, if and only
        // if the currently executing packet hash had been registered as sent on
        // the source. In other words, the market maker is unable to lie.
        if (intent) {
            return _marketMakerFill(
                caller, relayerMsg, quoteToken, receiver, order.quoteAmount
            );
        }

        (address wrappedToken, bytes32 wrappedTokenSalt) = _predictWrappedToken(
            path, ibcPacket.destinationChannelId, order.baseToken
        );
        bool baseAmountCoversQuoteAmount = order.baseAmount >= order.quoteAmount;
        if (quoteToken == wrappedToken && baseAmountCoversQuoteAmount) {
            _optionalRateLimit(quoteToken, order.quoteAmount);
            _deployWrappedToken(
                ibcPacket.destinationChannelId,
                path,
                wrappedToken,
                wrappedTokenSalt,
                order.baseTokenSymbol,
                order.baseTokenName,
                order.baseTokenDecimals
            );
            return _protocolFill(
                ibcPacket.destinationChannelId,
                path,
                wrappedToken,
                quoteToken,
                receiver,
                relayer,
                order.baseAmount,
                order.quoteAmount,
                true
            );
        } else if (order.baseTokenPath != 0 && baseAmountCoversQuoteAmount) {
            _optionalRateLimit(quoteToken, order.quoteAmount);
            return _protocolFill(
                ibcPacket.destinationChannelId,
                path,
                wrappedToken,
                quoteToken,
                receiver,
                relayer,
                order.baseAmount,
                order.quoteAmount,
                false
            );
        } else {
            // We also allow market makers to fill orders after finality. This
            // allow orders that combines protocol and mm filling (wrapped vs
            // non wrapped assets).
            return _marketMakerFill(
                caller, relayerMsg, quoteToken, receiver, order.quoteAmount
            );
        }
    }

    function onAcknowledgementPacket(
        address caller,
        IBCPacket calldata ibcPacket,
        bytes calldata ack,
        address relayer
    ) external virtual override onlyIBC whenNotPaused {
        ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(ibcPacket.data);
        if (ZkgmLib.isForwardedPacket(zkgmPacket.salt)) {
            bytes32 packetHash = IBCPacketLib.commitPacket(ibcPacket);
            IBCPacket memory parent = inFlightPacket[packetHash];
            if (parent.timeoutTimestamp != 0 || parent.timeoutHeight != 0) {
                delete inFlightPacket[packetHash];
                IBC_HANDLER.writeAcknowledgement(parent, ack);
                return;
            }
        }
        Ack calldata zkgmAck = ZkgmLib.decodeAck(ack);
        _acknowledgeInternal(
            caller,
            ibcPacket,
            relayer,
            zkgmPacket.path,
            zkgmPacket.salt,
            zkgmPacket.instruction,
            zkgmAck.tag == ZkgmLib.ACK_SUCCESS,
            zkgmAck.innerAck
        );
    }

    function _callStakeImpl(
        bytes memory data
    ) internal {
        STAKE_IMPL.delegateCallContract(data);
    }

    function _acknowledgeInternal(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        Instruction calldata instruction,
        bool successful,
        bytes calldata ack
    ) internal {
        if (
            instruction.isInst(
                ZkgmLib.OP_FUNGIBLE_ASSET_ORDER, ZkgmLib.INSTR_VERSION_1
            )
        ) {
            FungibleAssetOrder calldata order =
                ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
            _acknowledgeFungibleAssetOrder(
                ibcPacket,
                relayer,
                path,
                salt,
                order.sender,
                order.baseToken,
                order.baseTokenPath,
                order.baseAmount,
                successful,
                ack
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_BATCH, ZkgmLib.INSTR_VERSION_0)
        ) {
            _acknowledgeBatch(
                caller,
                ibcPacket,
                relayer,
                path,
                salt,
                ZkgmLib.decodeBatch(instruction.operand),
                successful,
                ack
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_FORWARD, ZkgmLib.INSTR_VERSION_0)
        ) {
            _acknowledgeForward(
                caller,
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeForward(instruction.operand),
                successful,
                ack
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_MULTIPLEX, ZkgmLib.INSTR_VERSION_0)
        ) {
            _acknowledgeMultiplex(
                caller,
                ibcPacket,
                relayer,
                path,
                salt,
                ZkgmLib.decodeMultiplex(instruction.operand),
                successful,
                ack
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_STAKE, ZkgmLib.INSTR_VERSION_0)
        ) {
            _callStakeImpl(
                abi.encodeCall(
                    UCS03ZkgmStakeImpl.acknowledgeStake,
                    (
                        ibcPacket,
                        ZkgmLib.decodeStake(instruction.operand),
                        successful
                    )
                )
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_UNSTAKE, ZkgmLib.INSTR_VERSION_0)
        ) {
            _callStakeImpl(
                abi.encodeCall(
                    UCS03ZkgmStakeImpl.acknowledgeUnstake,
                    (
                        ibcPacket,
                        ZkgmLib.decodeUnstake(instruction.operand),
                        successful,
                        ack
                    )
                )
            );
        } else if (
            instruction.isInst(
                ZkgmLib.OP_WITHDRAW_STAKE, ZkgmLib.INSTR_VERSION_0
            )
        ) {
            _callStakeImpl(
                abi.encodeCall(
                    UCS03ZkgmStakeImpl.acknowledgeWithdrawStake,
                    (
                        ibcPacket,
                        ZkgmLib.decodeWithdrawStake(instruction.operand),
                        successful,
                        ack
                    )
                )
            );
        } else {
            revert ZkgmLib.ErrUnknownOpcode();
        }
    }

    function _acknowledgeBatch(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        Batch calldata batch,
        bool successful,
        bytes calldata ack
    ) internal {
        uint256 l = batch.instructions.length;
        BatchAck calldata batchAck = ZkgmLib.decodeBatchAck(ack);
        for (uint256 i = 0; i < l; i++) {
            // The syscallAck is set to the ack by default just to satisfy the
            // compiler. The failure branch will never read the ack, hence the
            // assignation has no effect in the recursive handling semantic.
            bytes calldata syscallAck = ack;
            if (successful) {
                syscallAck = batchAck.acknowledgements[i];
            }
            _acknowledgeInternal(
                caller,
                ibcPacket,
                relayer,
                path,
                ZkgmLib.deriveBatchSalt(i, salt),
                batch.instructions[i],
                successful,
                syscallAck
            );
        }
    }

    function _acknowledgeForward(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        Forward calldata forward,
        bool successful,
        bytes calldata ack
    ) internal {
        _acknowledgeInternal(
            caller,
            ibcPacket,
            relayer,
            forward.path,
            salt,
            forward.instruction,
            successful,
            ack
        );
    }

    function _acknowledgeMultiplex(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        Multiplex calldata multiplex,
        bool successful,
        bytes calldata ack
    ) internal {
        if (successful && multiplex.eureka) {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: ZkgmLib.encodeMultiplexCalldata(
                    path, multiplex.sender, multiplex.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            IIBCModule(address(bytes20(multiplex.sender)))
                .onAcknowledgementPacket(caller, multiplexIbcPacket, ack, relayer);
        }
    }

    function _acknowledgeFungibleAssetOrder(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        bytes calldata orderSender,
        bytes calldata orderBaseToken,
        uint256 orderBaseTokenPath,
        uint256 orderBaseAmount,
        bool successful,
        bytes calldata ack
    ) internal {
        if (successful) {
            FungibleAssetOrderAck calldata assetOrderAck =
                ZkgmLib.decodeFungibleAssetOrderAck(ack);
            if (assetOrderAck.fillType == ZkgmLib.FILL_TYPE_PROTOCOL) {
                // The protocol filled, fee was paid to relayer.
            } else if (assetOrderAck.fillType == ZkgmLib.FILL_TYPE_MARKETMAKER)
            {
                // A market maker filled, we pay with the sent asset.
                address marketMaker =
                    address(bytes20(assetOrderAck.marketMaker));
                address baseToken = address(bytes20(orderBaseToken));
                if (orderBaseTokenPath != 0) {
                    IZkgmERC20(address(baseToken)).mint(
                        marketMaker, orderBaseAmount
                    );
                } else {
                    _decreaseOutstanding(
                        ibcPacket.sourceChannelId,
                        path,
                        baseToken,
                        orderBaseAmount
                    );
                    IERC20(baseToken).safeTransfer(marketMaker, orderBaseAmount);
                }
            } else {
                revert ZkgmLib.ErrInvalidFillType();
            }
        } else {
            _refund(
                ibcPacket.sourceChannelId,
                path,
                orderSender,
                orderBaseToken,
                orderBaseTokenPath,
                orderBaseAmount
            );
        }
    }

    function onTimeoutPacket(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer
    ) external virtual override onlyIBC whenNotPaused {
        ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(ibcPacket.data);
        if (ZkgmLib.isForwardedPacket(zkgmPacket.salt)) {
            bytes32 packetHash = IBCPacketLib.commitPacket(ibcPacket);
            IBCPacket memory parent = inFlightPacket[packetHash];
            if (parent.timeoutTimestamp != 0 || parent.timeoutHeight != 0) {
                // If the forwarded packet times out, we write a failure ACK for
                // the parent such that we ensure refund happens as the parent
                // wouldn't timeout itself.
                delete inFlightPacket[packetHash];
                IBC_HANDLER.writeAcknowledgement(
                    parent,
                    ZkgmLib.encodeAck(
                        Ack({
                            tag: ZkgmLib.ACK_FAILURE,
                            innerAck: ZkgmLib.ACK_EMPTY
                        })
                    )
                );
                return;
            }
        }
        _timeoutInternal(
            caller, ibcPacket, relayer, zkgmPacket.path, zkgmPacket.instruction
        );
    }

    function _timeoutInternal(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        Instruction calldata instruction
    ) internal {
        if (
            instruction.isInst(
                ZkgmLib.OP_FUNGIBLE_ASSET_ORDER, ZkgmLib.INSTR_VERSION_1
            )
        ) {
            FungibleAssetOrder calldata order =
                ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
            _timeoutFungibleAssetOrder(
                ibcPacket,
                path,
                order.sender,
                order.baseToken,
                order.baseTokenPath,
                order.baseAmount
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_BATCH, ZkgmLib.INSTR_VERSION_0)
        ) {
            _timeoutBatch(
                caller,
                ibcPacket,
                relayer,
                path,
                ZkgmLib.decodeBatch(instruction.operand)
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_FORWARD, ZkgmLib.INSTR_VERSION_0)
        ) {
            _timeoutForward(
                caller,
                ibcPacket,
                relayer,
                ZkgmLib.decodeForward(instruction.operand)
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_MULTIPLEX, ZkgmLib.INSTR_VERSION_0)
        ) {
            _timeoutMultiplex(
                caller,
                ibcPacket,
                relayer,
                path,
                ZkgmLib.decodeMultiplex(instruction.operand)
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_STAKE, ZkgmLib.INSTR_VERSION_0)
        ) {
            _callStakeImpl(
                abi.encodeCall(
                    UCS03ZkgmStakeImpl.timeoutStake,
                    (ibcPacket, ZkgmLib.decodeStake(instruction.operand))
                )
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_UNSTAKE, ZkgmLib.INSTR_VERSION_0)
        ) {
            _callStakeImpl(
                abi.encodeCall(
                    UCS03ZkgmStakeImpl.timeoutUnstake,
                    (ibcPacket, ZkgmLib.decodeUnstake(instruction.operand))
                )
            );
        } else if (
            instruction.isInst(
                ZkgmLib.OP_WITHDRAW_STAKE, ZkgmLib.INSTR_VERSION_0
            )
        ) {
            _callStakeImpl(
                abi.encodeCall(
                    UCS03ZkgmStakeImpl.timeoutWithdrawStake,
                    (
                        ibcPacket,
                        ZkgmLib.decodeWithdrawStake(instruction.operand)
                    )
                )
            );
        } else {
            revert ZkgmLib.ErrUnknownOpcode();
        }
    }

    function _timeoutBatch(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        Batch calldata batch
    ) internal {
        uint256 l = batch.instructions.length;
        for (uint256 i = 0; i < l; i++) {
            _timeoutInternal(
                caller, ibcPacket, relayer, path, batch.instructions[i]
            );
        }
    }

    function _timeoutForward(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        Forward calldata forward
    ) internal {
        _timeoutInternal(
            caller, ibcPacket, relayer, forward.path, forward.instruction
        );
    }

    function _timeoutMultiplex(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        Multiplex calldata multiplex
    ) internal {
        if (multiplex.eureka) {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: ZkgmLib.encodeMultiplexCalldata(
                    path, multiplex.contractAddress, multiplex.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            IIBCModule(address(bytes20(multiplex.sender))).onTimeoutPacket(
                caller, multiplexIbcPacket, relayer
            );
        }
    }

    function _timeoutFungibleAssetOrder(
        IBCPacket calldata ibcPacket,
        uint256 path,
        bytes calldata orderSender,
        bytes calldata orderBaseToken,
        uint256 orderBaseTokenPath,
        uint256 orderBaseAmount
    ) internal {
        _refund(
            ibcPacket.sourceChannelId,
            path,
            orderSender,
            orderBaseToken,
            orderBaseTokenPath,
            orderBaseAmount
        );
    }

    function _refund(
        uint32 sourceChannelId,
        uint256 path,
        bytes calldata orderSender,
        bytes calldata orderBaseToken,
        uint256 orderBaseTokenPath,
        uint256 orderBaseAmount
    ) internal {
        address sender = address(bytes20(orderSender));
        address baseToken = address(bytes20(orderBaseToken));
        if (orderBaseTokenPath != 0) {
            IZkgmERC20(address(baseToken)).mint(sender, orderBaseAmount);
        } else {
            _decreaseOutstanding(
                sourceChannelId, path, baseToken, orderBaseAmount
            );
            IERC20(baseToken).safeTransfer(sender, orderBaseAmount);
        }
    }

    function onChanOpenInit(
        address,
        uint32,
        uint32,
        string calldata version,
        address
    ) external virtual override onlyIBC {
        if (EfficientHashLib.hash(bytes(version)) != ZkgmLib.IBC_VERSION) {
            revert ZkgmLib.ErrInvalidIBCVersion();
        }
    }

    function onChanOpenTry(
        address,
        uint32,
        uint32,
        uint32,
        string calldata version,
        string calldata counterpartyVersion,
        address
    ) external virtual override onlyIBC {
        if (EfficientHashLib.hash(bytes(version)) != ZkgmLib.IBC_VERSION) {
            revert ZkgmLib.ErrInvalidIBCVersion();
        }
        if (
            EfficientHashLib.hash(bytes(counterpartyVersion))
                != ZkgmLib.IBC_VERSION
        ) {
            revert ZkgmLib.ErrInvalidIBCVersion();
        }
    }

    function onChanOpenAck(
        address,
        uint32 channelId,
        uint32,
        string calldata counterpartyVersion,
        address
    ) external virtual override onlyIBC {
        if (
            EfficientHashLib.hash(bytes(counterpartyVersion))
                != ZkgmLib.IBC_VERSION
        ) {
            revert ZkgmLib.ErrInvalidIBCVersion();
        }
    }

    function onChanOpenConfirm(
        address,
        uint32 channelId,
        address
    ) external virtual override onlyIBC {}

    function onChanCloseInit(
        address,
        uint32,
        address
    ) external virtual override onlyIBC {
        revert ZkgmLib.ErrInfiniteGame();
    }

    function onChanCloseConfirm(
        address,
        uint32,
        address
    ) external virtual override onlyIBC {
        revert ZkgmLib.ErrInfiniteGame();
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override restricted {}

    function pause() public restricted {
        _pause();
    }

    function unpause() public restricted {
        _unpause();
    }

    function setBucketConfig(
        address token,
        uint256 capacity,
        uint256 refillRate,
        bool reset
    ) public restricted {
        _setBucketConfig(token, capacity, refillRate, reset);
    }

    receive() external payable {}
}
