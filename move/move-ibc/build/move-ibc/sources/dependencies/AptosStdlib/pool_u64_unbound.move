///
/// Simple module for tracking and calculating shares of a pool of coins. The shares are worth more as the total coins in
/// the pool increases. New shareholder can buy more shares or redeem their existing shares.
///
/// Example flow:
/// 1. Pool start outs empty.
/// 2. Shareholder A buys in with 1000 coins. A will receive 1000 shares in the pool. Pool now has 1000 total coins and
/// 1000 total shares.
/// 3. Pool appreciates in value from rewards and now has 2000 coins. A's 1000 shares are now worth 2000 coins.
/// 4. Shareholder B now buys in with 1000 coins. Since before the buy in, each existing share is worth 2 coins, B will
/// receive 500 shares in exchange for 1000 coins. Pool now has 1500 shares and 3000 coins.
/// 5. Pool appreciates in value from rewards and now has 6000 coins.
/// 6. A redeems 500 shares. Each share is worth 6000 / 1500 = 4. A receives 2000 coins. Pool has 4000 coins and 1000
/// shares left.
///
module aptos_std::pool_u64_unbound {
    use aptos_std::table_with_length::{Self as table, TableWithLength as Table};
    use std::error;

    /// Shareholder not present in pool.
    const ESHAREHOLDER_NOT_FOUND: u64 = 1;
    /// There are too many shareholders in the pool.
    const ETOO_MANY_SHAREHOLDERS: u64 = 2;
    /// Cannot destroy non-empty pool.
    const EPOOL_IS_NOT_EMPTY: u64 = 3;
    /// Cannot redeem more shares than the shareholder has in the pool.
    const EINSUFFICIENT_SHARES: u64 = 4;
    /// Shareholder cannot have more than u64.max shares.
    const ESHAREHOLDER_SHARES_OVERFLOW: u64 = 5;
    /// Pool's total coins cannot exceed u64.max.
    const EPOOL_TOTAL_COINS_OVERFLOW: u64 = 6;
    /// Pool's total shares cannot exceed u64.max.
    const EPOOL_TOTAL_SHARES_OVERFLOW: u64 = 7;

    const MAX_U64: u64 = 18446744073709551615;

    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    struct Pool has store {
        total_coins: u64,
        total_shares: u128,
        shares: Table<address, u128>,
        // Default to 1. This can be used to minimize rounding errors when computing shares and coins amount.
        // However, users need to make sure the coins amount don't overflow when multiplied by the scaling factor.
        scaling_factor: u64,
    }

    /// Create a new pool.
    public fun new(): Pool {
        // Default to a scaling factor of 1 (effectively no scaling).
        create_with_scaling_factor(1)
    }

    #[deprecated]
    /// Deprecated. Use `new` instead.
    /// Create a new pool.
    public fun create(): Pool {
        new()
    }

    /// Create a new pool with custom `scaling_factor`.
    public fun create_with_scaling_factor(scaling_factor: u64): Pool {
        Pool {
            total_coins: 0,
            total_shares: 0,
            shares: table::new<address, u128>(),
            scaling_factor,
        }
    }

    /// Destroy an empty pool. This will fail if the pool has any balance of coins.
    public fun destroy_empty(pool: Pool) {
        assert!(pool.total_coins == 0, error::invalid_state(EPOOL_IS_NOT_EMPTY));
        let Pool {
            total_coins: _,
            total_shares: _,
            shares,
            scaling_factor: _,
        } = pool;
        table::destroy_empty<address, u128>(shares);
    }

    /// Return `pool`'s total balance of coins.
    public fun total_coins(pool: &Pool): u64 {
        pool.total_coins
    }

    /// Return the total number of shares across all shareholders in `pool`.
    public fun total_shares(pool: &Pool): u128 {
        pool.total_shares
    }

    /// Return true if `shareholder` is in `pool`.
    public fun contains(pool: &Pool, shareholder: address): bool {
        table::contains(&pool.shares, shareholder)
    }

    /// Return the number of shares of `stakeholder` in `pool`.
    public fun shares(pool: &Pool, shareholder: address): u128 {
        if (contains(pool, shareholder)) {
            *table::borrow(&pool.shares, shareholder)
        } else {
            0
        }
    }

    /// Return the balance in coins of `shareholder` in `pool.`
    public fun balance(pool: &Pool, shareholder: address): u64 {
        let num_shares = shares(pool, shareholder);
        shares_to_amount(pool, num_shares)
    }

    /// Return the number of shareholders in `pool`.
    public fun shareholders_count(pool: &Pool): u64 {
        table::length(&pool.shares)
    }

    /// Update `pool`'s total balance of coins.
    public fun update_total_coins(pool: &mut Pool, new_total_coins: u64) {
        pool.total_coins = new_total_coins;
    }

    /// Allow an existing or new shareholder to add their coins to the pool in exchange for new shares.
    public fun buy_in(pool: &mut Pool, shareholder: address, coins_amount: u64): u128 {
        if (coins_amount == 0) return 0;

        let new_shares = amount_to_shares(pool, coins_amount);
        assert!(MAX_U64 - pool.total_coins >= coins_amount, error::invalid_argument(EPOOL_TOTAL_COINS_OVERFLOW));
        assert!(MAX_U128 - pool.total_shares >= new_shares, error::invalid_argument(EPOOL_TOTAL_SHARES_OVERFLOW));

        pool.total_coins = pool.total_coins + coins_amount;
        pool.total_shares = pool.total_shares + new_shares;
        add_shares(pool, shareholder, new_shares);
        new_shares
    }

    /// Add the number of shares directly for `shareholder` in `pool`.
    /// This would dilute other shareholders if the pool's balance of coins didn't change.
    fun add_shares(pool: &mut Pool, shareholder: address, new_shares: u128): u128 {
        if (contains(pool, shareholder)) {
            let existing_shares = table::borrow_mut(&mut pool.shares, shareholder);
            let current_shares = *existing_shares;
            assert!(MAX_U128 - current_shares >= new_shares, error::invalid_argument(ESHAREHOLDER_SHARES_OVERFLOW));

            *existing_shares = current_shares + new_shares;
            *existing_shares
        } else if (new_shares > 0) {
            table::add(&mut pool.shares, shareholder, new_shares);
            new_shares
        } else {
            new_shares
        }
    }

    /// Allow `shareholder` to redeem their shares in `pool` for coins.
    public fun redeem_shares(pool: &mut Pool, shareholder: address, shares_to_redeem: u128): u64 {
        assert!(contains(pool, shareholder), error::invalid_argument(ESHAREHOLDER_NOT_FOUND));
        assert!(shares(pool, shareholder) >= shares_to_redeem, error::invalid_argument(EINSUFFICIENT_SHARES));

        if (shares_to_redeem == 0) return 0;

        let redeemed_coins = shares_to_amount(pool, shares_to_redeem);
        pool.total_coins = pool.total_coins - redeemed_coins;
        pool.total_shares = pool.total_shares - shares_to_redeem;
        deduct_shares(pool, shareholder, shares_to_redeem);

        redeemed_coins
    }

    /// Transfer shares from `shareholder_1` to `shareholder_2`.
    public fun transfer_shares(
        pool: &mut Pool,
        shareholder_1: address,
        shareholder_2: address,
        shares_to_transfer: u128,
    ) {
        assert!(contains(pool, shareholder_1), error::invalid_argument(ESHAREHOLDER_NOT_FOUND));
        assert!(shares(pool, shareholder_1) >= shares_to_transfer, error::invalid_argument(EINSUFFICIENT_SHARES));
        if (shares_to_transfer == 0) return;

        deduct_shares(pool, shareholder_1, shares_to_transfer);
        add_shares(pool, shareholder_2, shares_to_transfer);
    }

    /// Directly deduct `shareholder`'s number of shares in `pool` and return the number of remaining shares.
    fun deduct_shares(pool: &mut Pool, shareholder: address, num_shares: u128): u128 {
        assert!(contains(pool, shareholder), error::invalid_argument(ESHAREHOLDER_NOT_FOUND));
        assert!(shares(pool, shareholder) >= num_shares, error::invalid_argument(EINSUFFICIENT_SHARES));

        let existing_shares = table::borrow_mut(&mut pool.shares, shareholder);
        *existing_shares = *existing_shares - num_shares;

        // Remove the shareholder completely if they have no shares left.
        let remaining_shares = *existing_shares;
        if (remaining_shares == 0) {
            table::remove(&mut pool.shares, shareholder);
        };

        remaining_shares
    }

    /// Return the number of new shares `coins_amount` can buy in `pool`.
    /// `amount` needs to big enough to avoid rounding number.
    public fun amount_to_shares(pool: &Pool, coins_amount: u64): u128 {
        amount_to_shares_with_total_coins(pool, coins_amount, pool.total_coins)
    }

    /// Return the number of new shares `coins_amount` can buy in `pool` with a custom total coins number.
    /// `amount` needs to big enough to avoid rounding number.
    public fun amount_to_shares_with_total_coins(pool: &Pool, coins_amount: u64, total_coins: u64): u128 {
        // No shares yet so amount is worth the same number of shares.
        if (pool.total_coins == 0 || pool.total_shares == 0) {
            // Multiply by scaling factor to minimize rounding errors during internal calculations for buy ins/redeems.
            // This can overflow but scaling factor is expected to be chosen carefully so this would not overflow.
            to_u128(coins_amount) * to_u128(pool.scaling_factor)
        } else {
            // Shares price = total_coins / total existing shares.
            // New number of shares = new_amount / shares_price = new_amount * existing_shares / total_amount.
            // We rearrange the calc and do multiplication first to avoid rounding errors.
            multiply_then_divide(pool, to_u128(coins_amount), pool.total_shares, to_u128(total_coins))
        }
    }

    /// Return the number of coins `shares` are worth in `pool`.
    /// `shares` needs to big enough to avoid rounding number.
    public fun shares_to_amount(pool: &Pool, shares: u128): u64 {
        shares_to_amount_with_total_coins(pool, shares, pool.total_coins)
    }

    /// Return the number of coins `shares` are worth in `pool` with a custom total coins number.
    /// `shares` needs to big enough to avoid rounding number.
    public fun shares_to_amount_with_total_coins(pool: &Pool, shares: u128, total_coins: u64): u64 {
        // No shares or coins yet so shares are worthless.
        if (pool.total_coins == 0 || pool.total_shares == 0) {
            0
        } else {
            // Shares price = total_coins / total existing shares.
            // Shares worth = shares * shares price = shares * total_coins / total existing shares.
            // We rearrange the calc and do multiplication first to avoid rounding errors.
            (multiply_then_divide(pool, shares, to_u128(total_coins), pool.total_shares) as u64)
        }
    }

    /// Return the number of coins `shares` are worth in `pool` with custom total coins and shares numbers.
    public fun shares_to_amount_with_total_stats(
        pool: &Pool,
        shares: u128,
        total_coins: u64,
        total_shares: u128,
    ): u64 {
        if (pool.total_coins == 0 || total_shares == 0) {
            0
        } else {
            (multiply_then_divide(pool, shares, to_u128(total_coins), total_shares) as u64)
        }
    }

    public fun multiply_then_divide(_pool: &Pool, x: u128, y: u128, z: u128): u128 {
        let result = (to_u256(x) * to_u256(y)) / to_u256(z);
        (result as u128)
    }

    fun to_u128(num: u64): u128 {
        (num as u128)
    }

    fun to_u256(num: u128): u256 {
        (num as u256)
    }
}
