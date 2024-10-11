package prompt

import (
	"errors"
	"fmt"
	"net/url"

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

// ValidatePromptCoins validates that the input contains valid sdk.Coins
func ValidatePromptCoins(input string) error {
	if _, err := sdk.ParseCoinsNormalized(input); err != nil {
		return fmt.Errorf("invalid coins: %w", err)
	}

	return nil
}
