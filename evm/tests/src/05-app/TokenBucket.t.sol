pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "solady/utils/FixedPointMathLib.sol";

import "../../../contracts/apps/ucs/03-zkgm/Zkgm.sol";

contract TestTokenBucket is TokenBucket {
    function rateLimit(address token, uint256 amount) public {
        _rateLimit(token, amount);
    }

    function setConfig(
        address token,
        uint256 capacity,
        uint256 refillRate,
        bool reset
    ) public {
        _setBucketConfig(token, capacity, refillRate, reset);
    }
}

contract TokenBucketTests is Test {
    using FixedPointMathLib for *;

    TestTokenBucket tokenBucket;

    function setUp() public {
        tokenBucket = new TestTokenBucket();
    }

    function test_noConfig_rateLimitExceeded(
        address token,
        uint256 amount
    ) public {
        vm.assume(amount > 0);
        vm.expectRevert(TokenBucket.ErrTokenBucketRateLimitExceeded.selector);
        tokenBucket.rateLimit(token, amount);
    }

    function test_setConfig_zeroCapacity(
        address token,
        uint256 refillRate,
        bool reset
    ) public {
        vm.assume(refillRate > 0);
        vm.expectRevert(TokenBucket.ErrTokenBucketZeroCapacity.selector);
        tokenBucket.setConfig(token, 0, refillRate, reset);
    }

    function test_setConfig_zeroRefillRate(
        address token,
        uint256 capacity,
        bool reset
    ) public {
        vm.assume(capacity > 0);
        vm.expectRevert(TokenBucket.ErrTokenBucketZeroRefillRate.selector);
        tokenBucket.setConfig(token, capacity, 0, reset);
    }

    function test_setConfig_reset_refillAvailable(
        address token,
        uint256 capacity,
        uint256 refillRate
    ) public {
        vm.assume(capacity > 0);
        vm.assume(refillRate > 0);
        tokenBucket.setConfig(token, capacity, refillRate, false);
        tokenBucket.rateLimit(token, capacity);
        assertEq(tokenBucket.getBucket(token).available, 0);
        tokenBucket.setConfig(token, capacity, refillRate, true);
        assertEq(tokenBucket.getBucket(token).available, capacity);
    }

    function test_setConfig_emitEvent(
        address token,
        uint256 capacity,
        uint256 refillRate
    ) public {
        vm.assume(capacity > 0);
        vm.assume(refillRate > 0);
        vm.expectEmit();
        emit TokenBucket.TokenBucketUpdate(token, capacity, refillRate);
        tokenBucket.setConfig(token, capacity, refillRate, false);
    }

    function test_rateLimit_consumesAvailable(
        address token,
        uint256 capacity,
        uint256 refillRate,
        bool reset,
        uint256 amount
    ) public {
        vm.assume(amount <= capacity);
        vm.assume(capacity > 0);
        vm.assume(refillRate > 0);
        tokenBucket.setConfig(token, capacity, refillRate, reset);
        tokenBucket.rateLimit(token, amount);
        assertEq(tokenBucket.getBucket(token).available, capacity - amount);
    }

    function test_rateLimit_capacityExceeded(
        address token,
        uint128 capacity,
        uint256 refillRate,
        bool reset,
        uint128 exceedAmount
    ) public {
        vm.assume(exceedAmount > 0);
        vm.assume(capacity > 0);
        vm.assume(refillRate > 0);
        tokenBucket.setConfig(token, capacity, refillRate, reset);
        vm.expectRevert(TokenBucket.ErrTokenBucketRateLimitExceeded.selector);
        tokenBucket.rateLimit(token, uint256(capacity) + exceedAmount);
    }

    function test_rateLimit_refillsAvailable(
        address token,
        uint256 capacity,
        uint256 refillRate,
        bool reset,
        uint64 startingTimestamp,
        uint64 endingTimestamp
    ) public {
        vm.assume(startingTimestamp < endingTimestamp);
        vm.assume(capacity > 0);
        vm.assume(refillRate > 0);
        vm.warp(startingTimestamp);
        tokenBucket.setConfig(token, capacity, refillRate, reset);
        tokenBucket.rateLimit(token, capacity - 1);
        vm.warp(endingTimestamp);
        uint256 availableBeforeRefill = tokenBucket.getBucket(token).available;
        tokenBucket.rateLimit(token, 1);
        uint256 delta = endingTimestamp - startingTimestamp;
        uint256 availableAfterRefill =
            availableBeforeRefill.saturatingAdd(refillRate.saturatingMul(delta));
        assertEq(
            tokenBucket.getBucket(token).available,
            (availableAfterRefill > capacity ? capacity : availableAfterRefill)
                - 1
        );
    }

    function test_refill_neverExceedCapacity(
        address token,
        uint256 capacity,
        uint256 refillRate,
        bool reset,
        uint64 startingTimestamp,
        uint64 endingTimestamp
    ) public {
        vm.assume(startingTimestamp < endingTimestamp);
        vm.assume(capacity > 0);
        vm.assume(refillRate > 0);
        vm.warp(startingTimestamp);
        tokenBucket.setConfig(token, capacity, refillRate, reset);
        tokenBucket.rateLimit(token, capacity - 1);
        vm.warp(endingTimestamp);
        tokenBucket.rateLimit(token, 1);
        assertLe(tokenBucket.getBucket(token).available, capacity);
    }
}
