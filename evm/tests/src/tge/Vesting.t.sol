pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/manager/IAccessManaged.sol";

import "../../../contracts/Manager.sol";
import "../../../contracts/tge/Vesting.sol";
import "../05-app/Zkgm.t.sol";

contract VestingTests is Test {
    Manager mgr;
    TestIBCHandler handler;
    TestERC20 erc20;
    ZkgmERC20 erc20Impl;
    TestWETH weth;
    TestZkgm zkgm;
    VestingManager vestingMgr;

    function setUp() public {
        handler = new TestIBCHandler();
        erc20Impl = new ZkgmERC20();
        mgr = Manager(
            address(
                new ERC1967Proxy(
                    address(new Manager()),
                    abi.encodeCall(Manager.initialize, address(this))
                )
            )
        );
        zkgm = TestZkgm(
            payable(
                address(
                    new ERC1967Proxy(
                        address(new TestZkgm(handler, weth, erc20Impl)),
                        abi.encodeCall(UCS03Zkgm.initialize, (address(mgr)))
                    )
                )
            )
        );
        erc20 = new TestERC20("Ether", "ETH", 18);
        vestingMgr = VestingManager(
            address(
                new ERC1967Proxy(
                    address(new VestingManager()),
                    abi.encodeCall(
                        VestingManager.initialize,
                        (address(mgr), address(new VestingAccount()), zkgm)
                    )
                )
            )
        );
    }

    function _isDeployed(
        address addr
    ) internal returns (bool) {
        uint32 size = 0;
        assembly {
            size := extcodesize(addr)
        }
        return size > 0;
    }

    function test_update_ok(
        bytes32 key,
        address beneficiary,
        uint64 start,
        uint64 cliff,
        uint64 duration
    ) public {
        vm.assume(cliff < duration);
        vm.assume(beneficiary != address(0));
        vestingMgr.update(key, beneficiary, start, cliff, duration);
        assertEq(vestingMgr.beneficiary(key), beneficiary);
        assertEq(vestingMgr.start(key), start);
        assertEq(vestingMgr.cliff(key), cliff);
        assertEq(vestingMgr.duration(key), duration);
    }

    function test_release_anybody_ok(
        address operator,
        bytes32 key,
        address beneficiary,
        // Use smaller than u64 to avoid overflow because of fuzzing. In practice the `start + cliff < max(u64)`
        uint32 start,
        uint16 cliff,
        uint32 duration
    ) public {
        vm.assume(cliff < duration);
        vm.assume(beneficiary != address(0));
        vm.assume(operator != address(this));
        vestingMgr.update(key, beneficiary, start, cliff, duration);
        VestingAccount account = vestingMgr.vestingAccount(key);
        erc20.mint(address(account), 10000);
        vm.prank(operator);
        vestingMgr.release(key, address(erc20));
    }

    function test_update_deploys_account(
        bytes32 key,
        address beneficiary,
        uint64 start,
        uint64 cliff,
        uint64 duration
    ) public {
        vm.assume(cliff < duration);
        vm.assume(beneficiary != address(0));
        VestingAccount account = vestingMgr.vestingAccount(key);
        assertFalse(_isDeployed(address(account)));
        vestingMgr.update(key, beneficiary, start, cliff, duration);
        assertTrue(_isDeployed(address(account)));
    }

    function test_account_transfer_onlyVestingManager(
        bytes32 key,
        address beneficiary,
        uint64 start,
        uint64 cliff,
        uint64 duration
    ) public {
        vm.assume(cliff < duration);
        vm.assume(beneficiary != address(0));
        vestingMgr.update(key, beneficiary, start, cliff, duration);
        VestingAccount account = vestingMgr.vestingAccount(key);
        vm.expectRevert(VestingAccount.VestingAccount_Unauthorized.selector);
        account.transfer(address(0), beneficiary, 1);
    }

    function test_account_execute_onlyAuthority(
        address malicious,
        bytes32 key,
        address beneficiary,
        uint64 start,
        uint64 cliff,
        uint64 duration
    ) public {
        vm.assume(cliff < duration);
        vm.assume(beneficiary != address(0));
        vm.assume(malicious != address(this));
        vestingMgr.update(key, beneficiary, start, cliff, duration);
        VestingAccount account = vestingMgr.vestingAccount(key);
        vm.prank(malicious);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessManaged.AccessManagedUnauthorized.selector, malicious
            )
        );
        account.execute(address(0), 0, hex"");
    }

    function test_update_onlyAuthority(
        address malicious,
        bytes32 key,
        address beneficiary,
        uint64 start,
        uint64 cliff,
        uint64 duration
    ) public {
        vm.assume(cliff < duration);
        vm.assume(beneficiary != address(0));
        vm.assume(malicious != address(this));
        vm.prank(malicious);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessManaged.AccessManagedUnauthorized.selector, malicious
            )
        );
        vestingMgr.update(key, beneficiary, start, cliff, duration);
    }

    function test_release_zero_untill_cliff(
        address operator,
        bytes32 key,
        address beneficiary,
        // Use smaller than u64 to avoid overflow because of fuzzing. In practice the `start + cliff < max(u64)`
        uint32 start,
        uint16 cliff,
        uint32 duration,
        uint64 timestamp,
        uint256 amount
    ) public {
        vm.assume(amount > 0);
        vm.assume(cliff < duration);
        vm.assume(timestamp < uint64(start) + uint64(cliff));
        vm.assume(operator != address(this));
        vm.assume(beneficiary != address(0));
        vestingMgr.update(key, beneficiary, start, cliff, duration);
        VestingAccount account = vestingMgr.vestingAccount(key);
        erc20.mint(address(account), amount);
        vm.expectEmit();
        emit VestingManager.Released(key, address(erc20), 0);
        vm.warp(timestamp);
        vm.prank(operator);
        vestingMgr.release(key, address(erc20));
    }

    function test_release_total_after_duration(
        address operator,
        bytes32 key,
        address beneficiary,
        // Use smaller than u64 to avoid overflow because of fuzzing. In practice the `start + cliff < max(u64)`
        uint32 start,
        uint16 cliff,
        uint32 duration,
        uint64 timestamp,
        uint256 amount
    ) public {
        vm.assume(amount > 0);
        vm.assume(cliff < duration);
        vm.assume(timestamp >= uint64(start) + uint64(duration));
        vm.assume(operator != address(this));
        vm.assume(beneficiary != address(0));
        vestingMgr.update(key, beneficiary, start, cliff, duration);
        VestingAccount account = vestingMgr.vestingAccount(key);
        erc20.mint(address(account), amount);
        vm.expectEmit();
        emit IERC20.Transfer(address(account), beneficiary, amount);
        vm.expectEmit();
        emit VestingManager.Released(key, address(erc20), amount);
        vm.warp(timestamp);
        vm.prank(operator);
        vestingMgr.release(key, address(erc20));
    }

    function test_upgrade_vesting_account_beacon() public {
        vestingMgr.upgradeVestingAccountBeacon(address(new VestingAccount()));
    }

    function test_upgrade_vesting_account_beacon_onlyAuthority(
        address malicious
    ) public {
        vm.assume(malicious != address(this));
        VestingAccount newImpl = new VestingAccount();
        vm.prank(malicious);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessManaged.AccessManagedUnauthorized.selector, malicious
            )
        );
        vestingMgr.upgradeVestingAccountBeacon(address(newImpl));
    }

    function test_stake_ok(
        bytes32 key,
        address beneficiary,
        // Use smaller than u64 to avoid overflow because of fuzzing. In practice the `start + cliff < max(u64)`
        uint32 start,
        uint16 cliff,
        uint32 duration,
        uint64 timestamp,
        uint256 amount,
        uint256 tokenId,
        bytes32 salt,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address governanceToken,
        bytes calldata validator
    ) public {
        vm.assume(governanceToken != address(0));
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        vm.assume(amount > 0);
        vm.assume(cliff < duration);
        vm.assume(timestamp < uint64(start) + uint64(cliff));
        vm.assume(beneficiary != address(0));
        assumeUnusedAddress(governanceToken);
        handler.setChannel(sourceChannelId, destinationChannelId);
        vestingMgr.update(key, beneficiary, start, cliff, duration);
        VestingAccount account = vestingMgr.vestingAccount(key);
        erc20.mint(address(account), amount);
        address localGovernanceToken = governanceToken;
        zkgm.registerGovernanceToken(sourceChannelId, governanceToken);
        // Clone our ERC20 to the predicted token address.
        vm.cloneAccount(address(erc20), localGovernanceToken);
        vm.prank(beneficiary);
        vm.expectEmit();
        emit VestingManager.Staked(key, localGovernanceToken, tokenId, amount);
        vestingMgr.stake(
            key,
            sourceChannelId,
            type(uint64).max,
            salt,
            localGovernanceToken,
            Stake({
                tokenId: tokenId,
                stakedToken: localGovernanceToken,
                sender: abi.encodePacked(address(account)),
                beneficiary: abi.encodePacked(address(account)),
                validator: validator,
                amount: amount
            })
        );
    }

    function test_stake_onlyBeneficiary(
        address operator,
        bytes32 key,
        address beneficiary,
        // Use smaller than u64 to avoid overflow because of fuzzing. In practice the `start + cliff < max(u64)`
        uint32 start,
        uint16 cliff,
        uint32 duration,
        uint64 timestamp,
        uint256 amount,
        uint256 tokenId,
        bytes32 salt,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        address governanceToken,
        bytes calldata validator
    ) public {
        vm.assume(governanceToken != address(0));
        vm.assume(sourceChannelId != 0);
        vm.assume(destinationChannelId != 0);
        vm.assume(amount > 0);
        vm.assume(cliff < duration);
        vm.assume(timestamp < uint64(start) + uint64(cliff));
        vm.assume(operator != address(this) && operator != beneficiary);
        vm.assume(beneficiary != address(0));
        handler.setChannel(sourceChannelId, destinationChannelId);
        vestingMgr.update(key, beneficiary, start, cliff, duration);
        VestingAccount account = vestingMgr.vestingAccount(key);
        erc20.mint(address(account), amount);
        address localGovernanceToken = governanceToken;
        vm.expectRevert(VestingManager.VestingManager_OnlyBeneficiary.selector);
        vm.prank(operator);
        vestingMgr.stake(
            key,
            sourceChannelId,
            type(uint64).max,
            salt,
            localGovernanceToken,
            Stake({
                tokenId: tokenId,
                stakedToken: localGovernanceToken,
                sender: abi.encodePacked(address(account)),
                beneficiary: abi.encodePacked(address(account)),
                validator: validator,
                amount: amount
            })
        );
    }
}
