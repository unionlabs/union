package coins

import (
	"fmt"
	"regexp"
	"sort"
	"strings"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	"cosmossdk.io/math"
)

const emptyCoins = "zero"

// Amount can be a whole number or a decimal number. Denominations can be 3 ~ 128
// characters long and support letters, followed by either a letter, a number or
// a separator ('/', ':', '.', '_' or '-').
var coinRegex = regexp.MustCompile(`^(\d+(\.\d+)?)([a-zA-Z][a-zA-Z0-9\/\:\._\-]{2,127})$`)

// formatCoin formats a sdk.Coin into a value-rendered string, using the
// given metadata about the denom. It returns the formatted coin string, the
// display denom, and an optional error.
func formatCoin(coin *basev1beta1.Coin, metadata *bankv1beta1.Metadata) (string, error) {
	coinDenom := coin.Denom

	// Return early if no display denom or display denom is the current coin denom.
	if metadata == nil || metadata.Display == "" || coinDenom == metadata.Display {
		vr, err := math.FormatDec(coin.Amount)
		return vr + " " + coin.Denom, err
	}

	dispDenom := metadata.Display

	// Find exponents of both denoms.
	var coinExp, dispExp uint32
	foundCoinExp, foundDispExp := false, false
	for _, unit := range metadata.DenomUnits {
		if coinDenom == unit.Denom {
			coinExp = unit.Exponent
			foundCoinExp = true
		}
		if dispDenom == unit.Denom {
			dispExp = unit.Exponent
			foundDispExp = true
		}
	}

	// If we didn't find either exponent, then we return early.
	if !foundCoinExp || !foundDispExp {
		vr, err := math.FormatInt(coin.Amount)
		return vr + " " + coin.Denom, err
	}

	dispAmount, err := math.LegacyNewDecFromStr(coin.Amount)
	if err != nil {
		return "", err
	}

	if coinExp > dispExp {
		dispAmount = dispAmount.Mul(math.LegacyNewDec(10).Power(uint64(coinExp - dispExp)))
	} else {
		dispAmount = dispAmount.Quo(math.LegacyNewDec(10).Power(uint64(dispExp - coinExp)))
	}

	vr, err := math.FormatDec(dispAmount.String())
	return vr + " " + dispDenom, err
}

// FormatCoins formats Coins into a value-rendered string, which uses
// `formatCoin` separated by ", " (a comma and a space), and sorted
// alphabetically by value-rendered denoms. It expects an array of metadata
// (optionally nil), where each metadata at index `i` MUST match the coin denom
// at the same index.
func FormatCoins(coins []*basev1beta1.Coin, metadata []*bankv1beta1.Metadata) (string, error) {
	if len(coins) != len(metadata) {
		return "", fmt.Errorf("formatCoins expect one metadata for each coin; expected %d, got %d", len(coins), len(metadata))
	}

	formatted := make([]string, len(coins))
	for i, coin := range coins {
		var err error
		formatted[i], err = formatCoin(coin, metadata[i])
		if err != nil {
			return "", err
		}
	}

	if len(coins) == 0 {
		return emptyCoins, nil
	}

	// Sort the formatted coins by display denom.
	sort.SliceStable(formatted, func(i, j int) bool {
		denomI := strings.Split(formatted[i], " ")[1]
		denomJ := strings.Split(formatted[j], " ")[1]

		return denomI < denomJ
	})

	return strings.Join(formatted, ", "), nil
}

// ParseCoin parses a coin from a string. The string must be in the format
// <amount><denom>, where <amount> is a number and <denom> is a valid denom.
func ParseCoin(input string) (*basev1beta1.Coin, error) {
	input = strings.TrimSpace(input)

	if input == "" {
		return nil, fmt.Errorf("empty input when parsing coin")
	}

	matches := coinRegex.FindStringSubmatch(input)

	if len(matches) == 0 {
		return nil, fmt.Errorf("invalid input format")
	}

	return &basev1beta1.Coin{
		Amount: matches[1],
		Denom:  matches[3],
	}, nil
}
