pragma solidity ^0.8.23;

import "forge-std/Vm.sol";
import "forge-std/Script.sol";

import "@openzeppelin-foundry-upgradeable/Upgrades.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/access/Ownable.sol";

import "../contracts/Glue.sol";
import "../contracts/Multicall.sol";
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

bytes constant DEPLOYER_BYTECODE_SOLIDITY_8_23_f704f362 =
    hex"6080806040523461001657610387908161001c8239f35b600080fdfe608060405260048036101561001357600080fd5b6000803560e01c63d83c11381461002957600080fd5b346102495760607ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc3601126102495767ffffffffffffffff90823582811161024557602390366023820112156102415761008c90369060248188013591016102bc565b6024359084821161023d573660238301121561023d578186013594851161023d57366024868401011161023d576040519060808201604052600f936f30313233343536373839616263646566600f5260028301946028865286604a8501523360601b906001885b808001870160228d86841a9086821651898501531c5191015301926014841461011f57926001906100f3565b505050506101a661017a916101b4956130786002825101915284526040519283916001610150602085018098610326565b7f2f0000000000000000000000000000000000000000000000000000000000000081520190610326565b037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0810183528261024c565b5190209360243692016102bc565b916f67363d3d37363d34f03d5260086018f3825260108083f5918215610231578180918460145261d694825260016034536017601e209460208251920190604435905af11561022557813b156102255760208273ffffffffffffffffffffffffffffffffffffffff60405191168152f35b6319b991a8915052601cfd5b8363301164258352601cfd5b8380fd5b8280fd5b5080fd5b80fd5b90601f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0910116810190811067ffffffffffffffff82111761028d57604052565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b92919267ffffffffffffffff821161028d576040519161030460207fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0601f840116018461024c565b829481845281830111610321578281602093846000960137010152565b600080fd5b9081519160005b83811061033e575050016000815290565b806020809284010151818501520161032d56fea26469706673582212201caa34dba5b3bdaed3ef803daa8e62d2b80d0641a0b29ee824018fad5f2e76cd64736f6c63430008170033";
/* contract Deployer { */
/*     using LibString for *; */

/*     function deploy( */
/*         string memory salt, */
/*         bytes calldata creationCode, */
/*         uint256 value */
/*     ) public returns (address) { */
/*         return CREATE3.deploy( */
/*             keccak256(abi.encodePacked(msg.sender.toHexString(), "/", salt)), */
/*             creationCode, */
/*             value */
/*         ); */
/*     } */
/* } */

interface Deployer {
    function deploy(
        string memory salt,
        bytes calldata creationCode,
        uint256 value
    ) external returns (address);
}

library LIB {
    string constant MULTICALL = "multicall";
}

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
    string constant UCS00 = "ucs00";
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

abstract contract UnionBase is Script {
    function deployDeployer() internal returns (Deployer) {
        bytes memory bytecode = DEPLOYER_BYTECODE_SOLIDITY_8_23_f704f362;
        Deployer deployer;
        assembly {
            deployer := create(0, add(bytecode, 0x20), mload(bytecode))
        }
        return deployer;
    }
}

abstract contract UnionScript is UnionBase {
    function getDeployer() internal virtual returns (Deployer);

    function deploy(
        string memory salt,
        bytes memory args
    ) internal returns (address) {
        return getDeployer().deploy(
            salt, abi.encodePacked(type(ERC1967Proxy).creationCode, args), 0
        );
    }

    function deployMulticall() internal returns (Multicall) {
        return Multicall(
            getDeployer().deploy(
                LIB.MULTICALL, abi.encodePacked(type(Multicall).creationCode), 0
            )
        );
    }

    function deployIBCHandler(address owner) internal returns (IBCHandler) {
        return IBCHandler(
            deploy(
                IBC.BASED,
                abi.encode(
                    address(new OwnableIBCHandler()),
                    abi.encodeCall(
                        IBCHandler.initialize,
                        (
                            address(new IBCClient()),
                            address(new IBCConnection()),
                            address(new IBCChannelHandshake()),
                            address(new IBCPacket()),
                            owner
                        )
                    )
                )
            )
        );
    }

    function deployCometbls(
        IBCHandler handler,
        address owner
    ) internal returns (CometblsClient) {
        return CometblsClient(
            deploy(
                LightClients.make(LightClients.COMETBLS),
                abi.encode(
                    address(new CometblsClient()),
                    abi.encodeCall(
                        CometblsClient.initialize, (address(handler), owner)
                    )
                )
            )
        );
    }

    function deployUCS01(
        IBCHandler handler,
        address owner
    ) internal returns (UCS01Relay) {
        return UCS01Relay(
            deploy(
                Protocols.make(Protocols.UCS01),
                abi.encode(
                    address(new UCS01Relay()),
                    abi.encodeCall(UCS01Relay.initialize, (handler, owner))
                )
            )
        );
    }

    function deployUCS02(
        IBCHandler handler,
        address owner
    ) internal returns (UCS02NFT) {
        return UCS02NFT(
            deploy(
                Protocols.make(Protocols.UCS02),
                abi.encode(
                    address(new UCS02NFT()),
                    abi.encodeCall(UCS01Relay.initialize, (handler, owner))
                )
            )
        );
    }

    function deployIBC(address owner)
        internal
        returns (IBCHandler, CometblsClient, UCS01Relay, UCS02NFT, Multicall)
    {
        IBCHandler handler = deployIBCHandler(owner);
        CometblsClient client = deployCometbls(handler, owner);
        UCS01Relay relay = deployUCS01(handler, owner);
        UCS02NFT nft = deployUCS02(handler, owner);
        Multicall multicall = deployMulticall();
        return (handler, client, relay, nft, multicall);
    }
}

contract DeployDeployer is UnionBase {
    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(privateKey);
        deployDeployer();

        vm.stopBroadcast();
    }
}

contract DeployIBC is UnionScript {
    Deployer immutable deployer;

    constructor() {
        deployer = Deployer(vm.envAddress("DEPLOYER"));
    }

    function getDeployer() internal view override returns (Deployer) {
        return deployer;
    }

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(privateKey);

        (
            IBCHandler handler,
            CometblsClient client,
            UCS01Relay relay,
            UCS02NFT nft,
            Multicall multicall
        ) = deployIBC(vm.addr(privateKey));
        handler.registerClient(LightClients.COMETBLS, client);

        vm.stopBroadcast();
    }
}

contract DeployDeployerAndIBC is UnionScript {
    Deployer deployer;

    function getDeployer() internal view override returns (Deployer) {
        return deployer;
    }

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(privateKey);

        deployer = deployDeployer();

        (
            IBCHandler handler,
            CometblsClient client,
            UCS01Relay relay,
            UCS02NFT nft,
            Multicall multicall
        ) = deployIBC(vm.addr(privateKey));
        handler.registerClient(LightClients.COMETBLS, client);

        vm.stopBroadcast();
    }
}

contract GetDeployed is Script {
    using LibString for *;

    address immutable deployer;
    address immutable sender;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
    }

    function getDeployed(string memory salt) internal view returns (address) {
        return CREATE3.getDeployed(
            keccak256(abi.encodePacked(sender.toHexString(), "/", salt)),
            deployer
        );
    }

    function run() public view {
        address handler = getDeployed(IBC.BASED);
        address cometblsClient =
            getDeployed(LightClients.make(LightClients.COMETBLS));
        address ucs01 = getDeployed(Protocols.make(Protocols.UCS01));
        address ucs02 = getDeployed(Protocols.make(Protocols.UCS02));

        console.log(
            string(abi.encodePacked("IBCHandler: ", handler.toHexString()))
        );
        console.log(
            string(
                abi.encodePacked(
                    "CometblsClient: ", cometblsClient.toHexString()
                )
            )
        );
        console.log(string(abi.encodePacked("UCS01: ", ucs01.toHexString())));
        console.log(string(abi.encodePacked("UCS02: ", ucs02.toHexString())));
    }
}

contract DryUpgradeUCS01 is Script {
    using LibString for *;

    address immutable deployer;
    address immutable sender;
    address immutable owner;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
        owner = vm.envAddress("OWNER");
    }

    function getDeployed(string memory salt) internal returns (address) {
        return CREATE3.getDeployed(
            keccak256(abi.encodePacked(sender.toHexString(), "/", salt)),
            deployer
        );
    }

    function run() public {
        address ucs01 = getDeployed(Protocols.make(Protocols.UCS01));
        console.log(string(abi.encodePacked("UCS01: ", ucs01.toHexString())));
        address newImplementation = address(new UCS01Relay());
        vm.prank(owner);
        UCS01Relay(ucs01).upgradeToAndCall(newImplementation, new bytes(0));
    }
}

contract UpgradeUCS01 is Script {
    using LibString for *;

    address immutable deployer;
    address immutable sender;
    uint256 immutable privateKey;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
        privateKey = vm.envUint("PRIVATE_KEY");
    }

    function getDeployed(string memory salt) internal returns (address) {
        return CREATE3.getDeployed(
            keccak256(abi.encodePacked(sender.toHexString(), "/", salt)),
            deployer
        );
    }

    function run() public {
        address ucs01 = getDeployed(Protocols.make(Protocols.UCS01));

        console.log(string(abi.encodePacked("UCS01: ", ucs01.toHexString())));

        vm.startBroadcast(privateKey);
        address newImplementation = address(new UCS01Relay());
        UCS01Relay(ucs01).upgradeToAndCall(newImplementation, new bytes(0));
        vm.stopBroadcast();
    }
}

contract DryUpgradeIBCHandler is Script {
    using LibString for *;

    address immutable deployer;
    address immutable sender;
    address immutable owner;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
        owner = vm.envAddress("OWNER");
    }

    function getDeployed(string memory salt) internal returns (address) {
        return CREATE3.getDeployed(
            keccak256(abi.encodePacked(sender.toHexString(), "/", salt)),
            deployer
        );
    }

    function run() public {
        address handler = getDeployed(IBC.BASED);
        console.log(
            string(abi.encodePacked("IBCHandler: ", handler.toHexString()))
        );
        address newHandlerImplementation = address(new OwnableIBCHandler());
        bytes memory upgradeImplsCall = abi.encodeCall(
            IBCHandler.upgradeImpls,
            (
                address(new IBCClient()),
                address(new IBCConnection()),
                address(new IBCChannelHandshake()),
                address(new IBCPacket())
            )
        );
        vm.prank(owner);
        IBCHandler(handler).upgradeToAndCall(
            newHandlerImplementation, upgradeImplsCall
        );
    }
}

contract UpgradeIBCHandler is Script {
    using LibString for *;

    address immutable deployer;
    address immutable sender;
    uint256 immutable privateKey;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
        privateKey = vm.envUint("PRIVATE_KEY");
    }

    function getDeployed(string memory salt) internal returns (address) {
        return CREATE3.getDeployed(
            keccak256(abi.encodePacked(sender.toHexString(), "/", salt)),
            deployer
        );
    }

    function run() public {
        address handler = getDeployed(IBC.BASED);
        console.log(
            string(abi.encodePacked("IBCHandler: ", handler.toHexString()))
        );
        vm.startBroadcast(privateKey);
        address newHandlerImplementation = address(new OwnableIBCHandler());
        IBCHandler(handler).upgradeToAndCall(
            newHandlerImplementation,
            abi.encodeCall(
                IBCHandler.upgradeImpls,
                (
                    address(new IBCClient()),
                    address(new IBCConnection()),
                    address(new IBCChannelHandshake()),
                    address(new IBCPacket())
                )
            )
        );
        vm.stopBroadcast();
    }
}
