package keyring

import (
	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
)

// autoCLIKeyring represents the keyring interface used by the AutoCLI.
// It purposely does not import the AutoCLI package to avoid circular dependencies.
type autoCLIKeyring interface {
	// List returns the names of all keys stored in the keyring.
	List() ([]string, error)

	// LookupAddressByKeyName returns the address of the key with the given name.
	LookupAddressByKeyName(name string) ([]byte, error)

	// GetPubKey returns the public key of the key with the given name.
	GetPubKey(name string) (cryptotypes.PubKey, error)

	// Sign signs the given bytes with the key with the given name.
	Sign(name string, msg []byte, signMode signingv1beta1.SignMode) ([]byte, error)
}

// NewAutoCLIKeyring wraps the SDK keyring and make it compatible with the AutoCLI keyring interfaces.
func NewAutoCLIKeyring(kr Keyring) (autoCLIKeyring, error) {
	return &autoCLIKeyringAdapter{kr}, nil
}

type autoCLIKeyringAdapter struct {
	Keyring
}

func (a *autoCLIKeyringAdapter) List() ([]string, error) {
	list, err := a.Keyring.List()
	if err != nil {
		return nil, err
	}

	names := make([]string, len(list))
	for i, key := range list {
		names[i] = key.Name
	}

	return names, nil
}

// LookupAddressByKeyName returns the address of a key stored in the keyring
func (a *autoCLIKeyringAdapter) LookupAddressByKeyName(name string) ([]byte, error) {
	record, err := a.Keyring.Key(name)
	if err != nil {
		return nil, err
	}

	addr, err := record.GetAddress()
	if err != nil {
		return nil, err
	}

	return addr, nil
}

func (a *autoCLIKeyringAdapter) GetPubKey(name string) (cryptotypes.PubKey, error) {
	record, err := a.Keyring.Key(name)
	if err != nil {
		return nil, err
	}

	return record.GetPubKey()
}

func (a *autoCLIKeyringAdapter) Sign(name string, msg []byte, signMode signingv1beta1.SignMode) ([]byte, error) {
	record, err := a.Keyring.Key(name)
	if err != nil {
		return nil, err
	}

	sdkSignMode, err := authsigning.APISignModeToInternal(signMode)
	if err != nil {
		return nil, err
	}

	signBytes, _, err := a.Keyring.Sign(record.Name, msg, sdkSignMode)
	return signBytes, err
}
