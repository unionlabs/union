pragma solidity ^0.8.27;

import "forge-std/Vm.sol";
import "forge-std/StdJson.sol";
import "forge-std/Script.sol";

import "solady/utils/CREATE3.sol";
import "solady/utils/LibString.sol";
import "@openzeppelin-foundry-upgradeable/Upgrades.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Utils.sol";

import "../contracts/Manager.sol";
import "../contracts/Multicall.sol";
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
    string constant MULTICALL = "lib/multicall-v2";
    string constant ZKGM_ERC20 = "lib/zkgm-erc20";
}

library IBC {
    string constant BASED = "ibc-is-based";
    string constant MANAGER = "manager";
}

library LightClients {
    string constant COMETBLS = "lightclients/cometbls";
    string constant STATE_LENS_ICS23_MPT = "lightclients/state-lens/ics23/mpt";
    string constant STATE_LENS_ICS23_ICS23 =
        "lightclients/state-lens/ics23/ics23";
    string constant STATE_LENS_ICS23_SMT = "lightclients/state-lens/ics23/smt";
}

library Protocols {
    string constant UCS00 = "protocols/ucs00";
    string constant UCS03 = "protocols/ucs03";
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

    function deployMulticall(
        Manager manager
    ) internal returns (Multicall) {
        return Multicall(
            deploy(
                LIB.MULTICALL,
                abi.encode(
                    address(new Multicall()),
                    abi.encodeCall(Multicall.initialize, (address(manager)))
                )
            )
        );
    }

    function deployManager(
        address owner
    ) internal returns (Manager) {
        return Manager(
            deploy(
                IBC.MANAGER,
                abi.encode(
                    address(new Manager()),
                    abi.encodeCall(Manager.initialize, (owner))
                )
            )
        );
    }

    function deployZkgmERC20() internal returns (ZkgmERC20) {
        return ZkgmERC20(
            getDeployer().deploy(
                LIB.ZKGM_ERC20,
                abi.encodePacked(type(ZkgmERC20).creationCode),
                0
            )
        );
    }

    function deployIBCHandler(
        Manager manager
    ) internal returns (IBCHandler) {
        return IBCHandler(
            deploy(
                IBC.BASED,
                abi.encode(
                    address(new IBCHandler()),
                    abi.encodeCall(IBCHandler.initialize, (address(manager)))
                )
            )
        );
    }

    function deployStateLensIcs23MptClient(
        IBCHandler handler,
        Manager manager
    ) internal returns (StateLensIcs23MptClient) {
        return StateLensIcs23MptClient(
            deploy(
                LightClients.STATE_LENS_ICS23_MPT,
                abi.encode(
                    address(new StateLensIcs23MptClient(address(handler))),
                    abi.encodeCall(
                        StateLensIcs23MptClient.initialize, (address(manager))
                    )
                )
            )
        );
    }

    function deployStateLensIcs23Ics23Client(
        IBCHandler handler,
        Manager manager
    ) internal returns (StateLensIcs23Ics23Client) {
        return StateLensIcs23Ics23Client(
            deploy(
                LightClients.STATE_LENS_ICS23_ICS23,
                abi.encode(
                    address(new StateLensIcs23Ics23Client(address(handler))),
                    abi.encodeCall(
                        StateLensIcs23Ics23Client.initialize, (address(manager))
                    )
                )
            )
        );
    }

    function deployStateLensIcs23SmtClient(
        IBCHandler handler,
        Manager manager
    ) internal returns (StateLensIcs23SmtClient) {
        return StateLensIcs23SmtClient(
            deploy(
                LightClients.STATE_LENS_ICS23_SMT,
                abi.encode(
                    address(new StateLensIcs23SmtClient(address(handler))),
                    abi.encodeCall(
                        StateLensIcs23SmtClient.initialize, (address(manager))
                    )
                )
            )
        );
    }

    function deployCometbls(
        IBCHandler handler,
        Manager manager
    ) internal returns (CometblsClient) {
        return CometblsClient(
            deploy(
                LightClients.COMETBLS,
                abi.encode(
                    address(new CometblsClient(address(handler))),
                    abi.encodeCall(
                        CometblsClient.initialize, (address(manager))
                    )
                )
            )
        );
    }

    function deployUCS00(
        IBCHandler handler,
        Manager manager,
        uint64 timeout
    ) internal returns (PingPong) {
        return PingPong(
            deploy(
                Protocols.UCS00,
                abi.encode(
                    address(new PingPong()),
                    abi.encodeCall(
                        PingPong.initialize,
                        (handler, address(manager), timeout)
                    )
                )
            )
        );
    }

    function deployUCS03(
        IBCHandler handler,
        Manager manager,
        IWETH weth,
        ZkgmERC20 erc20
    ) internal returns (UCS03Zkgm) {
        UCS03Zkgm zkgm = UCS03Zkgm(
            payable(
                deploy(
                    Protocols.UCS03,
                    abi.encode(
                        address(new UCS03Zkgm(handler, weth, erc20)),
                        abi.encodeCall(UCS03Zkgm.initialize, (address(manager)))
                    )
                )
            )
        );
        return zkgm;
    }

    function deployIBC(
        address owner,
        IWETH weth
    )
        internal
        returns (
            IBCHandler,
            CometblsClient,
            StateLensIcs23MptClient,
            StateLensIcs23Ics23Client,
            StateLensIcs23SmtClient,
            PingPong,
            ZkgmERC20,
            UCS03Zkgm,
            Multicall,
            Manager
        )
    {
        Manager manager = deployManager(owner);
        IBCHandler handler = deployIBCHandler(manager);
        CometblsClient cometblsClient = deployCometbls(handler, manager);
        StateLensIcs23MptClient stateLensIcs23MptClient =
            deployStateLensIcs23MptClient(handler, manager);
        StateLensIcs23Ics23Client stateLensIcs23Ics23Client =
            deployStateLensIcs23Ics23Client(handler, manager);
        StateLensIcs23SmtClient stateLensIcs23SmtClient =
            deployStateLensIcs23SmtClient(handler, manager);
        PingPong pingpong = deployUCS00(handler, manager, 100000000000000);
        ZkgmERC20 zkgmERC20 = deployZkgmERC20();
        UCS03Zkgm ucs03 = deployUCS03(handler, manager, weth, zkgmERC20);
        Multicall multicall = deployMulticall(manager);
        setupRoles(owner, manager, handler, cometblsClient, ucs03, multicall);
        return (
            handler,
            cometblsClient,
            stateLensIcs23MptClient,
            stateLensIcs23Ics23Client,
            stateLensIcs23SmtClient,
            pingpong,
            zkgmERC20,
            ucs03,
            multicall,
            manager
        );
    }

    function setupRoles(
        address owner,
        Manager manager,
        IBCHandler handler,
        CometblsClient cometbls,
        UCS03Zkgm zkgm,
        Multicall multicall
    ) internal {
        bytes4[] memory relayerSelectors = new bytes4[](11);
        relayerSelectors[0] = IBCClient.registerClient.selector;
        relayerSelectors[1] = IBCClient.createClient.selector;
        relayerSelectors[2] = IBCClient.updateClient.selector;
        relayerSelectors[3] = IBCClient.misbehaviour.selector;
        relayerSelectors[4] = IBCPacketImpl.batchSend.selector;
        relayerSelectors[5] = IBCPacketImpl.batchAcks.selector;
        relayerSelectors[6] = IBCPacketImpl.recvPacket.selector;
        relayerSelectors[7] = IBCPacketImpl.recvIntentPacket.selector;
        relayerSelectors[8] = IBCPacketImpl.acknowledgePacket.selector;
        relayerSelectors[9] = IBCPacketImpl.timeoutPacket.selector;
        relayerSelectors[10] = Multicall.multicall.selector;
        manager.setTargetFunctionRole(
            address(handler), relayerSelectors, Roles.RELAYER
        );
        manager.labelRole(Roles.RELAYER, "RELAYER");

        // Pause selector is the same accross contracts
        bytes4[] memory pauserSelectors = new bytes4[](1);
        pauserSelectors[0] = CometblsClient.pause.selector;
        manager.setTargetFunctionRole(
            address(cometbls), pauserSelectors, Roles.PAUSER
        );
        manager.setTargetFunctionRole(
            address(zkgm), pauserSelectors, Roles.PAUSER
        );
        manager.labelRole(Roles.PAUSER, "PAUSER");

        // Unpause selector is the same accross contracts
        bytes4[] memory unpauserSelectors = new bytes4[](1);
        pauserSelectors[0] = CometblsClient.unpause.selector;
        manager.setTargetFunctionRole(
            address(cometbls), unpauserSelectors, Roles.UNPAUSER
        );
        manager.setTargetFunctionRole(
            address(zkgm), unpauserSelectors, Roles.UNPAUSER
        );
        manager.labelRole(Roles.UNPAUSER, "UNPAUSER");

        bytes4[] memory rateLimiterSelectors = new bytes4[](1);
        rateLimiterSelectors[0] = UCS03Zkgm.setBucketConfig.selector;
        manager.setTargetFunctionRole(
            address(zkgm), rateLimiterSelectors, Roles.RATE_LIMITER
        );
        manager.labelRole(Roles.RATE_LIMITER, "RATE_LIMITER");

        // Owner is granted all roles.
        manager.grantRole(Roles.RELAYER, owner, 0);
        manager.grantRole(Roles.PAUSER, owner, 0);
        manager.grantRole(Roles.UNPAUSER, owner, 0);
        manager.grantRole(Roles.RATE_LIMITER, owner, 0);

        // Multicall is granted relayer such that relayers can batch through it.
        manager.grantRole(Roles.RELAYER, address(multicall), 0);
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

        Manager manager = Manager(getDeployed(IBC.MANAGER));

        vm.startBroadcast(privateKey);
        deployMulticall(manager);
        vm.stopBroadcast();
    }
}

contract DeployManager is UnionScript {
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

        vm.startBroadcast(privateKey);

        Manager manager = deployManager(owner);

        vm.stopBroadcast();

        console.log("Manager: ", address(manager));
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

        Manager manager = Manager(getDeployed(IBC.MANAGER));
        IBCHandler handler = IBCHandler(getDeployed(IBC.BASED));

        vm.startBroadcast(privateKey);
        StateLensIcs23MptClient stateLensIcs23MptClient =
            deployStateLensIcs23MptClient(handler, manager);
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
        IWETH weth = IWETH(vm.envAddress("WETH_ADDRESS"));

        Manager manager = Manager(getDeployed(IBC.MANAGER));
        IBCHandler handler = IBCHandler(getDeployed(IBC.BASED));
        ZkgmERC20 zkgmERC20 = ZkgmERC20(getDeployed(LIB.ZKGM_ERC20));

        vm.startBroadcast(privateKey);
        UCS03Zkgm zkgm = deployUCS03(handler, manager, weth, zkgmERC20);
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

        Manager manager = Manager(getDeployed(IBC.MANAGER));
        IBCHandler handler = IBCHandler(getDeployed(IBC.BASED));

        vm.startBroadcast(privateKey);
        StateLensIcs23Ics23Client stateLensIcs23Ics23Client =
            deployStateLensIcs23Ics23Client(handler, manager);
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

        Manager manager = Manager(getDeployed(IBC.MANAGER));
        IBCHandler handler = IBCHandler(getDeployed(IBC.BASED));

        vm.startBroadcast(privateKey);
        StateLensIcs23SmtClient stateLensIcs23SmtClient =
            deployStateLensIcs23SmtClient(handler, manager);
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
        IWETH weth = IWETH(vm.envAddress("WETH_ADDRESS"));

        address owner = vm.addr(privateKey);
        vm.startBroadcast(privateKey);
        (
            IBCHandler handler,
            CometblsClient cometblsClient,
            StateLensIcs23MptClient stateLensIcs23MptClient,
            StateLensIcs23Ics23Client stateLensIcs23Ics23Client,
            StateLensIcs23SmtClient stateLensIcs23SmtClient,
            PingPong pingpong,
            ZkgmERC20 zkgmERC20,
            UCS03Zkgm ucs03,
            Multicall multicall,
            Manager manager
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

        console.log("Manager: ", address(manager));
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
        console.log("ZkgmERC20: ", address(zkgmERC20));
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
        IWETH weth = IWETH(vm.envAddress("WETH_ADDRESS"));

        vm.startBroadcast(privateKey);
        deployer = deployDeployer();
        (
            IBCHandler handler,
            CometblsClient cometblsClient,
            StateLensIcs23MptClient stateLensIcs23MptClient,
            StateLensIcs23Ics23Client stateLensIcs23Ics23Client,
            StateLensIcs23SmtClient stateLensIcs23SmtClient,
            PingPong pingpong,
            ZkgmERC20 zkgmERC20,
            UCS03Zkgm ucs03,
            Multicall multicall,
            Manager manager
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

        console.log("Manager: ", address(manager));
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
        console.log("ZkgmERC20: ", address(zkgmERC20));
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
        address multicall = getDeployed(LIB.MULTICALL);
        address zkgmERC20 = getDeployed(LIB.ZKGM_ERC20);

        address manager = getDeployed(IBC.MANAGER);
        address handler = getDeployed(IBC.BASED);

        address cometblsClient = getDeployed(LightClients.COMETBLS);
        address stateLensIcs23MptClient =
            getDeployed(LightClients.STATE_LENS_ICS23_MPT);
        address stateLensIcs23Ics23Client =
            getDeployed(LightClients.STATE_LENS_ICS23_ICS23);
        address stateLensIcs23SmtClient =
            getDeployed(LightClients.STATE_LENS_ICS23_SMT);

        address ucs00 = getDeployed(Protocols.UCS00);
        address ucs03 = getDeployed(Protocols.UCS03);

        console.log(
            string(abi.encodePacked("Manager: ", manager.toHexString()))
        );
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

        string memory proxyManager = "proxyManager";
        proxyManager.serialize(
            "contract",
            string(
                "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
            )
        );
        proxyManager = proxyManager.serialize(
            "args",
            abi.encode(
                implOf(manager), abi.encodeCall(Manager.initialize, sender)
            )
        );
        impls.serialize(manager.toHexString(), proxyManager);

        string memory proxyMulticall = "proxyMulticall";
        proxyMulticall.serialize(
            "contract",
            string(
                "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
            )
        );
        proxyMulticall = proxyMulticall.serialize(
            "args",
            abi.encode(
                implOf(multicall), abi.encodeCall(Multicall.initialize, manager)
            )
        );
        impls.serialize(multicall.toHexString(), proxyMulticall);

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
                implOf(handler), abi.encodeCall(IBCHandler.initialize, manager)
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
                abi.encodeCall(CometblsClient.initialize, (manager))
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
                abi.encodeCall(StateLensIcs23MptClient.initialize, (manager))
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
                    (IIBCPacket(handler), manager, 100000000000000)
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
                implOf(ucs03), abi.encodeCall(UCS03Zkgm.initialize, (manager))
            )
        );
        impls.serialize(ucs03.toHexString(), proxyUCS03);

        string memory implMulticall = "implMulticall";
        implMulticall.serialize(
            "contract", string("contracts/Multicall.sol:Multicall")
        );
        implMulticall = implMulticall.serialize("args", bytes(hex""));
        impls.serialize(implOf(multicall).toHexString(), implMulticall);

        string memory implZkgmERC20 = "implZkgmERC20";
        implZkgmERC20.serialize(
            "contract",
            string("contracts/apps/ucs/03-zkgm/ZkgmERC20.sol:ZkgmERC20")
        );
        implZkgmERC20 = implZkgmERC20.serialize("args", bytes(hex""));
        impls.serialize(zkgmERC20.toHexString(), implZkgmERC20);

        string memory implManager = "implManager";
        implManager.serialize(
            "contract", string("contracts/Manager.sol:Manager")
        );
        implManager = implManager.serialize("args", bytes(hex""));
        impls.serialize(implOf(manager).toHexString(), implManager);

        string memory implHandler = "implHandler";
        implHandler.serialize(
            "contract",
            string("contracts/core/25-handler/IBCHandler.sol:IBCHandler")
        );
        implHandler = implHandler.serialize("args", bytes(hex""));
        impls.serialize(implOf(handler).toHexString(), implHandler);

        string memory implComet = "implComet";
        implComet.serialize(
            "contract",
            string("contracts/clients/CometblsClient.sol:CometblsClient")
        );
        implComet = implComet.serialize("args", abi.encode(handler));
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
            implStateLensIcs23MptClient.serialize("args", abi.encode(handler));
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
            implStateLensIcs23Ics23Client.serialize("args", abi.encode(handler));
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
            implStateLensIcs23SmtClient.serialize("args", abi.encode(handler));
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
        implUCS03 =
            implUCS03.serialize("args", abi.encode(handler, weth, zkgmERC20));
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
        IWETH weth = IWETH(vm.envAddress("WETH_ADDRESS"));
        IIBCModulePacket handler = IIBCModulePacket(getDeployed(IBC.BASED));
        ZkgmERC20 zkgmERC20 = ZkgmERC20(getDeployed(LIB.ZKGM_ERC20));
        UCS03Zkgm ucs03 = UCS03Zkgm(payable(getDeployed(Protocols.UCS03)));

        console.log(
            string(abi.encodePacked("UCS03: ", address(ucs03).toHexString()))
        );

        address newImplementation =
            address(new UCS03Zkgm(handler, weth, zkgmERC20));
        vm.prank(owner);
        ucs03.upgradeToAndCall(newImplementation, new bytes(0));
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
        IWETH weth = IWETH(vm.envAddress("WETH_ADDRESS"));
        IIBCModulePacket handler = IIBCModulePacket(getDeployed(IBC.BASED));
        ZkgmERC20 zkgmERC20 = ZkgmERC20(getDeployed(LIB.ZKGM_ERC20));
        UCS03Zkgm ucs03 = UCS03Zkgm(payable(getDeployed(Protocols.UCS03)));

        console.log(
            string(abi.encodePacked("UCS03: ", address(ucs03).toHexString()))
        );

        vm.startBroadcast(privateKey);
        address newImplementation =
            address(new UCS03Zkgm(handler, weth, zkgmERC20));
        ucs03.upgradeToAndCall(newImplementation, new bytes(0));
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
        address ucs00 = getDeployed(Protocols.UCS00);

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
        address newImplementation = address(new IBCHandler());
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
        address newImplementation = address(new IBCHandler());
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
        address handler = getDeployed(IBC.BASED);
        CometblsClient cometblsClient =
            CometblsClient(getDeployed(LightClients.COMETBLS));
        console.log(
            string(
                abi.encodePacked(
                    "CometblsClient: ", address(cometblsClient).toHexString()
                )
            )
        );
        address newImplementation = address(new CometblsClient(handler));
        vm.prank(owner);
        cometblsClient.upgradeToAndCall(newImplementation, new bytes(0));
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
        address handler = getDeployed(IBC.BASED);
        CometblsClient cometblsClient =
            CometblsClient(getDeployed(LightClients.COMETBLS));
        console.log(
            string(
                abi.encodePacked(
                    "CometblsClient: ", address(cometblsClient).toHexString()
                )
            )
        );
        vm.startBroadcast(privateKey);
        address newImplementation = address(new CometblsClient(handler));
        cometblsClient.upgradeToAndCall(newImplementation, new bytes(0));
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
        address handler = getDeployed(IBC.BASED);
        StateLensIcs23MptClient stateLensIcs23MptClient =
        StateLensIcs23MptClient(getDeployed(LightClients.STATE_LENS_ICS23_MPT));
        console.log(
            string(
                abi.encodePacked(
                    "StateLensIcs23MptClient: ",
                    address(stateLensIcs23MptClient).toHexString()
                )
            )
        );
        vm.startBroadcast(privateKey);
        address newImplementation =
            address(new StateLensIcs23MptClient(handler));
        stateLensIcs23MptClient.upgradeToAndCall(
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
        address handler = getDeployed(IBC.BASED);
        StateLensIcs23Ics23Client stateLensIcs23Ics23Client =
        StateLensIcs23Ics23Client(
            getDeployed(LightClients.STATE_LENS_ICS23_ICS23)
        );
        console.log(
            string(
                abi.encodePacked(
                    "StateLensIcs23Ics23Client: ",
                    address(stateLensIcs23Ics23Client).toHexString()
                )
            )
        );
        vm.startBroadcast(privateKey);
        address newImplementation =
            address(new StateLensIcs23Ics23Client(handler));
        stateLensIcs23Ics23Client.upgradeToAndCall(
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
        address handler = getDeployed(IBC.BASED);
        StateLensIcs23SmtClient stateLensIcs23SmtClient =
        StateLensIcs23SmtClient(getDeployed(LightClients.STATE_LENS_ICS23_SMT));
        console.log(
            string(
                abi.encodePacked(
                    "StateLensIcs23SmtClient: ",
                    address(stateLensIcs23SmtClient).toHexString()
                )
            )
        );
        vm.startBroadcast(privateKey);
        address newImplementation =
            address(new StateLensIcs23SmtClient(handler));
        stateLensIcs23SmtClient.upgradeToAndCall(
            newImplementation, new bytes(0)
        );
        vm.stopBroadcast();
    }
}

contract DeployRoles is UnionScript {
    using LibString for *;

    Deployer immutable deployer;
    address immutable sender;

    constructor() {
        deployer = Deployer(vm.envAddress("DEPLOYER"));
        sender = vm.envAddress("SENDER");
    }

    function getDeployer() internal view override returns (Deployer) {
        return deployer;
    }

    function getDeployed(
        string memory salt
    ) internal view returns (address) {
        return CREATE3.predictDeterministicAddress(
            keccak256(abi.encodePacked(sender.toHexString(), "/", salt)),
            address(deployer)
        );
    }

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        address owner = vm.addr(privateKey);

        Manager manager = Manager(getDeployed(IBC.MANAGER));
        IBCHandler handler = IBCHandler(getDeployed(IBC.BASED));
        CometblsClient cometbls =
            CometblsClient(getDeployed(LightClients.COMETBLS));
        UCS03Zkgm ucs03 = UCS03Zkgm(payable(getDeployed(Protocols.UCS03)));
        Multicall multicall = Multicall(getDeployed(LIB.MULTICALL));

        vm.startBroadcast(privateKey);
        setupRoles(owner, manager, handler, cometbls, ucs03, multicall);
        vm.stopBroadcast();
    }
}
