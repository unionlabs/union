package types

import (
	"encoding/json"
	"errors"
	"fmt"
	"regexp"
	"sort"
	"strings"
	"unicode"

	"cosmossdk.io/math"
)

//-----------------------------------------------------------------------------
// Coin

// NewCoin returns a new coin with a denomination and amount. It will panic if
// the amount is negative or if the denomination is invalid.
func NewCoin(denom string, amount math.Int) Coin {
	coin := Coin{
		Denom:  denom,
		Amount: amount,
	}

	if err := coin.Validate(); err != nil {
		panic(err)
	}

	return coin
}

// NewInt64Coin returns a new coin with a denomination and amount. It will panic
// if the amount is negative.
func NewInt64Coin(denom string, amount int64) Coin {
	return NewCoin(denom, math.NewInt(amount))
}

// String provides a human-readable representation of a coin
func (coin Coin) String() string {
	return fmt.Sprintf("%v%s", coin.Amount, coin.Denom)
}

// Validate returns an error if the Coin has a negative amount or if
// the denom is invalid.
func (coin Coin) Validate() error {
	if err := ValidateDenom(coin.Denom); err != nil {
		return err
	}

	if coin.Amount.IsNil() {
		return errors.New("amount is nil")
	}

	if coin.Amount.IsNegative() {
		return fmt.Errorf("negative coin amount: %v", coin.Amount)
	}

	return nil
}

// IsValid returns true if the Coin has a non-negative amount and the denom is valid.
func (coin Coin) IsValid() bool {
	return coin.Validate() == nil
}

// IsZero returns if this represents no money
func (coin Coin) IsZero() bool {
	return coin.Amount.IsZero()
}

// IsGT returns true if they are the same type and the receiver is
// a greater value
func (coin Coin) IsGT(other Coin) bool {
	if coin.Denom != other.Denom {
		panic(fmt.Sprintf("invalid coin denominations; %s, %s", coin.Denom, other.Denom))
	}

	return coin.Amount.GT(other.Amount)
}

// IsGTE returns true if they are the same type and the receiver is
// an equal or greater value
func (coin Coin) IsGTE(other Coin) bool {
	if coin.Denom != other.Denom {
		panic(fmt.Sprintf("invalid coin denominations; %s, %s", coin.Denom, other.Denom))
	}

	return !coin.Amount.LT(other.Amount)
}

// IsLT returns true if they are the same type and the receiver is
// a smaller value
func (coin Coin) IsLT(other Coin) bool {
	if coin.Denom != other.Denom {
		panic(fmt.Sprintf("invalid coin denominations; %s, %s", coin.Denom, other.Denom))
	}

	return coin.Amount.LT(other.Amount)
}

// IsLTE returns true if they are the same type and the receiver is
// an equal or smaller value
func (coin Coin) IsLTE(other Coin) bool {
	if coin.Denom != other.Denom {
		panic(fmt.Sprintf("invalid coin denominations; %s, %s", coin.Denom, other.Denom))
	}

	return !coin.Amount.GT(other.Amount)
}

// IsEqual returns true if the two sets of Coins have the same value
// Deprecated: Use Coin.Equal instead.
func (coin Coin) IsEqual(other Coin) bool {
	return coin.Equal(other)
}

// Add adds amounts of two coins with same denom. If the coins differ in denom then
// it panics.
func (coin Coin) Add(coinB Coin) Coin {
	if coin.Denom != coinB.Denom {
		panic(fmt.Sprintf("invalid coin denominations; %s, %s", coin.Denom, coinB.Denom))
	}

	return Coin{coin.Denom, coin.Amount.Add(coinB.Amount)}
}

// AddAmount adds an amount to the Coin.
func (coin Coin) AddAmount(amount math.Int) Coin {
	return Coin{coin.Denom, coin.Amount.Add(amount)}
}

// Sub subtracts amounts of two coins with same denom and panics on error.
func (coin Coin) Sub(coinB Coin) Coin {
	res, err := coin.SafeSub(coinB)
	if err != nil {
		panic(err)
	}

	return res
}

// SafeSub safely subtracts the amounts of two coins. It returns an error if the coins differ
// in denom or subtraction results in negative coin denom.
func (coin Coin) SafeSub(coinB Coin) (Coin, error) {
	if coin.Denom != coinB.Denom {
		return Coin{}, fmt.Errorf("invalid coin denoms: %s, %s", coin.Denom, coinB.Denom)
	}

	res := Coin{coin.Denom, coin.Amount.Sub(coinB.Amount)}
	if res.IsNegative() {
		return Coin{}, fmt.Errorf("negative coin amount: %s", res)
	}

	return res, nil
}

// SubAmount subtracts an amount from the Coin.
func (coin Coin) SubAmount(amount math.Int) Coin {
	res := Coin{coin.Denom, coin.Amount.Sub(amount)}
	if res.IsNegative() {
		panic("negative coin amount")
	}

	return res
}

// IsPositive returns true if coin amount is positive.
//
// TODO: Remove once unsigned integers are used.
func (coin Coin) IsPositive() bool {
	return coin.Amount.Sign() == 1
}

// IsNegative returns true if the coin amount is negative and false otherwise.
//
// TODO: Remove once unsigned integers are used.
func (coin Coin) IsNegative() bool {
	return coin.Amount.Sign() == -1
}

// IsNil returns true if the coin amount is nil and false otherwise.
func (coin Coin) IsNil() bool {
	return coin.Amount.BigInt() == nil
}

//-----------------------------------------------------------------------------
// Coins

// Coins is a set of Coin, one per currency
type Coins []Coin

// NewCoins constructs a new coin set. The provided coins will be sanitized by removing
// zero coins and sorting the coin set. A panic will occur if the coin set is not valid.
func NewCoins(coins ...Coin) Coins {
	newCoins := sanitizeCoins(coins)
	if err := newCoins.Validate(); err != nil {
		panic(fmt.Errorf("invalid coin set %s: %w", newCoins, err))
	}

	return newCoins
}

func sanitizeCoins(coins []Coin) Coins {
	newCoins := removeZeroCoins(coins)
	if len(newCoins) == 0 {
		return Coins{}
	}

	return newCoins.Sort()
}

type coinsJSON Coins

// MarshalJSON implements a custom JSON marshaller for the Coins type to allow
// nil Coins to be encoded as an empty array.
func (coins Coins) MarshalJSON() ([]byte, error) {
	if coins == nil {
		return json.Marshal(coinsJSON(Coins{}))
	}

	return json.Marshal(coinsJSON(coins))
}

func (coins Coins) String() string {
	if len(coins) == 0 {
		return ""
	} else if len(coins) == 1 {
		return coins[0].String()
	}

	// Build the string with a string builder
	var out strings.Builder
	for _, coin := range coins[:len(coins)-1] {
		out.WriteString(coin.String())
		out.WriteByte(',')
	}
	out.WriteString(coins[len(coins)-1].String())
	return out.String()
}

// Validate checks that the Coins are sorted, have positive amount, with a valid and unique
// denomination (i.e no duplicates). Otherwise, it returns an error.
func (coins Coins) Validate() error {
	switch len(coins) {
	case 0:
		return nil

	case 1:
		if err := ValidateDenom(coins[0].Denom); err != nil {
			return err
		}
		if !coins[0].IsPositive() {
			return fmt.Errorf("coin %s amount is not positive", coins[0])
		}
		return nil

	default:
		// check single coin case
		if err := (Coins{coins[0]}).Validate(); err != nil {
			return err
		}

		lowDenom := coins[0].Denom

		for _, coin := range coins[1:] {
			if err := ValidateDenom(coin.Denom); err != nil {
				return err
			}
			if coin.Denom < lowDenom {
				return fmt.Errorf("denomination %s is not sorted", coin.Denom)
			}
			if coin.Denom == lowDenom {
				return fmt.Errorf("duplicate denomination %s", coin.Denom)
			}
			if !coin.IsPositive() {
				return fmt.Errorf("coin %s amount is not positive", coin.Denom)
			}

			// we compare each coin against the last denom
			lowDenom = coin.Denom
		}

		return nil
	}
}

// IsSorted returns true when coins are sorted in ASC order by denoms.
func (coins Coins) IsSorted() bool {
	for i := 1; i < len(coins); i++ {
		if coins[i-1].Denom > coins[i].Denom {
			return false
		}
	}
	return true
}

// IsValid calls Validate and returns true when the Coins are sorted, have positive amount, with a
// valid and unique denomination (i.e no duplicates).
func (coins Coins) IsValid() bool {
	return coins.Validate() == nil
}

// Denoms returns all denoms associated with a Coins object
func (coins Coins) Denoms() []string {
	res := make([]string, len(coins))
	for i, coin := range coins {
		res[i] = coin.Denom
	}
	return res
}

// Add adds two sets of coins.
//
// e.g.
// {2A} + {A, 2B} = {3A, 2B}
// {2A} + {0B} = {2A}
//
// NOTE: Add operates under the invariant that coins are sorted by
// denominations.
//
// CONTRACT: Add will never return Coins where one Coin has a non-positive
// amount. In otherwords, IsValid will always return true.
// The function panics if `coins` or  `coinsB` are not sorted (ascending).
func (coins Coins) Add(coinsB ...Coin) Coins {
	return coins.safeAdd(coinsB)
}

// safeAdd will perform addition of two coins sets. If both coin sets are
// empty, then an empty set is returned. If only a single set is empty, the
// other set is returned. Otherwise, the coins are compared in order of their
// denomination and addition only occurs when the denominations match, otherwise
// the coin is simply added to the sum assuming it's not zero.
// The function panics if `coins` or  `coinsB` are not sorted (ascending).
func (coins Coins) safeAdd(coinsB Coins) (coalesced Coins) {
	// probably the best way will be to make Coins and interface and hide the structure
	// definition (type alias)
	if !coins.IsSorted() {
		panic("Coins (self) must be sorted")
	}
	if !coinsB.IsSorted() {
		panic("Wrong argument: coins must be sorted")
	}

	uniqCoins := make(map[string]Coin, len(coins)+len(coinsB))
	// Traverse all the coins for each of the coins and coinsB.
	for _, cL := range []Coins{coins, coinsB} {
		for _, c := range cL {
			if uc, ok := uniqCoins[c.Denom]; ok {
				uniqCoins[c.Denom] = uc.Add(c)
			} else {
				uniqCoins[c.Denom] = c
			}
		}
	}

	coalesced = make(Coins, 0, len(uniqCoins))
	for denom, c := range uniqCoins { //#nosec
		if c.IsZero() {
			continue
		}
		c.Denom = denom
		coalesced = append(coalesced, c)
	}
	return coalesced.Sort()
}

// DenomsSubsetOf returns true if receiver's denom set
// is subset of coinsB's denoms.
func (coins Coins) DenomsSubsetOf(coinsB Coins) bool {
	// more denoms in B than in receiver
	if len(coins) > len(coinsB) {
		return false
	}

	for _, coin := range coins {
		if coinsB.AmountOf(coin.Denom).IsZero() {
			return false
		}
	}

	return true
}

// Sub subtracts a set of coins from another.
//
// e.g.
// {2A, 3B} - {A} = {A, 3B}
// {2A} - {0B} = {2A}
// {A, B} - {A} = {B}
//
// CONTRACT: Sub will never return Coins where one Coin has a non-positive
// amount. In otherwords, IsValid will always return true.
func (coins Coins) Sub(coinsB ...Coin) Coins {
	diff, hasNeg := coins.SafeSub(coinsB...)
	if hasNeg {
		panic("negative coin amount")
	}

	return diff
}

// SafeSub performs the same arithmetic as Sub but returns a boolean if any
// negative coin amount was returned.
// The function panics if `coins` or  `coinsB` are not sorted (ascending).
func (coins Coins) SafeSub(coinsB ...Coin) (Coins, bool) {
	diff := coins.safeAdd(NewCoins(coinsB...).negative())
	return diff, diff.IsAnyNegative()
}

// MulInt performs the scalar multiplication of coins with a `multiplier`
// All coins are multiplied by x
// e.g.
// {2A, 3B} * 2 = {4A, 6B}
// {2A} * 0 panics
// Note, if IsValid was true on Coins, IsValid stays true.
func (coins Coins) MulInt(x math.Int) Coins {
	coins, ok := coins.SafeMulInt(x)
	if !ok {
		panic("multiplying by zero is an invalid operation on coins")
	}

	return coins
}

// SafeMulInt performs the same arithmetic as MulInt but returns false
// if the `multiplier` is zero because it makes IsValid return false.
func (coins Coins) SafeMulInt(x math.Int) (Coins, bool) {
	if x.IsZero() {
		return nil, false
	}

	res := make(Coins, len(coins))
	for i, coin := range coins {
		res[i] = NewCoin(coin.Denom, coin.Amount.Mul(x))
	}

	return res, true
}

// QuoInt performs the scalar division of coins with a `divisor`
// All coins are divided by x and truncated.
// e.g.
// {2A, 30B} / 2 = {1A, 15B}
// {2A} / 2 = {1A}
// {4A} / {8A} = {0A}
// {2A} / 0 = panics
// Note, if IsValid was true on Coins, IsValid stays true,
// unless the `divisor` is greater than the smallest coin amount.
func (coins Coins) QuoInt(x math.Int) Coins {
	coins, ok := coins.SafeQuoInt(x)
	if !ok {
		panic("dividing by zero is an invalid operation on coins")
	}

	return coins
}

// SafeQuoInt performs the same arithmetic as QuoInt but returns an error
// if the division cannot be done.
func (coins Coins) SafeQuoInt(x math.Int) (Coins, bool) {
	if x.IsZero() {
		return nil, false
	}

	var res Coins
	for _, coin := range coins {
		res = append(res, NewCoin(coin.Denom, coin.Amount.Quo(x)))
	}

	return res, true
}

// Max takes two valid Coins inputs and returns a valid Coins result
// where for every denom D, AmountOf(D) of the result is the maximum
// of AmountOf(D) of the inputs.  Note that the result might be not
// be equal to either input. For any valid Coins a, b, and c, the
// following are always true:
//
//	a.IsAllLTE(a.Max(b))
//	b.IsAllLTE(a.Max(b))
//	a.IsAllLTE(c) && b.IsAllLTE(c) == a.Max(b).IsAllLTE(c)
//	a.Add(b...).Equal(a.Min(b).Add(a.Max(b)...))
//
// E.g.
// {1A, 3B, 2C}.Max({4A, 2B, 2C}) == {4A, 3B, 2C}
// {2A, 3B}.Max({1B, 4C}) == {2A, 3B, 4C}
// {1A, 2B}.Max({}) == {1A, 2B}
func (coins Coins) Max(coinsB Coins) Coins {
	max := make([]Coin, 0)
	indexA, indexB := 0, 0
	for indexA < len(coins) && indexB < len(coinsB) {
		coinA, coinB := coins[indexA], coinsB[indexB]
		switch strings.Compare(coinA.Denom, coinB.Denom) {
		case -1: // denom missing from coinsB
			max = append(max, coinA)
			indexA++
		case 0: // same denom in both
			maxCoin := coinA
			if coinB.Amount.GT(maxCoin.Amount) {
				maxCoin = coinB
			}
			max = append(max, maxCoin)
			indexA++
			indexB++
		case 1: // denom missing from coinsA
			max = append(max, coinB)
			indexB++
		}
	}
	for ; indexA < len(coins); indexA++ {
		max = append(max, coins[indexA])
	}
	for ; indexB < len(coinsB); indexB++ {
		max = append(max, coinsB[indexB])
	}
	return NewCoins(max...)
}

// Min takes two valid Coins inputs and returns a valid Coins result
// where for every denom D, AmountOf(D) of the result is the minimum
// of AmountOf(D) of the inputs.  Note that the result might be not
// be equal to either input. For any valid Coins a, b, and c, the
// following are always true:
//
//	a.Min(b).IsAllLTE(a)
//	a.Min(b).IsAllLTE(b)
//	c.IsAllLTE(a) && c.IsAllLTE(b) == c.IsAllLTE(a.Min(b))
//	a.Add(b...).Equal(a.Min(b).Add(a.Max(b)...))
//
// E.g.
// {1A, 3B, 2C}.Min({4A, 2B, 2C}) == {1A, 2B, 2C}
// {2A, 3B}.Min({1B, 4C}) == {1B}
// {1A, 2B}.Min({3C}) == empty
//
// See also DecCoins.Intersect().
func (coins Coins) Min(coinsB Coins) Coins {
	min := make([]Coin, 0)
	for indexA, indexB := 0, 0; indexA < len(coins) && indexB < len(coinsB); {
		coinA, coinB := coins[indexA], coinsB[indexB]
		switch strings.Compare(coinA.Denom, coinB.Denom) {
		case -1: // denom missing from coinsB
			indexA++
		case 0: // same denom in both
			minCoin := coinA
			if coinB.Amount.LT(minCoin.Amount) {
				minCoin = coinB
			}
			if !minCoin.IsZero() {
				min = append(min, minCoin)
			}
			indexA++
			indexB++
		case 1: // denom missing from coins
			indexB++
		}
	}
	return NewCoins(min...)
}

// IsAllGT returns true if for every denom in coinsB,
// the denom is present at a greater amount in coins.
func (coins Coins) IsAllGT(coinsB Coins) bool {
	if len(coins) == 0 {
		return false
	}

	if len(coinsB) == 0 {
		return true
	}

	if !coinsB.DenomsSubsetOf(coins) {
		return false
	}

	for _, coinB := range coinsB {
		amountA, amountB := coins.AmountOf(coinB.Denom), coinB.Amount
		if !amountA.GT(amountB) {
			return false
		}
	}

	return true
}

// IsAllGTE returns false if for any denom in coinsB,
// the denom is present at a smaller amount in coins;
// else returns true.
func (coins Coins) IsAllGTE(coinsB Coins) bool {
	if len(coinsB) == 0 {
		return true
	}

	if len(coins) == 0 {
		return false
	}

	for _, coinB := range coinsB {
		if coinB.Amount.GT(coins.AmountOf(coinB.Denom)) {
			return false
		}
	}

	return true
}

// IsAllLT returns True iff for every denom in coins, the denom is present at
// a smaller amount in coinsB.
func (coins Coins) IsAllLT(coinsB Coins) bool {
	return coinsB.IsAllGT(coins)
}

// IsAllLTE returns true iff for every denom in coins, the denom is present at
// a smaller or equal amount in coinsB.
func (coins Coins) IsAllLTE(coinsB Coins) bool {
	return coinsB.IsAllGTE(coins)
}

// IsAnyGT returns true iff for any denom in coins, the denom is present at a
// greater amount in coinsB.
//
// e.g.
// {2A, 3B}.IsAnyGT{A} = true
// {2A, 3B}.IsAnyGT{5C} = false
// {}.IsAnyGT{5C} = false
// {2A, 3B}.IsAnyGT{} = false
func (coins Coins) IsAnyGT(coinsB Coins) bool {
	if len(coinsB) == 0 {
		return false
	}

	for _, coin := range coins {
		amt := coinsB.AmountOf(coin.Denom)
		if coin.Amount.GT(amt) && !amt.IsZero() {
			return true
		}
	}

	return false
}

// IsAnyGTE returns true iff coins contains at least one denom that is present
// at a greater or equal amount in coinsB; it returns false otherwise.
//
// NOTE: IsAnyGTE operates under the invariant that both coin sets are sorted
// by denominations and there exists no zero coins.
func (coins Coins) IsAnyGTE(coinsB Coins) bool {
	if len(coinsB) == 0 {
		return false
	}

	for _, coin := range coins {
		amt := coinsB.AmountOf(coin.Denom)
		if coin.Amount.GTE(amt) && !amt.IsZero() {
			return true
		}
	}

	return false
}

// IsZero returns true if there are no coins or all coins are zero.
func (coins Coins) IsZero() bool {
	for _, coin := range coins {
		if !coin.IsZero() {
			return false
		}
	}
	return true
}

// Equal returns true if the two sets of Coins have the same value
func (coins Coins) Equal(coinsB Coins) bool {
	if len(coins) != len(coinsB) {
		return false
	}

	coins = coins.Sort()
	coinsB = coinsB.Sort()

	for i := 0; i < len(coins); i++ {
		if !coins[i].Equal(coinsB[i]) {
			return false
		}
	}

	return true
}

// Empty returns true if there are no coins and false otherwise.
func (coins Coins) Empty() bool {
	return len(coins) == 0
}

// AmountOf returns the amount of a denom from coins
// CONTRACT: coins must be valid (sorted).
func (coins Coins) AmountOf(denom string) math.Int {
	mustValidateDenom(denom)
	return coins.AmountOfNoDenomValidation(denom)
}

// AmountOfNoDenomValidation returns the amount of a denom from coins
// without validating the denomination.
// CONTRACT: coins must be valid (sorted).
func (coins Coins) AmountOfNoDenomValidation(denom string) math.Int {
	if ok, c := coins.Find(denom); ok {
		return c.Amount
	}
	return math.ZeroInt()
}

// Find returns true and coin if the denom exists in coins. Otherwise it returns false
// and a zero coin. Uses binary search.
// CONTRACT: coins must be valid (sorted).
func (coins Coins) Find(denom string) (bool, Coin) {
	switch len(coins) {
	case 0:
		return false, Coin{}

	case 1:
		coin := coins[0]
		if coin.Denom == denom {
			return true, coin
		}
		return false, Coin{}

	default:
		midIdx := len(coins) / 2 // 2:1, 3:1, 4:2
		coin := coins[midIdx]
		switch {
		case denom < coin.Denom:
			return coins[:midIdx].Find(denom)
		case denom == coin.Denom:
			return true, coin
		default:
			return coins[midIdx+1:].Find(denom)
		}
	}
}

// GetDenomByIndex returns the Denom of the certain coin to make the findDup generic
func (coins Coins) GetDenomByIndex(i int) string {
	return coins[i].Denom
}

// IsAllPositive returns true if there is at least one coin and all currencies
// have a positive value.
func (coins Coins) IsAllPositive() bool {
	if len(coins) == 0 {
		return false
	}

	for _, coin := range coins {
		if !coin.IsPositive() {
			return false
		}
	}

	return true
}

// IsAnyNegative returns true if there is at least one coin whose amount
// is negative; returns false otherwise. It returns false if the coin set
// is empty too.
//
// TODO: Remove once unsigned integers are used.
func (coins Coins) IsAnyNegative() bool {
	for _, coin := range coins {
		if coin.IsNegative() {
			return true
		}
	}

	return false
}

// IsAnyNil returns true if there is at least one coin whose amount
// is nil; returns false otherwise. It returns false if the coin set
// is empty too.
func (coins Coins) IsAnyNil() bool {
	for _, coin := range coins {
		if coin.IsNil() {
			return true
		}
	}

	return false
}

// negative returns a set of coins with all amount negative.
//
// TODO: Remove once unsigned integers are used.
func (coins Coins) negative() Coins {
	res := make([]Coin, 0, len(coins))

	for _, coin := range coins {
		res = append(res, Coin{
			Denom:  coin.Denom,
			Amount: coin.Amount.Neg(),
		})
	}

	return res
}

// removeZeroCoins removes all zero coins from the given coin set in-place.
func removeZeroCoins(coins Coins) Coins {
	nonZeros := make([]Coin, 0, len(coins))

	for _, coin := range coins {
		if !coin.IsZero() {
			nonZeros = append(nonZeros, coin)
		}
	}

	return nonZeros
}

//-----------------------------------------------------------------------------
// Sort interface

// Len implements sort.Interface for Coins
func (coins Coins) Len() int { return len(coins) }

// Less implements sort.Interface for Coins
func (coins Coins) Less(i, j int) bool { return coins[i].Denom < coins[j].Denom }

// Swap implements sort.Interface for Coins
func (coins Coins) Swap(i, j int) { coins[i], coins[j] = coins[j], coins[i] }

var _ sort.Interface = Coins{}

// Sort is a helper function to sort the set of coins in-place
func (coins Coins) Sort() Coins {
	// sort.Sort(coins) does a costly runtime copy as part of `runtime.convTSlice`
	// So we avoid this heap allocation if len(coins) <= 1. In the future, we should hopefully find
	// a strategy to always avoid this.
	if len(coins) > 1 {
		sort.Sort(coins)
	}
	return coins
}

var (
	reDecAmt = `[[:digit:]]+(?:\.[[:digit:]]+)?|\.[[:digit:]]+`
	reSpc    = `[[:space:]]*`

	coinDenomRegex func() string

	reDnm     *regexp.Regexp
	reDecCoin *regexp.Regexp
)

// SetCoinDenomRegex allows for coin's custom validation by overriding the regular
// expression string used for denom validation.
func SetCoinDenomRegex(reFn func() string) {
	coinDenomRegex = reFn

	reDnm = regexp.MustCompile(fmt.Sprintf(`^%s$`, coinDenomRegex()))
	reDecCoin = regexp.MustCompile(fmt.Sprintf(`^(%s)%s(%s)$`, reDecAmt, reSpc, coinDenomRegex()))
}

// ValidateDenom is the default validation function for Coin.Denom.
func ValidateDenom(denom string) error {
	if reDnm == nil || reDecCoin == nil {
		// Call the Ragel-generated function.
		if !MatchDenom(denom) {
			return fmt.Errorf("invalid denom: %s", denom)
		}
	} else if !reDnm.MatchString(denom) { // If reDnm has been initialized, use it for matching.
		return fmt.Errorf("invalid denom: %s", denom)
	}

	return nil
}

// isValidRune checks if a given rune is a valid character for a rune.
// It returns true if the rune is a letter, digit, '/', ':', '.', '_', or '-'.
func isValidRune(r rune) bool {
	return unicode.IsLetter(r) || unicode.IsDigit(r) || r == '/' || r == ':' || r == '.' || r == '_' || r == '-'
}

// MatchDenom checks if the given string is a valid denomination.
// A valid denomination must have a length between 3 and 128 characters,
// start with a letter, and only contain valid runes.
func MatchDenom(s string) bool {
	length := len(s)
	if length < 3 || length > 128 {
		return false
	}

	firstRune := rune(s[0])
	if !unicode.IsLetter(firstRune) {
		return false
	}

	for _, r := range s[1:] {
		if !isValidRune(r) {
			return false
		}
	}

	return true
}

func mustValidateDenom(denom string) {
	if err := ValidateDenom(denom); err != nil {
		panic(err)
	}
}

// ParseCoinNormalized parses and normalize a cli input for one coin type, returning errors if invalid or on an empty string
// as well.
// Expected format: "{amount}{denomination}"
func ParseCoinNormalized(coinStr string) (coin Coin, err error) {
	decCoin, err := ParseDecCoin(coinStr)
	if err != nil {
		return Coin{}, err
	}

	coin, _ = NewDecCoinFromDec(decCoin.Denom, decCoin.Amount).TruncateDecimal()

	return coin, nil
}

// ParseCoinsNormalized will parse out a list of coins separated by commas, and normalize them by converting to the smallest
// unit. If the parsing is successful, the provided coins will be sanitized by removing zero coins and sorting the coin
// set. Lastly a validation of the coin set is executed. If the check passes, ParseCoinsNormalized will return the
// sanitized coins.
// Otherwise, it will return an error.
// If an empty string is provided to ParseCoinsNormalized, it returns nil Coins.
// ParseCoinsNormalized supports decimal coins as inputs, and truncate them to int after converted to the smallest unit.
// Expected format: "{amount0}{denomination},...,{amountN}{denominationN}"
func ParseCoinsNormalized(coinStr string) (Coins, error) {
	coins, err := ParseDecCoins(coinStr)
	if err != nil {
		return Coins{}, err
	}
	return NormalizeCoins(coins), nil
}

// ----------------------------------------------------------------------------

// NormalizeCoins normalize and truncate a list of decimal coins
func NormalizeCoins(coins []DecCoin) Coins {
	if coins == nil {
		return nil
	}
	result := make([]Coin, 0, len(coins))

	for _, coin := range coins {
		newCoin, _ := NewDecCoinFromDec(coin.Denom, coin.Amount).TruncateDecimal()
		result = append(result, newCoin)
	}

	return result
}
