package prompt

import (
	"errors"
	"strconv"
	"strings"
	"unicode"
)

const (
	// Constants for passwords.
	minPasswordLength = 8
)

var (
	errIncorrectPhrase = errors.New("input does not match wanted phrase")
	errPasswordWeak    = errors.New("password must have at least 8 characters")
)

// NotEmpty is a validation function to make sure the input given isn't empty and is valid unicode.
func NotEmpty(input string) error {
	if input == "" {
		return errors.New("input cannot be empty")
	}
	if !IsValidUnicode(input) {
		return errors.New("not valid unicode")
	}
	return nil
}

// ValidateNumber makes sure the entered text is a valid number.
func ValidateNumber(input string) error {
	_, err := strconv.Atoi(input)
	if err != nil {
		return err
	}
	return nil
}

// ValidateConfirmation makes sure the entered text is the user confirming.
func ValidateConfirmation(input string) error {
	if input != "Y" && input != "y" {
		return errors.New("please confirm the above text")
	}
	return nil
}

// ValidateYesOrNo ensures the user input either Y, y or N, n.
func ValidateYesOrNo(input string) error {
	lowercase := strings.ToLower(input)
	if lowercase != "y" && lowercase != "n" {
		return errors.New("please enter y or n")
	}
	return nil
}

// IsValidUnicode checks if an input string is a valid unicode string comprised of only
// letters, numbers, punctuation, or symbols.
func IsValidUnicode(input string) bool {
	for _, char := range input {
		if !(unicode.IsLetter(char) ||
			unicode.IsNumber(char) ||
			unicode.IsPunct(char) ||
			unicode.IsSymbol(char) ||
			unicode.IsSpace(char)) {
			return false
		}
	}
	return true
}

// ValidatePasswordInput validates a strong password input for new accounts,
// including a min length, at least 1 number and at least
// 1 special character.
func ValidatePasswordInput(input string) error {
	if len(input) < minPasswordLength {
		return errPasswordWeak
	}
	return nil
}

// ValidatePhrase checks whether the user input is equal to the wanted phrase(s).
func ValidatePhrase(input string, wantedPhrases ...string) error {
	for _, wantedPhrase := range wantedPhrases {
		if strings.EqualFold(strings.TrimSpace(input), wantedPhrase) {
			return nil
		}
	}
	return errIncorrectPhrase
}
