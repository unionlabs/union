//go:build blst_disabled

package blst

import (
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
)

// This stub file exists until build issues can be resolved for libfuzz.
const err = "blst is only supported on linux,darwin,windows"

// SecretKey -- stub
type SecretKey struct{}

// PublicKey -- stub
func (s SecretKey) PublicKey() common.PublicKey {
	panic(err)
}

// Sign -- stub
func (s SecretKey) Sign(_ []byte) common.Signature {
	panic(err)
}

// Marshal -- stub
func (s SecretKey) Marshal() []byte {
	panic(err)
}

// IsZero -- stub
func (s SecretKey) IsZero() bool {
	panic(err)
}

// PublicKey -- stub
type PublicKey struct{}

// Marshal -- stub
func (p PublicKey) Marshal() []byte {
	panic(err)
}

// Copy -- stub
func (p PublicKey) Copy() common.PublicKey {
	panic(err)
}

// Aggregate -- stub
func (p PublicKey) Aggregate(_ common.PublicKey) common.PublicKey {
	panic(err)
}

// IsInfinite -- stub
func (p PublicKey) IsInfinite() bool {
	panic(err)
}

// Equals -- stub
func (p PublicKey) Equals(_ common.PublicKey) bool {
	panic(err)
}

// Signature -- stub
type Signature struct{}

// Verify -- stub
func (s Signature) Verify(_ common.PublicKey, _ []byte) bool {
	panic(err)
}

// AggregateVerify -- stub
func (s Signature) AggregateVerify(_ []common.PublicKey, _ [][32]byte) bool {
	panic(err)
}

// FastAggregateVerify -- stub
func (s Signature) FastAggregateVerify(_ []common.PublicKey, _ [32]byte) bool {
	panic(err)
}

// Eth2FastAggregateVerify -- stub
func (s Signature) Eth2FastAggregateVerify(_ []common.PublicKey, _ [32]byte) bool {
	panic(err)
}

// Marshal -- stub
func (s Signature) Marshal() []byte {
	panic(err)
}

// Copy -- stub
func (s Signature) Copy() common.Signature {
	panic(err)
}

// SecretKeyFromBytes -- stub
func SecretKeyFromBytes(_ []byte) (SecretKey, error) {
	panic(err)
}

// PublicKeyFromBytes -- stub
func PublicKeyFromBytes(_ []byte) (PublicKey, error) {
	panic(err)
}

// SignatureFromBytes -- stub
func SignatureFromBytes(_ []byte) (Signature, error) {
	panic(err)
}

// MultipleSignaturesFromBytes -- stub
func MultipleSignaturesFromBytes(multiSigs [][]byte) ([]common.Signature, error) {
	panic(err)
}

// AggregatePublicKeys -- stub
func AggregatePublicKeys(_ [][]byte) (PublicKey, error) {
	panic(err)
}

// AggregateSignatures -- stub
func AggregateSignatures(_ []common.Signature) common.Signature {
	panic(err)
}

// AggregateMultiplePubkeys -- stub
func AggregateMultiplePubkeys(pubs []common.PublicKey) common.PublicKey {
	panic(err)
}

// AggregateCompressedSignatures -- stub
func AggregateCompressedSignatures(multiSigs [][]byte) (common.Signature, error) {
	panic(err)
}

// VerifyMultipleSignatures -- stub
func VerifyMultipleSignatures(_ [][]byte, _ [][32]byte, _ []common.PublicKey) (bool, error) {
	panic(err)
}

// NewAggregateSignature -- stub
func NewAggregateSignature() common.Signature {
	panic(err)
}

// RandKey -- stub
func RandKey() (common.SecretKey, error) {
	panic(err)
}

// VerifyCompressed -- stub
func VerifyCompressed(_, _, _ []byte) bool {
	panic(err)
}
