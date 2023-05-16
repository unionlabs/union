// Copyright (c) 2019 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package bech32

import (
	"fmt"
)

// ErrMixedCase is returned when the bech32 string has both lower and uppercase
// characters.
type ErrMixedCase struct{}

func (e ErrMixedCase) Error() string {
	return "string not all lowercase or all uppercase"
}

// ErrInvalidBitGroups is returned when conversion is attempted between byte
// slices using bit-per-element of unsupported value.
type ErrInvalidBitGroups struct{}

func (e ErrInvalidBitGroups) Error() string {
	return "only bit groups between 1 and 8 allowed"
}

// ErrInvalidIncompleteGroup is returned when then byte slice used as input has
// data of wrong length.
type ErrInvalidIncompleteGroup struct{}

func (e ErrInvalidIncompleteGroup) Error() string {
	return "invalid incomplete group"
}

// ErrInvalidLength is returned when the bech32 string has an invalid length
// given the BIP-173 defined restrictions.
type ErrInvalidLength int

func (e ErrInvalidLength) Error() string {
	return fmt.Sprintf("invalid bech32 string length %d", int(e))
}

// ErrInvalidCharacter is returned when the bech32 string has a character
// outside the range of the supported charset.
type ErrInvalidCharacter rune

func (e ErrInvalidCharacter) Error() string {
	return fmt.Sprintf("invalid character in string: '%c'", rune(e))
}

// ErrInvalidSeparatorIndex is returned when the separator character '1' is
// in an invalid position in the bech32 string.
type ErrInvalidSeparatorIndex int

func (e ErrInvalidSeparatorIndex) Error() string {
	return fmt.Sprintf("invalid separator index %d", int(e))
}

// ErrNonCharsetChar is returned when a character outside of the specific
// bech32 charset is used in the string.
type ErrNonCharsetChar rune

func (e ErrNonCharsetChar) Error() string {
	return fmt.Sprintf("invalid character not part of charset: %v", int(e))
}

// ErrInvalidChecksum is returned when the extracted checksum of the string
// is different than what was expected.
type ErrInvalidChecksum struct {
	Expected string
	Actual   string
}

func (e ErrInvalidChecksum) Error() string {
	return fmt.Sprintf("invalid checksum (expected %v got %v)",
		e.Expected, e.Actual)
}

// ErrInvalidDataByte is returned when a byte outside the range required for
// conversion into a string was found.
type ErrInvalidDataByte byte

func (e ErrInvalidDataByte) Error() string {
	return fmt.Sprintf("invalid data byte: %v", byte(e))
}
