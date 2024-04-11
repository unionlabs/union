pragma solidity ^0.8.23;

import "../24-host/IBCHost.sol";
import "./IBCClientHandler.sol";
import "./IBCConnectionHandler.sol";
import "./IBCChannelHandler.sol";
import "./IBCPacketHandler.sol";
import "./IBCQuerier.sol";

import "@openzeppelin-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";
import "@openzeppelin-upgradeable/utils/ContextUpgradeable.sol";
import "@openzeppelin/utils/Context.sol";

/**
 * @dev IBCHandler is a contract that implements [ICS-25](https://github.com/cosmos/ibc/tree/main/spec/core/ics-025-handler-interface).
 */
abstract contract IBCHandler is
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable,
    IBCHost,
    IBCClientHandler,
    IBCConnectionHandler,
    IBCChannelHandler,
    IBCPacketHandler,
    IBCQuerier
{
    constructor() {
        _disableInitializers();
    }

    /**
     * @dev The arguments of constructor must satisfy the followings:
     * @param _ibcClient is the address of a contract that implements `IIBCClient`.
     * @param _ibcConnection is the address of a contract that implements `IIBCConnectionHandshake`.
     * @param _ibcChannel is the address of a contract that implements `IIBCChannelHandshake`.
     * @param _ibcPacket is the address of a contract that implements `IIBCPacket`.
     */
    function initialize(
        address _ibcClient,
        address _ibcConnection,
        address _ibcChannel,
        address _ibcPacket
    ) public virtual initializer {
        __Ownable_init(msg.sender);
        __UUPSUpgradeable_init();
        ibcClient = _ibcClient;
        ibcConnection = _ibcConnection;
        ibcChannel = _ibcChannel;
        ibcPacket = _ibcPacket;
    }

    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyOwner
    {}

    function _msgSender()
        internal
        view
        virtual
        override(Context, ContextUpgradeable)
        returns (address sender)
    {
        return ContextUpgradeable._msgSender();
    }

    function _msgData()
        internal
        view
        virtual
        override(Context, ContextUpgradeable)
        returns (bytes calldata)
    {
        return ContextUpgradeable._msgData();
    }

    function _contextSuffixLength()
        internal
        view
        virtual
        override(Context, ContextUpgradeable)
        returns (uint256)
    {
        return ContextUpgradeable._contextSuffixLength();
    }
}
