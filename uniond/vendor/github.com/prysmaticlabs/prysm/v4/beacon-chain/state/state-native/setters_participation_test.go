package state_native_test

import (
	"testing"

	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func BenchmarkParticipationBits(b *testing.B) {
	st, err := state_native.InitializeFromProtoCapella(&ethpb.BeaconStateCapella{})
	require.NoError(b, err)

	max := uint64(16777216)
	for i := uint64(0); i < max-2; i++ {
		require.NoError(b, st.AppendCurrentParticipationBits(byte(1)))
	}

	ref := st.Copy()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		require.NoError(b, ref.AppendCurrentParticipationBits(byte(2)))
		ref = st.Copy()
	}
}
