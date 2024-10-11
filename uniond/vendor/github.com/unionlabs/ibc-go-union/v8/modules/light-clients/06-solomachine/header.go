package solomachine

import (
	"strings"

	errorsmod "cosmossdk.io/errors"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// SentinelHeaderPath defines a placeholder path value used for headers in solomachine client updates
const SentinelHeaderPath = "solomachine:header"

var _ exported.ClientMessage = (*Header)(nil)

// ClientType defines that the Header is a Solo Machine.
func (Header) ClientType() string {
	return exported.Solomachine
}

// GetPubKey unmarshals the new public key into a cryptotypes.PubKey type.
// An error is returned if the new public key is nil or the cached value
// is not a PubKey.
func (h Header) GetPubKey() (cryptotypes.PubKey, error) {
	if h.NewPublicKey == nil {
		return nil, errorsmod.Wrap(ErrInvalidHeader, "header NewPublicKey cannot be nil")
	}

	publicKey, ok := h.NewPublicKey.GetCachedValue().(cryptotypes.PubKey)
	if !ok {
		return nil, errorsmod.Wrap(ErrInvalidHeader, "header NewPublicKey is not cryptotypes.PubKey")
	}

	return publicKey, nil
}

// ValidateBasic ensures that the timestamp, signature and public key have all
// been initialized.
func (h Header) ValidateBasic() error {
	if h.Timestamp == 0 {
		return errorsmod.Wrap(clienttypes.ErrInvalidHeader, "timestamp cannot be zero")
	}

	if h.NewDiversifier != "" && strings.TrimSpace(h.NewDiversifier) == "" {
		return errorsmod.Wrap(clienttypes.ErrInvalidHeader, "diversifier cannot contain only spaces")
	}

	if len(h.Signature) == 0 {
		return errorsmod.Wrap(clienttypes.ErrInvalidHeader, "signature cannot be empty")
	}

	newPublicKey, err := h.GetPubKey()
	if err != nil || newPublicKey == nil || len(newPublicKey.Bytes()) == 0 {
		return errorsmod.Wrap(clienttypes.ErrInvalidHeader, "new public key cannot be empty")
	}

	return nil
}
