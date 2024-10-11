package secp256r1

import (
	"encoding/base64"

	cmtcrypto "github.com/cometbft/cometbft/crypto"
	"github.com/cosmos/gogoproto/proto"

	"github.com/cosmos/cosmos-sdk/crypto/keys/internal/ecdsa"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

// customProtobufType is here to make sure that ecdsaPK and ecdsaSK implement the
// gogoproto customtype interface.
type customProtobufType interface {
	Marshal() ([]byte, error)
	MarshalTo(data []byte) (n int, err error)
	Unmarshal(data []byte) error
	Size() int

	MarshalJSON() ([]byte, error)
	UnmarshalJSON(data []byte) error
}

var _ customProtobufType = (*ecdsaPK)(nil)

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

// Marshal implements customProtobufType.
func (pk ecdsaPK) Marshal() ([]byte, error) {
	return pk.PubKey.Bytes(), nil
}

// MarshalJSON implements customProtobufType.
func (pk ecdsaPK) MarshalJSON() ([]byte, error) {
	b64 := base64.StdEncoding.EncodeToString(pk.PubKey.Bytes())
	return []byte("\"" + b64 + "\""), nil
}

// UnmarshalJSON implements customProtobufType.
func (pk *ecdsaPK) UnmarshalJSON(data []byte) error {
	// the string is quoted so we need to remove them
	bz, err := base64.StdEncoding.DecodeString(string(data[1 : len(data)-1]))
	if err != nil {
		return err
	}

	return pk.PubKey.Unmarshal(bz, secp256r1, pubKeySize)
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
