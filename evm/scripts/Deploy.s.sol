pragma solidity ^0.8.27;

import "forge-std/Vm.sol";
import "forge-std/StdJson.sol";
import "forge-std/Script.sol";

import "solady/utils/CREATE3.sol";
import "solady/utils/LibString.sol";
import "solady/utils/LibBytes.sol";
import "solady/utils/LibSort.sol";
import "solady/utils/MerkleTreeLib.sol";
import "solady/utils/EfficientHashLib.sol";

import "@openzeppelin-foundry-upgradeable/Upgrades.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Utils.sol";

import "@safe-utils/Safe.sol";

import "../contracts/CrosschainVault.sol";
import "../contracts/U.sol";
import "../contracts/ProxyAccount.sol";
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
import "../contracts/tge/Vesting.sol";
import "../contracts/tge/UDrop.sol";

import "./Deployer.sol";

library SafeLib {
    address constant ADDRESS = 0x6a742078f90c2e16d8c9F83E41749410C314eDD4;
}

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

library INSTANCE_SALT {
    bytes constant U =
        hex"12c206e42a6e7773c97d1f1b855d7848492f9e4e396b33fcf0172d6758e9b047";
    bytes constant UDROP =
        hex"96de8fc8c256fa1e1556d41af431cace7dca68707c78dd88c3acab8b17177504";
    bytes constant EU =
        hex"0dec0db7b56214f189bc3d33052145c6d7558c6a7ee0da79e34bdd8a29d569c2";
}

library LIB_SALT {
    string constant MULTICALL = "lib/multicall-v2";
    string constant UCS03_ZKGM_ERC20_IMPL = "lib/zkgm-erc20-v2";
    string constant UCS03_ZKGM_ACCOUNT_IMPL = "lib/proxy-account-v1";
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
    using LibString for address;

    address private immutable deployer;
    address private immutable sender;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
    }

    function getDeployer() internal view virtual returns (Deployer) {
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

    function getSender() internal view returns (address) {
        return sender;
    }

    function deploy(
        string memory salt,
        bytes memory args
    ) internal returns (address) {
        return getDeployer().deploy(
            salt, abi.encodePacked(type(ERC1967Proxy).creationCode, args), 0
        );
    }

    function isContractDeployed(
        address contractAddress
    ) internal view returns (bool) {
        return contractAddress.code.length > 0;
    }

    function deployIfNotExists(
        string memory salt,
        bytes memory args,
        string memory contractName
    ) internal returns (address) {
        address predicted = getDeployed(salt);

        if (isContractDeployed(predicted)) {
            console.log(
                string(abi.encodePacked(contractName, " already deployed at:")),
                predicted
            );
            return predicted;
        }

        return getDeployer().deploy(
            salt, abi.encodePacked(type(ERC1967Proxy).creationCode, args), 0
        );
    }

    function deployDirectIfNotExists(
        string memory salt,
        bytes memory creationCode,
        string memory contractName
    ) internal returns (address) {
        address predicted = getDeployed(salt);

        if (isContractDeployed(predicted)) {
            console.log(
                string(abi.encodePacked(contractName, " already deployed at:")),
                predicted
            );
            return predicted;
        }

        return getDeployer().deploy(salt, creationCode, 0);
    }

    function deployMulticall(
        Manager manager
    ) internal returns (Multicall) {
        return Multicall(
            deployIfNotExists(
                LIB_SALT.MULTICALL,
                abi.encode(
                    address(new Multicall()),
                    abi.encodeCall(Multicall.initialize, (address(manager)))
                ),
                "Multicall"
            )
        );
    }

    function deployManager(
        address owner
    ) internal returns (Manager) {
        return Manager(
            deployIfNotExists(
                IBC_SALT.MANAGER,
                abi.encode(
                    address(new Manager()),
                    abi.encodeCall(Manager.initialize, (owner))
                ),
                "Manager"
            )
        );
    }

    function deployZkgmERC20() internal returns (ZkgmERC20) {
        return ZkgmERC20(
            deployDirectIfNotExists(
                LIB_SALT.UCS03_ZKGM_ERC20_IMPL,
                abi.encodePacked(type(ZkgmERC20).creationCode),
                "ZkgmERC20"
            )
        );
    }

    function deployProxyAccount() internal returns (ProxyAccount) {
        return ProxyAccount(
            deployDirectIfNotExists(
                LIB_SALT.UCS03_ZKGM_ACCOUNT_IMPL,
                abi.encodePacked(type(ProxyAccount).creationCode),
                "ProxyAccount"
            )
        );
    }

    function deployU(
        Manager authority,
        UCS03Zkgm zkgm,
        string memory name,
        string memory symbol,
        uint8 decimals
    ) internal returns (U) {
        return U(
            deployIfNotExists(
                string(INSTANCE_SALT.U),
                abi.encode(
                    address(new U()),
                    abi.encodeCall(
                        U.initialize,
                        (
                            address(authority),
                            address(zkgm),
                            name,
                            symbol,
                            decimals,
                            hex""
                        )
                    )
                ),
                "U token"
            )
        );
    }

    function deployEU(
        Manager authority,
        UCS03Zkgm zkgm,
        string memory name,
        string memory symbol,
        uint8 decimals
    ) internal returns (U) {
        return U(
            deployIfNotExists(
                string(INSTANCE_SALT.EU),
                abi.encode(
                    address(new U()),
                    abi.encodeCall(
                        U.initialize,
                        (
                            address(authority),
                            address(zkgm),
                            name,
                            symbol,
                            decimals,
                            hex""
                        )
                    )
                ),
                "EU token"
            )
        );
    }

    function deployUDrop(
        Manager authority,
        bytes32 root,
        address token,
        bool active
    ) internal returns (UDrop) {
        return UDrop(
            deployIfNotExists(
                string(INSTANCE_SALT.UDROP),
                abi.encode(
                    address(new UDrop(root, token)),
                    abi.encodeCall(
                        UDrop.initialize, (address(authority), active)
                    )
                ),
                "UDrop"
            )
        );
    }

    function deployIBCHandler(
        Manager manager
    ) internal returns (IBCHandler) {
        return IBCHandler(
            deployIfNotExists(
                IBC_SALT.BASED,
                abi.encode(
                    address(new IBCHandler()),
                    abi.encodeCall(IBCHandler.initialize, (address(manager)))
                ),
                "IBCHandler"
            )
        );
    }

    function deployStateLensIcs23MptClient(
        IBCHandler handler,
        Manager manager
    ) internal returns (StateLensIcs23MptClient) {
        return StateLensIcs23MptClient(
            deployIfNotExists(
                LIGHT_CLIENT_SALT.STATE_LENS_ICS23_MPT,
                abi.encode(
                    address(new StateLensIcs23MptClient(address(handler))),
                    abi.encodeCall(
                        StateLensIcs23MptClient.initialize, (address(manager))
                    )
                ),
                "StateLensIcs23MptClient"
            )
        );
    }

    function deployStateLensIcs23Ics23Client(
        IBCHandler handler,
        Manager manager
    ) internal returns (StateLensIcs23Ics23Client) {
        return StateLensIcs23Ics23Client(
            deployIfNotExists(
                LIGHT_CLIENT_SALT.STATE_LENS_ICS23_ICS23,
                abi.encode(
                    address(new StateLensIcs23Ics23Client(address(handler))),
                    abi.encodeCall(
                        StateLensIcs23Ics23Client.initialize, (address(manager))
                    )
                ),
                "StateLensIcs23Ics23Client"
            )
        );
    }

    function deployStateLensIcs23SmtClient(
        IBCHandler handler,
        Manager manager
    ) internal returns (StateLensIcs23SmtClient) {
        return StateLensIcs23SmtClient(
            deployIfNotExists(
                LIGHT_CLIENT_SALT.STATE_LENS_ICS23_SMT,
                abi.encode(
                    address(new StateLensIcs23SmtClient(address(handler))),
                    abi.encodeCall(
                        StateLensIcs23SmtClient.initialize, (address(manager))
                    )
                ),
                "StateLensIcs23SmtClient"
            )
        );
    }

    function deployCometbls(
        IBCHandler handler,
        Manager manager
    ) internal returns (CometblsClient) {
        return CometblsClient(
            deployIfNotExists(
                LIGHT_CLIENT_SALT.COMETBLS,
                abi.encode(
                    address(new CometblsClient(address(handler))),
                    abi.encodeCall(
                        CometblsClient.initialize, (address(manager))
                    )
                ),
                "CometblsClient"
            )
        );
    }

    function deployUCS00(
        IBCHandler handler,
        Manager manager,
        uint64 timeout
    ) internal returns (PingPong) {
        return PingPong(
            deployIfNotExists(
                Protocols.UCS00,
                abi.encode(
                    address(new PingPong()),
                    abi.encodeCall(
                        PingPong.initialize,
                        (handler, address(manager), timeout)
                    )
                ),
                "UCS00 PingPong"
            )
        );
    }

    function deployUCS03(
        IBCHandler handler,
        Manager manager,
        ZkgmERC20 zkgmERC20,
        ProxyAccount accountImpl,
        UCS03Parameters memory params
    ) internal returns (UCS03Zkgm) {
        UCS03Zkgm zkgm = UCS03Zkgm(
            payable(
                deployIfNotExists(
                    Protocols.UCS03,
                    abi.encode(
                        address(
                            new UCS03Zkgm(
                                handler,
                                new UCS03ZkgmSendImpl(
                                    handler,
                                    params.weth,
                                    zkgmERC20,
                                    params.nativeTokenName,
                                    params.nativeTokenSymbol,
                                    params.nativeTokenDecimals
                                ),
                                new UCS03ZkgmTokenOrderImpl(
                                    params.weth,
                                    zkgmERC20,
                                    params.rateLimitEnabled
                                ),
                                accountImpl
                            )
                        ),
                        abi.encodeCall(UCS03Zkgm.initialize, (address(manager)))
                    ),
                    "UCS03 Zkgm"
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
                deployIfNotExists(
                    Protocols.UCS06,
                    abi.encode(
                        address(new UCS06FundedDispatch()),
                        abi.encodeCall(
                            UCS06FundedDispatch.initialize, (address(manager))
                        )
                    ),
                    "UCS06 FundedDispatch"
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
        ProxyAccount accountImpl = deployProxyAccount();
        UCS03Zkgm ucs03 =
            deployUCS03(handler, manager, zkgmERC20, accountImpl, ucs03Params);
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

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(privateKey);
        ZkgmERC20 zkgmERC20 = deployZkgmERC20();
        vm.stopBroadcast();

        console.log("ZkgmERC20: ", address(zkgmERC20));
    }
}

contract DeployProxyAccount is UnionScript, VersionedScript {
    using LibString for *;

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(privateKey);
        ProxyAccount accountImpl = deployProxyAccount();
        vm.stopBroadcast();

        console.log("ProxyAccount: ", address(accountImpl));
    }
}

contract DeployUCS03 is UnionScript, VersionedScript {
    using LibString for *;

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));
        IBCHandler handler = IBCHandler(getDeployed(IBC_SALT.BASED));
        ZkgmERC20 zkgmERC20 =
            ZkgmERC20(getDeployed(LIB_SALT.UCS03_ZKGM_ERC20_IMPL));
        ProxyAccount accountImpl =
            ProxyAccount(getDeployed(LIB_SALT.UCS03_ZKGM_ACCOUNT_IMPL));

        vm.startBroadcast(privateKey);
        UCS03Zkgm zkgm = deployUCS03(
            handler, manager, zkgmERC20, accountImpl, getUCS03Params()
        );
        vm.stopBroadcast();

        console.log("UCS03: ", address(zkgm));
    }
}

contract DeployStateLensIcs23Ics23Client is UnionScript, VersionedScript {
    using LibString for *;

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
        console.log("Deployer: ", address(getDeployer()));
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
        U u = deployU(contracts.manager, contracts.ucs03, "Union", "U", 18);
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
        console.log("Deployer: ", address(getDeployer()));
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
        console.log("U: ", address(u));
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
        address proxyAccount = getDeployed(LIB_SALT.UCS03_ZKGM_ACCOUNT_IMPL);

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

        address u = getDeployed(string(INSTANCE_SALT.U));
        address eu = getDeployed(string(INSTANCE_SALT.EU));
        address udrop = getDeployed(string(INSTANCE_SALT.UDROP));

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
        console.log(string(abi.encodePacked("U: ", u.toHexString())));
        console.log(string(abi.encodePacked("UDrop: ", udrop.toHexString())));

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

        string memory proxyU = "proxyU";
        proxyU.serialize(
            "contract",
            string(
                "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
            )
        );
        proxyU = proxyU.serialize(
            "args",
            abi.encode(
                implOf(u),
                abi.encodeCall(
                    U.initialize, (manager, ucs03, "Union", "U", 18, hex"")
                )
            )
        );
        impls.serialize(u.toHexString(), proxyU);

        if (eu.code.length > 0) {
            string memory proxyEU = "proxyEU";
            proxyEU.serialize(
                "contract",
                string(
                    "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
                )
            );
            proxyEU = proxyEU.serialize(
                "args",
                abi.encode(
                    implOf(eu),
                    abi.encodeCall(
                        U.initialize,
                        (
                            manager,
                            ucs03,
                            "Escher Staked U",
                            "eU",
                            18,
                            bytes(hex"")
                        )
                    )
                )
            );
            impls.serialize(eu.toHexString(), proxyEU);
        }

        if (udrop.code.length > 0) {
            string memory proxyUDrop = "proxyUDrop";
            proxyUDrop.serialize(
                "contract",
                string(
                    "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
                )
            );
            proxyUDrop = proxyUDrop.serialize(
                "args",
                abi.encode(
                    implOf(udrop),
                    abi.encodeCall(
                        UDrop.initialize, (manager, UDrop(udrop).active())
                    )
                )
            );
            impls.serialize(udrop.toHexString(), proxyUDrop);
        }

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

        string memory proxyStateLensIcs23Ics23Client =
            "proxyStateLensIcs23Ics23Client";
        proxyStateLensIcs23Ics23Client.serialize(
            "contract",
            string(
                "libs/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy"
            )
        );
        proxyStateLensIcs23Ics23Client = proxyStateLensIcs23Ics23Client
            .serialize(
            "args",
            abi.encode(
                implOf(stateLensIcs23Ics23Client),
                abi.encodeCall(StateLensIcs23Ics23Client.initialize, (manager))
            )
        );
        impls.serialize(
            stateLensIcs23Ics23Client.toHexString(),
            proxyStateLensIcs23Ics23Client
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

        string memory implProxyAccount = "implProxyAccount";
        implProxyAccount.serialize(
            "contract", string("contracts/ProxyAccount.sol:ProxyAccount")
        );
        implProxyAccount = implProxyAccount.serialize("args", bytes(hex""));
        impls.serialize(proxyAccount.toHexString(), implProxyAccount);

        string memory implManager = "implManager";
        implManager.serialize(
            "contract", string("contracts/Manager.sol:Manager")
        );
        implManager = implManager.serialize("args", bytes(hex""));
        impls.serialize(implOf(manager).toHexString(), implManager);

        string memory implU = "implU";
        implU.serialize("contract", string("contracts/U.sol:U"));
        implU = implU.serialize("args", bytes(hex""));
        impls.serialize(implOf(u).toHexString(), implU);

        if (eu.code.length > 0) {
            string memory implEU = "implEU";
            implEU.serialize("contract", string("contracts/U.sol:U"));
            implEU = implEU.serialize("args", bytes(hex""));
            impls.serialize(implOf(eu).toHexString(), implEU);
        }

        if (udrop.code.length > 0) {
            string memory implUDrop = "implUDrop";
            implUDrop.serialize(
                "contract", string("contracts/tge/UDrop.sol:UDrop")
            );
            implUDrop = implUDrop.serialize(
                "args", abi.encode(UDrop(udrop).ROOT(), UDrop(udrop).TOKEN())
            );
            impls.serialize(implOf(udrop).toHexString(), implUDrop);
        }

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

        string memory implUCS03Send = "implUCS03Send";
        implUCS03Send.serialize(
            "contract",
            string("contracts/apps/ucs/03-zkgm/Send.sol:UCS03ZkgmSendImpl")
        );
        implUCS03Send = implUCS03Send.serialize(
            "args",
            abi.encode(
                handler,
                weth,
                zkgmERC20,
                nativeTokenName,
                nativeTokenSymbol,
                nativeTokenDecimals
            )
        );
        impls.serialize(
            UCS03Zkgm(payable(ucs03)).SEND_IMPL().toHexString(), implUCS03Send
        );

        string memory implUCS03FAO = "implUCS03FAO";
        implUCS03FAO.serialize(
            "contract",
            string(
                "contracts/apps/ucs/03-zkgm/TokenOrder.sol:UCS03ZkgmTokenOrderImpl"
            )
        );
        implUCS03FAO = implUCS03FAO.serialize(
            "args", abi.encode(weth, zkgmERC20, rateLimitEnabled)
        );
        impls.serialize(
            UCS03Zkgm(payable(ucs03)).FAO_IMPL().toHexString(), implUCS03FAO
        );

        string memory implUCS03 = "implUCS03";
        implUCS03.serialize(
            "contract", string("contracts/apps/ucs/03-zkgm/Zkgm.sol:UCS03Zkgm")
        );
        implUCS03 = implUCS03.serialize(
            "args",
            abi.encode(
                handler,
                UCS03Zkgm(payable(ucs03)).SEND_IMPL(),
                UCS03Zkgm(payable(ucs03)).FAO_IMPL()
            )
        );
        impls = impls.serialize(implOf(ucs03).toHexString(), implUCS03);

        impls.write(vm.envString("OUTPUT"));
    }
}

// Base contract for all upgrade operations (direct, safe, and dry-run)
abstract contract BaseUpgrade is VersionedScript {
    using LibString for *;
    using Safe for *;

    address immutable deployer;
    address immutable sender;
    uint256 immutable privateKey;
    address immutable owner;
    bool immutable useSafe;
    bool immutable isDryRun;

    Safe.Client safe;

    constructor(bool _useSafe, bool _isDryRun) {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
        useSafe = _useSafe;
        isDryRun = _isDryRun;

        if (isDryRun) {
            owner = vm.envAddress("OWNER");
        } else {
            privateKey = vm.envUint("PRIVATE_KEY");
        }

        if (useSafe) {
            safe.initialize(SafeLib.ADDRESS);
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

    // Must be implemented by derived contracts to provide upgrade parameters
    function upgradeParameters()
        internal
        virtual
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        );

    function run() public {
        if (isDryRun) {
            (
                address targetContract,
                address newImplementation,
                bytes memory upgradeCall
            ) = upgradeParameters();
            console.log(
                string(
                    abi.encodePacked(
                        "Dry run upgrade: ", targetContract.toHexString()
                    )
                )
            );
            vm.prank(owner);
            UUPSUpgradeable(targetContract).upgradeToAndCall(
                newImplementation, upgradeCall
            );
        } else {
            vm.startBroadcast(privateKey);
            (
                address targetContract,
                address newImplementation,
                bytes memory upgradeCall
            ) = upgradeParameters();
            if (useSafe) {
                safe.proposeTransaction(
                    targetContract,
                    abi.encodeCall(
                        UUPSUpgradeable.upgradeToAndCall,
                        (newImplementation, upgradeCall)
                    ),
                    vm.createWallet(privateKey).addr
                );
            } else {
                console.log(
                    string(
                        abi.encodePacked(
                            "Upgrading: ", targetContract.toHexString()
                        )
                    )
                );
                UUPSUpgradeable(targetContract).upgradeToAndCall(
                    newImplementation, upgradeCall
                );
            }
            vm.stopBroadcast();
        }
    }
}

contract DryUpgradeUCS03 is BaseUpgrade {
    constructor() BaseUpgrade(false, true) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        IWETH weth = IWETH(vm.envAddress("WETH_ADDRESS"));
        bool rateLimitEnabled = vm.envBool("RATE_LIMIT_ENABLED");
        string memory nativeTokenName = vm.envString("NATIVE_TOKEN_NAME");
        string memory nativeTokenSymbol = vm.envString("NATIVE_TOKEN_SYMBOL");
        uint8 nativeTokenDecimals = uint8(vm.envUint("NATIVE_TOKEN_DECIMALS"));

        IIBCModulePacket handler = IIBCModulePacket(getDeployed(IBC_SALT.BASED));
        ZkgmERC20 zkgmERC20 =
            ZkgmERC20(getDeployed(LIB_SALT.UCS03_ZKGM_ERC20_IMPL));
        ProxyAccount accountImpl =
            ProxyAccount(getDeployed(LIB_SALT.UCS03_ZKGM_ACCOUNT_IMPL));

        targetContract = getDeployed(Protocols.UCS03);
        newImplementation = address(
            new UCS03Zkgm(
                handler,
                new UCS03ZkgmSendImpl(
                    handler,
                    weth,
                    zkgmERC20,
                    nativeTokenName,
                    nativeTokenSymbol,
                    nativeTokenDecimals
                ),
                new UCS03ZkgmTokenOrderImpl(weth, zkgmERC20, rateLimitEnabled),
                accountImpl
            )
        );
        upgradeCall = new bytes(0);
    }
}

contract UpgradeUCS03 is BaseUpgrade {
    constructor() BaseUpgrade(false, false) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        IWETH weth = IWETH(vm.envAddress("WETH_ADDRESS"));
        bool rateLimitEnabled = vm.envBool("RATE_LIMIT_ENABLED");
        string memory nativeTokenName = vm.envString("NATIVE_TOKEN_NAME");
        string memory nativeTokenSymbol = vm.envString("NATIVE_TOKEN_SYMBOL");
        uint8 nativeTokenDecimals = uint8(vm.envUint("NATIVE_TOKEN_DECIMALS"));

        IIBCModulePacket handler = IIBCModulePacket(getDeployed(IBC_SALT.BASED));
        ZkgmERC20 zkgmERC20 =
            ZkgmERC20(getDeployed(LIB_SALT.UCS03_ZKGM_ERC20_IMPL));
        ProxyAccount accountImpl =
            ProxyAccount(getDeployed(LIB_SALT.UCS03_ZKGM_ACCOUNT_IMPL));

        targetContract = getDeployed(Protocols.UCS03);
        newImplementation = address(
            new UCS03Zkgm(
                handler,
                new UCS03ZkgmSendImpl(
                    handler,
                    weth,
                    zkgmERC20,
                    nativeTokenName,
                    nativeTokenSymbol,
                    nativeTokenDecimals
                ),
                new UCS03ZkgmTokenOrderImpl(weth, zkgmERC20, rateLimitEnabled),
                accountImpl
            )
        );
        upgradeCall = new bytes(0);
    }
}

abstract contract UCS03FromV1ToV2 is VersionedScript {
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

    function migrations()
        internal
        virtual
        returns (V1ToV2Migration[] memory, V1ToV2WrappedTokenMigration[] memory);

    function run() public {
        IWETH weth = IWETH(vm.envAddress("WETH_ADDRESS"));
        bool rateLimitEnabled = vm.envBool("RATE_LIMIT_ENABLED");
        string memory nativeTokenName = vm.envString("NATIVE_TOKEN_NAME");
        string memory nativeTokenSymbol = vm.envString("NATIVE_TOKEN_SYMBOL");
        uint8 nativeTokenDecimals = uint8(vm.envUint("NATIVE_TOKEN_DECIMALS"));

        IIBCModulePacket handler = IIBCModulePacket(getDeployed(IBC_SALT.BASED));
        ZkgmERC20 zkgmERC20 =
            ZkgmERC20(getDeployed(LIB_SALT.UCS03_ZKGM_ERC20_IMPL));
        ProxyAccount accountImpl =
            ProxyAccount(getDeployed(LIB_SALT.UCS03_ZKGM_ACCOUNT_IMPL));
        UCS03Zkgm ucs03 = UCS03Zkgm(payable(getDeployed(Protocols.UCS03)));

        console.log(
            string(abi.encodePacked("UCS03: ", address(ucs03).toHexString()))
        );

        vm.startBroadcast(privateKey);
        address newImplementation = address(
            new UCS03Zkgm(
                handler,
                new UCS03ZkgmSendImpl(
                    handler,
                    weth,
                    zkgmERC20,
                    nativeTokenName,
                    nativeTokenSymbol,
                    nativeTokenDecimals
                ),
                new UCS03ZkgmTokenOrderImpl(weth, zkgmERC20, rateLimitEnabled),
                accountImpl
            )
        );

        (
            V1ToV2Migration[] memory balanceMigrations,
            V1ToV2WrappedTokenMigration[] memory wrappedMigrations
        ) = migrations();
        ucs03.upgradeToAndCall(
            newImplementation,
            abi.encodeCall(
                UCS03Zkgm.migrateV1ToV2, (balanceMigrations, wrappedMigrations)
            )
        );
        vm.stopBroadcast();
    }
}

struct BridgedToken {
    string wrappedChainId;
    string direction;
    uint32 sourceChannelId;
    uint32 destinationChannelId;
    bytes baseToken;
    bytes quoteToken;
}

contract UpgradeUCS03FromV1ToV2 is UCS03FromV1ToV2 {
    using LibString for *;
    using stdJson for *;

    V1ToV2Migration[] balanceMigrations;
    V1ToV2WrappedTokenMigration[] wrappedMigrations;

    function migrations()
        internal
        override
        returns (V1ToV2Migration[] memory, V1ToV2WrappedTokenMigration[] memory)
    {
        string memory universalChainId = vm.envString("UNIVERSAL_CHAIN_ID");
        string memory json = vm.readFile("bridged_tokens_v1.json");

        uint256 length = 0;
        bool found = true;
        while (found) {
            try vm.parseJsonString(
                json,
                string(
                    abi.encodePacked("[", vm.toString(length), "].direction")
                )
            ) returns (string memory) {
                length++;
            } catch {
                found = false;
            }
        }

        BridgedToken[] memory bridgedTokens = new BridgedToken[](length);
        for (uint256 i = 0; i < length; i++) {
            BridgedToken memory bridgedToken = bridgedTokens[i];
            string memory prefix = string.concat(".[", vm.toString(i), "].");
            bridgedToken.wrappedChainId = vm.parseJsonString(
                json, string.concat(prefix, "wrapped_universal_chain_id")
            );
            bridgedToken.direction =
                vm.parseJsonString(json, string.concat(prefix, "direction"));
            bridgedToken.sourceChannelId = uint32(
                vm.parseJsonUint(
                    json, string.concat(prefix, "source_channel_id")
                )
            );
            bridgedToken.destinationChannelId = uint32(
                vm.parseJsonUint(
                    json, string.concat(prefix, "destination_channel_id")
                )
            );
            bridgedToken.baseToken =
                vm.parseJsonBytes(json, string.concat(prefix, "base_token"));
            bridgedToken.quoteToken =
                vm.parseJsonBytes(json, string.concat(prefix, "quote_token"));
            if (bridgedToken.wrappedChainId.eq(universalChainId)) {
                if (bridgedToken.direction.eq("in")) {
                    require(
                        bridgedToken.quoteToken.length == 20,
                        "in: invalid quote token length"
                    );
                    console.log("Wrapped migration:");
                    console.logBytes(bridgedToken.baseToken);
                    console.logBytes(bridgedToken.quoteToken);
                    wrappedMigrations.push(
                        V1ToV2WrappedTokenMigration({
                            path: 0,
                            channelId: bridgedToken.destinationChannelId,
                            baseToken: bridgedToken.baseToken,
                            quoteToken: address(bytes20(bridgedToken.quoteToken))
                        })
                    );
                } else if (bridgedToken.direction.eq("out")) {
                    require(
                        bridgedToken.baseToken.length == 20,
                        "out: invalid base token length"
                    );
                    console.log("Balance migration:");
                    console.logBytes(bridgedToken.baseToken);
                    console.logBytes(bridgedToken.quoteToken);
                    balanceMigrations.push(
                        V1ToV2Migration({
                            path: 0,
                            channelId: bridgedToken.sourceChannelId,
                            baseToken: address(bytes20(bridgedToken.baseToken)),
                            quoteToken: bridgedToken.quoteToken
                        })
                    );
                } else {
                    revert(bridgedToken.direction);
                }
            }
        }

        console.log(balanceMigrations.length);
        console.log(wrappedMigrations.length);

        return (balanceMigrations, wrappedMigrations);
    }
}

contract UpgradeUCS00 is BaseUpgrade {
    constructor() BaseUpgrade(false, false) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        targetContract = getDeployed(Protocols.UCS00);
        newImplementation = address(new PingPong());
        upgradeCall = new bytes(0);
    }
}

contract DryUpgradeIBCHandler is BaseUpgrade {
    constructor() BaseUpgrade(false, true) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        targetContract = getDeployed(IBC_SALT.BASED);
        newImplementation = address(new IBCHandler());
        upgradeCall = new bytes(0);
    }
}

contract UpgradeIBCHandler is BaseUpgrade {
    constructor() BaseUpgrade(false, false) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        targetContract = getDeployed(IBC_SALT.BASED);
        newImplementation = address(new IBCHandler());
        upgradeCall = new bytes(0);
    }
}

contract DryUpgradeCometblsClient is BaseUpgrade {
    constructor() BaseUpgrade(false, true) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        address handler = getDeployed(IBC_SALT.BASED);
        targetContract = getDeployed(LIGHT_CLIENT_SALT.COMETBLS);
        newImplementation = address(new CometblsClient(handler));
        upgradeCall = new bytes(0);
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

contract UpgradeU is BaseUpgrade {
    constructor() BaseUpgrade(false, false) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        targetContract = getDeployed(string(INSTANCE_SALT.U));
        newImplementation = address(new U());
        upgradeCall = new bytes(0);
    }
}

contract UpgradeEU is BaseUpgrade {
    constructor() BaseUpgrade(false, false) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        targetContract = getDeployed(string(INSTANCE_SALT.EU));
        newImplementation = address(new U());
        upgradeCall = new bytes(0);
    }
}

contract DeployRoles is UnionScript {
    using LibString for *;

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

contract DeployRegisterClients is UnionScript {
    using LibString for *;

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

        Contracts memory contracts = getContracts();

        vm.startBroadcast(privateKey);

        try contracts.handler.registerClient(
            LightClients.COMETBLS, contracts.cometblsClient
        ) {} catch Error(string memory reason) {
            console.log(
                "error deploying client ", LightClients.COMETBLS, ": ", reason
            );
        }

        try contracts.handler.registerClient(
            LightClients.STATE_LENS_ICS23_MPT, contracts.stateLensIcs23MptClient
        ) {} catch Error(string memory reason) {
            console.log(
                "error deploying client ", LightClients.COMETBLS, ": ", reason
            );
        }

        try contracts.handler.registerClient(
            LightClients.STATE_LENS_ICS23_ICS23,
            contracts.stateLensIcs23Ics23Client
        ) {} catch Error(string memory reason) {
            console.log(
                "error deploying client ", LightClients.COMETBLS, ": ", reason
            );
        }

        try contracts.handler.registerClient(
            LightClients.STATE_LENS_ICS23_SMT, contracts.stateLensIcs23SmtClient
        ) {} catch Error(string memory reason) {
            console.log(
                "error deploying client ", LightClients.COMETBLS, ": ", reason
            );
        }

        vm.stopBroadcast();
    }
}

contract DeployU is UnionScript, VersionedScript {
    using LibString for *;

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));
        UCS03Zkgm ucs03 = UCS03Zkgm(payable(getDeployed(Protocols.UCS03)));

        vm.startBroadcast(privateKey);
        U u = deployU(manager, ucs03, "Union", "U", 18);
        vm.stopBroadcast();

        console.log("U: ", address(u));
    }
}

contract DryDeployU is UnionScript, VersionedScript {
    using LibString for *;

    function run() public {
        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));
        UCS03Zkgm ucs03 = UCS03Zkgm(payable(getDeployed(Protocols.UCS03)));

        vm.startPrank(getSender());
        U u = deployU(manager, ucs03, "Union", "U", 18);
        vm.stopPrank();

        console.log("U: ", address(u));
    }
}

contract DeployEU is UnionScript, VersionedScript {
    using LibString for *;

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));
        UCS03Zkgm ucs03 = UCS03Zkgm(payable(getDeployed(Protocols.UCS03)));

        vm.startBroadcast(privateKey);
        U eu = deployEU(manager, ucs03, "Escher Staked U", "eU", 18);
        vm.stopBroadcast();

        console.log("eU: ", address(eu));
    }
}

contract DryDeployEU is UnionScript, VersionedScript {
    using LibString for *;

    function run() public {
        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));
        UCS03Zkgm ucs03 = UCS03Zkgm(payable(getDeployed(Protocols.UCS03)));

        vm.startPrank(getSender());
        U eu = deployEU(manager, ucs03, "Escher Staked U", "eU", 18);
        vm.stopPrank();

        console.log("eU: ", address(eu));
    }
}

contract DeployUDrop is UnionScript, VersionedScript {
    using LibString for *;

    bytes32 immutable root;
    bool immutable active;

    constructor() {
        root = vm.envBytes32("MERKLE_ROOT");
        active = vm.envBool("ACTIVE");
    }

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");

        Manager manager = Manager(getDeployed(IBC_SALT.MANAGER));
        address token = getDeployed(string(INSTANCE_SALT.U));

        vm.startBroadcast(privateKey);
        UDrop udrop = deployUDrop(manager, root, token, active);
        vm.stopBroadcast();

        console.log("UDrop: ", address(udrop));
    }
}

contract UpgradeUDrop is VersionedScript {
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
        UDrop udrop = UDrop(getDeployed(string(INSTANCE_SALT.UDROP)));

        console.log(
            string(abi.encodePacked("UDrop: ", address(udrop).toHexString()))
        );

        vm.startBroadcast(privateKey);
        address newImplementation =
            address(new UDrop(udrop.ROOT(), udrop.TOKEN()));
        udrop.upgradeToAndCall(newImplementation, new bytes(0));
        vm.stopBroadcast();
    }
}

contract MintedAddress is VersionedScript {
    using LibString for *;
    using LibBytes for *;

    address immutable deployer;
    address immutable sender;
    bytes salt;

    constructor() {
        deployer = vm.envAddress("DEPLOYER");
        sender = vm.envAddress("SENDER");
        salt = vm.envBytes("SALT");
    }

    function run() public {
        vm.pauseGasMetering();
        address mintedAddress = CREATE3.predictDeterministicAddress(
            keccak256(abi.encodePacked(sender.toHexString(), "/", salt)),
            deployer
        );
        console.log("Salt");
        console.logBytes(salt);
        console.log("Minted address");
        console.log(mintedAddress);
    }
}

struct AirdropEntry {
    address beneficiary;
    uint256 amount;
}

contract SimpleMerkleTree is Script {
    mapping(bytes32 => AirdropEntry) preimages;

    function pushEntry(
        address beneficiary,
        uint256 amount
    ) internal returns (bytes32) {
        bytes32 image =
            EfficientHashLib.hash(abi.encodePacked(beneficiary, amount));
        preimages[image] =
            AirdropEntry({beneficiary: beneficiary, amount: amount});
        return image;
    }

    function run() public {
        bytes32[] memory leaves = new bytes32[](8);
        leaves[0] = pushEntry(address(1), 1);
        leaves[1] = pushEntry(address(2), 2);
        leaves[2] =
            pushEntry(address(0x50A22f95bcB21E7bFb63c7A8544AC0683dCeA302), 3);
        leaves[3] = pushEntry(address(4), 4);
        leaves[4] = pushEntry(address(5), 5);
        leaves[5] =
            pushEntry(address(0x2FB055fC77D751e2E6B7c88A1B404505154521c3), 6);
        leaves[6] = pushEntry(address(7), 7);
        leaves[7] = pushEntry(address(8), 8);
        LibSort.sort(leaves);

        bytes32[] memory tree = MerkleTreeLib.build(leaves);
        bytes32 root = MerkleTreeLib.root(tree);
        console.log("Root: ");
        console.logBytes32(root);
        for (uint256 i = 0; i < leaves.length; i++) {
            bytes32[] memory proof = MerkleTreeLib.leafProof(tree, i);
            console.log("==================================");
            console.log("index: ");
            console.log(i);
            console.log("beneficiary: ");
            console.log(preimages[leaves[i]].beneficiary);
            console.log("amount: ");
            console.log(preimages[leaves[i]].amount);
            console.log("proof: ");
            console.logBytes(abi.encode(proof));
        }
    }
}

// SafeUpgrade contracts - use the same BaseUpgrade with useSafe=true
contract SafeUpgradeU is BaseUpgrade {
    constructor() BaseUpgrade(true, false) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        targetContract = getDeployed(string(INSTANCE_SALT.U));
        newImplementation = address(new U());
        upgradeCall = new bytes(0);
    }
}

contract SafeUpgradeEU is BaseUpgrade {
    constructor() BaseUpgrade(true, false) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        targetContract = getDeployed(string(INSTANCE_SALT.EU));
        newImplementation = address(new U());
        upgradeCall = new bytes(0);
    }
}

contract SafeUpgradeIBCHandler is BaseUpgrade {
    constructor() BaseUpgrade(true, false) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        targetContract = getDeployed(IBC_SALT.BASED);
        newImplementation = address(new IBCHandler());
        upgradeCall = new bytes(0);
    }
}

contract SafeUpgradeUCS03 is BaseUpgrade {
    constructor() BaseUpgrade(true, false) {}

    function upgradeParameters()
        internal
        override
        returns (
            address targetContract,
            address newImplementation,
            bytes memory upgradeCall
        )
    {
        IWETH weth = IWETH(vm.envAddress("WETH_ADDRESS"));
        bool rateLimitEnabled = vm.envBool("RATE_LIMIT_ENABLED");
        string memory nativeTokenName = vm.envString("NATIVE_TOKEN_NAME");
        string memory nativeTokenSymbol = vm.envString("NATIVE_TOKEN_SYMBOL");
        uint8 nativeTokenDecimals = uint8(vm.envUint("NATIVE_TOKEN_DECIMALS"));
        IIBCModulePacket handler = IIBCModulePacket(getDeployed(IBC_SALT.BASED));
        ZkgmERC20 zkgmERC20 =
            ZkgmERC20(getDeployed(LIB_SALT.UCS03_ZKGM_ERC20_IMPL));
        ProxyAccount accountImpl =
            ProxyAccount(getDeployed(LIB_SALT.UCS03_ZKGM_ACCOUNT_IMPL));

        targetContract = getDeployed(Protocols.UCS03);
        newImplementation = address(
            new UCS03Zkgm(
                handler,
                new UCS03ZkgmSendImpl(
                    handler,
                    weth,
                    zkgmERC20,
                    nativeTokenName,
                    nativeTokenSymbol,
                    nativeTokenDecimals
                ),
                new UCS03ZkgmTokenOrderImpl(weth, zkgmERC20, rateLimitEnabled),
                accountImpl
            )
        );
        upgradeCall = new bytes(0);
    }
}
