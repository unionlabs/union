pragma solidity ^0.8.23;

import "forge-std/Vm.sol";
import "forge-std/Script.sol";

import "@openzeppelin-foundry-upgradeable/Upgrades.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/access/Ownable.sol";

import "../contracts/core/02-client/IBCClient.sol";
import "../contracts/core/03-connection/IBCConnection.sol";
import "../contracts/core/04-channel/IBCChannelHandshake.sol";
import "../contracts/core/04-channel/IBCPacket.sol";
import "../contracts/core/OwnableIBCHandler.sol";
import "../contracts/clients/CometblsClientV2.sol";
import "../contracts/apps/ucs/01-relay/Relay.sol";
import "../contracts/apps/ucs/02-nft/NFT.sol";
import "../contracts/lib/Hex.sol";

import "solady/utils/CREATE3.sol";
import "solady/utils/LibString.sol";

library IBC {
    string constant BASED = "ibc-is-based";
}

library LightClients {
    string constant NAMESPACE = "lightclients";
    string constant COMETBLS = "cometbls";

    function make(string memory lightClient)
        internal
        pure
        returns (string memory)
    {
        return string(abi.encodePacked(NAMESPACE, "/", lightClient));
    }
}

library Protocols {
    string constant NAMESPACE = "protocols";
    string constant UCS01 = "ucs01";
    string constant UCS02 = "ucs02";

    function make(string memory protocol)
        internal
        pure
        returns (string memory)
    {
        return string(abi.encodePacked(NAMESPACE, "/", protocol));
    }
}

contract Deployer {
    using LibString for *;

    function deploy(
        string memory salt,
        bytes calldata creationCode,
        uint256 value
    ) public returns (address) {
        return CREATE3.deploy(
            keccak256(abi.encodePacked(msg.sender.toHexString(), "/", salt)),
            creationCode,
            value
        );
    }
}

contract DeployDeployer is Script {
    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(privateKey);
        new Deployer();
        vm.stopBroadcast();
    }
}

contract DeployIBC is Script {
    function run() public {
        Deployer deployer = Deployer(vm.envAddress("DEPLOYER"));
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        VmSafe.Wallet memory wallet = vm.createWallet(privateKey);
        vm.startBroadcast(privateKey);

        // 0xed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5
        IBCHandler handler = IBCHandler(
            deployer.deploy(
                IBC.BASED,
                abi.encodePacked(
                    type(ERC1967Proxy).creationCode,
                    abi.encode(
                        address(new OwnableIBCHandler()),
                        abi.encodeCall(
                            IBCHandler.initialize,
                            (
                                address(new IBCClient()),
                                address(new IBCConnection()),
                                address(new IBCChannelHandshake()),
                                address(new IBCPacket()),
                                wallet.addr
                            )
                        )
                    )
                ),
                0
            )
        );

        // 0xc4f27a952faBa4174ce0Ee6D9d0c6F4c41524d49
        CometblsClient client = CometblsClient(
            deployer.deploy(
                LightClients.make(LightClients.COMETBLS),
                abi.encodePacked(
                    type(ERC1967Proxy).creationCode,
                    abi.encode(
                        address(new CometblsClient()),
                        abi.encodeCall(
                            CometblsClient.initialize,
                            (address(handler), wallet.addr)
                        )
                    )
                ),
                0
            )
        );

        // 0xa9d03ba6E27B43c69a64C87F845485b73A8e5d46
        UCS01Relay relay = UCS01Relay(
            deployer.deploy(
                Protocols.make(Protocols.UCS01),
                abi.encodePacked(
                    type(ERC1967Proxy).creationCode,
                    abi.encode(
                        address(new UCS01Relay()),
                        abi.encodeCall(
                            UCS01Relay.initialize, (handler, wallet.addr)
                        )
                    )
                ),
                0
            )
        );

        // 0x524D4d28fc90dc5A257162abE37081f52681C7D6
        UCS02NFT nft = UCS02NFT(
            deployer.deploy(
                Protocols.make(Protocols.UCS02),
                abi.encodePacked(
                    type(ERC1967Proxy).creationCode,
                    abi.encode(
                        address(new UCS02NFT()),
                        abi.encodeCall(
                            UCS01Relay.initialize, (handler, wallet.addr)
                        )
                    )
                ),
                0
            )
        );
        vm.stopBroadcast();

        console.log(address(handler));
        console.log(address(client));
        console.log(address(relay));
        console.log(address(nft));
    }
}
