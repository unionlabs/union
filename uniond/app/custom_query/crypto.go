package custom_query

import (
	"fmt"

	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
)

func AggregatePublicKeys(publicKeys [][]byte) (bls.PublicKey, error) {
	return bls.AggregatePublicKeys(publicKeys)
}

func VerifySignature(signature []byte, message [32]byte, publicKeys [][]byte) (bool, error) {
	aggregatedPublicKey, err := AggregatePublicKeys(publicKeys)
	if err != nil {
		return false, fmt.Errorf("failed to aggregate public keys %v", err)
	}
	return bls.VerifySignature(signature, message, aggregatedPublicKey)
}
