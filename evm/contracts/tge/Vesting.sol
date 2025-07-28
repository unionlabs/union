pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/PausableUpgradeable.sol";

import "@openzeppelin/contracts/proxy/beacon/BeaconProxy.sol";
import "@openzeppelin/contracts/proxy/beacon/UpgradeableBeacon.sol";

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

import "solady/utils/CREATE3.sol";
import "solady/utils/LibBytes.sol";
import "solady/utils/LibCall.sol";

import "../internal/Versioned.sol";
import "../apps/ucs/03-zkgm/IZkgm.sol";
import "../apps/ucs/03-zkgm/Lib.sol";

contract VestingAccount is
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    PausableUpgradeable,
    Versioned
{
    using LibBytes for *;
    using LibCall for *;

    error VestingAccount_Unauthorized();
    error VestingAccount_StakeBeneficiaryMustBeSelf();
    error VestingAccount_StakeSenderMustBeSelf();
    error VestingAccount_UnstakeSenderMustBeSelf();
    error VestingAccount_WithdrawStakeSenderMustBeSelf();
    error VestingAccount_WithdrawStakeBeneficiaryMustBeSelf();

    bytes32 internal constant VESTING_ACCOUNT_STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("union.storage.tge.vestingAccount")) - 1)
    ) & ~bytes32(uint256(0xff));

    struct VestingAccountStorage {
        address vestingManager;
    }

    function _getVestingAccountStorage()
        private
        pure
        returns (VestingAccountStorage storage $)
    {
        bytes32 slot = VESTING_ACCOUNT_STORAGE_SLOT;
        assembly {
            $.slot := slot
        }
    }

    constructor() {
        _disableInitializers();
    }

    function initialize(
        address _authority,
        address _vestingManager
    ) public initializer {
        __AccessManaged_init(_authority);
        __UUPSUpgradeable_init();
        VestingAccountStorage storage $ = _getVestingAccountStorage();
        $.vestingManager = _vestingManager;
    }

    function transfer(
        address token,
        address beneficiary,
        uint256 amount
    ) public onlyVestingManager {
        SafeERC20.safeTransfer(IERC20(token), beneficiary, amount);
    }

    function stake(
        IZkgm zkgm,
        uint32 channelId,
        uint64 timeout,
        bytes32 salt,
        address governanceToken,
        Stake calldata stake
    ) public onlyVestingManager {
        if (!stake.beneficiary.eq(abi.encodePacked(address(this)))) {
            revert VestingAccount_StakeBeneficiaryMustBeSelf();
        }
        if (!stake.sender.eq(abi.encodePacked(address(this)))) {
            revert VestingAccount_StakeSenderMustBeSelf();
        }
        IERC20(governanceToken).approve(address(zkgm), stake.amount);
        zkgm.send(
            channelId,
            0,
            timeout,
            salt,
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_STAKE,
                operand: ZkgmLib.encodeStake(stake)
            })
        );
    }

    function unstake(
        IZkgm zkgm,
        uint32 channelId,
        uint64 timeout,
        bytes32 salt,
        Unstake calldata unstake
    ) public onlyVestingManager {
        if (!unstake.sender.eq(abi.encodePacked(address(this)))) {
            revert VestingAccount_UnstakeSenderMustBeSelf();
        }
        zkgm.predictStakeManagerAddress().approve(
            address(zkgm), unstake.tokenId
        );
        zkgm.send(
            channelId,
            0,
            timeout,
            salt,
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_UNSTAKE,
                operand: ZkgmLib.encodeUnstake(unstake)
            })
        );
    }

    function withdrawStake(
        IZkgm zkgm,
        uint32 channelId,
        uint64 timeout,
        bytes32 salt,
        WithdrawStake calldata withdrawStake
    ) public onlyVestingManager {
        if (!withdrawStake.beneficiary.eq(abi.encodePacked(address(this)))) {
            revert VestingAccount_WithdrawStakeBeneficiaryMustBeSelf();
        }
        if (!withdrawStake.sender.eq(abi.encodePacked(address(this)))) {
            revert VestingAccount_WithdrawStakeSenderMustBeSelf();
        }
        zkgm.predictStakeManagerAddress().approve(
            address(zkgm), withdrawStake.tokenId
        );
        zkgm.send(
            channelId,
            0,
            timeout,
            salt,
            Instruction({
                version: ZkgmLib.INSTR_VERSION_0,
                opcode: ZkgmLib.OP_WITHDRAW_STAKE,
                operand: ZkgmLib.encodeWithdrawStake(withdrawStake)
            })
        );
    }

    function execute(
        address target,
        uint256 value,
        bytes calldata payload
    ) public restricted {
        (bool success,, bytes memory result) =
            target.tryCall(value, gasleft(), type(uint16).max, payload);
        if (!success) {
            LibCall.bubbleUpRevert(result);
        }
    }

    modifier onlyVestingManager() {
        if (msg.sender != _getVestingAccountStorage().vestingManager) {
            revert VestingAccount_Unauthorized();
        }
        _;
    }

    function _authorizeUpgrade(
        address
    ) internal override restricted {}
}

contract VestingManager is
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    PausableUpgradeable,
    Versioned
{
    error VestingManager_ZeroAddress();
    error VestingManager_CliffExceedsDuration();
    error VestingManager_OnlyBeneficiary();

    event Released(bytes32 indexed key, address indexed token, uint256 amount);

    event Updated(
        bytes32 indexed key,
        address indexed beneficiary,
        uint64 cliff,
        uint64 start,
        uint64 duration
    );

    event Staked(
        bytes32 indexed key,
        address indexed governanceToken,
        uint256 indexed nftId,
        uint256 amount
    );

    event Unstaked(bytes32 indexed key, uint256 indexed nftId);

    event StakeWithdrawn(bytes32 indexed key, uint256 indexed nftId);

    bytes32 internal constant VESTING_MANAGER_STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("union.storage.tge.vestingManager")) - 1)
    ) & ~bytes32(uint256(0xff));

    struct VestingSchedule {
        address beneficiary;
        uint64 cliff;
        uint64 start;
        uint64 duration;
        mapping(address => uint256) released;
    }

    struct VestingManagerStorage {
        IZkgm zkgm;
        UpgradeableBeacon vestingAccountBeacon;
        mapping(bytes32 => VestingSchedule) schedules;
    }

    function _getVestingManagerStorage()
        private
        pure
        returns (VestingManagerStorage storage $)
    {
        bytes32 slot = VESTING_MANAGER_STORAGE_SLOT;
        assembly {
            $.slot := slot
        }
    }

    constructor() {
        _disableInitializers();
    }

    function initialize(
        address _authority,
        address _vestingAccountImpl,
        IZkgm _zkgm
    ) public initializer {
        __AccessManaged_init(_authority);
        __UUPSUpgradeable_init();
        VestingManagerStorage storage $ = _getVestingManagerStorage();
        $.zkgm = _zkgm;
        $.vestingAccountBeacon =
            new UpgradeableBeacon(_vestingAccountImpl, address(this));
    }

    function cliff(
        bytes32 key
    ) public view returns (uint64) {
        return _getVestingManagerStorage().schedules[key].cliff;
    }

    function start(
        bytes32 key
    ) public view returns (uint64) {
        return _getVestingManagerStorage().schedules[key].start;
    }

    function duration(
        bytes32 key
    ) public view returns (uint64) {
        return _getVestingManagerStorage().schedules[key].duration;
    }

    function end(
        bytes32 key
    ) public view returns (uint64) {
        return start(key) + duration(key);
    }

    function beneficiary(
        bytes32 key
    ) public view returns (address) {
        return _getVestingManagerStorage().schedules[key].beneficiary;
    }

    function released(
        bytes32 key,
        address token
    ) public view returns (uint256) {
        return _getVestingManagerStorage().schedules[key].released[token];
    }

    function releasable(
        bytes32 key,
        address token
    ) public view returns (uint256) {
        return vestedAmount(key, token, uint64(block.timestamp))
            - released(key, token);
    }

    function vestedAmount(
        bytes32 key,
        address token,
        uint64 timestamp
    ) public view returns (uint256) {
        VestingAccount vestingAccount = _vestingAccount(key);
        uint256 currentBalance =
            IERC20(token).balanceOf(address(vestingAccount));
        return _vestingSchedule(
            key, currentBalance + released(key, token), timestamp
        );
    }

    function _vestingSchedule(
        bytes32 key,
        uint256 totalAllocation,
        uint64 timestamp
    ) internal view returns (uint256) {
        if (timestamp < start(key) + cliff(key)) {
            return 0;
        } else if (timestamp >= end(key)) {
            return totalAllocation;
        } else {
            return (totalAllocation * (timestamp - start(key))) / duration(key);
        }
    }

    function _vestingAccount(
        bytes32 key
    ) internal view returns (VestingAccount) {
        return VestingAccount(CREATE3.predictDeterministicAddress(key));
    }

    function vestingAccount(
        bytes32 key
    ) public view returns (VestingAccount) {
        return _vestingAccount(key);
    }

    function release(bytes32 key, address token) public {
        uint256 amount = releasable(key, token);
        VestingSchedule storage $ = _getVestingManagerStorage().schedules[key];
        $.released[token] += amount;
        VestingAccount vestingAccount = _vestingAccount(key);
        vestingAccount.transfer(token, $.beneficiary, amount);
        emit Released(key, token, amount);
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

    function _deployVestingAccount(
        bytes32 key
    ) internal {
        if (!_isDeployed(address(_vestingAccount(key)))) {
            CREATE3.deployDeterministic(
                abi.encodePacked(
                    type(BeaconProxy).creationCode,
                    abi.encode(
                        _getVestingManagerStorage().vestingAccountBeacon,
                        abi.encodeCall(
                            VestingAccount.initialize,
                            (authority(), address(this))
                        )
                    )
                ),
                key
            );
        }
    }

    function update(
        bytes32 key,
        address beneficiary,
        uint64 start,
        uint64 cliff,
        uint64 duration
    ) public restricted {
        if (beneficiary == address(0)) {
            revert VestingManager_ZeroAddress();
        }
        if (cliff > duration) {
            revert VestingManager_CliffExceedsDuration();
        }
        VestingSchedule storage $ = _getVestingManagerStorage().schedules[key];
        $.beneficiary = beneficiary;
        $.cliff = cliff;
        $.start = start;
        $.duration = duration;
        _deployVestingAccount(key);
        emit Updated(key, beneficiary, start, cliff, duration);
    }

    function stake(
        bytes32 key,
        uint32 channelId,
        uint64 timeout,
        bytes32 salt,
        address governanceToken,
        Stake calldata stake
    ) public {
        VestingManagerStorage storage $ = _getVestingManagerStorage();
        VestingSchedule storage schedule = $.schedules[key];
        if (msg.sender != schedule.beneficiary) {
            revert VestingManager_OnlyBeneficiary();
        }
        _vestingAccount(key).stake(
            $.zkgm, channelId, timeout, salt, governanceToken, stake
        );
        emit Staked(key, governanceToken, stake.tokenId, stake.amount);
    }

    function unstake(
        bytes32 key,
        uint32 channelId,
        uint64 timeout,
        bytes32 salt,
        Unstake calldata unstake
    ) public {
        VestingManagerStorage storage $ = _getVestingManagerStorage();
        VestingSchedule storage schedule = $.schedules[key];
        if (msg.sender != schedule.beneficiary) {
            revert VestingManager_OnlyBeneficiary();
        }
        _vestingAccount(key).unstake($.zkgm, channelId, timeout, salt, unstake);
        emit Unstaked(key, unstake.tokenId);
    }

    function withdrawStake(
        bytes32 key,
        uint32 channelId,
        uint64 timeout,
        bytes32 salt,
        WithdrawStake calldata withdrawStake
    ) public {
        VestingManagerStorage storage $ = _getVestingManagerStorage();
        VestingSchedule storage schedule = $.schedules[key];
        if (msg.sender != schedule.beneficiary) {
            revert VestingManager_OnlyBeneficiary();
        }
        _vestingAccount(key).withdrawStake(
            $.zkgm, channelId, timeout, salt, withdrawStake
        );
        emit StakeWithdrawn(key, withdrawStake.tokenId);
    }

    function _authorizeUpgrade(
        address
    ) internal override restricted {}

    function upgradeVestingAccountBeacon(
        address newImplementation
    ) public restricted {
        _getVestingManagerStorage().vestingAccountBeacon.upgradeTo(
            newImplementation
        );
    }
}
