pragma solidity ^0.8.18;

import "../../contracts/proto/ibc/core/channel/v1/channel.sol";
import "../../contracts/core/05-port/IIBCModule.sol";
import "../../contracts/core/25-handler/IBCHandler.sol";
import "../../contracts/core/24-host/IBCHost.sol";
import "@openzeppelin/contracts/utils/Context.sol";

contract MockApp is IIBCModule {
    event MockRecv(bool ok);

    /// Module callbacks ///

    function onRecvPacket(IbcCoreChannelV1Packet.Data calldata, address) external virtual override returns (bytes memory) {
        emit MockRecv(true);
        return bytes("1");
    }

    function onAcknowledgementPacket(IbcCoreChannelV1Packet.Data calldata packet, bytes calldata acknowledgement, address relayer)
        external
        virtual
        override
    {}

    function onChanOpenInit(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata,
        string calldata,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata,
        string calldata
    ) external virtual override {}

    function onChanOpenTry(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata,
        string calldata,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata,
        string calldata,
        string calldata
    ) external virtual override {}

    function onChanOpenAck(string calldata portId, string calldata channelId, string calldata counterpartyVersion)
        external
        virtual
        override
    {}

    function onChanOpenConfirm(string calldata portId, string calldata channelId) external virtual override {}

    function onChanCloseInit(string calldata portId, string calldata channelId) external virtual override {}

    function onChanCloseConfirm(string calldata portId, string calldata channelId) external virtual override {}
}
