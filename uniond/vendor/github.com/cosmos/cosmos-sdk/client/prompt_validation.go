package client

import (
	"errors"
	"fmt"
	"net/url"
	"unicode"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// ValidatePromptNotEmpty validates that the input is not empty.
func ValidatePromptNotEmpty(input string) error {
	if input == "" {
		return errors.New("input cannot be empty")
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
func ValidatePromptAddress(input string) error { // TODO(@julienrbrt) remove and add prompts in AutoCLI
	_, err := sdk.AccAddressFromBech32(input)
	if err == nil {
		return nil
	}

	_, err = sdk.ValAddressFromBech32(input)
	if err == nil {
		return nil
	}

	_, err = sdk.ConsAddressFromBech32(input)
	if err == nil {
		return nil
	}

	return fmt.Errorf("invalid address: %w", err)
}

// ValidatePromptCoins validates that the input contains valid sdk.Coins
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
