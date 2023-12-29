package sr25519

import (
	"fmt"

	"github.com/oasisprotocol/curve25519-voi/primitives/sr25519"

	"github.com/cometbft/cometbft/crypto"
)

var _ crypto.BatchVerifier = &BatchVerifier{}

// BatchVerifier implements batch verification for sr25519.
type BatchVerifier struct {
	*sr25519.BatchVerifier
}

func NewBatchVerifier() crypto.BatchVerifier {
	return &BatchVerifier{sr25519.NewBatchVerifier()}
}

func (b *BatchVerifier) Add(key crypto.PubKey, msg, signature []byte) error {
	pk, ok := key.(PubKey)
	if !ok {
		return fmt.Errorf("sr25519: pubkey is not sr25519")
	}

	var srpk sr25519.PublicKey
	if err := srpk.UnmarshalBinary(pk); err != nil {
		return fmt.Errorf("sr25519: invalid public key: %w", err)
	}

	var sig sr25519.Signature
	if err := sig.UnmarshalBinary(signature); err != nil {
		return fmt.Errorf("sr25519: unable to decode signature: %w", err)
	}

	st := signingCtx.NewTranscriptBytes(msg)
	b.BatchVerifier.Add(&srpk, st, &sig)

	return nil
}

func (b *BatchVerifier) Verify() (bool, []bool) {
	return b.BatchVerifier.Verify(crypto.CReader())
}
