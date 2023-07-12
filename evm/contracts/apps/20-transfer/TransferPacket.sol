pragma solidity ^0.8.18;

struct TransferPacket {
    uint256 amount;
    string denom;
    string receiver;
    string sender;
}

library TransferPacketHelp {
    function encode(
        TransferPacket memory packet
    ) internal pure returns (bytes memory) {
        return
            abi.encode(
                packet.amount,
                packet.denom,
                packet.receiver,
                packet.sender
            );
    }

    function decode(
        bytes memory packet
    ) internal pure returns (TransferPacket memory) {
        (
            uint256 amount,
            string memory denom,
            string memory receiver,
            string memory sender
        ) = abi.decode(packet, (uint256, string, string, string));
        return
            TransferPacket({
                amount: amount,
                denom: denom,
                receiver: receiver,
                sender: sender
            });
    }
}
