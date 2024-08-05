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
module aptos_std::pool_u64 {
    use aptos_std::simple_map::{Self, SimpleMap};
    use std::error;
    use std::vector;

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

    struct Pool has store {
        shareholders_limit: u64,
        total_coins: u64,
        total_shares: u64,
        shares: SimpleMap<address, u64>,
        shareholders: vector<address>,
        // Default to 1. This can be used to minimize rounding errors when computing shares and coins amount.
        // However, users need to make sure the coins amount don't overflow when multiplied by the scaling factor.
        scaling_factor: u64,
    }

    /// Create a new pool.
    public fun new(shareholders_limit: u64): Pool {
        // Default to a scaling factor of 1 (effectively no scaling).
        create_with_scaling_factor(shareholders_limit, 1)
    }

    #[deprecated]
    /// Deprecated. Use `new` instead.
    /// Create a new pool.
    public fun create(shareholders_limit: u64): Pool {
        new(shareholders_limit)
    }

    /// Create a new pool with custom `scaling_factor`.
    public fun create_with_scaling_factor(shareholders_limit: u64, scaling_factor: u64): Pool {
        Pool {
            shareholders_limit,
            total_coins: 0,
            total_shares: 0,
            shares: simple_map::create<address, u64>(),
            shareholders: vector::empty<address>(),
            scaling_factor,
        }
    }

    /// Destroy an empty pool. This will fail if the pool has any balance of coins.
    public fun destroy_empty(pool: Pool) {
        assert!(pool.total_coins == 0, error::invalid_state(EPOOL_IS_NOT_EMPTY));
        let Pool {
            shareholders_limit: _,
            total_coins: _,
            total_shares: _,
            shares: _,
            shareholders: _,
            scaling_factor: _,
        } = pool;
    }

    /// Return `pool`'s total balance of coins.
    public fun total_coins(pool: &Pool): u64 {
        pool.total_coins
    }

    /// Return the total number of shares across all shareholders in `pool`.
    public fun total_shares(pool: &Pool): u64 {
        pool.total_shares
    }

    /// Return true if `shareholder` is in `pool`.
    public fun contains(pool: &Pool, shareholder: address): bool {
        simple_map::contains_key(&pool.shares, &shareholder)
    }

    /// Return the number of shares of `stakeholder` in `pool`.
    public fun shares(pool: &Pool, shareholder: address): u64 {
        if (contains(pool, shareholder)) {
            *simple_map::borrow(&pool.shares, &shareholder)
        } else {
            0
        }
    }

    /// Return the balance in coins of `shareholder` in `pool.`
    public fun balance(pool: &Pool, shareholder: address): u64 {
        let num_shares = shares(pool, shareholder);
        shares_to_amount(pool, num_shares)
    }

    /// Return the list of shareholders in `pool`.
    public fun shareholders(pool: &Pool): vector<address> {
        pool.shareholders
    }

    /// Return the number of shareholders in `pool`.
    public fun shareholders_count(pool: &Pool): u64 {
        vector::length(&pool.shareholders)
    }

    /// Update `pool`'s total balance of coins.
    public fun update_total_coins(pool: &mut Pool, new_total_coins: u64) {
        pool.total_coins = new_total_coins;
    }

    /// Allow an existing or new shareholder to add their coins to the pool in exchange for new shares.
    public fun buy_in(pool: &mut Pool, shareholder: address, coins_amount: u64): u64 {
        if (coins_amount == 0) return 0;

        let new_shares = amount_to_shares(pool, coins_amount);
        assert!(MAX_U64 - pool.total_coins >= coins_amount, error::invalid_argument(EPOOL_TOTAL_COINS_OVERFLOW));
        assert!(MAX_U64 - pool.total_shares >= new_shares, error::invalid_argument(EPOOL_TOTAL_COINS_OVERFLOW));

        pool.total_coins = pool.total_coins + coins_amount;
        pool.total_shares = pool.total_shares + new_shares;
        add_shares(pool, shareholder, new_shares);
        new_shares
    }

    /// Add the number of shares directly for `shareholder` in `pool`.
    /// This would dilute other shareholders if the pool's balance of coins didn't change.
    fun add_shares(pool: &mut Pool, shareholder: address, new_shares: u64): u64 {
        if (contains(pool, shareholder)) {
            let existing_shares = simple_map::borrow_mut(&mut pool.shares, &shareholder);
            let current_shares = *existing_shares;
            assert!(MAX_U64 - current_shares >= new_shares, error::invalid_argument(ESHAREHOLDER_SHARES_OVERFLOW));

            *existing_shares = current_shares + new_shares;
            *existing_shares
        } else if (new_shares > 0) {
            assert!(
                vector::length(&pool.shareholders) < pool.shareholders_limit,
                error::invalid_state(ETOO_MANY_SHAREHOLDERS),
            );

            vector::push_back(&mut pool.shareholders, shareholder);
            simple_map::add(&mut pool.shares, shareholder, new_shares);
            new_shares
        } else {
            new_shares
        }
    }

    /// Allow `shareholder` to redeem their shares in `pool` for coins.
    public fun redeem_shares(pool: &mut Pool, shareholder: address, shares_to_redeem: u64): u64 {
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
        shares_to_transfer: u64,
    ) {
        assert!(contains(pool, shareholder_1), error::invalid_argument(ESHAREHOLDER_NOT_FOUND));
        assert!(shares(pool, shareholder_1) >= shares_to_transfer, error::invalid_argument(EINSUFFICIENT_SHARES));
        if (shares_to_transfer == 0) return;

        deduct_shares(pool, shareholder_1, shares_to_transfer);
        add_shares(pool, shareholder_2, shares_to_transfer);
    }

    /// Directly deduct `shareholder`'s number of shares in `pool` and return the number of remaining shares.
    fun deduct_shares(pool: &mut Pool, shareholder: address, num_shares: u64): u64 {
        assert!(contains(pool, shareholder), error::invalid_argument(ESHAREHOLDER_NOT_FOUND));
        assert!(shares(pool, shareholder) >= num_shares, error::invalid_argument(EINSUFFICIENT_SHARES));

        let existing_shares = simple_map::borrow_mut(&mut pool.shares, &shareholder);
        *existing_shares = *existing_shares - num_shares;

        // Remove the shareholder completely if they have no shares left.
        let remaining_shares = *existing_shares;
        if (remaining_shares == 0) {
            let (_, shareholder_index) = vector::index_of(&pool.shareholders, &shareholder);
            vector::remove(&mut pool.shareholders, shareholder_index);
            simple_map::remove(&mut pool.shares, &shareholder);
        };

        remaining_shares
    }

    /// Return the number of new shares `coins_amount` can buy in `pool`.
    /// `amount` needs to big enough to avoid rounding number.
    public fun amount_to_shares(pool: &Pool, coins_amount: u64): u64 {
        amount_to_shares_with_total_coins(pool, coins_amount, pool.total_coins)
    }

    /// Return the number of new shares `coins_amount` can buy in `pool` with a custom total coins number.
    /// `amount` needs to big enough to avoid rounding number.
    public fun amount_to_shares_with_total_coins(pool: &Pool, coins_amount: u64, total_coins: u64): u64 {
        // No shares yet so amount is worth the same number of shares.
        if (pool.total_coins == 0 || pool.total_shares == 0) {
            // Multiply by scaling factor to minimize rounding errors during internal calculations for buy ins/redeems.
            // This can overflow but scaling factor is expected to be chosen carefully so this would not overflow.
            coins_amount * pool.scaling_factor
        } else {
            // Shares price = total_coins / total existing shares.
            // New number of shares = new_amount / shares_price = new_amount * existing_shares / total_amount.
            // We rearrange the calc and do multiplication first to avoid rounding errors.
            multiply_then_divide(pool, coins_amount, pool.total_shares, total_coins)
        }
    }

    /// Return the number of coins `shares` are worth in `pool`.
    /// `shares` needs to big enough to avoid rounding number.
    public fun shares_to_amount(pool: &Pool, shares: u64): u64 {
        shares_to_amount_with_total_coins(pool, shares, pool.total_coins)
    }

    /// Return the number of coins `shares` are worth in `pool` with a custom total coins number.
    /// `shares` needs to big enough to avoid rounding number.
    public fun shares_to_amount_with_total_coins(pool: &Pool, shares: u64, total_coins: u64): u64 {
        // No shares or coins yet so shares are worthless.
        if (pool.total_coins == 0 || pool.total_shares == 0) {
            0
        } else {
            // Shares price = total_coins / total existing shares.
            // Shares worth = shares * shares price = shares * total_coins / total existing shares.
            // We rearrange the calc and do multiplication first to avoid rounding errors.
            multiply_then_divide(pool, shares, total_coins, pool.total_shares)
        }
    }

    public fun multiply_then_divide(_pool: &Pool, x: u64, y: u64, z: u64): u64 {
        let result = (to_u128(x) * to_u128(y)) / to_u128(z);
        (result as u64)
    }

    fun to_u128(num: u64): u128 {
        (num as u128)
    }

    #[test_only]
    public fun destroy_pool(pool: Pool) {
        let Pool {
            shareholders_limit: _,
            total_coins: _,
            total_shares: _,
            shares: _,
            shareholders: _,
            scaling_factor: _,
        } = pool;
    }

    #[test]
    public entry fun test_buy_in_and_redeem() {
        let pool = new(5);

        // Shareholders 1 and 2 buy in first.
        buy_in(&mut pool, @1, 1000);
        buy_in(&mut pool, @2, 2000);
        assert!(shareholders_count(&pool) == 2, 0);
        assert!(total_coins(&pool) == 3000, 1);
        assert!(total_shares(&pool) == 3000, 2);
        assert!(shares(&pool, @1) == 1000, 3);
        assert!(shares(&pool, @2) == 2000, 4);
        assert!(balance(&pool, @1) == 1000, 5);
        assert!(balance(&pool, @2) == 2000, 6);

        // Pool increases in value.
        update_total_coins(&mut pool, 5000);
        assert!(shares(&pool, @1) == 1000, 7);
        assert!(shares(&pool, @2) == 2000, 8);
        let expected_balance_1 = 1000 * 5000 / 3000;
        assert!(balance(&pool, @1) == expected_balance_1, 9);
        let expected_balance_2 = 2000 * 5000 / 3000;
        assert!(balance(&pool, @2) == expected_balance_2, 10);

        // Shareholder 3 buys in into the 5000-coin pool with 1000 coins. There are 3000 existing shares.
        let expected_shares = 1000 * 3000 / 5000;
        buy_in(&mut pool, @3, 1000);
        assert!(shares(&pool, @3) == expected_shares, 11);
        assert!(balance(&pool, @3) == 1000, 12);

        // Pool increases more in value.
        update_total_coins(&mut pool, 8000);

        // Shareholders 1 and 2 redeem.
        let all_shares = 3000 + expected_shares;
        assert!(total_shares(&pool) == all_shares, 13);
        let expected_value_per_500_shares = 500 * 8000 / all_shares;
        assert!(redeem_shares(&mut pool, @1, 500) == expected_value_per_500_shares, 14);
        assert!(redeem_shares(&mut pool, @1, 500) == expected_value_per_500_shares, 15);
        assert!(redeem_shares(&mut pool, @2, 2000) == expected_value_per_500_shares * 4, 16);

        // Due to a very small rounding error of 1, shareholder 3 actually has 1 more coin.
        let shareholder_3_balance = expected_value_per_500_shares * 6 / 5 + 1;
        assert!(balance(&pool, @3) == shareholder_3_balance, 17);
        assert!(total_coins(&pool) == shareholder_3_balance, 18);
        assert!(shareholders_count(&pool) == 1, 19);
        let num_shares_3 = shares(&pool, @3);
        assert!(redeem_shares(&mut pool, @3, num_shares_3) == shareholder_3_balance, 20);

        // Nothing left.
        assert!(shareholders_count(&pool) == 0, 21);
        destroy_empty(pool);
    }

    #[test]
    #[expected_failure(abort_code = 196611, location = Self)]
    public entry fun test_destroy_empty_should_fail_if_not_empty() {
        let pool = new(1);
        update_total_coins(&mut pool, 100);
        destroy_empty(pool);
    }

    #[test]
    public entry fun test_buy_in_and_redeem_large_numbers() {
        let pool = new(2);
        let half_max_u64 = MAX_U64 / 2;
        let shares_1 = buy_in(&mut pool, @1, half_max_u64);
        assert!(shares_1 == half_max_u64, 0);
        let shares_2 = buy_in(&mut pool, @2, half_max_u64 + 1);
        assert!(shares_2 == half_max_u64 + 1, 1);
        assert!(total_shares(&pool) == MAX_U64, 2);
        assert!(total_coins(&pool) == MAX_U64, 3);
        assert!(redeem_shares(&mut pool, @1, shares_1) == half_max_u64, 4);
        assert!(redeem_shares(&mut pool, @2, shares_2) == half_max_u64 + 1, 5);
        destroy_empty(pool);
    }

    #[test]
    public entry fun test_buy_in_and_redeem_large_numbers_with_scaling_factor() {
        let scaling_factor = 100;
        let pool = create_with_scaling_factor(2, scaling_factor);
        let coins_amount = MAX_U64 / 100;
        let shares = buy_in(&mut pool, @1, coins_amount);
        assert!(total_shares(&pool) == coins_amount * scaling_factor, 0);
        assert!(total_coins(&pool) == coins_amount, 1);
        assert!(redeem_shares(&mut pool, @1, shares) == coins_amount, 2);
        destroy_empty(pool);
    }

    #[test]
    public entry fun test_buy_in_zero_amount() {
        let pool = new(2);
        buy_in(&mut pool, @1, 100);
        assert!(buy_in(&mut pool, @2, 0) == 0, 0);
        assert!(total_shares(&pool) == shares(&pool, @1), 1);
        assert!(shareholders_count(&pool) == 1, 2);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_buy_in_with_small_coins_amount() {
        let pool = new(2);
        // Shareholder 1 buys in with 1e17 coins.
        buy_in(&mut pool, @1, 100000000000000000);
        // Shareholder 2 buys in with a very small amount.
        assert!(buy_in(&mut pool, @2, 1) == 1, 0);
        // Pool's total coins increases by 20%. Shareholder 2 shouldn't see any actual balance increase as it gets
        // rounded down.
        let total_coins = total_coins(&pool);
        update_total_coins(&mut pool, total_coins * 6 / 5);
        // Minus 1 due to rounding error.
        assert!(balance(&pool, @1) == 100000000000000000 * 6 / 5 - 1, 1);
        assert!(balance(&pool, @2) == 1, 2);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_add_zero_shares_should_not_add_shareholder() {
        let pool = new(1);
        update_total_coins(&mut pool, 1000);
        assert!(add_shares(&mut pool, @1, 0) == 0, 0);
        assert!(shareholders_count(&pool) == 0, 1);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_add_zero_shares_returns_existing_number_of_shares() {
        let pool = new(1);
        update_total_coins(&mut pool, 1000);
        add_shares(&mut pool, @1, 1);
        assert!(shares(&pool, @1) == add_shares(&mut pool, @1, 0), 0);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_add_shares_existing_shareholder() {
        let pool = new(1);
        update_total_coins(&mut pool, 1000);
        add_shares(&mut pool, @1, 1);
        add_shares(&mut pool, @1, 2);
        assert!(shares(&mut pool, @1) == 3, 0);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_add_shares_new_shareholder() {
        let pool = new(2);
        update_total_coins(&mut pool, 1000);
        add_shares(&mut pool, @1, 1);
        add_shares(&mut pool, @2, 2);
        assert!(shares(&mut pool, @1) == 1, 0);
        assert!(shares(&mut pool, @2) == 2, 1);
        destroy_pool(pool);
    }

    #[test]
    #[expected_failure(abort_code = 196610, location = Self)]
    public entry fun test_add_shares_should_enforce_shareholder_limit() {
        let pool = new(2);
        add_shares(&mut pool, @1, 1);
        add_shares(&mut pool, @2, 2);
        add_shares(&mut pool, @3, 2);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_add_shares_should_work_after_reducing_shareholders_below_limit() {
        let pool = new(3);
        add_shares(&mut pool, @1, 1);
        add_shares(&mut pool, @2, 2);
        deduct_shares(&mut pool, @2, 2);
        add_shares(&mut pool, @3, 3);
        assert!(shares(&pool, @3) == 3, 0);
        destroy_pool(pool);
    }

    #[test]
    #[expected_failure(abort_code = 65537, location = Self)]
    public entry fun test_redeem_shares_non_existent_shareholder() {
        let pool = new(1);
        add_shares(&mut pool, @1, 1);
        redeem_shares(&mut pool, @2, 1);
        destroy_pool(pool);
    }

    #[test]
    #[expected_failure(abort_code = 65540, location = Self)]
    public entry fun test_redeem_shares_insufficient_shares() {
        let pool = new(1);
        add_shares(&mut pool, @1, 1);
        redeem_shares(&mut pool, @1, 2);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_redeem_small_number_of_shares() {
        let pool = new(2);
        // 1e17 shares and coins.
        buy_in(&mut pool, @1, 100000000000000000);
        buy_in(&mut pool, @2, 100000000000000000);
        assert!(redeem_shares(&mut pool, @1, 1) == 1, 0);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_redeem_zero_shares() {
        let pool = new(2);
        buy_in(&mut pool, @1, 1);
        assert!(redeem_shares(&mut pool, @1, 0) == 0, 0);
        assert!(shares(&pool, @1) == 1, 1);
        assert!(total_coins(&pool) == 1, 2);
        assert!(total_shares(&pool) == 1, 3);
        destroy_pool(pool);
    }

    #[test]
    #[expected_failure(abort_code = 65537, location = Self)]
    public entry fun test_deduct_shares_non_existent_shareholder() {
        let pool = new(1);
        add_shares(&mut pool, @1, 1);
        deduct_shares(&mut pool, @2, 1);
        destroy_pool(pool);
    }

    #[test]
    #[expected_failure(abort_code = 65540, location = Self)]
    public entry fun test_deduct_shares_insufficient_shares() {
        let pool = new(1);
        add_shares(&mut pool, @1, 1);
        deduct_shares(&mut pool, @1, 2);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_deduct_shares_remove_shareholder_with_no_shares() {
        let pool = new(2);
        add_shares(&mut pool, @1, 1);
        add_shares(&mut pool, @2, 2);
        assert!(shareholders_count(&pool) == 2, 0);
        deduct_shares(&mut pool, @1, 1);
        assert!(shareholders_count(&pool) == 1, 1);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_transfer_shares() {
        let pool = new(2);
        add_shares(&mut pool, @1, 2);
        add_shares(&mut pool, @2, 2);
        assert!(shareholders_count(&pool) == 2, 0);
        transfer_shares(&mut pool, @1, @2, 1);
        assert!(shares(&pool, @1) == 1, 0);
        assert!(shares(&pool, @2) == 3, 0);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_amount_to_shares_empty_pool() {
        let pool = new(1);
        // No total shares and total coins.
        assert!(amount_to_shares(&pool, 1000) == 1000, 0);

        // No total shares but some total coins.
        update_total_coins(&mut pool, 1000);
        assert!(amount_to_shares(&pool, 1000) == 1000, 1);

        // No total coins but some total shares.
        update_total_coins(&mut pool, 0);
        add_shares(&mut pool, @1, 100);
        assert!(amount_to_shares(&pool, 1000) == 1000, 2);
        destroy_pool(pool);
    }

    #[test]
    public entry fun test_shares_to_amount_empty_pool() {
        let pool = new(1);
        // No total shares and total coins.
        assert!(shares_to_amount(&pool, 1000) == 0, 0);

        // No total shares but some total coins.
        update_total_coins(&mut pool, 1000);
        assert!(shares_to_amount(&pool, 1000) == 0, 1);

        // No total coins but some total shares.
        update_total_coins(&mut pool, 0);
        add_shares(&mut pool, @1, 100);
        assert!(shares_to_amount(&pool, 1000) == 0, 2);
        destroy_pool(pool);
    }
}
