package types

import (
	"fmt"
	"sort"
	"strings"
	"unicode"

	"cosmossdk.io/errors"
	"cosmossdk.io/math"
)

// ----------------------------------------------------------------------------
// Decimal Coin

// NewDecCoin creates a new DecCoin instance from an Int.
func NewDecCoin(denom string, amount math.Int) DecCoin {
	coin := NewCoin(denom, amount)

	return DecCoin{
		Denom:  coin.Denom,
		Amount: math.LegacyNewDecFromInt(coin.Amount),
	}
}

// NewDecCoinFromDec creates a new DecCoin instance from a Dec.
func NewDecCoinFromDec(denom string, amount math.LegacyDec) DecCoin {
	mustValidateDenom(denom)

	if amount.IsNegative() {
		panic(fmt.Sprintf("negative decimal coin amount: %v\n", amount))
	}

	return DecCoin{
		Denom:  denom,
		Amount: amount,
	}
}

// NewDecCoinFromCoin creates a new DecCoin from a Coin.
func NewDecCoinFromCoin(coin Coin) DecCoin {
	if err := coin.Validate(); err != nil {
		panic(err)
	}

	return DecCoin{
		Denom:  coin.Denom,
		Amount: math.LegacyNewDecFromInt(coin.Amount),
	}
}

// NewInt64DecCoin returns a new DecCoin with a denomination and amount. It will
// panic if the amount is negative or denom is invalid.
func NewInt64DecCoin(denom string, amount int64) DecCoin {
	return NewDecCoin(denom, math.NewInt(amount))
}

// IsZero returns if the DecCoin amount is zero.
func (coin DecCoin) IsZero() bool {
	return coin.Amount.IsZero()
}

// IsGTE returns true if they are the same type and the receiver is
// an equal or greater value.
func (coin DecCoin) IsGTE(other DecCoin) bool {
	if coin.Denom != other.Denom {
		panic(fmt.Sprintf("invalid coin denominations; %s, %s", coin.Denom, other.Denom))
	}

	return !coin.Amount.LT(other.Amount)
}

// IsLT returns true if they are the same type and the receiver is
// a smaller value.
func (coin DecCoin) IsLT(other DecCoin) bool {
	if coin.Denom != other.Denom {
		panic(fmt.Sprintf("invalid coin denominations; %s, %s", coin.Denom, other.Denom))
	}

	return coin.Amount.LT(other.Amount)
}

// IsEqual returns true if the two sets of Coins have the same value.
// Deprecated: Use DecCoin.Equal instead.
func (coin DecCoin) IsEqual(other DecCoin) bool {
	return coin.Equal(other)
}

// Add adds amounts of two decimal coins with same denom.
func (coin DecCoin) Add(coinB DecCoin) DecCoin {
	if coin.Denom != coinB.Denom {
		panic(fmt.Sprintf("coin denom different: %v %v\n", coin.Denom, coinB.Denom))
	}
	return DecCoin{coin.Denom, coin.Amount.Add(coinB.Amount)}
}

// Sub subtracts amounts of two decimal coins with same denom.
func (coin DecCoin) Sub(coinB DecCoin) DecCoin {
	if coin.Denom != coinB.Denom {
		panic(fmt.Sprintf("coin denom different: %v %v\n", coin.Denom, coinB.Denom))
	}
	res := DecCoin{coin.Denom, coin.Amount.Sub(coinB.Amount)}
	if res.IsNegative() {
		panic("negative decimal coin amount")
	}
	return res
}

// TruncateDecimal returns a Coin with a truncated decimal and a DecCoin for the
// change. Note, the change may be zero.
func (coin DecCoin) TruncateDecimal() (Coin, DecCoin) {
	truncated := coin.Amount.TruncateInt()
	change := coin.Amount.Sub(math.LegacyNewDecFromInt(truncated))
	return NewCoin(coin.Denom, truncated), NewDecCoinFromDec(coin.Denom, change)
}

// IsPositive returns true if coin amount is positive.
//
// TODO: Remove once unsigned integers are used.
func (coin DecCoin) IsPositive() bool {
	return coin.Amount.IsPositive()
}

// IsNegative returns true if the coin amount is negative and false otherwise.
//
// TODO: Remove once unsigned integers are used.
func (coin DecCoin) IsNegative() bool {
	return coin.Amount.IsNegative()
}

// String implements the Stringer interface for DecCoin. It returns a
// human-readable representation of a decimal coin.
func (coin DecCoin) String() string {
	return fmt.Sprintf("%v%v", coin.Amount, coin.Denom)
}

// Validate returns an error if the DecCoin has a negative amount or if the denom is invalid.
func (coin DecCoin) Validate() error {
	if err := ValidateDenom(coin.Denom); err != nil {
		return err
	}
	if coin.IsNegative() {
		return fmt.Errorf("decimal coin %s amount cannot be negative", coin)
	}
	return nil
}

// IsValid returns true if the DecCoin has a non-negative amount and the denom is valid.
func (coin DecCoin) IsValid() bool {
	return coin.Validate() == nil
}

// ----------------------------------------------------------------------------
// Decimal Coins

// DecCoins defines a slice of coins with decimal values
type DecCoins []DecCoin

// NewDecCoins constructs a new coin set with decimal values
// from DecCoins. The provided coins will be sanitized by removing
// zero coins and sorting the coin set. A panic will occur if the coin set is not valid.
func NewDecCoins(decCoins ...DecCoin) DecCoins {
	newDecCoins := sanitizeDecCoins(decCoins)
	if err := newDecCoins.Validate(); err != nil {
		panic(fmt.Errorf("invalid coin set %s: %w", newDecCoins, err))
	}

	return newDecCoins
}

func sanitizeDecCoins(decCoins []DecCoin) DecCoins {
	// remove zeroes
	newDecCoins := removeZeroDecCoins(decCoins)
	if len(newDecCoins) == 0 {
		return DecCoins{}
	}

	return newDecCoins.Sort()
}

// NewDecCoinsFromCoins constructs a new coin set with decimal values
// from regular Coins.
func NewDecCoinsFromCoins(coins ...Coin) DecCoins {
	if len(coins) == 0 {
		return DecCoins{}
	}

	decCoins := make([]DecCoin, 0, len(coins))
	newCoins := NewCoins(coins...)
	for _, coin := range newCoins {
		decCoins = append(decCoins, NewDecCoinFromCoin(coin))
	}

	return decCoins
}

// String implements the Stringer interface for DecCoins. It returns a
// human-readable representation of decimal coins.
func (coins DecCoins) String() string {
	if len(coins) == 0 {
		return ""
	}

	out := ""
	for _, coin := range coins {
		out += fmt.Sprintf("%v,", coin.String())
	}

	return out[:len(out)-1]
}

// TruncateDecimal returns the coins with truncated decimals and returns the
// change. Note, it will not return any zero-amount coins in either the truncated or
// change coins.
func (coins DecCoins) TruncateDecimal() (truncatedCoins Coins, changeCoins DecCoins) {
	for _, coin := range coins {
		truncated, change := coin.TruncateDecimal()
		if !truncated.IsZero() {
			truncatedCoins = truncatedCoins.Add(truncated)
		}
		if !change.IsZero() {
			changeCoins = changeCoins.Add(change)
		}
	}

	return truncatedCoins, changeCoins
}

// Add adds two sets of DecCoins.
//
// NOTE: Add operates under the invariant that coins are sorted by
// denominations.
//
// CONTRACT: Add will never return Coins where one Coin has a non-positive
// amount. In otherwords, IsValid will always return true.
func (coins DecCoins) Add(coinsB ...DecCoin) DecCoins {
	return coins.safeAdd(coinsB)
}

// safeAdd will perform addition of two DecCoins sets. If both coin sets are
// empty, then an empty set is returned. If only a single set is empty, the
// other set is returned. Otherwise, the coins are compared in order of their
// denomination and addition only occurs when the denominations match, otherwise
// the coin is simply added to the sum assuming it's not zero.
func (coins DecCoins) safeAdd(coinsB DecCoins) DecCoins {
	sum := make(DecCoins, 0, len(coins)+len(coinsB))
	indexA, indexB := 0, 0
	lenA, lenB := len(coins), len(coinsB)

	for {
		if indexA == lenA {
			if indexB == lenB {
				// return nil coins if both sets are empty
				return sum
			}

			// return set B (excluding zero coins) if set A is empty
			return append(sum, removeZeroDecCoins(coinsB[indexB:])...)
		} else if indexB == lenB {
			// return set A (excluding zero coins) if set B is empty
			return append(sum, removeZeroDecCoins(coins[indexA:])...)
		}

		coinA, coinB := coins[indexA], coinsB[indexB]

		switch strings.Compare(coinA.Denom, coinB.Denom) {
		case -1: // coin A denom < coin B denom
			if !coinA.IsZero() {
				sum = append(sum, coinA)
			}

			indexA++

		case 0: // coin A denom == coin B denom
			res := coinA.Add(coinB)
			if !res.IsZero() {
				sum = append(sum, res)
			}

			indexA++
			indexB++

		case 1: // coin A denom > coin B denom
			if !coinB.IsZero() {
				sum = append(sum, coinB)
			}

			indexB++
		}
	}
}

// negative returns a set of coins with all amount negative.
func (coins DecCoins) negative() DecCoins {
	res := make([]DecCoin, 0, len(coins))
	for _, coin := range coins {
		res = append(res, DecCoin{
			Denom:  coin.Denom,
			Amount: coin.Amount.Neg(),
		})
	}
	return res
}

// Sub subtracts a set of DecCoins from another (adds the inverse).
func (coins DecCoins) Sub(coinsB DecCoins) DecCoins {
	diff, hasNeg := coins.SafeSub(coinsB)
	if hasNeg {
		panic("negative coin amount")
	}

	return diff
}

// SafeSub performs the same arithmetic as Sub but returns a boolean if any
// negative coin amount was returned.
func (coins DecCoins) SafeSub(coinsB DecCoins) (DecCoins, bool) {
	diff := coins.safeAdd(coinsB.negative())
	return diff, diff.IsAnyNegative()
}

// Intersect will return a new set of coins which contains the minimum DecCoin
// for common denoms found in both `coins` and `coinsB`. For denoms not common
// to both `coins` and `coinsB` the minimum is considered to be 0, thus they
// are not added to the final set. In other words, trim any denom amount from
// coin which exceeds that of coinB, such that (coin.Intersect(coinB)).IsLTE(coinB).
// See also Coins.Min().
func (coins DecCoins) Intersect(coinsB DecCoins) DecCoins {
	res := make([]DecCoin, len(coins))
	for i, coin := range coins {
		minCoin := DecCoin{
			Denom:  coin.Denom,
			Amount: math.LegacyMinDec(coin.Amount, coinsB.AmountOf(coin.Denom)),
		}
		res[i] = minCoin
	}
	return removeZeroDecCoins(res)
}

// GetDenomByIndex returns the Denom to make the findDup generic
func (coins DecCoins) GetDenomByIndex(i int) string {
	return coins[i].Denom
}

// IsAnyNegative returns true if there is at least one coin whose amount
// is negative; returns false otherwise. It returns false if the DecCoins set
// is empty too.
//
// TODO: Remove once unsigned integers are used.
func (coins DecCoins) IsAnyNegative() bool {
	for _, coin := range coins {
		if coin.IsNegative() {
			return true
		}
	}

	return false
}

// MulDec multiplies all the coins by a decimal.
//
// CONTRACT: No zero coins will be returned.
func (coins DecCoins) MulDec(d math.LegacyDec) DecCoins {
	var res DecCoins
	for _, coin := range coins {
		product := DecCoin{
			Denom:  coin.Denom,
			Amount: coin.Amount.Mul(d),
		}

		if !product.IsZero() {
			res = res.Add(product)
		}
	}

	return res
}

// MulDecTruncate multiplies all the decimal coins by a decimal, truncating. It
// returns nil DecCoins if d is zero.
//
// CONTRACT: No zero coins will be returned.
func (coins DecCoins) MulDecTruncate(d math.LegacyDec) DecCoins {
	if d.IsZero() {
		return DecCoins{}
	}

	var res DecCoins
	for _, coin := range coins {
		product := DecCoin{
			Denom:  coin.Denom,
			Amount: coin.Amount.MulTruncate(d),
		}

		if !product.IsZero() {
			res = res.Add(product)
		}
	}

	return res
}

// QuoDec divides all the decimal coins by a decimal. It panics if d is zero.
//
// CONTRACT: No zero coins will be returned.
func (coins DecCoins) QuoDec(d math.LegacyDec) DecCoins {
	if d.IsZero() {
		panic("invalid zero decimal")
	}

	var res DecCoins
	for _, coin := range coins {
		quotient := DecCoin{
			Denom:  coin.Denom,
			Amount: coin.Amount.Quo(d),
		}

		if !quotient.IsZero() {
			res = res.Add(quotient)
		}
	}

	return res
}

// QuoDecTruncate divides all the decimal coins by a decimal, truncating. It
// panics if d is zero.
//
// CONTRACT: No zero coins will be returned.
func (coins DecCoins) QuoDecTruncate(d math.LegacyDec) DecCoins {
	if d.IsZero() {
		panic("invalid zero decimal")
	}

	var res DecCoins
	for _, coin := range coins {
		quotient := DecCoin{
			Denom:  coin.Denom,
			Amount: coin.Amount.QuoTruncate(d),
		}

		if !quotient.IsZero() {
			res = res.Add(quotient)
		}
	}

	return res
}

// Empty returns true if there are no coins and false otherwise.
func (coins DecCoins) Empty() bool {
	return len(coins) == 0
}

// AmountOf returns the amount of a denom from deccoins. It panics if the denom
// is invalid.
func (coins DecCoins) AmountOf(denom string) math.LegacyDec {
	mustValidateDenom(denom)
	return coins.AmountOfNoValidation(denom)
}

// AmountOfNoValidation returns the amount of a denom from deccoins without checking
// the correctness of the denom.
func (coins DecCoins) AmountOfNoValidation(denom string) math.LegacyDec {
	switch len(coins) {
	case 0:
		return math.LegacyZeroDec()

	case 1:
		coin := coins[0]
		if coin.Denom == denom {
			return coin.Amount
		}
		return math.LegacyZeroDec()

	default:
		midIdx := len(coins) / 2 // 2:1, 3:1, 4:2
		coin := coins[midIdx]

		switch {
		case denom < coin.Denom:
			return coins[:midIdx].AmountOfNoValidation(denom)
		case denom == coin.Denom:
			return coin.Amount
		default:
			return coins[midIdx+1:].AmountOfNoValidation(denom)
		}
	}
}

// Equal returns true if the two sets of DecCoins have the same value.
func (coins DecCoins) Equal(coinsB DecCoins) bool {
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

// IsZero returns whether all coins are zero
func (coins DecCoins) IsZero() bool {
	for _, coin := range coins {
		if !coin.Amount.IsZero() {
			return false
		}
	}
	return true
}

// Validate checks that the DecCoins are sorted, have positive amount, with a valid and unique
// denomination (i.e no duplicates). Otherwise, it returns an error.
func (coins DecCoins) Validate() error {
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
		if err := (DecCoins{coins[0]}).Validate(); err != nil {
			return err
		}

		lowDenom := coins[0].Denom
		seenDenoms := make(map[string]bool)
		seenDenoms[lowDenom] = true

		for _, coin := range coins[1:] {
			if seenDenoms[coin.Denom] {
				return fmt.Errorf("duplicate denomination %s", coin.Denom)
			}
			if err := ValidateDenom(coin.Denom); err != nil {
				return err
			}
			if coin.Denom <= lowDenom {
				return fmt.Errorf("denomination %s is not sorted", coin.Denom)
			}
			if !coin.IsPositive() {
				return fmt.Errorf("coin %s amount is not positive", coin.Denom)
			}

			// we compare each coin against the last denom
			lowDenom = coin.Denom
			seenDenoms[coin.Denom] = true
		}

		return nil
	}
}

// IsValid calls Validate and returns true when the DecCoins are sorted, have positive amount, with a
// valid and unique denomination (i.e no duplicates).
func (coins DecCoins) IsValid() bool {
	return coins.Validate() == nil
}

// IsAllPositive returns true if there is at least one coin and all currencies
// have a positive value.
//
// TODO: Remove once unsigned integers are used.
func (coins DecCoins) IsAllPositive() bool {
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

func removeZeroDecCoins(coins DecCoins) DecCoins {
	result := make([]DecCoin, 0, len(coins))

	for _, coin := range coins {
		if !coin.IsZero() {
			result = append(result, coin)
		}
	}

	return result
}

//-----------------------------------------------------------------------------
// Sorting

var _ sort.Interface = DecCoins{}

// Len implements sort.Interface for DecCoins
func (coins DecCoins) Len() int { return len(coins) }

// Less implements sort.Interface for DecCoins
func (coins DecCoins) Less(i, j int) bool { return coins[i].Denom < coins[j].Denom }

// Swap implements sort.Interface for DecCoins
func (coins DecCoins) Swap(i, j int) { coins[i], coins[j] = coins[j], coins[i] }

// Sort is a helper function to sort the set of decimal coins in-place.
func (coins DecCoins) Sort() DecCoins {
	// sort.Sort(coins) does a costly runtime copy as part of `runtime.convTSlice`
	// So we avoid this heap allocation if len(coins) <= 1. In the future, we should hopefully find
	// a strategy to always avoid this.
	if len(coins) > 1 {
		sort.Sort(coins)
	}
	return coins
}

// ----------------------------------------------------------------------------
// Parsing

// ParseDecCoin parses a decimal coin from a string, returning an error if
// invalid. An empty string is considered invalid.
func ParseDecCoin(coinStr string) (coin DecCoin, err error) {
	var amountStr, denomStr string
	// if custom parsing has not been set, use default coin regex
	if reDecCoin == nil {
		amountStr, denomStr, err = ParseDecAmount(coinStr)
		if err != nil {
			return DecCoin{}, err
		}
	} else {
		coinStr = strings.TrimSpace(coinStr)

		matches := reDecCoin.FindStringSubmatch(coinStr)
		if matches == nil {
			return DecCoin{}, fmt.Errorf("invalid decimal coin expression: %s", coinStr)
		}

		amountStr, denomStr = matches[1], matches[2]
	}

	amount, err := math.LegacyNewDecFromStr(amountStr)
	if err != nil {
		return DecCoin{}, errors.Wrap(err, fmt.Sprintf("failed to parse decimal coin amount: %s", amountStr))
	}

	if err := ValidateDenom(denomStr); err != nil {
		return DecCoin{}, fmt.Errorf("invalid denom cannot contain spaces: %w", err)
	}

	return NewDecCoinFromDec(denomStr, amount), nil
}

// ParseDecAmount parses the given string into amount, denomination.
func ParseDecAmount(coinStr string) (string, string, error) {
	var amountRune, denomRune []rune

	// Indicates the start of denom parsing
	seenLetter := false
	// Indicates we're currently parsing the amount
	parsingAmount := true

	for _, r := range strings.TrimSpace(coinStr) {
		if parsingAmount {
			if unicode.IsDigit(r) || r == '.' {
				amountRune = append(amountRune, r)
			} else if unicode.IsSpace(r) { // if space is seen, indicates that we have finished parsing amount
				parsingAmount = false
			} else if unicode.IsLetter(r) { // if letter is seen, indicates that it is the start of denom
				parsingAmount = false
				seenLetter = true
				denomRune = append(denomRune, r)
			} else { // Invalid character encountered in amount part
				return "", "", fmt.Errorf("invalid character in coin string: %s", string(r))
			}
		} else if !seenLetter { // This logic flow is for skipping spaces between amount and denomination
			if unicode.IsLetter(r) {
				seenLetter = true
				denomRune = append(denomRune, r)
			} else if !unicode.IsSpace(r) {
				// Invalid character before denomination starts
				return "", "", fmt.Errorf("invalid start of denomination: %s", string(r))
			}
		} else {
			// Parsing the denomination
			if unicode.IsLetter(r) || unicode.IsDigit(r) || r == '/' || r == ':' || r == '.' || r == '_' || r == '-' {
				denomRune = append(denomRune, r)
			} else {
				// Invalid character encountered in denomination part
				return "", "", fmt.Errorf("invalid character in denomination: %s", string(r))
			}
		}
	}

	return string(amountRune), string(denomRune), nil
}

// ParseDecCoins will parse out a list of decimal coins separated by commas. If the parsing is successuful,
// the provided coins will be sanitized by removing zero coins and sorting the coin set. Lastly
// a validation of the coin set is executed. If the check passes, ParseDecCoins will return the sanitized coins.
// Otherwise it will return an error.
// If an empty string is provided to ParseDecCoins, it returns nil Coins.
// Expected format: "{amount0}{denomination},...,{amountN}{denominationN}"
func ParseDecCoins(coinsStr string) (DecCoins, error) {
	coinsStr = strings.TrimSpace(coinsStr)
	if len(coinsStr) == 0 {
		return nil, nil
	}

	coinStrs := strings.Split(coinsStr, ",")
	decCoins := make(DecCoins, len(coinStrs))
	for i, coinStr := range coinStrs {
		coin, err := ParseDecCoin(coinStr)
		if err != nil {
			return nil, err
		}

		decCoins[i] = coin
	}

	newDecCoins := sanitizeDecCoins(decCoins)
	if err := newDecCoins.Validate(); err != nil {
		return nil, err
	}

	return newDecCoins, nil
}
