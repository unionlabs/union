//go:build bls12381

package bls12381

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/json"
	"errors"

	blst "github.com/supranational/blst/bindings/go"

	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/tmhash"
	cmtjson "github.com/cometbft/cometbft/libs/json"
)

const (
	// Enabled indicates if this curve is enabled.
	Enabled = true
)

var (
	// ErrDeserialization is returned when deserialization fails.
	ErrDeserialization = errors.New("bls12381: deserialization error")

	dstMinSig = []byte("BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_")
)

// For minimal-pubkey-size operations.
//
// Changing this to 'minimal-signature-size' would render CometBFT not Ethereum
// compatible.
type (
	blstPublicKey          = blst.P1Affine
	blstSignature          = blst.P2Affine
	blstAggregateSignature = blst.P1Aggregate
	blstAggregatePublicKey = blst.P2Aggregate
)

// -------------------------------------.

func init() {
	cmtjson.RegisterType(PubKey{}, PubKeyName)
	cmtjson.RegisterType(PrivKey{}, PrivKeyName)
}

// ===============================================================================================
// Private Key
// ===============================================================================================

// PrivKey is a wrapper around the Ethereum BLS12-381 private key type. This
// wrapper conforms to crypto.Pubkey to allow for the use of the Ethereum
// BLS12-381 private key type.

var _ crypto.PrivKey = &PrivKey{}

type PrivKey struct {
	sk *blst.SecretKey
}

// NewPrivateKeyFromBytes build a new key from the given bytes.
func NewPrivateKeyFromBytes(bz []byte) (*PrivKey, error) {
	sk := new(blst.SecretKey).Deserialize(bz)
	if sk == nil {
		return nil, ErrDeserialization
	}
	return &PrivKey{sk: sk}, nil
}

// GenPrivKey generates a new key.
func GenPrivKey() (*PrivKey, error) {
	var ikm [32]byte
	_, err := rand.Read(ikm[:])
	if err != nil {
		return nil, err
	}
	sk := blst.KeyGen(ikm[:])
	return &PrivKey{sk: sk}, nil
}

// Bytes returns the byte representation of the Key.
func (privKey PrivKey) Bytes() []byte {
	return privKey.sk.Serialize()
}

// PubKey returns the private key's public key. If the privkey is not valid
// it returns a nil value.
func (privKey PrivKey) PubKey() crypto.PubKey {
	return &PubKey{pk: new(blstPublicKey).From(privKey.sk)}
}

// Type returns the type.
func (PrivKey) Type() string {
	return KeyType
}

// Sign signs the given byte array. If msg is larger than
// MaxMsgLen, SHA256 sum will be signed instead of the raw bytes.
func (privKey PrivKey) Sign(msg []byte) ([]byte, error) {
	if len(msg) > MaxMsgLen {
		hash := sha256.Sum256(msg)
		signature := new(blstSignature).Sign(privKey.sk, hash[:], dstMinSig)
		return signature.Compress(), nil
	}

	signature := new(blstSignature).Sign(privKey.sk, msg, dstMinSig)
	return signature.Compress(), nil
}

// Zeroize clears the private key.
func (privKey *PrivKey) Zeroize() {
	privKey.sk.Zeroize()
}

// MarshalJSON marshals the private key to JSON.
func (privKey *PrivKey) MarshalJSON() ([]byte, error) {
	return json.Marshal(privKey.Bytes())
}

// UnmarshalJSON unmarshals the private key from JSON.
func (privKey *PrivKey) UnmarshalJSON(bz []byte) error {
	var rawBytes []byte
	if err := json.Unmarshal(bz, &rawBytes); err != nil {
		return err
	}
	pk, err := NewPrivateKeyFromBytes(rawBytes)
	if err != nil {
		return err
	}
	privKey.sk = pk.sk
	return nil
}

// ===============================================================================================
// Public Key
// ===============================================================================================

// Pubkey is a wrapper around the Ethereum BLS12-381 public key type. This
// wrapper conforms to crypto.Pubkey to allow for the use of the Ethereum
// BLS12-381 public key type.

var _ crypto.PubKey = &PubKey{}

type PubKey struct {
	pk *blstPublicKey
}

// NewPublicKeyFromBytes returns a new public key from the given bytes.
func NewPublicKeyFromBytes(bz []byte) (*PubKey, error) {
	pk := new(blstPublicKey).Deserialize(bz)
	if pk == nil {
		return nil, ErrDeserialization
	}
	return &PubKey{pk: pk}, nil
}

// Address returns the address of the key.
//
// The function will panic if the public key is invalid.
func (pubKey PubKey) Address() crypto.Address {
	return crypto.Address(tmhash.SumTruncated(pubKey.pk.Serialize()))
}

// VerifySignature verifies the given signature.
func (pubKey PubKey) VerifySignature(msg, sig []byte) bool {
	signature := new(blstSignature).Uncompress(sig)
	if signature == nil {
		return false
	}

	// Group check signature. Do not check for infinity since an aggregated signature
	// could be infinite.
	if !signature.SigValidate(false) {
		return false
	}

	if len(msg) > MaxMsgLen {
		hash := sha256.Sum256(msg)
		return signature.Verify(false, pubKey.pk, false, hash[:], dstMinSig)
	}

	return signature.Verify(false, pubKey.pk, false, msg, dstMinSig)
}

// Bytes returns the byte format.
func (pubKey PubKey) Bytes() []byte {
	return pubKey.pk.Serialize()
}

// Type returns the key's type.
func (PubKey) Type() string {
	return KeyType
}

// MarshalJSON marshals the public key to JSON.
func (pubkey PubKey) MarshalJSON() ([]byte, error) {
	return json.Marshal(pubkey.Bytes())
}

// UnmarshalJSON unmarshals the public key from JSON.
func (pubkey *PubKey) UnmarshalJSON(bz []byte) error {
	var rawBytes []byte
	if err := json.Unmarshal(bz, &rawBytes); err != nil {
		return err
	}
	pk, err := NewPublicKeyFromBytes(rawBytes)
	if err != nil {
		return err
	}
	pubkey.pk = pk.pk
	return nil
}
