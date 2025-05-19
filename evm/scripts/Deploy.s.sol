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
import "../contracts/apps/ucs/06-funded-dispatch/FundedDispatch.sol";

import "./Deployer.sol";

struct Contracts {
    Manager manager;
    Multicall multicall;
    IBCHandler handler;
    CometblsClient cometblsClient;
    StateLensIcs23MptClient stateLensIcs23MptClient;
    StateLensIcs23Ics23Client stateLensIcs23Ics23Client;
    StateLensIcs23SmtClient stateLensIcs23SmtClient;
    PingPong ucs00;
    ZkgmERC20 ucs03Erc20Impl;
    UCS03Zkgm ucs03;
    UCS06FundedDispatch ucs06;
}

struct UCS03Parameters {
    IWETH weth;
    bool rateLimitEnabled;
    string nativeTokenName;
    string nativeTokenSymbol;
    uint8 nativeTokenDecimals;
}

library LIB_SALT {
    string constant MULTICALL = "lib/multicall-v2";
    string constant UCS03_ZKGM_ERC20_IMPL = "lib/zkgm-erc20-v2";
}

library IBC_SALT {
    string constant BASED = "ibc-is-based";
    string constant MANAGER = "manager";
}

library LIGHT_CLIENT_SALT {
    string constant COMETBLS = "lightclients/cometbls";
    string constant STATE_LENS_ICS23_MPT = "lightclients/state-lens/ics23/mpt";
    string constant STATE_LENS_ICS23_ICS23 =
        "lightclients/state-lens/ics23/ics23";
    string constant STATE_LENS_ICS23_SMT = "lightclients/state-lens/ics23/smt";
}

library LightClients {
    string constant COMETBLS = "cometbls";
    string constant STATE_LENS_ICS23_MPT = "state-lens/ics23/mpt";
    string constant STATE_LENS_ICS23_ICS23 = "state-lens/ics23/ics23";
    string constant STATE_LENS_ICS23_SMT = "state-lens/ics23/smt";
}

library Protocols {
    string constant UCS00 = "protocols/ucs00";
    string constant UCS03 = "protocols/ucs03";
    string constant UCS06 = "protocols/ucs06";
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

abstract contract UnionBase is Script {
    function deployDeployer() internal returns (Deployer) {
        return new Deployer();
    }

    function getUCS03Params() internal returns (UCS03Parameters memory) {
        IWETH weth = IWETH(vm.envAddress("WETH_ADDRESS"));
        bool rateLimitEnabled = vm.envBool("RATE_LIMIT_ENABLED");
        string memory nativeTokenName = vm.envString("NATIVE_TOKEN_NAME");
        string memory nativeTokenSymbol = vm.envString("NATIVE_TOKEN_SYMBOL");
        uint8 nativeTokenDecimals = uint8(vm.envUint("NATIVE_TOKEN_DECIMALS"));
        return UCS03Parameters({
            weth: weth,
            rateLimitEnabled: rateLimitEnabled,
            nativeTokenName: nativeTokenName,
            nativeTokenSymbol: nativeTokenSymbol,
            nativeTokenDecimals: nativeTokenDecimals
        });
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
                LIB_SALT.MULTICALL,
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
                IBC_SALT.MANAGER,
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
                LIB_SALT.UCS03_ZKGM_ERC20_IMPL,
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
                IBC_SALT.BASED,
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
                LIGHT_CLIENT_SALT.STATE_LENS_ICS23_MPT,
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
                LIGHT_CLIENT_SALT.STATE_LENS_ICS23_ICS23,
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
                LIGHT_CLIENT_SALT.STATE_LENS_ICS23_SMT,
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
                LIGHT_CLIENT_SALT.COMETBLS,
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
        ZkgmERC20 zkgmERC20,
        UCS03Parameters memory params
    ) internal returns (UCS03Zkgm) {
        UCS03Zkgm zkgm = UCS03Zkgm(
            payable(
                deploy(
                    Protocols.UCS03,
                    abi.encode(
                        address(
                            new UCS03Zkgm(
                                handler,
                                params.weth,
                                zkgmERC20,
                                params.rateLimitEnabled,
                                new UCS03ZkgmSendImpl(
                                    handler,
                                    params.weth,
                                    params.nativeTokenName,
                                    params.nativeTokenSymbol,
                                    params.nativeTokenDecimals
                                ),
                                new UCS03ZkgmStakeImpl(handler)
                            )
                        ),
                        abi.encodeCall(UCS03Zkgm.initialize, (address(manager)))
                    )
                )
            )
        );
        return zkgm;
    }

    function deployUCS06(
        Manager manager
    ) internal returns (UCS06FundedDispatch) {
        UCS06FundedDispatch fundedDispatch = UCS06FundedDispatch(
            payable(
                deploy(
                    Protocols.UCS06,
                    abi.encode(
                        address(new UCS06FundedDispatch()),
                        abi.encodeCall(
                            UCS06FundedDispatch.initialize, (address(manager))
                        )
                    )
                )
            )
        );
        return fundedDispatch;
    }

    function deployIBC(
        address owner,
        UCS03Parameters memory ucs03Params
    ) internal returns (Contracts memory) {
        Manager manager = deployManager(owner);
        IBCHandler handler = deployIBCHandler(manager);
        CometblsClient cometblsClient = deployCometbls(handler, manager);
        StateLensIcs23MptClient stateLensIcs23MptClient =
            deployStateLensIcs23MptClient(handler, manager);
        StateLensIcs23Ics23Client stateLensIcs23Ics23Client =
            deployStateLensIcs23Ics23Client(handler, manager);
        StateLensIcs23SmtClient stateLensIcs23SmtClient =
            deployStateLensIcs23SmtClient(handler, manager);
        PingPong ucs00 = deployUCS00(handler, manager, 100000000000000);
        ZkgmERC20 zkgmERC20 = deployZkgmERC20();
        UCS03Zkgm ucs03 = deployUCS03(handler, manager, zkgmERC20, ucs03Params);
        UCS06FundedDispatch ucs06 = deployUCS06(manager);
        Multicall multicall = deployMulticall(manager);
        Contracts memory contracts = Contracts({
            handler: handler,
            cometblsClient: cometblsClient,
            stateLensIcs23MptClient: stateLensIcs23MptClient,
            stateLensIcs23Ics23Client: stateLensIcs23Ics23Client,
            stateLensIcs23SmtClient: stateLensIcs23SmtClient,
            ucs00: ucs00,
            ucs03Erc20Impl: zkgmERC20,
            ucs03: ucs03,
            ucs06: ucs06,
            multicall: multicall,
            manager: manager
        });
        setupRoles(owner, contracts);
        return contracts;
    }

    function setupRoles(address owner, Contracts memory contracts) internal {
        bytes4[] memory relayerSelectors = new bytes4[](10);
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
        contracts.manager.setTargetFunctionRole(
            address(contracts.handler), relayerSelectors, Roles.RELAYER
        );

        bytes4[] memory multicallSelectors = new bytes4[](1);
        multicallSelectors[0] = Multicall.multicall.selector;
        contracts.manager.setTargetFunctionRole(
            address(contracts.multicall), multicallSelectors, Roles.RELAYER
        );

        contracts.manager.labelRole(Roles.RELAYER, "RELAYER");

        // Pause selector is the same accross contracts
        bytes4[] memory pauserSelectors = new bytes4[](1);
        pauserSelectors[0] = CometblsClient.pause.selector;
        contracts.manager.setTargetFunctionRole(
            address(contracts.cometblsClient), pauserSelectors, Roles.PAUSER
        );
        contracts.manager.setTargetFunctionRole(
            address(contracts.ucs03), pauserSelectors, Roles.PAUSER
        );
        contracts.manager.labelRole(Roles.PAUSER, "PAUSER");

        // Unpause selector is the same accross contracts
        bytes4[] memory unpauserSelectors = new bytes4[](1);
        pauserSelectors[0] = CometblsClient.unpause.selector;
        contracts.manager.setTargetFunctionRole(
            address(contracts.cometblsClient), unpauserSelectors, Roles.UNPAUSER
        );
        contracts.manager.setTargetFunctionRole(
            address(contracts.ucs03), unpauserSelectors, Roles.UNPAUSER
        );
        contracts.manager.labelRole(Roles.UNPAUSER, "UNPAUSER");

        bytes4[] memory rateLimiterSelectors = new bytes4[](1);
        rateLimiterSelectors[0] = UCS03Zkgm.setBucketConfig.selector;
        contracts.manager.setTargetFunctionRole(
            address(contracts.ucs03), rateLimiterSelectors, Roles.RATE_LIMITER
        );
        contracts.manager.labelRole(Roles.RATE_LIMITER, "RATE_LIMITER");

        // Owner is granted all roles.
        contracts.manager.grantRole(Roles.RELAYER, owner, 0);
        contracts.manager.grantRole(Roles.PAUSER, owner, 0);
        contracts.manager.grantRole(Roles.UNPAUSER, owner, 0);
        contracts.manager.grantRole(Roles.RATE_LIMITER, owner, 0);

        // Multicall is granted relayer such that relayers can batch through it.
        contracts.manager.grantRole(
            Roles.RELAYER, address(contracts.multicall), 0
        );
    }
}

contract DeployDeployer is UnionBase, VersionedScript {
    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(privateKey);
        deployDeployer();
        vm.stopBroadcast();
    }
}

contract DeployMulticall is UnionScript, VersionedScript {
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

        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));

        vm.startBroadcast(privateKey);
        deployMulticall(manager);
        vm.stopBroadcast();
    }
}

contract DeployManager is UnionScript, VersionedScript {
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

contract DeployStateLensIcs23MptClient is UnionScript, VersionedScript {
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

        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));
        IBCHandler handler = IBCHandler(getDeployed(IBC_SALT.BASED));

        vm.startBroadcast(privateKey);
        StateLensIcs23MptClient stateLensIcs23MptClient =
            deployStateLensIcs23MptClient(handler, manager);
        vm.stopBroadcast();

        console.log(
            "StateLensIcs23MptClient: ", address(stateLensIcs23MptClient)
        );
    }
}

contract DeployZkgmERC20 is UnionScript, VersionedScript {
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

        vm.startBroadcast(privateKey);
        ZkgmERC20 zkgmERC20 = deployZkgmERC20();
        vm.stopBroadcast();

        console.log("ZkgmERC20: ", address(zkgmERC20));
    }
}

contract DeployUCS03 is UnionScript, VersionedScript {
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

        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));
        IBCHandler handler = IBCHandler(getDeployed(IBC_SALT.BASED));
        ZkgmERC20 zkgmERC20 =
            ZkgmERC20(getDeployed(LIB_SALT.UCS03_ZKGM_ERC20_IMPL));

        vm.startBroadcast(privateKey);
        UCS03Zkgm zkgm =
            deployUCS03(handler, manager, zkgmERC20, getUCS03Params());
        vm.stopBroadcast();

        console.log("UCS03: ", address(zkgm));
    }
}

contract DeployStateLensIcs23Ics23Client is UnionScript, VersionedScript {
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

        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));
        IBCHandler handler = IBCHandler(getDeployed(IBC_SALT.BASED));

        vm.startBroadcast(privateKey);
        StateLensIcs23Ics23Client stateLensIcs23Ics23Client =
            deployStateLensIcs23Ics23Client(handler, manager);
        vm.stopBroadcast();

        console.log(
            "StateLensIcs23Ics23Client: ", address(stateLensIcs23Ics23Client)
        );
    }
}

contract DeployStateLensIcs23SmtClient is UnionScript, VersionedScript {
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

        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));
        IBCHandler handler = IBCHandler(getDeployed(IBC_SALT.BASED));

        vm.startBroadcast(privateKey);
        StateLensIcs23SmtClient stateLensIcs23SmtClient =
            deployStateLensIcs23SmtClient(handler, manager);
        vm.stopBroadcast();

        console.log(
            "StateLensIcs23SmtClient: ", address(stateLensIcs23SmtClient)
        );
    }
}

contract DeployIBC is UnionScript, VersionedScript {
    Deployer immutable deployer;

    constructor() {
        deployer = Deployer(vm.envAddress("DEPLOYER"));
    }

    function getDeployer() internal view override returns (Deployer) {
        return deployer;
    }

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        address owner = vm.addr(privateKey);
        vm.startBroadcast(privateKey);
        Contracts memory contracts =
            deployIBC(vm.addr(privateKey), getUCS03Params());
        contracts.handler.registerClient(
            LightClients.COMETBLS, contracts.cometblsClient
        );
        contracts.handler.registerClient(
            LightClients.STATE_LENS_ICS23_MPT, contracts.stateLensIcs23MptClient
        );
        contracts.handler.registerClient(
            LightClients.STATE_LENS_ICS23_ICS23,
            contracts.stateLensIcs23Ics23Client
        );
        contracts.handler.registerClient(
            LightClients.STATE_LENS_ICS23_SMT, contracts.stateLensIcs23SmtClient
        );
        vm.stopBroadcast();

        console.log("Manager: ", address(contracts.manager));
        console.log("Deployer: ", address(deployer));
        console.log("Sender: ", vm.addr(privateKey));
        console.log("IBCHandler: ", address(contracts.handler));
        console.log("CometblsClient: ", address(contracts.cometblsClient));
        console.log(
            "StateLensIcs23MptClient: ",
            address(contracts.stateLensIcs23MptClient)
        );
        console.log(
            "StateLensIcs23Ics23Client: ",
            address(contracts.stateLensIcs23Ics23Client)
        );
        console.log(
            "StateLensIcs23SmtClient: ",
            address(contracts.stateLensIcs23SmtClient)
        );
        console.log("UCS00: ", address(contracts.ucs00));
        console.log("ZkgmERC20: ", address(contracts.ucs03Erc20Impl));
        console.log("UCS03: ", address(contracts.ucs03));
        console.log("UCS06: ", address(contracts.ucs06));
        console.log("Multicall: ", address(contracts.multicall));
    }
}

contract DeployDeployerAndIBC is UnionScript, VersionedScript {
    Deployer deployer;

    function getDeployer() internal view override returns (Deployer) {
        return deployer;
    }

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(privateKey);
        deployer = deployDeployer();
        Contracts memory contracts =
            deployIBC(vm.addr(privateKey), getUCS03Params());
        contracts.handler.registerClient(
            LightClients.COMETBLS, contracts.cometblsClient
        );
        contracts.handler.registerClient(
            LightClients.STATE_LENS_ICS23_MPT, contracts.stateLensIcs23MptClient
        );
        contracts.handler.registerClient(
            LightClients.STATE_LENS_ICS23_ICS23,
            contracts.stateLensIcs23Ics23Client
        );
        contracts.handler.registerClient(
            LightClients.STATE_LENS_ICS23_SMT, contracts.stateLensIcs23SmtClient
        );
        vm.stopBroadcast();

        console.log("Manager: ", address(contracts.manager));
        console.log("Deployer: ", address(deployer));
        console.log("Sender: ", vm.addr(privateKey));
        console.log("IBCHandler: ", address(contracts.handler));
        console.log("CometblsClient: ", address(contracts.cometblsClient));
        console.log(
            "StateLensIcs23MptClient: ",
            address(contracts.stateLensIcs23MptClient)
        );
        console.log(
            "StateLensIcs23Ics23Client: ",
            address(contracts.stateLensIcs23Ics23Client)
        );
        console.log(
            "StateLensIcs23SmtClient: ",
            address(contracts.stateLensIcs23SmtClient)
        );
        console.log("UCS00: ", address(contracts.ucs00));
        console.log("ZkgmERC20: ", address(contracts.ucs03Erc20Impl));
        console.log("UCS03: ", address(contracts.ucs03));
        console.log("UCS06: ", address(contracts.ucs06));
        console.log("Multicall: ", address(contracts.multicall));
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
        bool rateLimitEnabled = vm.envBool("RATE_LIMIT_ENABLED");
        string memory nativeTokenName = vm.envString("NATIVE_TOKEN_NAME");
        string memory nativeTokenSymbol = vm.envString("NATIVE_TOKEN_SYMBOL");
        uint8 nativeTokenDecimals = uint8(vm.envUint("NATIVE_TOKEN_DECIMALS"));

        address multicall = getDeployed(LIB_SALT.MULTICALL);
        address zkgmERC20 = getDeployed(LIB_SALT.UCS03_ZKGM_ERC20_IMPL);

        address manager = getDeployed(IBC_SALT.MANAGER);
        address handler = getDeployed(IBC_SALT.BASED);

        address cometblsClient = getDeployed(LIGHT_CLIENT_SALT.COMETBLS);
        address stateLensIcs23MptClient =
            getDeployed(LIGHT_CLIENT_SALT.STATE_LENS_ICS23_MPT);
        address stateLensIcs23Ics23Client =
            getDeployed(LIGHT_CLIENT_SALT.STATE_LENS_ICS23_ICS23);
        address stateLensIcs23SmtClient =
            getDeployed(LIGHT_CLIENT_SALT.STATE_LENS_ICS23_SMT);

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
        implUCS03 = implUCS03.serialize(
            "args",
            abi.encode(
                handler,
                weth,
                zkgmERC20,
                rateLimitEnabled,
                nativeTokenName,
                nativeTokenSymbol,
                nativeTokenDecimals
            )
        );
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
        bool rateLimitEnabled = vm.envBool("RATE_LIMIT_ENABLED");
        string memory nativeTokenName = vm.envString("NATIVE_TOKEN_NAME");
        string memory nativeTokenSymbol = vm.envString("NATIVE_TOKEN_SYMBOL");
        uint8 nativeTokenDecimals = uint8(vm.envUint("NATIVE_TOKEN_DECIMALS"));

        IIBCModulePacket handler = IIBCModulePacket(getDeployed(IBC_SALT.BASED));
        ZkgmERC20 zkgmERC20 =
            ZkgmERC20(getDeployed(LIB_SALT.UCS03_ZKGM_ERC20_IMPL));
        UCS03Zkgm ucs03 = UCS03Zkgm(payable(getDeployed(Protocols.UCS03)));

        console.log(
            string(abi.encodePacked("UCS03: ", address(ucs03).toHexString()))
        );

        address newImplementation = address(
            new UCS03Zkgm(
                handler,
                weth,
                zkgmERC20,
                rateLimitEnabled,
                new UCS03ZkgmSendImpl(
                    handler,
                    weth,
                    nativeTokenName,
                    nativeTokenSymbol,
                    nativeTokenDecimals
                ),
                new UCS03ZkgmStakeImpl(handler)
            )
        );

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
        bool rateLimitEnabled = vm.envBool("RATE_LIMIT_ENABLED");
        string memory nativeTokenName = vm.envString("NATIVE_TOKEN_NAME");
        string memory nativeTokenSymbol = vm.envString("NATIVE_TOKEN_SYMBOL");
        uint8 nativeTokenDecimals = uint8(vm.envUint("NATIVE_TOKEN_DECIMALS"));

        IIBCModulePacket handler = IIBCModulePacket(getDeployed(IBC_SALT.BASED));
        ZkgmERC20 zkgmERC20 =
            ZkgmERC20(getDeployed(LIB_SALT.UCS03_ZKGM_ERC20_IMPL));
        UCS03Zkgm ucs03 = UCS03Zkgm(payable(getDeployed(Protocols.UCS03)));

        console.log(
            string(abi.encodePacked("UCS03: ", address(ucs03).toHexString()))
        );

        vm.startBroadcast(privateKey);
        address newImplementation = address(
            new UCS03Zkgm(
                handler,
                weth,
                zkgmERC20,
                rateLimitEnabled,
                new UCS03ZkgmSendImpl(
                    handler,
                    weth,
                    nativeTokenName,
                    nativeTokenSymbol,
                    nativeTokenDecimals
                ),
                new UCS03ZkgmStakeImpl(handler)
            )
        );
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
        address handler = getDeployed(IBC_SALT.BASED);
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
        address handler = getDeployed(IBC_SALT.BASED);
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
        address handler = getDeployed(IBC_SALT.BASED);
        CometblsClient cometblsClient =
            CometblsClient(getDeployed(LIGHT_CLIENT_SALT.COMETBLS));
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
        address handler = getDeployed(IBC_SALT.BASED);
        CometblsClient cometblsClient =
            CometblsClient(getDeployed(LIGHT_CLIENT_SALT.COMETBLS));
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
        address handler = getDeployed(IBC_SALT.BASED);
        StateLensIcs23MptClient stateLensIcs23MptClient =
        StateLensIcs23MptClient(
            getDeployed(LIGHT_CLIENT_SALT.STATE_LENS_ICS23_MPT)
        );
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
    uint32[] public clientIds;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
        privateKey = vm.envUint("PRIVATE_KEY");

        uint256[] memory u256ClientIds = vm.envUint("CLIENT_IDS", ",");

        clientIds = new uint32[](u256ClientIds.length);

        for (uint256 i = 0; i < u256ClientIds.length; ++i) {
            clientIds[i] = uint32(u256ClientIds[i]);
        }
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
        address handler = getDeployed(IBC_SALT.BASED);
        StateLensIcs23Ics23Client stateLensIcs23Ics23Client =
        StateLensIcs23Ics23Client(
            getDeployed(LIGHT_CLIENT_SALT.STATE_LENS_ICS23_ICS23)
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
            newImplementation,
            abi.encodeWithSelector(
                StateLensIcs23Ics23Client.migrateClientStateToV1.selector,
                clientIds
            )
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
        address handler = getDeployed(IBC_SALT.BASED);
        StateLensIcs23SmtClient stateLensIcs23SmtClient =
        StateLensIcs23SmtClient(
            getDeployed(LIGHT_CLIENT_SALT.STATE_LENS_ICS23_SMT)
        );
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

    function getContracts() internal returns (Contracts memory) {
        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));
        IBCHandler handler = IBCHandler(getDeployed(IBC_SALT.BASED));
        CometblsClient cometblsClient =
            CometblsClient(getDeployed(LIGHT_CLIENT_SALT.COMETBLS));
        PingPong ucs00 = PingPong(getDeployed(Protocols.UCS00));
        ZkgmERC20 ucs03Erc20Impl =
            ZkgmERC20(getDeployed(LIB_SALT.UCS03_ZKGM_ERC20_IMPL));
        UCS03Zkgm ucs03 = UCS03Zkgm(payable(getDeployed(Protocols.UCS03)));
        UCS06FundedDispatch ucs06 =
            UCS06FundedDispatch(getDeployed(Protocols.UCS06));
        Multicall multicall = Multicall(getDeployed(LIB_SALT.MULTICALL));
        StateLensIcs23MptClient stateLensIcs23MptClient =
        StateLensIcs23MptClient(
            getDeployed(LIGHT_CLIENT_SALT.STATE_LENS_ICS23_MPT)
        );
        StateLensIcs23Ics23Client stateLensIcs23Ics23Client =
        StateLensIcs23Ics23Client(
            getDeployed(LIGHT_CLIENT_SALT.STATE_LENS_ICS23_ICS23)
        );
        StateLensIcs23SmtClient stateLensIcs23SmtClient =
        StateLensIcs23SmtClient(
            getDeployed(LIGHT_CLIENT_SALT.STATE_LENS_ICS23_SMT)
        );
        return Contracts({
            manager: manager,
            multicall: multicall,
            handler: handler,
            cometblsClient: cometblsClient,
            stateLensIcs23MptClient: stateLensIcs23MptClient,
            stateLensIcs23Ics23Client: stateLensIcs23Ics23Client,
            stateLensIcs23SmtClient: stateLensIcs23SmtClient,
            ucs00: ucs00,
            ucs03Erc20Impl: ucs03Erc20Impl,
            ucs03: ucs03,
            ucs06: ucs06
        });
    }

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        address owner = vm.addr(privateKey);

        vm.startBroadcast(privateKey);
        setupRoles(owner, getContracts());
        vm.stopBroadcast();
    }
}
