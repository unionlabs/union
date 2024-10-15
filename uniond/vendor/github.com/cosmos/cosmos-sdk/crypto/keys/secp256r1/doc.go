// Package secp256r1 implements Cosmos-SDK compatible ECDSA public and private key. The keys
// can be protobuf serialized and packed in Any.
package secp256r1

import (
	"crypto/elliptic"
	"fmt"

	"cosmossdk.io/core/registry"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

const (
	// fieldSize is the curve domain size.
	fieldSize  = 32
	pubKeySize = fieldSize + 1

	name = "secp256r1"
)

var secp256r1 elliptic.Curve

func init() {
	secp256r1 = elliptic.P256()
	// pubKeySize is ceil of field bit size + 1 for the sign
	expected := (secp256r1.Params().BitSize + 7) / 8
	if expected != fieldSize {
		panic(fmt.Sprintf("Wrong secp256r1 curve fieldSize=%d, expecting=%d", fieldSize, expected))
	}
}

// RegisterInterfaces adds secp256r1 PubKey to pubkey registry
func RegisterInterfaces(registry registry.InterfaceRegistrar) {
	registry.RegisterImplementations((*cryptotypes.PubKey)(nil), &PubKey{})
}
