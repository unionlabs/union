package client

import (
	"fmt"
	"net/url"
	"unicode"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// ValidatePromptNotEmpty validates that the input is not empty.
func ValidatePromptNotEmpty(input string) error {
	if input == "" {
		return fmt.Errorf("input cannot be empty")
	}

	return nil
}

// ValidatePromptURL validates that the input is a valid URL.
func ValidatePromptURL(input string) error {
	_, err := url.ParseRequestURI(input)
	if err != nil {
		return fmt.Errorf("invalid URL: %w", err)
	}

	return nil
}

// ValidatePromptAddress validates that the input is a valid Bech32 address.
func ValidatePromptAddress(input string) error {
	if _, err := sdk.AccAddressFromBech32(input); err != nil {
		return fmt.Errorf("invalid address: %w", err)
	}

	return nil
}

// ValidatePromptYesNo validates that the input is valid sdk.COins
func ValidatePromptCoins(input string) error {
	if _, err := sdk.ParseCoinsNormalized(input); err != nil {
		return fmt.Errorf("invalid coins: %w", err)
	}

	return nil
}

// CamelCaseToString converts a camel case string to a string with spaces.
func CamelCaseToString(str string) string {
	w := []rune(str)
	for i := len(w) - 1; i > 1; i-- {
		if unicode.IsUpper(w[i]) {
			w = append(w[:i], append([]rune{' '}, w[i:]...)...)
		}
	}
	return string(w)
}
