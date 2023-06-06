package types

import (
	"fmt"
	"net/url"

	errorsmod "cosmossdk.io/errors"
	"github.com/docker/distribution/reference"
)

// MaxSaltSize is the longest salt that can be used when instantiating a contract
const MaxSaltSize = 64

var (
	// MaxLabelSize is the longest label that can be used when instantiating a contract
	MaxLabelSize = 128 // extension point for chains to customize via compile flag.

	// MaxWasmSize is the largest a compiled contract code can be when storing code on chain
	MaxWasmSize = 800 * 1024 // extension point for chains to customize via compile flag.

	// MaxProposalWasmSize is the largest a gov proposal compiled contract code can be when storing code on chain
	MaxProposalWasmSize = 3 * 1024 * 1024 // extension point for chains to customize via compile flag.
)

func validateWasmCode(s []byte, maxSize int) error {
	if len(s) == 0 {
		return errorsmod.Wrap(ErrEmpty, "is required")
	}
	if len(s) > maxSize {
		return errorsmod.Wrapf(ErrLimit, "cannot be longer than %d bytes", maxSize)
	}
	return nil
}

// ValidateLabel ensure label constraints
func ValidateLabel(label string) error {
	if label == "" {
		return errorsmod.Wrap(ErrEmpty, "is required")
	}
	if len(label) > MaxLabelSize {
		return ErrLimit.Wrapf("cannot be longer than %d characters", MaxLabelSize)
	}
	return nil
}

// ValidateSalt ensure salt constraints
func ValidateSalt(salt []byte) error {
	switch n := len(salt); {
	case n == 0:
		return errorsmod.Wrap(ErrEmpty, "is required")
	case n > MaxSaltSize:
		return ErrLimit.Wrapf("cannot be longer than %d characters", MaxSaltSize)
	}
	return nil
}

// ValidateVerificationInfo ensure source, builder and checksum constraints
func ValidateVerificationInfo(source, builder string, codeHash []byte) error {
	// if any set require others to be set
	if len(source) != 0 || len(builder) != 0 || len(codeHash) != 0 {
		if source == "" {
			return fmt.Errorf("source is required")
		}
		if _, err := url.ParseRequestURI(source); err != nil {
			return fmt.Errorf("source: %s", err)
		}
		if builder == "" {
			return fmt.Errorf("builder is required")
		}
		if _, err := reference.ParseDockerRef(builder); err != nil {
			return fmt.Errorf("builder: %s", err)
		}
		if codeHash == nil {
			return fmt.Errorf("code hash is required")
		}
		// code hash checksum match validation is done in the keeper, ungzipping consumes gas
	}
	return nil
}
