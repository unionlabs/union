pragma solidity ^0.8.27;

/**
 * @title [TokenBucket](https://en.wikipedia.org/wiki/Token_bucket) (ERC-7201 Storage Compliant)
 * @dev Abstract contract implementing the Token Bucket algorithm with namespaced storage.
 */
abstract contract TokenBucket {
    error ErrTokenBucketZeroCapacity();
    error ErrTokenBucketZeroRefillRate();
    error ErrTokenBucketRateLimitExceeded();

    bytes32 internal constant _TOKEN_BUCKT_STORAGE_SLOT = keccak256(
        abi.encode(uint256(keccak256("union.token-bucket.zkgm")) - 1)
    ) & ~bytes32(uint256(0xff));

    struct Bucket {
        // Maximum capacity
        uint256 capacity;
        // Available amount
        uint256 available;
        // Refill rate per second
        uint256 refillRate;
        // Last refill checkpoint
        uint256 lastRefill;
    }

    struct TokenBucketStorage {
        mapping(address => Bucket) buckets;
    }

    function _getTokenBucketStorage()
        internal
        pure
        returns (TokenBucketStorage storage $)
    {
        bytes32 slot = _TOKEN_BUCKT_STORAGE_SLOT;
        assembly {
            $.slot := slot
        }
    }

    function _rateLimit(address token, uint256 amount) internal {
        _refill(token);
        Bucket storage $ = _getTokenBucketStorage().buckets[token];
        if ($.available < amount) {
            revert ErrTokenBucketRateLimitExceeded();
        }
        $.available -= amount;
    }

    function _refill(
        address token
    ) internal {
        Bucket storage $ = _getTokenBucketStorage().buckets[token];
        if ($.available >= $.capacity) {
            $.lastRefill = block.timestamp;
            return;
        }
        uint256 elapsed = block.timestamp - $.lastRefill;
        uint256 toRefill = elapsed * $.refillRate;
        if (toRefill > 0) {
            $.available = _clampToCapacity($.capacity, $.available + toRefill);
            $.lastRefill = block.timestamp;
        }
    }

    function _setBucketConfig(
        address token,
        uint256 capacity,
        uint256 refillRate,
        bool reset
    ) internal {
        if (capacity == 0) {
            revert ErrTokenBucketZeroCapacity();
        }
        if (refillRate == 0) {
            revert ErrTokenBucketZeroRefillRate();
        }
        Bucket storage $ = _getTokenBucketStorage().buckets[token];
        $.capacity = capacity;
        $.refillRate = refillRate;
        if ($.lastRefill == 0 || reset) {
            $.available = capacity;
            $.lastRefill = block.timestamp;
        }
    }

    function getTokenBucket(
        address token
    ) external view returns (Bucket memory) {
        return _getTokenBucketStorage().buckets[token];
    }

    function _clampToCapacity(
        uint256 capacity,
        uint256 amount
    ) internal pure returns (uint256) {
        return capacity < amount ? capacity : amount;
    }
}
