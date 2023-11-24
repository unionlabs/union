package secp256r1

import (
	cmtcrypto "github.com/cometbft/cometbft/crypto"
	"github.com/cosmos/gogoproto/proto"

	ecdsa "github.com/cosmos/cosmos-sdk/crypto/keys/internal/ecdsa"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

// String implements proto.Message interface.
func (m *PubKey) String() string {
	return m.Key.String(name)
}

// Bytes implements SDK PubKey interface.
func (m *PubKey) Bytes() []byte {
	if m == nil {
		return nil
	}
	return m.Key.Bytes()
}

// Equals implements SDK PubKey interface.
func (m *PubKey) Equals(other cryptotypes.PubKey) bool {
	pk2, ok := other.(*PubKey)
	if !ok {
		return false
	}
	return m.Key.Equal(&pk2.Key.PublicKey)
}

// Address implements SDK PubKey interface.
func (m *PubKey) Address() cmtcrypto.Address {
	return m.Key.Address(proto.MessageName(m))
}

// Type returns key type name. Implements SDK PubKey interface.
func (m *PubKey) Type() string {
	return name
}

// VerifySignature implements SDK PubKey interface.
func (m *PubKey) VerifySignature(msg, sig []byte) bool {
	return m.Key.VerifySignature(msg, sig)
}

type ecdsaPK struct {
	ecdsa.PubKey
}

// Size implements proto.Marshaler interface
func (pk *ecdsaPK) Size() int {
	if pk == nil {
		return 0
	}
	return pubKeySize
}

// Unmarshal implements proto.Marshaler interface
func (pk *ecdsaPK) Unmarshal(bz []byte) error {
	return pk.PubKey.Unmarshal(bz, secp256r1, pubKeySize)
}
