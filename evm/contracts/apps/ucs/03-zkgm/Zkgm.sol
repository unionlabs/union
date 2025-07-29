pragma solidity ^0.8.27;

import "./Stake.sol";
import "./Send.sol";
import "./TokenOrder.sol";
import "./Store.sol";

// Dummy lib to ensure all types are exported
contract AbiExport {
    function ensureExported(
        ZkgmPacket calldata,
        Instruction calldata,
        Forward calldata,
        Call calldata,
        Batch calldata,
        TokenOrderV1 calldata,
        Ack calldata,
        BatchAck calldata,
        TokenOrderAck calldata,
        TokenOrderV2 calldata,
        TokenMetadata calldata
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
    address public immutable SEND_IMPL;
    address public immutable STAKE_IMPL;
    address public immutable FAO_IMPL;

    constructor(
        IIBCModulePacket _ibcHandler,
        UCS03ZkgmSendImpl _sendImpl,
        UCS03ZkgmStakeImpl _stakeImpl,
        UCS03ZkgmTokenOrderImpl _faoImpl
    ) {
        _disableInitializers();
        IBC_HANDLER = _ibcHandler;
        SEND_IMPL = address(_sendImpl);
        STAKE_IMPL = address(_stakeImpl);
        FAO_IMPL = address(_faoImpl);
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
        GovernanceToken calldata governanceToken
    ) public restricted {
        if (channelGovernanceToken[channelId].unwrappedToken.length != 0) {
            revert ZkgmLib.ErrChannelGovernanceTokenAlreadySet();
        }
        channelGovernanceToken[channelId] = governanceToken;
    }

    function overwriteGovernanceToken(
        uint32 channelId,
        GovernanceToken calldata governanceToken
    ) public restricted {
        channelGovernanceToken[channelId] = governanceToken;
    }

    function predictWrappedToken(
        uint256 path,
        uint32 channel,
        bytes calldata token
    ) public returns (address, bytes32) {
        passthrough(address(SEND_IMPL));
    }

    function predictWrappedTokenV2(
        uint256 path,
        uint32 channel,
        bytes calldata token,
        TokenMetadata calldata metadata
    ) public returns (address, bytes32) {
        passthrough(address(SEND_IMPL));
    }

    function predictWrappedTokenFromMetadataImageV2(
        uint256 path,
        uint32 channel,
        bytes calldata token,
        bytes32 metadataImage
    ) public returns (address, bytes32) {
        passthrough(address(SEND_IMPL));
    }

    function send(
        uint32 channelId,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt,
        Instruction calldata instruction
    ) public payable whenNotPaused {
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
            // The acknowledgement may be asynchronous (forward/call).
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
        if (instruction.isInst(ZkgmLib.OP_TOKEN_ORDER, ZkgmLib.INSTR_VERSION_1))
        {
            TokenOrderV1 calldata order =
                ZkgmLib.decodeTokenOrderV1(instruction.operand);
            bytes memory rawResult = _callFAOImpl(
                abi.encodeCall(
                    UCS03ZkgmTokenOrderImpl.executeTokenOrderV1,
                    (
                        caller,
                        ibcPacket,
                        relayer,
                        relayerMsg,
                        path,
                        order,
                        intent
                    )
                )
            );
            return abi.decode(rawResult, (bytes));
        } else if (
            instruction.isInst(ZkgmLib.OP_TOKEN_ORDER, ZkgmLib.INSTR_VERSION_2)
        ) {
            TokenOrderV2 calldata order =
                ZkgmLib.decodeTokenOrderV2(instruction.operand);
            bytes memory rawResult = _callFAOImpl(
                abi.encodeCall(
                    UCS03ZkgmTokenOrderImpl.executeTokenOrderV2,
                    (
                        caller,
                        ibcPacket,
                        relayer,
                        relayerMsg,
                        path,
                        order,
                        intent
                    )
                )
            );
            return abi.decode(rawResult, (bytes));
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
        } else if (instruction.isInst(ZkgmLib.OP_CALL, ZkgmLib.INSTR_VERSION_0))
        {
            return _executeCall(
                caller,
                ibcPacket,
                relayer,
                relayerMsg,
                path,
                salt,
                ZkgmLib.decodeCall(instruction.operand),
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
            // the only instructions allowed in a batch are call and
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

    function _executeCall(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        uint256 path,
        bytes32 salt,
        Call calldata call,
        bool intent
    ) internal returns (bytes memory) {
        address contractAddress = address(bytes20(call.contractAddress));
        if (!call.eureka) {
            if (intent) {
                IZkgmable(contractAddress).onIntentZkgm(
                    caller,
                    path,
                    ibcPacket.sourceChannelId,
                    ibcPacket.destinationChannelId,
                    call.sender,
                    call.contractCalldata,
                    relayer,
                    relayerMsg
                );
            } else {
                IZkgmable(contractAddress).onZkgm(
                    caller,
                    path,
                    ibcPacket.sourceChannelId,
                    ibcPacket.destinationChannelId,
                    call.sender,
                    call.contractCalldata,
                    relayer,
                    relayerMsg
                );
            }
            return abi.encode(ZkgmLib.ACK_SUCCESS);
        } else {
            IBCPacket memory callIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: ZkgmLib.encodeCallCalldata(
                    path, call.sender, call.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            bytes memory acknowledgement;
            if (intent) {
                acknowledgement = IIBCModuleRecv(contractAddress)
                    .onRecvIntentPacket(caller, callIbcPacket, relayer, relayerMsg);
            } else {
                acknowledgement = IIBCModuleRecv(contractAddress).onRecvPacket(
                    caller, callIbcPacket, relayer, relayerMsg
                );
            }
            if (acknowledgement.length == 0) {
                revert ZkgmLib.ErrAsyncCallUnsupported();
            }
            return acknowledgement;
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

    function _callFAOImpl(
        bytes memory data
    ) internal returns (bytes memory) {
        return FAO_IMPL.delegateCallContract(data);
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
        if (instruction.isInst(ZkgmLib.OP_TOKEN_ORDER, ZkgmLib.INSTR_VERSION_1))
        {
            TokenOrderV1 calldata order =
                ZkgmLib.decodeTokenOrderV1(instruction.operand);
            _callFAOImpl(
                abi.encodeCall(
                    UCS03ZkgmTokenOrderImpl.acknowledgeTokenOrderV1,
                    (ibcPacket, relayer, path, salt, order, successful, ack)
                )
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_TOKEN_ORDER, ZkgmLib.INSTR_VERSION_2)
        ) {
            TokenOrderV2 calldata order =
                ZkgmLib.decodeTokenOrderV2(instruction.operand);
            _callFAOImpl(
                abi.encodeCall(
                    UCS03ZkgmTokenOrderImpl.acknowledgeTokenOrderV2,
                    (ibcPacket, relayer, path, salt, order, successful, ack)
                )
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
        } else if (instruction.isInst(ZkgmLib.OP_CALL, ZkgmLib.INSTR_VERSION_0))
        {
            _acknowledgeCall(
                caller,
                ibcPacket,
                relayer,
                path,
                salt,
                ZkgmLib.decodeCall(instruction.operand),
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
        } else if (
            instruction.isInst(
                ZkgmLib.OP_WITHDRAW_REWARDS, ZkgmLib.INSTR_VERSION_0
            )
        ) {
            _callStakeImpl(
                abi.encodeCall(
                    UCS03ZkgmStakeImpl.acknowledgeWithdrawRewards,
                    (
                        ibcPacket,
                        ZkgmLib.decodeWithdrawRewards(instruction.operand),
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

    function _acknowledgeCall(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        Call calldata call,
        bool successful,
        bytes calldata ack
    ) internal {
        if (successful && call.eureka) {
            IBCPacket memory callIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: ZkgmLib.encodeCallCalldata(
                    path, call.sender, call.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            IIBCModule(address(bytes20(call.sender))).onAcknowledgementPacket(
                caller, callIbcPacket, ack, relayer
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
        if (instruction.isInst(ZkgmLib.OP_TOKEN_ORDER, ZkgmLib.INSTR_VERSION_1))
        {
            TokenOrderV1 calldata order =
                ZkgmLib.decodeTokenOrderV1(instruction.operand);
            _callFAOImpl(
                abi.encodeCall(
                    UCS03ZkgmTokenOrderImpl.timeoutTokenOrderV1,
                    (ibcPacket, path, order)
                )
            );
        } else if (
            instruction.isInst(ZkgmLib.OP_TOKEN_ORDER, ZkgmLib.INSTR_VERSION_2)
        ) {
            TokenOrderV2 calldata order =
                ZkgmLib.decodeTokenOrderV2(instruction.operand);
            _callFAOImpl(
                abi.encodeCall(
                    UCS03ZkgmTokenOrderImpl.timeoutTokenOrderV2,
                    (ibcPacket, path, order)
                )
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
        } else if (instruction.isInst(ZkgmLib.OP_CALL, ZkgmLib.INSTR_VERSION_0))
        {
            _timeoutCall(
                caller,
                ibcPacket,
                relayer,
                path,
                ZkgmLib.decodeCall(instruction.operand)
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
        } else if (
            instruction.isInst(
                ZkgmLib.OP_WITHDRAW_REWARDS, ZkgmLib.INSTR_VERSION_0
            )
        ) {
            _callStakeImpl(
                abi.encodeCall(
                    UCS03ZkgmStakeImpl.timeoutWithdrawRewards,
                    (
                        ibcPacket,
                        ZkgmLib.decodeWithdrawRewards(instruction.operand)
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

    function _timeoutCall(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        Call calldata call
    ) internal {
        if (call.eureka) {
            IBCPacket memory callIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: ZkgmLib.encodeCallCalldata(
                    path, call.contractAddress, call.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            IIBCModule(address(bytes20(call.sender))).onTimeoutPacket(
                caller, callIbcPacket, relayer
            );
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

    function migrateV1ToV2(
        V1ToV2Migration[] calldata migrations
    ) public restricted {
        for (uint256 i = 0; i < migrations.length; i++) {
            V1ToV2Migration calldata migration = migrations[i];
            uint256 balance = _deprecated_channelBalanceV1[migration.channelId][migration
                .path][migration.baseToken];
            if (balance == 0) {
                revert("no balance");
            }
            _deprecated_channelBalanceV1[migration.channelId][migration.path][migration
                .baseToken] = 0;
            _increaseOutstandingV2(
                migration.channelId,
                migration.path,
                migration.baseToken,
                migration.quoteToken,
                balance
            );
        }
    }

    receive() external payable {}
}
