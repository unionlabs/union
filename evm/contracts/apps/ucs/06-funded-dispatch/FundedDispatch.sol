pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/PausableUpgradeable.sol";

import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/utils/Address.sol";

import "solady/utils/CREATE3.sol";
import "solady/utils/LibCall.sol";
import "solady/utils/EfficientHashLib.sol";

import "../../../core/25-handler/IBCHandler.sol";
import "../03-zkgm/IZkgmable.sol";
import "../03-zkgm/Lib.sol";

struct FundedDispatchFund {
    bytes token;
    uint256 amount;
}

struct FundedDispatchParameters {
    uint256 flags;
    FundedDispatchFund[] funds;
    bytes contractAddress;
    bytes contractCalldata;
    bytes beneficiary;
}

library FundedDispatchLib {
    error FundedDispatch_OnlyZkgm();
    error FundedDispatch_NoMarketMaker();

    uint256 constant FLAG_DEFAULT = 0;
    uint256 constant FLAG_ALLOW_FAILURE = 1;
    uint256 constant FLAG_ALLOW_MARKET_MAKER = 2;

    function hasFlag(
        uint256 value,
        uint256 flag
    ) internal pure returns (bool) {
        return (value & flag) == flag;
    }

    function encode(
        FundedDispatchParameters memory params
    ) internal pure returns (bytes memory) {
        return abi.encode(
            params.flags,
            params.funds,
            params.contractAddress,
            params.contractCalldata,
            params.beneficiary
        );
    }

    function decode(
        bytes calldata stream
    ) internal pure returns (FundedDispatchParameters calldata) {
        FundedDispatchParameters calldata params;
        assembly {
            params := stream.offset
        }
        return params;
    }
}

contract UCS06FundedDispatch is
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    PausableUpgradeable,
    IZkgmable,
    Versioned
{
    using Address for *;
    using SafeERC20 for *;
    using FundedDispatchLib for *;

    constructor() {
        _disableInitializers();
    }

    function initialize(
        address _authority
    ) public initializer {
        __UUPSUpgradeable_init();
        __AccessManaged_init(_authority);
        __Pausable_init();
    }

    // Execute a contract call with funds. Allowance is set and reset
    // before/after calling the contract. In case of failure, either the user
    // allows for a local beneficiary to receive the funds or the transaction
    // will revert.
    //
    // /!\ This contract must be atomically funded then called. Any other
    // scenario would allow a third party to drain it.
    //
    // /!\ The sent funds must exactly match the target contract expectations. Any
    // extra token can be drained.
    function _execute(bytes calldata message, bool intent) internal {
        FundedDispatchParameters calldata params =
            FundedDispatchLib.decode(message);
        if (
            intent
                && !params.flags.hasFlag(FundedDispatchLib.FLAG_ALLOW_MARKET_MAKER)
        ) {
            revert FundedDispatchLib.FundedDispatch_NoMarketMaker();
        }
        address contractAddress = address(bytes20(params.contractAddress));
        uint256 value;
        for (uint256 i = 0; i < params.funds.length; i++) {
            FundedDispatchFund calldata fund = params.funds[i];
            address token = address(bytes20(fund.token));
            if (token == ZkgmLib.NATIVE_ETH_ERC_7528_ADDRESS) {
                value = fund.amount;
            } else {
                IERC20(token).approve(contractAddress, fund.amount);
            }
        }
        (bool success, bytes memory returnData) =
            contractAddress.call{value: value}(params.contractCalldata);
        bool allowFailure =
            params.flags.hasFlag(FundedDispatchLib.FLAG_ALLOW_FAILURE);
        // Bubble up revert will cancel the entire pipeline.
        if (!success && !allowFailure) {
            LibCall.bubbleUpRevert(returnData);
        }
        address payable beneficiary =
            payable(address(bytes20(params.beneficiary)));
        for (uint256 i = 0; i < params.funds.length; i++) {
            FundedDispatchFund calldata fund = params.funds[i];
            address token = address(bytes20(fund.token));
            if (token == ZkgmLib.NATIVE_ETH_ERC_7528_ADDRESS) {
                if (!success && allowFailure) {
                    beneficiary.sendValue(fund.amount);
                }
            } else {
                IERC20(token).approve(contractAddress, 0);
                // Optionally forward funds if the user allowed failure.
                if (!success && allowFailure) {
                    IERC20(token).safeTransfer(beneficiary, fund.amount);
                }
            }
        }
    }

    function execute(bytes calldata message, bool intent) public {
        _execute(message, intent);
    }

    function onZkgm(
        address,
        uint256,
        uint32,
        uint32,
        bytes calldata,
        bytes calldata message,
        address,
        bytes calldata
    ) public {
        _execute(message, false);
    }

    function onIntentZkgm(
        address,
        uint256,
        uint32,
        uint32,
        bytes calldata,
        bytes calldata message,
        address,
        bytes calldata
    ) public {
        _execute(message, true);
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override restricted {}

    function pause() public restricted {
        _pause();
    }

    function unpause() public restricted {
        _unpause();
    }
}
