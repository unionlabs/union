package coins

import (
	"errors"
	"regexp"
	"strings"

	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
)

// Amount can be a whole number or a decimal number. Denominations can be 3 ~ 128
// characters long and support letters, followed by either a letter, a number or
// a separator ('/', ':', '.', '_' or '-').
var coinRegex = regexp.MustCompile(`^(\d+(\.\d+)?)([a-zA-Z][a-zA-Z0-9\/\:\._\-]{2,127})$`)

// ParseCoin parses a coin from a string. The string must be in the format
// <amount><denom>, where <amount> is a number and <denom> is a valid denom.
func ParseCoin(input string) (*basev1beta1.Coin, error) {
	input = strings.TrimSpace(input)

	if input == "" {
		return nil, errors.New("empty input when parsing coin")
	}

	matches := coinRegex.FindStringSubmatch(input)

	if len(matches) == 0 {
		return nil, errors.New("invalid input format")
	}

	return &basev1beta1.Coin{
		Amount: matches[1],
		Denom:  matches[3],
	}, nil
}
