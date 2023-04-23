package solomachine

import (
	"strings"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"

	clienttypes "github.com/cosmos/ibc-go/v7/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

var _ exported.ConsensusState = &ConsensusState{}

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
		return nil, sdkerrors.Wrap(clienttypes.ErrInvalidConsensus, "consensus state PublicKey cannot be nil")
	}

	publicKey, ok := cs.PublicKey.GetCachedValue().(cryptotypes.PubKey)
	if !ok {
		return nil, sdkerrors.Wrap(clienttypes.ErrInvalidConsensus, "consensus state PublicKey is not cryptotypes.PubKey")
	}

	return publicKey, nil
}

// ValidateBasic defines basic validation for the solo machine consensus state.
func (cs ConsensusState) ValidateBasic() error {
	if cs.Timestamp == 0 {
		return sdkerrors.Wrap(clienttypes.ErrInvalidConsensus, "timestamp cannot be 0")
	}
	if cs.Diversifier != "" && strings.TrimSpace(cs.Diversifier) == "" {
		return sdkerrors.Wrap(clienttypes.ErrInvalidConsensus, "diversifier cannot contain only spaces")
	}

	publicKey, err := cs.GetPubKey()
	if err != nil || publicKey == nil || len(publicKey.Bytes()) == 0 {
		return sdkerrors.Wrap(clienttypes.ErrInvalidConsensus, "public key cannot be empty")
	}

	return nil
}
