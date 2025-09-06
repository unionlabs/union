pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "../../contracts/ProxyAccount.sol";
import "../../contracts/apps/ucs/03-zkgm/IZkgmable.sol";

contract MockERC20 is ERC20 {
    constructor() ERC20("Mock Token", "MOCK") {
        _mint(msg.sender, 1000000 * 10 ** 18);
    }
}

contract MockZkgm {
    mapping(address => bool) public authorizedAccounts;

    function authorizeAccount(
        address account
    ) external {
        authorizedAccounts[account] = true;
    }
}

contract MaliciousContract {
    uint256 public attackCount;
    ProxyAccount public target;

    constructor(
        address _target
    ) {
        target = ProxyAccount(_target);
    }

    receive() external payable {
        attackCount++;
        if (attackCount < 3) {
            // Attempt reentrancy
            target.execute(address(this), 0, "");
        }
    }

    function attack() external {
        target.execute(address(this), 1 ether, "");
    }
}

contract ProxyAccountTest is Test {
    ProxyAccount public implementation;
    ProxyAccount public proxyAccount;
    MockZkgm public mockZkgm;
    MockERC20 public mockToken;

    address public constant ADMIN_1 = address(0x1);
    address public constant ADMIN_2 = address(0x2);
    address public constant NON_ADMIN = address(0x3);
    address public constant RECIPIENT = address(0x1234);

    bytes public constant REMOTE_ADMIN_1 = hex"0001";
    bytes public constant REMOTE_ADMIN_2 = hex"0002";
    uint256 public constant PATH_1 = 1;
    uint32 public constant CHANNEL_1 = 100;

    // Events from ProxyAccount contract
    event LocalAdminAdded(address indexed admin);
    event LocalAdminRemoved(address indexed admin);
    event RemoteAdminAdded(
        uint256 indexed path, uint32 indexed channelId, bytes admin
    );
    event RemoteAdminRemoved(
        uint256 indexed path, uint32 indexed channelId, bytes admin
    );

    function setUp() public {
        // Deploy mock contracts
        mockZkgm = new MockZkgm();
        mockToken = new MockERC20();

        // Deploy implementation
        implementation = new ProxyAccount();

        // Deploy and initialize proxy with remote admin
        bytes memory initData = abi.encodeCall(
            ProxyAccount.initializeRemote,
            (address(mockZkgm), PATH_1, CHANNEL_1, REMOTE_ADMIN_1)
        );

        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), initData);

        proxyAccount = ProxyAccount(payable(address(proxy)));

        // Authorize account in mock zkgm
        mockZkgm.authorizeAccount(address(proxyAccount));

        // Fund the proxy account
        vm.deal(address(proxyAccount), 10 ether);
        mockToken.transfer(address(proxyAccount), 1000 * 10 ** 18);
    }

    // Helper function to add a local admin via zkgm
    function _addLocalAdminViaZkgm(
        address admin
    ) internal {
        bytes memory addAdminCall =
            abi.encodeCall(ProxyAccount.addLocalAdmin, admin);
        bytes memory message =
            abi.encode(address(proxyAccount), 0, addAdminCall);

        vm.prank(address(mockZkgm));
        proxyAccount.onZkgm(
            address(this),
            PATH_1,
            1,
            CHANNEL_1,
            REMOTE_ADMIN_1,
            message,
            address(this),
            ""
        );
    }

    // Helper function to execute arbitrary call via zkgm
    function _executeViaZkgm(
        address target,
        uint256 value,
        bytes memory payload
    ) internal {
        bytes memory message = abi.encode(target, value, payload);

        vm.prank(address(mockZkgm));
        proxyAccount.onZkgm(
            address(this),
            PATH_1,
            1,
            CHANNEL_1,
            REMOTE_ADMIN_1,
            message,
            address(this),
            ""
        );
    }

    // ============ Initialization Tests ============

    function test_Initialize() public view {
        assertEq(proxyAccount.zkgm(), address(mockZkgm));
        assertTrue(
            proxyAccount.isRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_1)
        );
    }

    function test_InitializeEmitsEvent() public {
        // Deploy a new proxy to test initialization event emission
        ProxyAccount newImplementation = new ProxyAccount();

        // Expect the RemoteAdminAdded event to be emitted during initialization
        vm.expectEmit(true, true, false, true);
        emit RemoteAdminAdded(PATH_1, CHANNEL_1, REMOTE_ADMIN_1);

        bytes memory initData = abi.encodeCall(
            ProxyAccount.initializeRemote,
            (address(mockZkgm), PATH_1, CHANNEL_1, REMOTE_ADMIN_1)
        );

        new ERC1967Proxy(address(newImplementation), initData);
    }

    function test_CannotReinitialize() public {
        vm.expectRevert();
        proxyAccount.initializeRemote(address(0x5), 2, 200, hex"0003");

        vm.expectRevert();
        proxyAccount.initializeLocal(address(0x5));
    }

    function test_ImplementationInitializationDisabled() public {
        vm.expectRevert();
        implementation.initializeRemote(
            address(mockZkgm), PATH_1, CHANNEL_1, REMOTE_ADMIN_1
        );

        vm.expectRevert();
        implementation.initializeLocal(ADMIN_1);
    }

    // ============ Local Initialization Tests ============

    function test_InitializeLocal() public {
        // Deploy new implementation and proxy for local initialization
        ProxyAccount localImplementation = new ProxyAccount();

        vm.expectEmit(true, false, false, true);
        emit LocalAdminAdded(ADMIN_1);

        bytes memory initData =
            abi.encodeCall(ProxyAccount.initializeLocal, ADMIN_1);

        ERC1967Proxy localProxy =
            new ERC1967Proxy(address(localImplementation), initData);
        ProxyAccount localProxyAccount =
            ProxyAccount(payable(address(localProxy)));

        // Verify local admin was added
        assertTrue(localProxyAccount.isLocalAdmin(ADMIN_1));

        // Verify zkgm is not set initially (should be zero address)
        assertEq(localProxyAccount.zkgm(), address(0));

        // Verify no remote admins exist initially
        assertFalse(
            localProxyAccount.isRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_1)
        );
    }

    function test_InitializeLocalZeroAddressReverts() public {
        ProxyAccount localImplementation = new ProxyAccount();

        bytes memory initData =
            abi.encodeCall(ProxyAccount.initializeLocal, address(0));

        vm.expectRevert(ProxyAccount.ProxyAccount_ZeroAdmin.selector);
        new ERC1967Proxy(address(localImplementation), initData);
    }

    function test_PromoteLocalToRemoteCapabilities() public {
        // Deploy new proxy with local initialization
        ProxyAccount localImplementation = new ProxyAccount();

        bytes memory initData =
            abi.encodeCall(ProxyAccount.initializeLocal, ADMIN_1);

        ERC1967Proxy localProxy =
            new ERC1967Proxy(address(localImplementation), initData);
        ProxyAccount localProxyAccount =
            ProxyAccount(payable(address(localProxy)));

        // Verify initial local-only state
        assertTrue(localProxyAccount.isLocalAdmin(ADMIN_1));
        assertEq(localProxyAccount.zkgm(), address(0));

        // Promote to remote capabilities by setting zkgm
        vm.prank(ADMIN_1);
        localProxyAccount.setZkgm(address(mockZkgm));

        // Verify zkgm is now set
        assertEq(localProxyAccount.zkgm(), address(mockZkgm));

        // Add remote admin via local admin
        vm.expectEmit(true, true, false, true);
        emit RemoteAdminAdded(PATH_1, CHANNEL_1, REMOTE_ADMIN_1);

        vm.prank(ADMIN_1);
        localProxyAccount.addRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_1);

        // Verify remote admin was added
        assertTrue(
            localProxyAccount.isRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_1)
        );

        // Authorize the proxy in mock zkgm for testing onZkgm calls
        mockZkgm.authorizeAccount(address(localProxyAccount));

        // Fund the proxy account for testing
        vm.deal(address(localProxyAccount), 1 ether);

        // Now test that remote admin can execute via zkgm
        bytes memory message = abi.encode(RECIPIENT, 0.5 ether, "");
        uint256 initialBalance = RECIPIENT.balance;

        vm.prank(address(mockZkgm));
        localProxyAccount.onZkgm(
            address(this),
            PATH_1,
            1,
            CHANNEL_1,
            REMOTE_ADMIN_1,
            message,
            address(this),
            ""
        );

        assertEq(RECIPIENT.balance, initialBalance + 0.5 ether);
    }

    function test_LocalInitializedCannotReceiveZkgmCallsInitially() public {
        // Deploy new proxy with local initialization
        ProxyAccount localImplementation = new ProxyAccount();

        bytes memory initData =
            abi.encodeCall(ProxyAccount.initializeLocal, ADMIN_1);

        ERC1967Proxy localProxy =
            new ERC1967Proxy(address(localImplementation), initData);
        ProxyAccount localProxyAccount =
            ProxyAccount(payable(address(localProxy)));

        // Try to call onZkgm without setting zkgm first - should fail
        bytes memory message = abi.encode(RECIPIENT, 1 ether, "");

        vm.expectRevert(ProxyAccount.ProxyAccount_ZeroZkgm.selector);
        vm.prank(address(mockZkgm));
        localProxyAccount.onZkgm(
            address(this),
            PATH_1,
            1,
            CHANNEL_1,
            REMOTE_ADMIN_1,
            message,
            address(this),
            ""
        );
    }

    function test_SetZkgmOnlyAdmin() public {
        // Deploy new proxy with local initialization
        ProxyAccount localImplementation = new ProxyAccount();

        bytes memory initData =
            abi.encodeCall(ProxyAccount.initializeLocal, ADMIN_1);

        ERC1967Proxy localProxy =
            new ERC1967Proxy(address(localImplementation), initData);
        ProxyAccount localProxyAccount =
            ProxyAccount(payable(address(localProxy)));

        // Non-admin cannot set zkgm
        vm.prank(NON_ADMIN);
        vm.expectRevert(ProxyAccount.ProxyAccount_OnlyAdmin.selector);
        localProxyAccount.setZkgm(address(mockZkgm));

        // Admin can set zkgm
        vm.prank(ADMIN_1);
        localProxyAccount.setZkgm(address(mockZkgm));

        assertEq(localProxyAccount.zkgm(), address(mockZkgm));
    }

    // ============ Access Control Tests ============

    function test_OnlyAdminCanAddLocalAdmin() public {
        // Non-admin cannot add admin
        vm.prank(NON_ADMIN);
        vm.expectRevert(ProxyAccount.ProxyAccount_OnlyAdmin.selector);
        proxyAccount.addLocalAdmin(ADMIN_1);

        // Expect the LocalAdminAdded event to be emitted
        vm.expectEmit(true, false, false, true);
        emit LocalAdminAdded(ADMIN_1);

        // Remote admin via zkgm executes addLocalAdmin
        _addLocalAdminViaZkgm(ADMIN_1);

        assertTrue(proxyAccount.isLocalAdmin(ADMIN_1));
    }

    function test_LocalAdminCanAddOtherAdmins() public {
        // Setup: Add ADMIN_1 as local admin via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        // ADMIN_1 can add ADMIN_2 - expect event emission
        vm.expectEmit(true, false, false, true);
        emit LocalAdminAdded(ADMIN_2);

        vm.prank(ADMIN_1);
        proxyAccount.addLocalAdmin(ADMIN_2);

        assertTrue(proxyAccount.isLocalAdmin(ADMIN_2));
    }

    function test_RemoveLocalAdmin() public {
        // Setup: Add ADMIN_1 via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);
        assertTrue(proxyAccount.isLocalAdmin(ADMIN_1));

        // Remove ADMIN_1 via zkgm - expect event emission
        bytes memory removeCall =
            abi.encodeCall(ProxyAccount.removeLocalAdmin, ADMIN_1);

        vm.expectEmit(true, false, false, true);
        emit LocalAdminRemoved(ADMIN_1);

        _executeViaZkgm(address(proxyAccount), 0, removeCall);

        assertFalse(proxyAccount.isLocalAdmin(ADMIN_1));
    }

    function test_AddRemoteAdmin() public {
        // Add ADMIN_1 as local admin first via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        // ADMIN_1 adds new remote admin - expect event emission
        vm.expectEmit(true, true, false, true);
        emit RemoteAdminAdded(PATH_1, CHANNEL_1, REMOTE_ADMIN_2);

        vm.prank(ADMIN_1);
        proxyAccount.addRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_2);

        assertTrue(
            proxyAccount.isRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_2)
        );
    }

    function test_RemoveRemoteAdmin() public {
        // Add ADMIN_1 as local admin via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        // First add REMOTE_ADMIN_2
        vm.prank(ADMIN_1);
        proxyAccount.addRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_2);
        assertTrue(
            proxyAccount.isRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_2)
        );

        // ADMIN_1 removes REMOTE_ADMIN_2 - expect event emission
        vm.expectEmit(true, true, false, true);
        emit RemoteAdminRemoved(PATH_1, CHANNEL_1, REMOTE_ADMIN_2);

        vm.prank(ADMIN_1);
        proxyAccount.removeRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_2);

        assertFalse(
            proxyAccount.isRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_2)
        );
    }

    // ============ Event Emission Tests ============

    function test_EventEmissions_ComprehensiveTest() public {
        // Test all admin operations emit proper events

        // 1. Add local admin via remote admin - should emit LocalAdminAdded
        vm.expectEmit(true, false, false, true);
        emit LocalAdminAdded(ADMIN_1);
        _addLocalAdminViaZkgm(ADMIN_1);

        // 2. Local admin adds another local admin - should emit LocalAdminAdded
        vm.expectEmit(true, false, false, true);
        emit LocalAdminAdded(ADMIN_2);
        vm.prank(ADMIN_1);
        proxyAccount.addLocalAdmin(ADMIN_2);

        // 3. Local admin adds remote admin - should emit RemoteAdminAdded
        vm.expectEmit(true, true, false, true);
        emit RemoteAdminAdded(PATH_1, CHANNEL_1, REMOTE_ADMIN_2);
        vm.prank(ADMIN_1);
        proxyAccount.addRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_2);

        // 4. Local admin removes local admin - should emit LocalAdminRemoved
        vm.expectEmit(true, false, false, true);
        emit LocalAdminRemoved(ADMIN_2);
        vm.prank(ADMIN_1);
        proxyAccount.removeLocalAdmin(ADMIN_2);

        // 5. Local admin removes remote admin - should emit RemoteAdminRemoved
        vm.expectEmit(true, true, false, true);
        emit RemoteAdminRemoved(PATH_1, CHANNEL_1, REMOTE_ADMIN_2);
        vm.prank(ADMIN_1);
        proxyAccount.removeRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_2);

        // Verify final state
        assertTrue(proxyAccount.isLocalAdmin(ADMIN_1));
        assertFalse(proxyAccount.isLocalAdmin(ADMIN_2));
        assertFalse(
            proxyAccount.isRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_2)
        );
        assertTrue(
            proxyAccount.isRemoteAdmin(PATH_1, CHANNEL_1, REMOTE_ADMIN_1)
        ); // Original should still exist
    }

    // ============ Execution Tests ============

    function test_ExecuteTransferETH() public {
        // Add ADMIN_1 via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        uint256 initialBalance = RECIPIENT.balance;

        // Execute ETH transfer
        vm.prank(ADMIN_1);
        proxyAccount.execute(RECIPIENT, 1 ether, "");

        assertEq(RECIPIENT.balance, initialBalance + 1 ether);
    }

    function test_ExecuteERC20Transfer() public {
        // Add ADMIN_1 via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        uint256 initialBalance = mockToken.balanceOf(RECIPIENT);

        // Execute ERC20 transfer
        bytes memory transferData =
            abi.encodeCall(IERC20.transfer, (RECIPIENT, 100 * 10 ** 18));

        vm.prank(ADMIN_1);
        proxyAccount.execute(address(mockToken), 0, transferData);

        assertEq(
            mockToken.balanceOf(RECIPIENT), initialBalance + 100 * 10 ** 18
        );
    }

    function test_ExecuteViaZkgm() public {
        uint256 initialBalance = RECIPIENT.balance;

        // Remote admin executes via zkgm
        bytes memory message = abi.encode(RECIPIENT, 1 ether, "");

        vm.prank(address(mockZkgm));
        proxyAccount.onZkgm(
            address(this),
            PATH_1,
            1,
            CHANNEL_1,
            REMOTE_ADMIN_1,
            message,
            address(this),
            ""
        );

        assertEq(RECIPIENT.balance, initialBalance + 1 ether);
    }

    function test_NonAdminCannotExecute() public {
        vm.prank(NON_ADMIN);
        vm.expectRevert(ProxyAccount.ProxyAccount_OnlyAdmin.selector);
        proxyAccount.execute(RECIPIENT, 1 ether, "");
    }

    // ============ Multicall Tests ============

    function test_Multicall() public {
        // Add ADMIN_1 via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        // Prepare multicall data
        bytes[] memory calls = new bytes[](2);
        calls[0] = abi.encodeCall(ProxyAccount.addLocalAdmin, ADMIN_2);
        calls[1] =
            abi.encodeCall(ProxyAccount.execute, (RECIPIENT, 1 ether, ""));

        uint256 initialBalance = RECIPIENT.balance;

        // Execute multicall
        vm.prank(ADMIN_1);
        proxyAccount.multicall(calls);

        assertTrue(proxyAccount.isLocalAdmin(ADMIN_2));
        assertEq(RECIPIENT.balance, initialBalance + 1 ether);
    }

    function test_MulticallOnlyAdmin() public {
        bytes[] memory calls = new bytes[](1);
        calls[0] = abi.encodeCall(ProxyAccount.addLocalAdmin, ADMIN_1);

        vm.prank(NON_ADMIN);
        vm.expectRevert(ProxyAccount.ProxyAccount_OnlyAdmin.selector);
        proxyAccount.multicall(calls);
    }

    // ============ Upgrade Tests ============

    function test_UpgradeToNewImplementation() public {
        // Add ADMIN_1 via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        // Deploy new implementation
        ProxyAccount newImplementation = new ProxyAccount();

        // Upgrade
        vm.prank(ADMIN_1);
        proxyAccount.upgradeToAndCall(address(newImplementation), "");

        // Verify upgrade succeeded and state is preserved
        assertTrue(proxyAccount.isLocalAdmin(ADMIN_1));
        assertEq(proxyAccount.zkgm(), address(mockZkgm));
    }

    function test_OnlyAdminCanUpgrade() public {
        ProxyAccount newImplementation = new ProxyAccount();

        vm.prank(NON_ADMIN);
        vm.expectRevert(ProxyAccount.ProxyAccount_OnlyAdmin.selector);
        proxyAccount.upgradeToAndCall(address(newImplementation), "");
    }

    // ============ onZkgm Tests ============

    function test_OnlyZkgmCanCallOnZkgm() public {
        bytes memory message = abi.encode(RECIPIENT, 1 ether, "");

        vm.prank(NON_ADMIN);
        vm.expectRevert(ProxyAccount.ProxyAccount_OnlyZkgm.selector);
        proxyAccount.onZkgm(
            NON_ADMIN,
            PATH_1,
            1,
            CHANNEL_1,
            REMOTE_ADMIN_1,
            message,
            address(this),
            ""
        );
    }

    function test_OnlyRemoteAdminCanExecuteViaZkgm() public {
        bytes memory message = abi.encode(RECIPIENT, 1 ether, "");

        // Non-admin remote sender should fail
        vm.prank(address(mockZkgm));
        vm.expectRevert(ProxyAccount.ProxyAccount_OnlyAdmin.selector);
        proxyAccount.onZkgm(
            address(this),
            PATH_1,
            1,
            CHANNEL_1,
            hex"9999", // non-admin
            message,
            address(this),
            ""
        );
    }

    function test_OnIntentZkgmReverts() public {
        vm.prank(address(mockZkgm));
        vm.expectRevert(ProxyAccount.ProxyAccount_IntentUnsupported.selector);
        proxyAccount.onIntentZkgm(
            address(this),
            PATH_1,
            1,
            CHANNEL_1,
            REMOTE_ADMIN_1,
            "",
            address(this),
            ""
        );
    }

    // ============ Edge Case Tests ============

    function test_SelfCallAsAdmin() public {
        // Contract can call itself as it's considered admin
        bytes memory callData =
            abi.encodeCall(ProxyAccount.addLocalAdmin, ADMIN_1);
        bytes memory executeData = abi.encodeCall(
            ProxyAccount.execute, (address(proxyAccount), 0, callData)
        );

        // Execute via remote admin to make contract call itself
        _executeViaZkgm(address(proxyAccount), 0, executeData);

        assertTrue(proxyAccount.isLocalAdmin(ADMIN_1));
    }

    function test_AddLocalAdminZeroAddressReverts() public {
        // Add ADMIN_1 via zkgm
        bytes memory addCall =
            abi.encodeCall(ProxyAccount.addLocalAdmin, ADMIN_1);
        bytes memory message = abi.encode(address(proxyAccount), 0, addCall);

        vm.prank(address(mockZkgm));
        proxyAccount.onZkgm(
            address(this),
            PATH_1,
            1,
            CHANNEL_1,
            REMOTE_ADMIN_1,
            message,
            address(this),
            ""
        );

        // Should revert when trying to add zero address as local admin
        vm.prank(ADMIN_1);
        vm.expectRevert(ProxyAccount.ProxyAccount_ZeroAdmin.selector);
        proxyAccount.addLocalAdmin(address(0));

        // Verify zero address was not added
        assertFalse(proxyAccount.isLocalAdmin(address(0)));
    }

    function test_EmptyBytesRemoteAdmin() public {
        // Add ADMIN_1 via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        // Can add empty bytes as remote admin (potential issue)
        vm.prank(ADMIN_1);
        proxyAccount.addRemoteAdmin(PATH_1, CHANNEL_1, "");
        assertTrue(proxyAccount.isRemoteAdmin(PATH_1, CHANNEL_1, ""));
    }

    // ============ Reentrancy Test ============

    function test_ReentrancyAttack() public {
        // Deploy malicious contract
        MaliciousContract malicious =
            new MaliciousContract(address(proxyAccount));
        vm.deal(address(malicious), 1 ether);

        // Add ADMIN_1 via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        // Add malicious contract as admin (for testing)
        vm.prank(ADMIN_1);
        proxyAccount.addLocalAdmin(address(malicious));

        // Attempt reentrancy attack
        malicious.attack();

        // Check that multiple calls were attempted
        assertGt(malicious.attackCount(), 0);
    }

    // ============ Gas Tests ============

    function test_GasConsumption() public {
        // Add ADMIN_1 via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        // Measure gas for different operations
        uint256 gasStart = gasleft();
        vm.prank(ADMIN_1);
        proxyAccount.execute(RECIPIENT, 1 ether, "");
        uint256 gasUsed = gasStart - gasleft();

        console.log("Gas used for ETH transfer:", gasUsed);
        assertTrue(gasUsed < 50000, "Gas usage too high");
    }

    // ============ Fuzz Tests ============

    function testFuzz_AddMultipleAdmins(
        address[] memory admins
    ) public {
        vm.assume(admins.length > 0 && admins.length < 10);

        // Add first admin via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        // Add multiple admins
        for (uint256 i = 0; i < admins.length; i++) {
            if (admins[i] != address(0)) {
                vm.prank(ADMIN_1);
                proxyAccount.addLocalAdmin(admins[i]);
                assertTrue(proxyAccount.isLocalAdmin(admins[i]));
            }
        }
    }

    function testFuzz_ExecuteWithDifferentValues(
        uint256 value,
        address target
    ) public {
        vm.assume(value <= 5 ether);
        assumeUnusedAddress(target);

        // Add ADMIN_1 via zkgm
        _addLocalAdminViaZkgm(ADMIN_1);

        uint256 initialBalance = target.balance;

        // Execute transfer
        vm.prank(ADMIN_1);
        proxyAccount.execute(target, value, "");

        assertEq(target.balance, initialBalance + value);
    }
}
