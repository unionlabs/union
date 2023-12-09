package simulation

import (
	"errors"
	"math/big"
	"math/rand"
	"time"
	"unsafe"

	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	letterBytes   = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
	letterIdxBits = 6                    // 6 bits to represent a letter index
	letterIdxMask = 1<<letterIdxBits - 1 // All 1-bits, as many as letterIdxBits
	letterIdxMax  = 63 / letterIdxBits   // # of letter indices fitting in 63 bits
)

// shamelessly copied from
// https://stackoverflow.com/questions/22892120/how-to-generate-a-random-string-of-a-fixed-length-in-golang#31832326

// RandStringOfLength generates a random string of a particular length.
func RandStringOfLength(r *rand.Rand, n int) string {
	b := make([]byte, n)
	// A src.Int63() generates 63 random bits, enough for letterIdxMax characters!
	for i, cache, remain := n-1, r.Int63(), letterIdxMax; i >= 0; {
		if remain == 0 {
			cache, remain = r.Int63(), letterIdxMax
		}
		if idx := int(cache & letterIdxMask); idx < len(letterBytes) {
			b[i] = letterBytes[idx]
			i--
		}
		cache >>= letterIdxBits
		remain--
	}

	return *(*string)(unsafe.Pointer(&b))
}

// RandPositiveInt get a rand positive math.Int
func RandPositiveInt(r *rand.Rand, max math.Int) (math.Int, error) {
	if !max.GTE(math.OneInt()) {
		return math.Int{}, errors.New("max too small")
	}

	max = max.Sub(math.OneInt())

	return math.NewIntFromBigInt(new(big.Int).Rand(r, max.BigInt())).Add(math.OneInt()), nil
}

// RandomAmount generates a random amount
// Note: The range of RandomAmount includes max, and is, in fact, biased to return max as well as 0.
func RandomAmount(r *rand.Rand, max math.Int) math.Int {
	randInt := big.NewInt(0)

	switch r.Intn(10) {
	case 0:
		// randInt = big.NewInt(0)
	case 1:
		randInt = max.BigInt()
	default: // NOTE: there are 10 total cases.
		randInt = big.NewInt(0).Rand(r, max.BigInt()) // up to max - 1
	}

	return math.NewIntFromBigInt(randInt)
}

// RandomDecAmount generates a random decimal amount
// Note: The range of RandomDecAmount includes max, and is, in fact, biased to return max as well as 0.
func RandomDecAmount(r *rand.Rand, max math.LegacyDec) math.LegacyDec {
	randInt := big.NewInt(0)

	switch r.Intn(10) {
	case 0:
		// randInt = big.NewInt(0)
	case 1:
		randInt = max.BigInt() // the underlying big int with all precision bits.
	default: // NOTE: there are 10 total cases.
		randInt = big.NewInt(0).Rand(r, max.BigInt())
	}

	return math.LegacyNewDecFromBigIntWithPrec(randInt, math.LegacyPrecision)
}

// RandTimestamp generates a random timestamp
func RandTimestamp(r *rand.Rand) time.Time {
	// json.Marshal breaks for timestamps with year greater than 9999
	// UnixNano breaks with year greater than 2262
	start := time.Date(2062, time.Month(1), 1, 1, 1, 1, 1, time.UTC).UnixMilli()

	// Calculate a random amount of time in seconds between 0 and 200 years
	unixTime := r.Int63n(60*60*24*365*200) * 1000 // convert to milliseconds

	// Get milliseconds for a time between Jan 1, 2062 and Jan 1, 2262
	rtime := time.UnixMilli(start+unixTime).UnixMilli() / 1000
	return time.Unix(rtime, 0)
}

// RandIntBetween returns a random int in the range [min, max) using a given source of randomness.
func RandIntBetween(r *rand.Rand, min, max int) int {
	return r.Intn(max-min) + min
}

// RandSubsetCoins returns random subset of the provided coins
// will return at least one coin unless coins argument is empty or malformed
// i.e. 0 amt in coins
func RandSubsetCoins(r *rand.Rand, coins sdk.Coins) sdk.Coins {
	if len(coins) == 0 {
		return sdk.Coins{}
	}
	// make sure at least one coin added
	denomIdx := r.Intn(len(coins))
	coin := coins[denomIdx]
	amt, err := RandPositiveInt(r, coin.Amount)
	// malformed coin. 0 amt in coins
	if err != nil {
		return sdk.Coins{}
	}

	subset := sdk.Coins{sdk.NewCoin(coin.Denom, amt)}

	for i, c := range coins {
		// skip denom that we already chose earlier
		if i == denomIdx {
			continue
		}
		// coin flip if multiple coins
		// if there is single coin then return random amount of it
		if r.Intn(2) == 0 && len(coins) != 1 {
			continue
		}

		amt, err := RandPositiveInt(r, c.Amount)
		// ignore errors and try another denom
		if err != nil {
			continue
		}

		subset = append(subset, sdk.NewCoin(c.Denom, amt))
	}

	return subset.Sort()
}

// DeriveRand derives a new Rand deterministically from another random source.
// Unlike rand.New(rand.NewSource(seed)), the result is "more random"
// depending on the source and state of r.
//
// NOTE: not crypto safe.
func DeriveRand(r *rand.Rand) *rand.Rand {
	const num = 8 // TODO what's a good number?  Too large is too slow.
	ms := multiSource(make([]rand.Source, num))

	for i := 0; i < num; i++ {
		ms[i] = rand.NewSource(r.Int63())
	}

	return rand.New(ms)
}

type multiSource []rand.Source

func (ms multiSource) Int63() (r int64) {
	for _, source := range ms {
		r ^= source.Int63()
	}

	return r
}

func (ms multiSource) Seed(_ int64) {
	panic("multiSource Seed should not be called")
}
