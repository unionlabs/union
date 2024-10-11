package solomachine

import (
	"strings"

	errorsmod "cosmossdk.io/errors"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ exported.ConsensusState = (*ConsensusState)(nil)

// ClientType returns Solo Machine type.
func (ConsensusState) ClientType() string {
	return exported.Solomachine
}

// GetTimestamp returns zero.
func (cs ConsensusState) GetTimestamp() uint64 {
	return cs.Timestamp
}

// GetPubKey unmarshals the public key into a cryptotypes.PubKey type.
// An error is returned if the public key is nil or the cached value
// is not a PubKey.
func (cs ConsensusState) GetPubKey() (cryptotypes.PubKey, error) {
	if cs.PublicKey == nil {
		return nil, errorsmod.Wrap(clienttypes.ErrInvalidConsensus, "consensus state PublicKey cannot be nil")
	}

	publicKey, ok := cs.PublicKey.GetCachedValue().(cryptotypes.PubKey)
	if !ok {
		return nil, errorsmod.Wrap(clienttypes.ErrInvalidConsensus, "consensus state PublicKey is not cryptotypes.PubKey")
	}

	return publicKey, nil
}

// ValidateBasic defines basic validation for the solo machine consensus state.
func (cs ConsensusState) ValidateBasic() error {
	if cs.Timestamp == 0 {
		return errorsmod.Wrap(clienttypes.ErrInvalidConsensus, "timestamp cannot be 0")
	}
	if cs.Diversifier != "" && strings.TrimSpace(cs.Diversifier) == "" {
		return errorsmod.Wrap(clienttypes.ErrInvalidConsensus, "diversifier cannot contain only spaces")
	}

	publicKey, err := cs.GetPubKey()
	if err != nil || publicKey == nil || len(publicKey.Bytes()) == 0 {
		return errorsmod.Wrap(clienttypes.ErrInvalidConsensus, "public key cannot be empty")
	}

	return nil
}
