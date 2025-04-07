pragma solidity ^0.8.27;

import "forge-std/Vm.sol";
import "forge-std/StdJson.sol";
import "forge-std/Script.sol";

import "solady/utils/CREATE3.sol";
import "solady/utils/LibString.sol";
import "@openzeppelin-foundry-upgradeable/Upgrades.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Utils.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

import "../contracts/Multicall.sol";
import "../contracts/core/OwnableIBCHandler.sol";
import "../contracts/clients/CometblsClient.sol";
import {StateLensIcs23Ics23Client} from
    "../contracts/clients/StateLensIcs23Ics23Client.sol";
import {StateLensIcs23MptClient} from
    "../contracts/clients/StateLensIcs23MptClient.sol";
import {StateLensIcs23SmtClient} from
    "../contracts/clients/StateLensIcs23SmtClient.sol";
import "../contracts/apps/ucs/00-pingpong/PingPong.sol";
import "../contracts/apps/ucs/03-zkgm/Zkgm.sol";
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
    string constant STATE_LENS_ICS23_MPT = "state-lens/ics23/mpt";
    string constant STATE_LENS_ICS23_ICS23 = "state-lens/ics23/ics23";
    string constant STATE_LENS_ICS23_SMT = "state-lens/ics23/smt";

    function make(
        string memory lightClient
    ) internal pure returns (string memory) {
        return string(abi.encodePacked(NAMESPACE, "/", lightClient));
    }
}

library Protocols {
    string constant NAMESPACE = "protocols";
    string constant UCS00 = "ucs00";
    string constant UCS03 = "ucs03";

    function make(
        string memory protocol
    ) internal pure returns (string memory) {
        return string(abi.encodePacked(NAMESPACE, "/", protocol));
    }
}

abstract contract VersionedScript is Script {
    using LibString for *;

    constructor() {
        assertGitRevIsPresent();
    }

    function assertGitRevIsPresent() internal {
        if (!vm.envExists("BYPASS_GITREV") && VersionedLib.gitRev().eq("dirty"))
        {
            revert("git hash must be injected prior to script interactions");
        }
    }
}

abstract contract UnionBase is VersionedScript {
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

    function deployStateLensIcs23MptClient(
        IBCHandler handler,
        address owner
    ) internal returns (StateLensIcs23MptClient) {
        return StateLensIcs23MptClient(
            deploy(
                LightClients.make(LightClients.STATE_LENS_ICS23_MPT),
                abi.encode(
                    address(new StateLensIcs23MptClient()),
                    abi.encodeCall(
                        StateLensIcs23MptClient.initialize,
                        (address(handler), owner)
                    )
                )
            )
        );
    }

    function deployStateLensIcs23Ics23Client(
        IBCHandler handler,
        address owner
    ) internal returns (StateLensIcs23Ics23Client) {
        return StateLensIcs23Ics23Client(
            deploy(
                LightClients.make(LightClients.STATE_LENS_ICS23_ICS23),
                abi.encode(
                    address(new StateLensIcs23Ics23Client()),
                    abi.encodeCall(
                        StateLensIcs23Ics23Client.initialize,
                        (address(handler), owner)
                    )
                )
            )
        );
    }

    function deployStateLensIcs23SmtClient(
        IBCHandler handler,
        address owner
    ) internal returns (StateLensIcs23SmtClient) {
        return StateLensIcs23SmtClient(
            deploy(
                LightClients.make(LightClients.STATE_LENS_ICS23_SMT),
                abi.encode(
                    address(new StateLensIcs23SmtClient()),
                    abi.encodeCall(
                        StateLensIcs23SmtClient.initialize,
                        (address(handler), owner)
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

    function deployUCS03(
        IBCHandler handler,
        address owner,
        address weth
    ) internal returns (UCS03Zkgm) {
        UCS03Zkgm zkgm = UCS03Zkgm(
            payable(
                deploy(
                    Protocols.make(Protocols.UCS03),
                    abi.encode(
                        address(new UCS03Zkgm()),
                        abi.encodeCall(
                            UCS03Zkgm.initialize,
                            (IIBCModulePacket(handler), owner)
                        )
                    )
                )
            )
        );
        zkgm.setWETH(IWETH(weth));
        return zkgm;
    }

    function deployIBC(
        address owner,
        address weth
    )
        internal
        returns (
            IBCHandler,
            CometblsClient,
            StateLensIcs23MptClient,
            StateLensIcs23Ics23Client,
            StateLensIcs23SmtClient,
            PingPong,
            UCS03Zkgm,
            Multicall
        )
    {
        IBCHandler handler = deployIBCHandler(owner);
        CometblsClient cometblsClient = deployCometbls(handler, owner);
        StateLensIcs23MptClient stateLensIcs23MptClient =
            deployStateLensIcs23MptClient(handler, owner);
        StateLensIcs23Ics23Client stateLensIcs23Ics23Client =
            deployStateLensIcs23Ics23Client(handler, owner);
        StateLensIcs23SmtClient stateLensIcs23SmtClient =
            deployStateLensIcs23SmtClient(handler, owner);
        PingPong pingpong = deployUCS00(handler, owner, 100000000000000);
        UCS03Zkgm ucs03 = deployUCS03(handler, owner, weth);
        Multicall multicall = deployMulticall();
        return (
            handler,
            cometblsClient,
            stateLensIcs23MptClient,
            stateLensIcs23Ics23Client,
            stateLensIcs23SmtClient,
            pingpong,
            ucs03,
            multicall
        );
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

contract DeployStateLensIcs23MptClient is UnionScript {
    using LibString for *;

    address immutable deployer;
    address immutable sender;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
    }

    function getDeployer() internal view override returns (Deployer) {
        return Deployer(deployer);
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
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        address owner = vm.addr(privateKey);

        address handler = getDeployed(IBC.BASED);

        vm.startBroadcast(privateKey);

        StateLensIcs23MptClient stateLensIcs23MptClient =
            deployStateLensIcs23MptClient(IBCHandler(handler), owner);

        vm.stopBroadcast();

        console.log(
            "StateLensIcs23MptClient: ", address(stateLensIcs23MptClient)
        );
    }
}

contract DeployUCS03 is UnionScript {
    using LibString for *;

    address immutable deployer;
    address immutable sender;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
    }

    function getDeployer() internal view override returns (Deployer) {
        return Deployer(deployer);
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
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address weth = vm.envAddress("WETH_ADDRESS");

        address owner = vm.addr(privateKey);

        address handler = getDeployed(IBC.BASED);

        vm.startBroadcast(privateKey);

        UCS03Zkgm zkgm = deployUCS03(IBCHandler(handler), owner, weth);

        vm.stopBroadcast();

        console.log("UCS03: ", address(zkgm));
    }
}

contract DeployStateLensIcs23Ics23Client is UnionScript {
    using LibString for *;

    address immutable deployer;
    address immutable sender;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
    }

    function getDeployer() internal view override returns (Deployer) {
        return Deployer(deployer);
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
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        address owner = vm.addr(privateKey);

        address handler = getDeployed(IBC.BASED);

        vm.startBroadcast(privateKey);

        StateLensIcs23Ics23Client stateLensIcs23Ics23Client =
            deployStateLensIcs23Ics23Client(IBCHandler(handler), owner);

        vm.stopBroadcast();

        console.log(
            "StateLensIcs23Ics23Client: ", address(stateLensIcs23Ics23Client)
        );
    }
}

contract DeployStateLensIcs23SmtClient is UnionScript {
    using LibString for *;

    address immutable deployer;
    address immutable sender;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
    }

    function getDeployer() internal view override returns (Deployer) {
        return Deployer(deployer);
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
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        address owner = vm.addr(privateKey);

        address handler = getDeployed(IBC.BASED);

        vm.startBroadcast(privateKey);

        StateLensIcs23SmtClient stateLensIcs23SmtClient =
            deployStateLensIcs23SmtClient(IBCHandler(handler), owner);

        vm.stopBroadcast();

        console.log(
            "StateLensIcs23SmtClient: ", address(stateLensIcs23SmtClient)
        );
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
        address weth = vm.envAddress("WETH_ADDRESS");
        vm.startBroadcast(privateKey);

        (
            IBCHandler handler,
            CometblsClient cometblsClient,
            StateLensIcs23MptClient stateLensIcs23MptClient,
            StateLensIcs23Ics23Client stateLensIcs23Ics23Client,
            StateLensIcs23SmtClient stateLensIcs23SmtClient,
            PingPong pingpong,
            UCS03Zkgm ucs03,
            Multicall multicall
        ) = deployIBC(vm.addr(privateKey), weth);
        handler.registerClient(LightClients.COMETBLS, cometblsClient);
        handler.registerClient(
            LightClients.STATE_LENS_ICS23_MPT, stateLensIcs23MptClient
        );
        handler.registerClient(
            LightClients.STATE_LENS_ICS23_ICS23, stateLensIcs23Ics23Client
        );
        handler.registerClient(
            LightClients.STATE_LENS_ICS23_SMT, stateLensIcs23SmtClient
        );

        vm.stopBroadcast();

        console.log("Deployer: ", address(deployer));
        console.log("Sender: ", vm.addr(privateKey));
        console.log("IBCHandler: ", address(handler));
        console.log("CometblsClient: ", address(cometblsClient));
        console.log(
            "StateLensIcs23MptClient: ", address(stateLensIcs23MptClient)
        );
        console.log(
            "StateLensIcs23Ics23Client: ", address(stateLensIcs23Ics23Client)
        );
        console.log(
            "StateLensIcs23SmtClient: ", address(stateLensIcs23SmtClient)
        );
        console.log("UCS00: ", address(pingpong));
        console.log("UCS03: ", address(ucs03));
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
        address weth = vm.envAddress("WETH_ADDRESS");

        vm.startBroadcast(privateKey);

        deployer = deployDeployer();

        (
            IBCHandler handler,
            CometblsClient cometblsClient,
            StateLensIcs23MptClient stateLensIcs23MptClient,
            StateLensIcs23Ics23Client stateLensIcs23Ics23Client,
            StateLensIcs23SmtClient stateLensIcs23SmtClient,
            PingPong pingpong,
            UCS03Zkgm ucs03,
            Multicall multicall
        ) = deployIBC(vm.addr(privateKey), weth);
        handler.registerClient(LightClients.COMETBLS, cometblsClient);
        handler.registerClient(
            LightClients.STATE_LENS_ICS23_MPT, stateLensIcs23MptClient
        );
        handler.registerClient(
            LightClients.STATE_LENS_ICS23_ICS23, stateLensIcs23Ics23Client
        );
        handler.registerClient(
            LightClients.STATE_LENS_ICS23_SMT, stateLensIcs23SmtClient
        );

        vm.stopBroadcast();

        console.log("Deployer: ", address(deployer));
        console.log("Sender: ", vm.addr(privateKey));
        console.log("IBCHandler: ", address(handler));
        console.log("CometblsClient: ", address(cometblsClient));
        console.log(
            "StateLensIcs23MptClient: ", address(stateLensIcs23MptClient)
        );
        console.log(
            "StateLensIcs23Ics23Client: ", address(stateLensIcs23Ics23Client)
        );
        console.log(
            "StateLensIcs23SmtClient: ", address(stateLensIcs23SmtClient)
        );
        console.log("UCS00: ", address(pingpong));
        console.log("UCS03: ", address(ucs03));
        console.log("Multicall: ", address(multicall));
    }
}

contract GetDeployed is VersionedScript {
    using LibString for *;
    using stdJson for string;

    address immutable deployer;
    address immutable sender;
    address immutable weth;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
        weth = vm.envAddress("WETH_ADDRESS");
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
        address stateLensIcs23MptClient =
            getDeployed(LightClients.make(LightClients.STATE_LENS_ICS23_MPT));
        address stateLensIcs23Ics23Client =
            getDeployed(LightClients.make(LightClients.STATE_LENS_ICS23_ICS23));
        address stateLensIcs23SmtClient =
            getDeployed(LightClients.make(LightClients.STATE_LENS_ICS23_SMT));
        address ucs00 = getDeployed(Protocols.make(Protocols.UCS00));
        address ucs03 = getDeployed(Protocols.make(Protocols.UCS03));

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
        console.log(
            string(
                abi.encodePacked(
                    "StateLensIcs23MptClient: ",
                    stateLensIcs23MptClient.toHexString()
                )
            )
        );
        console.log(
            string(
                abi.encodePacked(
                    "StateLensIcs23Ics23Client: ",
                    stateLensIcs23Ics23Client.toHexString()
                )
            )
        );
        console.log(
            string(
                abi.encodePacked(
                    "StateLensIcs23SmtClient: ",
                    stateLensIcs23SmtClient.toHexString()
                )
            )
        );
        console.log(string(abi.encodePacked("UCS00: ", ucs00.toHexString())));
        console.log(string(abi.encodePacked("UCS03: ", ucs03.toHexString())));

        string memory impls = "base";

        string memory proxyHandler = "proxyHandler";
        proxyHandler.serialize(
            "contract",
            string(
                "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
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
                "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
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

        string memory proxyStateLensIcs23MptClient =
            "proxyStateLensIcs23MptClient";
        proxyStateLensIcs23MptClient.serialize(
            "contract",
            string(
                "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
            )
        );
        proxyStateLensIcs23MptClient = proxyStateLensIcs23MptClient.serialize(
            "args",
            abi.encode(
                implOf(stateLensIcs23MptClient),
                abi.encodeCall(
                    StateLensIcs23MptClient.initialize, (handler, sender)
                )
            )
        );
        impls.serialize(
            stateLensIcs23MptClient.toHexString(), proxyStateLensIcs23MptClient
        );

        string memory proxyUCS00 = "proxyUCS00";
        proxyUCS00.serialize(
            "contract",
            string(
                "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
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

        string memory proxyUCS03 = "proxyUCS03";
        proxyUCS03.serialize(
            "contract",
            string(
                "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
            )
        );
        proxyUCS03 = proxyUCS03.serialize(
            "args",
            abi.encode(
                implOf(ucs03),
                abi.encodeCall(
                    UCS03Zkgm.initialize, (IIBCModulePacket(handler), sender)
                )
            )
        );
        impls.serialize(ucs03.toHexString(), proxyUCS03);

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

        string memory implStateLensIcs23MptClient =
            "implStateLensIcs23MptClient";
        implStateLensIcs23MptClient.serialize(
            "contract",
            string(
                "contracts/clients/StateLensIcs23MptClient.sol:StateLensIcs23MptClient"
            )
        );
        implStateLensIcs23MptClient =
            implStateLensIcs23MptClient.serialize("args", bytes(hex""));
        impls.serialize(
            implOf(stateLensIcs23MptClient).toHexString(),
            implStateLensIcs23MptClient
        );

        string memory implStateLensIcs23Ics23Client =
            "implStateLensIcs23Ics23Client";
        implStateLensIcs23Ics23Client.serialize(
            "contract",
            string(
                "contracts/clients/StateLensIcs23Ics23Client.sol:StateLensIcs23Ics23Client"
            )
        );
        implStateLensIcs23Ics23Client =
            implStateLensIcs23Ics23Client.serialize("args", bytes(hex""));
        impls.serialize(
            implOf(stateLensIcs23Ics23Client).toHexString(),
            implStateLensIcs23Ics23Client
        );

        string memory implStateLensIcs23SmtClient =
            "implStateLensIcs23SmtClient";
        implStateLensIcs23SmtClient.serialize(
            "contract",
            string(
                "contracts/clients/StateLensIcs23SmtClient.sol:StateLensIcs23SmtClient"
            )
        );
        implStateLensIcs23SmtClient =
            implStateLensIcs23SmtClient.serialize("args", bytes(hex""));
        impls.serialize(
            implOf(stateLensIcs23SmtClient).toHexString(),
            implStateLensIcs23SmtClient
        );

        string memory implUCS00 = "implUCS00";
        implUCS00.serialize(
            "contract",
            string("contracts/apps/ucs/00-pingpong/PingPong.sol:PingPong")
        );
        implUCS00 = implUCS00.serialize("args", bytes(hex""));
        impls.serialize(implOf(ucs00).toHexString(), implUCS00);

        string memory implUCS03 = "implUCS03";
        implUCS03.serialize(
            "contract", string("contracts/apps/ucs/03-zkgm/Zkgm.sol:UCS03Zkgm")
        );
        implUCS03 = implUCS03.serialize("args", bytes(hex""));
        impls = impls.serialize(implOf(ucs03).toHexString(), implUCS03);

        impls.write(vm.envString("OUTPUT"));
    }
}

contract DryUpgradeUCS03 is VersionedScript {
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
        address ucs03 = getDeployed(Protocols.make(Protocols.UCS03));

        console.log(string(abi.encodePacked("UCS03: ", ucs03.toHexString())));

        address newImplementation = address(new UCS03Zkgm());
        vm.prank(owner);
        UCS03Zkgm(payable(ucs03)).upgradeToAndCall(
            newImplementation, new bytes(0)
        );
    }
}

contract UpgradeUCS03 is VersionedScript {
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
        address ucs03 = getDeployed(Protocols.make(Protocols.UCS03));

        console.log(string(abi.encodePacked("UCS03: ", ucs03.toHexString())));

        vm.startBroadcast(privateKey);
        address newImplementation = address(new UCS03Zkgm());
        UCS03Zkgm(payable(ucs03)).upgradeToAndCall(
            newImplementation, new bytes(0)
        );
        vm.stopBroadcast();
    }
}

contract UpgradeUCS00 is VersionedScript {
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
        address ucs00 = getDeployed(Protocols.make(Protocols.UCS00));

        console.log(string(abi.encodePacked("UCS00: ", ucs00.toHexString())));

        vm.startBroadcast(privateKey);
        address newImplementation = address(new PingPong());
        PingPong(ucs00).upgradeToAndCall(newImplementation, new bytes(0));
        vm.stopBroadcast();
    }
}

contract DryUpgradeIBCHandler is VersionedScript {
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

contract UpgradeIBCHandler is VersionedScript {
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

contract DryUpgradeCometblsClient is VersionedScript {
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

contract UpgradeCometblsClient is VersionedScript {
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

contract UpgradeStateLensIcs23MptClient is VersionedScript {
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
        address stateLensIcs23MptClient =
            getDeployed(LightClients.make(LightClients.STATE_LENS_ICS23_MPT));
        console.log(
            string(
                abi.encodePacked(
                    "StateLensIcs23MptClient: ",
                    stateLensIcs23MptClient.toHexString()
                )
            )
        );
        vm.startBroadcast(privateKey);
        address newImplementation = address(new StateLensIcs23MptClient());
        CometblsClient(stateLensIcs23MptClient).upgradeToAndCall(
            newImplementation, new bytes(0)
        );
        vm.stopBroadcast();
    }
}

contract UpgradeStateLensIcs23Ics23Client is VersionedScript {
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
        address stateLensIcs23Ics23Client =
            getDeployed(LightClients.make(LightClients.STATE_LENS_ICS23_ICS23));
        console.log(
            string(
                abi.encodePacked(
                    "StateLensIcs23Ics23Client: ",
                    stateLensIcs23Ics23Client.toHexString()
                )
            )
        );
        vm.startBroadcast(privateKey);
        address newImplementation = address(new StateLensIcs23Ics23Client());
        StateLensIcs23Ics23Client(stateLensIcs23Ics23Client).upgradeToAndCall(
            newImplementation, new bytes(0)
        );
        vm.stopBroadcast();
    }
}

contract UpgradeStateLensIcs23SmtClient is VersionedScript {
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
        address stateLensIcs23SmtClient =
            getDeployed(LightClients.make(LightClients.STATE_LENS_ICS23_SMT));
        console.log(
            string(
                abi.encodePacked(
                    "StateLensIcs23SmtClient: ",
                    stateLensIcs23SmtClient.toHexString()
                )
            )
        );
        vm.startBroadcast(privateKey);
        address newImplementation = address(new StateLensIcs23SmtClient());
        StateLensIcs23SmtClient(stateLensIcs23SmtClient).upgradeToAndCall(
            newImplementation, new bytes(0)
        );
        vm.stopBroadcast();
    }
}
