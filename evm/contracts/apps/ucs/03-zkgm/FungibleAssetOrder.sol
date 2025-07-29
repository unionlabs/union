pragma solidity ^0.8.27;

import "./Store.sol";

contract UCS03ZkgmFungibleAssetOrderImpl is
    Versioned,
    TokenBucket,
    UCS03ZkgmStore
{
    using ZkgmLib for *;
    using LibString for *;
    using LibBytes for *;
    using SafeERC20 for *;
    using Address for *;
    using LibCall for *;

    IWETH public immutable WETH;
    ZkgmERC20 public immutable ERC20_IMPL;
    bool public immutable RATE_LIMIT_ENABLED;

    constructor(IWETH _weth, ZkgmERC20 _erc20Impl, bool _rateLimitEnabled) {
        WETH = _weth;
        ERC20_IMPL = _erc20Impl;
        RATE_LIMIT_ENABLED = _rateLimitEnabled;
    }

    function _protocolFillMint(
        uint32 channelId,
        uint256 path,
        address wrappedToken,
        address receiver,
        address relayer,
        uint256 baseAmount,
        uint256 quoteAmount
    ) internal returns (bytes memory) {
        uint256 fee = baseAmount - quoteAmount;
        if (quoteAmount > 0) {
            IZkgmERC20(wrappedToken).mint(receiver, quoteAmount);
        }
        if (fee > 0) {
            IZkgmERC20(wrappedToken).mint(relayer, fee);
        }
        return ZkgmLib.encodeFungibleAssetOrderAck(
            FungibleAssetOrderAck({
                fillType: ZkgmLib.FILL_TYPE_PROTOCOL,
                marketMaker: ZkgmLib.ACK_EMPTY
            })
        );
    }

    function _protocolFillUnescrowV2(
        uint32 channelId,
        uint256 path,
        bytes calldata baseToken,
        address quoteToken,
        address receiver,
        address relayer,
        uint256 baseAmount,
        uint256 quoteAmount
    ) internal returns (bytes memory) {
        uint256 fee = baseAmount - quoteAmount;
        // If the base token path is being unwrapped, it's escrowed balance will be non zero.
        _decreaseOutstandingV2(
            channelId,
            ZkgmLib.reverseChannelPath(path),
            quoteToken,
            baseToken,
            baseAmount
        );
        // Specific case for native token.
        if (quoteToken == ZkgmLib.NATIVE_TOKEN_ERC_7528_ADDRESS) {
            if (quoteAmount + fee > 0) {
                WETH.withdraw(baseAmount);
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

    function _marketMakerFillV2(
        IBCPacket calldata packet,
        address caller,
        address relayer,
        bytes calldata relayerMsg,
        address quoteToken,
        address payable receiver,
        TokenOrderV2 calldata order,
        bool intent
    ) internal returns (bytes memory) {
        uint256 quoteAmount = order.quoteAmount;
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
            if (quoteAmount > 0) {
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
            }
        } else {
            bool solver = ZkgmLib.isSolver(quoteToken);
            bool solverFilled = false;
            if (solver) {
                // Even if the quote amount is zero, this may be some sort of
                // mechanism to interact with a contract, hence, we allows the
                // call to happen because the solver may constraint the
                // execution.
                (solverFilled,,) = quoteToken.tryCall(
                    0,
                    gasleft(),
                    type(uint16).max,
                    abi.encodeCall(
                        ISolver.solve,
                        (packet, order, caller, relayer, relayerMsg, intent)
                    )
                );
                if (!solverFilled && !ISolver(quoteToken).allowMarketMakers()) {
                    return ZkgmLib.ACK_ERR_ONLYMAKER;
                }
            }
            if (!solverFilled && quoteAmount > 0) {
                if (
                    !IERC20(quoteToken).trySafeTransferFrom(
                        caller, receiver, quoteAmount
                    )
                ) {
                    return ZkgmLib.ACK_ERR_ONLYMAKER;
                }
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

    function _deployWrappedTokenV2Memory(
        uint32 channelId,
        uint256 path,
        bytes calldata unwrappedToken,
        address wrappedToken,
        bytes32 wrappedTokenSalt,
        FungibleAssetMetadata memory metadata,
        bool canDeploy
    ) internal {
        if (!ZkgmLib.isDeployed(wrappedToken)) {
            if (!canDeploy) {
                revert ZkgmLib.ErrCannotDeploy();
            }
            CREATE3.deployDeterministic(
                abi.encodePacked(
                    type(ERC1967Proxy).creationCode,
                    abi.encode(
                        address(bytes20(metadata.implementation)),
                        metadata.initializer
                    )
                ),
                wrappedTokenSalt
            );
            tokenOrigin[wrappedToken] =
                ZkgmLib.updateChannelPath(path, channelId);
            metadataImageOf[wrappedToken] = EfficientHashLib.hash(
                ZkgmLib.encodeFungibleAssetMetadata(metadata)
            );
        }
    }

    function _deployWrappedTokenV2(
        uint32 channelId,
        uint256 path,
        bytes calldata unwrappedToken,
        address wrappedToken,
        bytes32 wrappedTokenSalt,
        FungibleAssetMetadata calldata metadata,
        bool canDeploy
    ) internal {
        if (!ZkgmLib.isDeployed(wrappedToken)) {
            if (!canDeploy) {
                revert ZkgmLib.ErrCannotDeploy();
            }
            CREATE3.deployDeterministic(
                abi.encodePacked(
                    type(ERC1967Proxy).creationCode,
                    abi.encode(
                        address(bytes20(metadata.implementation)),
                        metadata.initializer
                    )
                ),
                wrappedTokenSalt
            );
            tokenOrigin[wrappedToken] =
                ZkgmLib.updateChannelPath(path, channelId);
            metadataImageOf[wrappedToken] = EfficientHashLib.hash(
                ZkgmLib.encodeFungibleAssetMetadata(metadata)
            );
        }
    }

    function _makeDefaultFungibleAssetMetadata(
        FungibleAssetOrder calldata order
    ) internal view returns (FungibleAssetMetadata memory) {
        return FungibleAssetMetadata({
            implementation: abi.encodePacked(ERC20_IMPL),
            initializer: abi.encodeCall(
                ZkgmERC20.initialize,
                (
                    authority(),
                    address(this),
                    order.baseTokenName,
                    order.baseTokenSymbol,
                    order.baseTokenDecimals
                )
            )
        });
    }

    function _optionalRateLimit(address token, uint256 amount) internal {
        if (RATE_LIMIT_ENABLED) {
            _rateLimit(token, amount);
        }
    }

    function executeFungibleAssetOrder(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        uint256 path,
        FungibleAssetOrder calldata order,
        bool intent
    ) public returns (bytes memory) {
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
            FungibleAssetMetadata memory metadata =
                _makeDefaultFungibleAssetMetadata(order);
            _deployWrappedTokenV2Memory(
                ibcPacket.destinationChannelId,
                path,
                order.baseToken,
                wrappedToken,
                wrappedTokenSalt,
                metadata,
                true
            );
            return _protocolFillMint(
                ibcPacket.destinationChannelId,
                path,
                wrappedToken,
                receiver,
                relayer,
                order.baseAmount,
                order.quoteAmount
            );
        } else if (order.baseTokenPath != 0 && baseAmountCoversQuoteAmount) {
            _optionalRateLimit(quoteToken, order.quoteAmount);
            return _protocolFillUnescrowV2(
                ibcPacket.destinationChannelId,
                path,
                order.baseToken,
                quoteToken,
                receiver,
                relayer,
                order.baseAmount,
                order.quoteAmount
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

    function executeTokenOrderV2(
        address caller,
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        uint256 path,
        TokenOrderV2 calldata order,
        bool intent
    ) public returns (bytes memory) {
        address quoteToken = address(bytes20(order.quoteToken));
        address payable receiver = payable(address(bytes20(order.receiver)));

        // For intent packets, the protocol is not allowed to provide any fund
        // as the packet has not been checked for membership poof. Instead, we
        // know the market maker will be repaid on the source chain, if and only
        // if the currently executing packet hash had been registered as sent on
        // the source. In other words, the market maker is unable to lie.
        if (intent) {
            return _marketMakerFillV2(
                ibcPacket,
                caller,
                relayer,
                relayerMsg,
                quoteToken,
                receiver,
                order,
                intent
            );
        }

        bool baseAmountCoversQuoteAmount = order.baseAmount >= order.quoteAmount;

        if (
            order.kind == ZkgmLib.TOKEN_ORDER_KIND_UNESCROW
                && baseAmountCoversQuoteAmount
        ) {
            _optionalRateLimit(quoteToken, order.quoteAmount);
            return _protocolFillUnescrowV2(
                ibcPacket.destinationChannelId,
                path,
                order.baseToken,
                quoteToken,
                receiver,
                relayer,
                order.baseAmount,
                order.quoteAmount
            );
        } else {
            address wrappedToken;
            bytes32 wrappedTokenSalt;
            // Decode is noop here as it's directly indexing in the calldata,
            // even if the metadata is empty this will not fail.
            FungibleAssetMetadata calldata metadata =
                ZkgmLib.decodeFungibleAssetMetadata(order.metadata);
            if (order.kind == ZkgmLib.TOKEN_ORDER_KIND_ESCROW) {
                bytes32 metadataImage = metadataImageOf[quoteToken];
                if (metadataImage == 0) {
                    // V1
                    (wrappedToken, wrappedTokenSalt) = _predictWrappedToken(
                        path, ibcPacket.destinationChannelId, order.baseToken
                    );
                } else {
                    // V2
                    (wrappedToken, wrappedTokenSalt) =
                    _predictWrappedTokenFromMetadataImageV2(
                        path,
                        ibcPacket.destinationChannelId,
                        order.baseToken,
                        metadataImage
                    );
                }
            } else if (order.kind == ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE) {
                (wrappedToken, wrappedTokenSalt) = _predictWrappedTokenV2(
                    path,
                    ibcPacket.destinationChannelId,
                    order.baseToken,
                    metadata
                );
            }

            if (quoteToken == wrappedToken && baseAmountCoversQuoteAmount) {
                _optionalRateLimit(quoteToken, order.quoteAmount);
                // The asset can only be deployed if the metadata preimage is provided.
                bool canDeploy =
                    order.kind == ZkgmLib.TOKEN_ORDER_KIND_INITIALIZE;
                _deployWrappedTokenV2(
                    ibcPacket.destinationChannelId,
                    path,
                    order.baseToken,
                    wrappedToken,
                    wrappedTokenSalt,
                    metadata,
                    canDeploy
                );
                return _protocolFillMint(
                    ibcPacket.destinationChannelId,
                    path,
                    wrappedToken,
                    receiver,
                    relayer,
                    order.baseAmount,
                    order.quoteAmount
                );
            } else {
                // We also allow market makers to fill orders after finality. This
                // allow orders that combines protocol and mm filling (wrapped vs
                // non wrapped assets).
                return _marketMakerFillV2(
                    ibcPacket,
                    caller,
                    relayer,
                    relayerMsg,
                    quoteToken,
                    receiver,
                    order,
                    intent
                );
            }
        }
    }

    function _acknowledgeFungibleAssetOrder(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        FungibleAssetOrder calldata order,
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
                address baseToken = address(bytes20(order.baseToken));
                if (order.baseTokenPath != 0) {
                    IZkgmERC20(address(baseToken)).mint(
                        marketMaker, order.baseAmount
                    );
                } else {
                    _decreaseOutstandingV2(
                        ibcPacket.sourceChannelId,
                        path,
                        baseToken,
                        order.quoteToken,
                        order.baseAmount
                    );
                    IERC20(baseToken).safeTransfer(
                        marketMaker, order.baseAmount
                    );
                }
            } else {
                revert ZkgmLib.ErrInvalidFillType();
            }
        } else {
            _refund(ibcPacket.sourceChannelId, path, order);
        }
    }

    function acknowledgeFungibleAssetOrder(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        FungibleAssetOrder calldata order,
        bool successful,
        bytes calldata ack
    ) public {
        _acknowledgeFungibleAssetOrder(
            ibcPacket, relayer, path, salt, order, successful, ack
        );
    }

    function acknowledgeTokenOrderV2(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        TokenOrderV2 calldata order,
        bool successful,
        bytes calldata ack
    ) public {
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
                address baseToken = address(bytes20(order.baseToken));
                if (order.kind == ZkgmLib.TOKEN_ORDER_KIND_UNESCROW) {
                    IZkgmERC20(address(baseToken)).mint(
                        marketMaker, order.baseAmount
                    );
                } else {
                    _decreaseOutstandingV2(
                        ibcPacket.sourceChannelId,
                        path,
                        baseToken,
                        order.quoteToken,
                        order.baseAmount
                    );
                    IERC20(baseToken).safeTransfer(
                        marketMaker, order.baseAmount
                    );
                }
            } else {
                revert ZkgmLib.ErrInvalidFillType();
            }
        } else {
            _refundV2(ibcPacket.sourceChannelId, path, order);
        }
    }

    function timeoutFungibleAssetOrder(
        IBCPacket calldata ibcPacket,
        uint256 path,
        FungibleAssetOrder calldata order
    ) public {
        _refund(ibcPacket.sourceChannelId, path, order);
    }

    function timeoutTokenOrderV2(
        IBCPacket calldata ibcPacket,
        uint256 path,
        TokenOrderV2 calldata order
    ) public {
        _refundV2(ibcPacket.sourceChannelId, path, order);
    }

    function _refund(
        uint32 sourceChannelId,
        uint256 path,
        FungibleAssetOrder calldata order
    ) internal {
        address sender = address(bytes20(order.sender));
        address baseToken = address(bytes20(order.baseToken));
        if (order.baseTokenPath != 0) {
            IZkgmERC20(address(baseToken)).mint(sender, order.baseAmount);
        } else {
            _decreaseOutstandingV2(
                sourceChannelId,
                path,
                baseToken,
                order.quoteToken,
                order.baseAmount
            );
            IERC20(baseToken).safeTransfer(sender, order.baseAmount);
        }
    }

    function _refundV2(
        uint32 sourceChannelId,
        uint256 path,
        TokenOrderV2 calldata order
    ) internal {
        address sender = address(bytes20(order.sender));
        address baseToken = address(bytes20(order.baseToken));
        if (order.kind == ZkgmLib.TOKEN_ORDER_KIND_UNESCROW) {
            IZkgmERC20(address(baseToken)).mint(sender, order.baseAmount);
        } else {
            _decreaseOutstandingV2(
                sourceChannelId,
                path,
                baseToken,
                order.quoteToken,
                order.baseAmount
            );
            IERC20(baseToken).safeTransfer(sender, order.baseAmount);
        }
    }
}
