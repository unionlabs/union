pragma solidity ^0.8.27;

import "./Store.sol";

contract UCS03ZkgmSendImpl is Versioned, UCS03ZkgmStore {
    using ZkgmLib for *;
    using LibString for *;
    using LibBytes for *;
    using SafeERC20 for *;
    using Address for *;
    using LibCall for *;

    IIBCModulePacket public immutable IBC_HANDLER;
    IWETH public immutable WETH;
    ZkgmERC20 public immutable ERC20_IMPL;
    bytes32 public immutable NATIVE_TOKEN_NAME_HASH;
    bytes32 public immutable NATIVE_TOKEN_SYMBOL_HASH;
    uint8 public immutable NATIVE_TOKEN_DECIMALS;

    constructor(
        IIBCModulePacket _ibcHandler,
        IWETH _weth,
        ZkgmERC20 _erc20Impl,
        string memory _nativeTokenName,
        string memory _nativeTokenSymbol,
        uint8 _nativeTokenDecimals
    ) {
        IBC_HANDLER = _ibcHandler;
        WETH = _weth;
        ERC20_IMPL = _erc20Impl;
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
    ) public payable {
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
        if (instruction.isInst(ZkgmLib.OP_TOKEN_ORDER, ZkgmLib.INSTR_VERSION_1))
        {
            TokenOrderV1 calldata order =
                ZkgmLib.decodeTokenOrderV1(instruction.operand);
            _verifyTokenOrderV1(channelId, path, order);
        } else if (
            instruction.isInst(ZkgmLib.OP_TOKEN_ORDER, ZkgmLib.INSTR_VERSION_2)
        ) {
            TokenOrderV2 calldata order =
                ZkgmLib.decodeTokenOrderV2(instruction.operand);
            _verifyTokenOrderV2(channelId, path, order);
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
        } else if (instruction.isInst(ZkgmLib.OP_CALL, ZkgmLib.INSTR_VERSION_0))
        {
            _verifyCall(
                channelId, path, ZkgmLib.decodeCall(instruction.operand)
            );
        } else {
            revert ZkgmLib.ErrUnknownOpcode();
        }
    }

    function _verifyTokenOrderV1(
        uint32 channelId,
        uint256 path,
        TokenOrderV1 calldata order
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
            _increaseOutstandingV2(
                channelId,
                path,
                address(baseToken),
                order.quoteToken,
                order.baseAmount
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

    function _verifyTokenOrderV2(
        uint32 channelId,
        uint256 path,
        TokenOrderV2 calldata order
    ) internal {
        address baseToken = address(bytes20(order.baseToken));

        if (order.kind == ZkgmLib.TOKEN_ORDER_KIND_UNESCROW) {
            (uint256 intermediateChannelPath, uint32 destinationChannelId) =
                ZkgmLib.popChannelFromPath(tokenOrigin[baseToken]);
            bool isInverseIntermediatePath =
                path == ZkgmLib.reverseChannelPath(intermediateChannelPath);
            bool isSendingBackToSameChannel = destinationChannelId == channelId;

            // Predict V1
            (address wrappedTokenV1,) = _predictWrappedToken(
                intermediateChannelPath, channelId, order.quoteToken
            );

            // Predict V2
            bytes32 metadataImage = metadataImageOf[baseToken];
            (address wrappedTokenV2,) = _predictWrappedTokenFromMetadataImageV2(
                intermediateChannelPath,
                channelId,
                order.quoteToken,
                metadataImage
            );

            bool isUnwrappingV1 =
                order.baseToken.eq(abi.encodePacked(wrappedTokenV1));
            bool isUnwrappingV2 =
                order.baseToken.eq(abi.encodePacked(wrappedTokenV2));
            bool isUnwrapping = isUnwrappingV1 || isUnwrappingV2;

            if (
                !(
                    isUnwrapping && isInverseIntermediatePath
                        && isSendingBackToSameChannel
                )
            ) {
                revert ZkgmLib.ErrInvalidUnescrow();
            }

            IZkgmERC20(baseToken).burn(msg.sender, order.baseAmount);
        } else {
            _increaseOutstandingV2(
                channelId, path, baseToken, order.quoteToken, order.baseAmount
            );
            if (
                baseToken == ZkgmLib.NATIVE_TOKEN_ERC_7528_ADDRESS
                    && msg.value >= order.baseAmount
            ) {
                // Use the deposit as a mechanism to consume the order amount from the msg.value.
                // This avoids issue if multiple native eth orders are present.
                WETH.deposit{value: order.baseAmount}();
            } else {
                IERC20(baseToken).safeTransferFrom(
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

    function _verifyCall(
        uint32 channelId,
        uint256 path,
        Call calldata call
    ) internal {
        if (!call.sender.eq(abi.encodePacked(msg.sender))) {
            revert ZkgmLib.ErrInvalidCallSender();
        }
    }

    function predictWrappedToken(
        uint256 path,
        uint32 channel,
        bytes calldata token
    ) external view returns (address, bytes32) {
        return _predictWrappedToken(path, channel, token);
    }

    function predictWrappedTokenV2(
        uint256 path,
        uint32 channel,
        bytes calldata token,
        TokenMetadata calldata metadata
    ) external returns (address, bytes32) {
        return _predictWrappedTokenV2(path, channel, token, metadata);
    }

    function predictWrappedTokenFromMetadataImageV2(
        uint256 path,
        uint32 channel,
        bytes calldata token,
        bytes32 metadataImage
    ) external returns (address, bytes32) {
        return _predictWrappedTokenFromMetadataImageV2(
            path, channel, token, metadataImage
        );
    }

    function predictProxyAccount(
        uint256 path,
        uint32 channelId,
        bytes calldata sender
    ) external returns (bytes32, address) {
        return _predictProxyAccount(path, channelId, sender);
    }
}
