package types

import "cosmossdk.io/math"

// map coins is a map representation of sdk.Coins
// intended solely for use in bulk additions.
// All serialization and iteration should be done after conversion to sdk.Coins.
type MapCoins map[string]math.Int

func NewMapCoins(coins Coins) MapCoins {
	m := make(MapCoins, len(coins))
	m.Add(coins...)
	return m
}

func (m MapCoins) Add(coins ...Coin) {
	for _, coin := range coins {
		existAmt, exists := m[coin.Denom]
		// TODO: Once int supports in-place arithmetic, switch this to be in-place.
		if exists {
			m[coin.Denom] = existAmt.Add(coin.Amount)
		} else {
			m[coin.Denom] = coin.Amount
		}
	}
}

func (m MapCoins) ToCoins() Coins {
	if len(m) == 0 {
		return Coins{}
	}
	coins := make(Coins, 0, len(m))
	for denom, amount := range m {
		if amount.IsZero() {
			continue
		}
		coins = append(coins, NewCoin(denom, amount))
	}
	if len(coins) == 0 {
		return Coins{}
	}
	coins.Sort()
	return coins
}
