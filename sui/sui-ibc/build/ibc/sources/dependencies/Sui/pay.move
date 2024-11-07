// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// This module provides handy functionality for wallets and `sui::Coin` management.
module sui::pay;

use sui::coin::Coin;

/// For when empty vector is supplied into join function.
const ENoCoins: u64 = 0;

#[allow(lint(self_transfer))]
/// Transfer `c` to the sender of the current transaction
public fun keep<T>(c: Coin<T>, ctx: &TxContext) {
    transfer::public_transfer(c, ctx.sender())
}

/// Split coin `self` to two coins, one with balance `split_amount`,
/// and the remaining balance is left is `self`.
public entry fun split<T>(coin: &mut Coin<T>, split_amount: u64, ctx: &mut TxContext) {
    keep(coin.split(split_amount, ctx), ctx)
}

/// Split coin `self` into multiple coins, each with balance specified
/// in `split_amounts`. Remaining balance is left in `self`.
public entry fun split_vec<T>(self: &mut Coin<T>, split_amounts: vector<u64>, ctx: &mut TxContext) {
    let (mut i, len) = (0, split_amounts.length());
    while (i < len) {
        split(self, split_amounts[i], ctx);
        i = i + 1;
    };
}

/// Send `amount` units of `c` to `recipient`
/// Aborts with `EVALUE` if `amount` is greater than or equal to `amount`
public entry fun split_and_transfer<T>(
    c: &mut Coin<T>,
    amount: u64,
    recipient: address,
    ctx: &mut TxContext,
) {
    transfer::public_transfer(c.split(amount, ctx), recipient)
}

#[allow(lint(self_transfer))]
/// Divide coin `self` into `n - 1` coins with equal balances. If the balance is
/// not evenly divisible by `n`, the remainder is left in `self`.
public entry fun divide_and_keep<T>(self: &mut Coin<T>, n: u64, ctx: &mut TxContext) {
    let mut vec: vector<Coin<T>> = self.divide_into_n(n, ctx);
    let (mut i, len) = (0, vec.length());
    while (i < len) {
        transfer::public_transfer(vec.pop_back(), ctx.sender());
        i = i + 1;
    };
    vec.destroy_empty();
}

/// Join `coin` into `self`. Re-exports `coin::join` function.
/// Deprecated: you should call `coin.join(other)` directly.
public entry fun join<T>(self: &mut Coin<T>, coin: Coin<T>) {
    self.join(coin)
}

/// Join everything in `coins` with `self`
public entry fun join_vec<T>(self: &mut Coin<T>, mut coins: vector<Coin<T>>) {
    let (mut i, len) = (0, coins.length());
    while (i < len) {
        let coin = coins.pop_back();
        self.join(coin);
        i = i + 1
    };
    // safe because we've drained the vector
    coins.destroy_empty()
}

/// Join a vector of `Coin` into a single object and transfer it to `receiver`.
public entry fun join_vec_and_transfer<T>(mut coins: vector<Coin<T>>, receiver: address) {
    assert!(coins.length() > 0, ENoCoins);

    let mut self = coins.pop_back();
    join_vec(&mut self, coins);
    transfer::public_transfer(self, receiver)
}
