package types

import (
	"bytes"
	"fmt"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
)

// NewBaseAccountWithPubKey creates an account with an a pubkey.
func NewBaseAccountWithPubKey(pubkey cryptotypes.PubKey) (*BaseAccount, error) {
	if pubkey == nil {
		return nil, fmt.Errorf("pubkey cannot be nil")
	}

	baseAccount := NewBaseAccountWithAddress(sdk.AccAddress(pubkey.Address()))
	if err := baseAccount.SetPubKey(pubkey); err != nil {
		return nil, fmt.Errorf("failed to create a valid account with credentials: %w", err)
	}

	if err := baseAccount.Validate(); err != nil {
		return nil, fmt.Errorf("failed to create a valid account with credentials: %w", err)
	}

	return baseAccount, nil
}

const ModuleCredentialType = "ModuleCredential"

var _ cryptotypes.PubKey = &ModuleCredential{}

// NewModuleCredential creates new module credential key.
// All derivation keys must be non-empty.
func NewModuleCredential(moduleName string, derivationKeys ...[]byte) (*ModuleCredential, error) {
	for i := range derivationKeys {
		if len(derivationKeys[i]) == 0 {
			return nil, fmt.Errorf("module credential derivation keys at index %d is empty", i)
		}
	}
	return &ModuleCredential{
		ModuleName:     moduleName,
		DerivationKeys: derivationKeys,
	}, nil
}

func (m *ModuleCredential) Address() cryptotypes.Address {
	return address.Module(m.ModuleName, m.DerivationKeys...)
}

func (m *ModuleCredential) Bytes() []byte {
	return nil
}

// VerifySignature returns always false, making the account unclaimable
func (m *ModuleCredential) VerifySignature(_, _ []byte) bool {
	return false
}

func (m *ModuleCredential) Equals(other cryptotypes.PubKey) bool {
	om, ok := other.(*ModuleCredential)
	if !ok {
		return false
	}

	if m.ModuleName != om.ModuleName {
		return false
	}

	if len(m.DerivationKeys) != len(om.DerivationKeys) {
		return false
	}

	for i := range m.DerivationKeys {
		if !bytes.Equal(m.DerivationKeys[i], om.DerivationKeys[i]) {
			return false
		}
	}

	return true
}

func (m *ModuleCredential) Type() string {
	return ModuleCredentialType
}
