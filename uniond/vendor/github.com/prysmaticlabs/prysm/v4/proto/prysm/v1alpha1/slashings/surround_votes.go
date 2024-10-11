package slashings

import ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"

// IsSurround checks if an attestation, a, is surrounding
// another one, b, based on the Ethereum slashing conditions specified
// by @protolambda https://github.com/protolambda/eth2-surround#definition.
//
//	s: source
//	t: target
//
//	a surrounds b if: s_a < s_b and t_b < t_a
func IsSurround(a, b *ethpb.IndexedAttestation) bool {
	return a.Data.Source.Epoch < b.Data.Source.Epoch && b.Data.Target.Epoch < a.Data.Target.Epoch
}
