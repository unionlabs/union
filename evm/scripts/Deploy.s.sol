pragma solidity ^0.8.27;

import "forge-std/Vm.sol";
import "forge-std/StdJson.sol";
import "forge-std/Script.sol";

import "solady/utils/CREATE3.sol";
import "solady/utils/LibString.sol";
import "@openzeppelin-foundry-upgradeable/Upgrades.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Utils.sol";
import "@openzeppelin/access/Ownable.sol";

import "../contracts/Multicall.sol";
import "../contracts/core/OwnableIBCHandler.sol";
import "../contracts/clients/CometblsClient.sol";
import {CosmosInCosmosClient} from
    "../contracts/clients/CosmosInCosmosClient.sol";
import "../contracts/apps/ucs/00-pingpong/PingPong.sol";
import "../contracts/apps/ucs/01-relay/Relay.sol";
import "../contracts/apps/ucs/02-nft/NFT.sol";
import "../contracts/lib/Hex.sol";

import "./Deployer.sol";

library LIB {
    string constant NAMESPACE = "lib";
    string constant MULTICALL = "multicall";

    function make(
        string memory lib
    ) internal pure returns (string memory) {
        return string(abi.encodePacked(NAMESPACE, "/", lib));
    }
}

library IBC {
    string constant BASED = "ibc-is-based";
}

library LightClients {
    string constant NAMESPACE = "lightclients";
    string constant COMETBLS = "cometbls";

    function make(
        string memory lightClient
    ) internal pure returns (string memory) {
        return string(abi.encodePacked(NAMESPACE, "/", lightClient));
    }
}

library Protocols {
    string constant NAMESPACE = "protocols";
    string constant UCS00 = "ucs00";
    string constant UCS01 = "ucs01";
    string constant UCS02 = "ucs02";

    function make(
        string memory protocol
    ) internal pure returns (string memory) {
        return string(abi.encodePacked(NAMESPACE, "/", protocol));
    }
}

abstract contract UnionBase is Script {
    function deployDeployer() internal returns (Deployer) {
        return new Deployer();
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
                LIB.make(LIB.MULTICALL),
                abi.encodePacked(type(Multicall).creationCode),
                0
            )
        );
    }

    function deployIBCHandler(
        address owner
    ) internal returns (IBCHandler) {
        return IBCHandler(
            deploy(
                IBC.BASED,
                abi.encode(
                    address(new OwnableIBCHandler()),
                    abi.encodeCall(IBCHandler.initialize, (owner))
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

    function deployUCS00(
        IBCHandler handler,
        address owner,
        uint64 timeout
    ) internal returns (PingPong) {
        return PingPong(
            deploy(
                Protocols.make(Protocols.UCS00),
                abi.encode(
                    address(new PingPong()),
                    abi.encodeCall(
                        PingPong.initialize, (handler, owner, timeout)
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
                    abi.encodeCall(UCS02NFT.initialize, (handler, owner))
                )
            )
        );
    }

    function deployIBC(
        address owner
    )
        internal
        returns (
            IBCHandler,
            CometblsClient,
            PingPong,
            UCS01Relay,
            UCS02NFT,
            Multicall
        )
    {
        IBCHandler handler = deployIBCHandler(owner);
        CometblsClient client = deployCometbls(handler, owner);
        PingPong pingpong = deployUCS00(handler, owner, 100000000000000);
        UCS01Relay relay = deployUCS01(handler, owner);
        UCS02NFT nft = deployUCS02(handler, owner);
        Multicall multicall = deployMulticall();
        return (handler, client, pingpong, relay, nft, multicall);
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

contract DeployMulticall is UnionScript {
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

        deployMulticall();

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
            PingPong pingpong,
            UCS01Relay relay,
            UCS02NFT nft,
            Multicall multicall
        ) = deployIBC(vm.addr(privateKey));
        handler.registerClient(LightClients.COMETBLS, client);

        vm.stopBroadcast();

        console.log("Deployer: ", address(deployer));
        console.log("Sender: ", vm.addr(privateKey));
        console.log("IBCHandler: ", address(handler));
        console.log("CometblsClient: ", address(client));
        console.log("UCS00: ", address(pingpong));
        console.log("UCS01: ", address(relay));
        console.log("UCS02: ", address(nft));
        console.log("Multicall: ", address(multicall));
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
            PingPong pingpong,
            UCS01Relay relay,
            UCS02NFT nft,
            Multicall multicall
        ) = deployIBC(vm.addr(privateKey));
        handler.registerClient(LightClients.COMETBLS, client);

        vm.stopBroadcast();

        console.log("Deployer: ", address(deployer));
        console.log("Sender: ", vm.addr(privateKey));
        console.log("IBCHandler: ", address(handler));
        console.log("CometblsClient: ", address(client));
        console.log("UCS00: ", address(pingpong));
        console.log("UCS01: ", address(relay));
        console.log("UCS02: ", address(nft));
        console.log("Multicall: ", address(multicall));
    }
}

contract GetDeployed is Script {
    using LibString for *;
    using stdJson for string;

    address immutable deployer;
    address immutable sender;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
    }

    function getDeployed(
        string memory salt
    ) internal view returns (address) {
        return CREATE3.predictDeterministicAddress(
            keccak256(abi.encodePacked(sender.toHexString(), "/", salt)),
            deployer
        );
    }

    function implOf(
        address x
    ) internal returns (address) {
        return
            address(bytes20(vm.load(x, ERC1967Utils.IMPLEMENTATION_SLOT) << 96));
    }

    function run() public {
        address multicall = getDeployed(LIB.make(LIB.MULTICALL));
        address handler = getDeployed(IBC.BASED);
        address cometblsClient =
            getDeployed(LightClients.make(LightClients.COMETBLS));
        address ucs00 = getDeployed(Protocols.make(Protocols.UCS00));
        address ucs01 = getDeployed(Protocols.make(Protocols.UCS01));
        address ucs02 = getDeployed(Protocols.make(Protocols.UCS02));

        console.log(
            string(abi.encodePacked("Multicall: ", multicall.toHexString()))
        );
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
        console.log(string(abi.encodePacked("UCS00: ", ucs00.toHexString())));
        console.log(string(abi.encodePacked("UCS01: ", ucs01.toHexString())));
        console.log(string(abi.encodePacked("UCS02: ", ucs02.toHexString())));

        string memory impls = "base";

        string memory proxyHandler = "proxyHandler";
        proxyHandler.serialize(
            "contract",
            string(
                "libs/@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
            )
        );
        proxyHandler = proxyHandler.serialize(
            "args",
            abi.encode(
                implOf(handler), abi.encodeCall(IBCHandler.initialize, sender)
            )
        );
        impls.serialize(handler.toHexString(), proxyHandler);

        string memory proxyComet = "proxyComet";
        proxyComet.serialize(
            "contract",
            string(
                "libs/@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
            )
        );
        proxyComet = proxyComet.serialize(
            "args",
            abi.encode(
                implOf(cometblsClient),
                abi.encodeCall(CometblsClient.initialize, (handler, sender))
            )
        );
        impls.serialize(cometblsClient.toHexString(), proxyComet);

        string memory proxyUCS00 = "proxyUCS00";
        proxyUCS00.serialize(
            "contract",
            string(
                "libs/@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
            )
        );
        proxyUCS00 = proxyUCS00.serialize(
            "args",
            abi.encode(
                implOf(ucs00),
                abi.encodeCall(
                    PingPong.initialize,
                    (IIBCPacket(handler), sender, 100000000000000)
                )
            )
        );
        impls.serialize(ucs00.toHexString(), proxyUCS00);

        string memory proxyUCS01 = "proxyUCS01";
        proxyUCS01.serialize(
            "contract",
            string(
                "libs/@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
            )
        );
        proxyUCS01 = proxyUCS01.serialize(
            "args",
            abi.encode(
                implOf(ucs01),
                abi.encodeCall(
                    UCS01Relay.initialize, (IIBCPacket(handler), sender)
                )
            )
        );
        impls.serialize(ucs01.toHexString(), proxyUCS01);

        string memory proxyUCS02 = "proxyUCS02";
        proxyUCS02.serialize(
            "contract",
            string(
                "libs/@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
            )
        );
        proxyUCS02 = proxyUCS02.serialize(
            "args",
            abi.encode(
                implOf(ucs02),
                abi.encodeCall(
                    UCS02NFT.initialize, (IIBCPacket(handler), sender)
                )
            )
        );
        impls.serialize(ucs02.toHexString(), proxyUCS02);

        string memory implMulticall = "implMulticall";
        implMulticall.serialize(
            "contract", string("contracts/Multicall.sol:Multicall")
        );
        implMulticall = implMulticall.serialize("args", bytes(hex""));
        impls.serialize(multicall.toHexString(), implMulticall);

        string memory implHandler = "implHandler";
        implHandler.serialize(
            "contract",
            string("contracts/core/OwnableIBCHandler.sol:OwnableIBCHandler")
        );
        implHandler = implHandler.serialize("args", bytes(hex""));
        impls.serialize(implOf(handler).toHexString(), implHandler);

        string memory implComet = "implComet";
        implComet.serialize(
            "contract",
            string("contracts/clients/CometblsClient.sol:CometblsClient")
        );
        implComet = implComet.serialize("args", bytes(hex""));
        impls.serialize(implOf(cometblsClient).toHexString(), implComet);

        string memory implUCS00 = "implUCS00";
        implUCS00.serialize(
            "contract",
            string("contracts/apps/ucs/00-pingpong/PingPong.sol:PingPong")
        );
        implUCS00 = implUCS00.serialize("args", bytes(hex""));
        impls.serialize(implOf(ucs00).toHexString(), implUCS00);

        string memory implUCS01 = "implUCS01";
        implUCS01.serialize(
            "contract",
            string("contracts/apps/ucs/01-relay/Relay.sol:UCS01Relay")
        );
        implUCS01 = implUCS01.serialize("args", bytes(hex""));
        impls.serialize(implOf(ucs01).toHexString(), implUCS01);

        string memory implUCS02 = "implUCS02";
        implUCS02.serialize(
            "contract", string("contracts/apps/ucs/02-nft/NFT.sol:UCS02NFT")
        );
        implUCS02 = implUCS02.serialize("args", bytes(hex""));
        impls = impls.serialize(implOf(ucs01).toHexString(), implUCS02);

        impls.write(vm.envString("OUTPUT"));
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

    function getDeployed(
        string memory salt
    ) internal view returns (address) {
        return CREATE3.predictDeterministicAddress(
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

    function getDeployed(
        string memory salt
    ) internal view returns (address) {
        return CREATE3.predictDeterministicAddress(
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

    function getDeployed(
        string memory salt
    ) internal view returns (address) {
        return CREATE3.predictDeterministicAddress(
            keccak256(abi.encodePacked(sender.toHexString(), "/", salt)),
            deployer
        );
    }

    function run() public {
        address handler = getDeployed(IBC.BASED);
        console.log(
            string(abi.encodePacked("IBCHandler: ", handler.toHexString()))
        );
        address newImplementation = address(new OwnableIBCHandler());
        vm.prank(owner);
        IBCHandler(handler).upgradeToAndCall(newImplementation, new bytes(0));
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

    function getDeployed(
        string memory salt
    ) internal view returns (address) {
        return CREATE3.predictDeterministicAddress(
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
        address newImplementation = address(new OwnableIBCHandler());
        IBCHandler(handler).upgradeToAndCall(newImplementation, new bytes(0));
        vm.stopBroadcast();
    }
}

contract DryUpgradeCometblsClient is Script {
    using LibString for *;

    address immutable deployer;
    address immutable sender;
    address immutable owner;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
        owner = vm.envAddress("OWNER");
    }

    function getDeployed(
        string memory salt
    ) internal view returns (address) {
        return CREATE3.predictDeterministicAddress(
            keccak256(abi.encodePacked(sender.toHexString(), "/", salt)),
            deployer
        );
    }

    function run() public {
        address cometblsClient =
            getDeployed(LightClients.make(LightClients.COMETBLS));
        console.log(
            string(
                abi.encodePacked(
                    "CometblsClient: ", cometblsClient.toHexString()
                )
            )
        );
        address newImplementation = address(new CometblsClient());
        vm.prank(owner);
        CometblsClient(cometblsClient).upgradeToAndCall(
            newImplementation, new bytes(0)
        );
    }
}

contract UpgradeCometblsClient is Script {
    using LibString for *;

    address immutable deployer;
    address immutable sender;
    uint256 immutable privateKey;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
        privateKey = vm.envUint("PRIVATE_KEY");
    }

    function getDeployed(
        string memory salt
    ) internal view returns (address) {
        return CREATE3.predictDeterministicAddress(
            keccak256(abi.encodePacked(sender.toHexString(), "/", salt)),
            deployer
        );
    }

    function run() public {
        address cometblsClient =
            getDeployed(LightClients.make(LightClients.COMETBLS));
        console.log(
            string(
                abi.encodePacked(
                    "CometblsClient: ", cometblsClient.toHexString()
                )
            )
        );
        vm.startBroadcast(privateKey);
        address newImplementation = address(new CometblsClient());
        CometblsClient(cometblsClient).upgradeToAndCall(
            newImplementation, new bytes(0)
        );
        vm.stopBroadcast();
    }
}
