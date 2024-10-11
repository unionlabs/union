package signing

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestDomain_OK(t *testing.T) {
	state := &eth.BeaconState{
		Fork: &eth.Fork{
			Epoch:           3,
			PreviousVersion: []byte{0, 0, 0, 2},
			CurrentVersion:  []byte{0, 0, 0, 3},
		},
	}
	tests := []struct {
		epoch      primitives.Epoch
		domainType [4]byte
		result     []byte
	}{
		{epoch: 1, domainType: bytesutil.ToBytes4(bytesutil.Bytes4(4)), result: bytesutil.ToBytes(947067381421703172, 32)},
		{epoch: 2, domainType: bytesutil.ToBytes4(bytesutil.Bytes4(4)), result: bytesutil.ToBytes(947067381421703172, 32)},
		{epoch: 2, domainType: bytesutil.ToBytes4(bytesutil.Bytes4(5)), result: bytesutil.ToBytes(947067381421703173, 32)},
		{epoch: 3, domainType: bytesutil.ToBytes4(bytesutil.Bytes4(4)), result: bytesutil.ToBytes(9369798235163459588, 32)},
		{epoch: 3, domainType: bytesutil.ToBytes4(bytesutil.Bytes4(5)), result: bytesutil.ToBytes(9369798235163459589, 32)},
	}
	for _, tt := range tests {
		domain, err := Domain(state.Fork, tt.epoch, tt.domainType, nil)
		require.NoError(t, err)
		assert.DeepEqual(t, tt.result[:8], domain[:8], "Unexpected domain version")
	}
}
