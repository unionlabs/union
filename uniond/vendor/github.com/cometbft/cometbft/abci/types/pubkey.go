package types

import (
	"github.com/cometbft/cometbft/crypto"
)

// NewValidatorUpdate creates a new ValidatorUpdate from the given public
// key.
func NewValidatorUpdate(pubKey crypto.PubKey, power int64) ValidatorUpdate {
	return ValidatorUpdate{
		Power:       power,
		PubKeyType:  pubKey.Type(),
		PubKeyBytes: pubKey.Bytes(),
	}
}
