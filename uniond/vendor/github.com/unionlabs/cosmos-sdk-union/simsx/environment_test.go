package simsx

import (
	"math/rand"
	"testing"

	"github.com/stretchr/testify/assert"

	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

func TestChainDataSourceAnyAccount(t *testing.T) {
	codec := txConfig().SigningContext().AddressCodec()
	r := rand.New(rand.NewSource(1))
	accs := simtypes.RandomAccounts(r, 3)
	specs := map[string]struct {
		filters []SimAccountFilter
		assert  func(t *testing.T, got SimAccount, reporter SimulationReporter)
	}{
		"no filters": {
			assert: func(t *testing.T, got SimAccount, reporter SimulationReporter) { //nolint:thelper // not a helper
				assert.NotEmpty(t, got.AddressBech32)
				assert.False(t, reporter.IsSkipped())
			},
		},
		"filter": {
			filters: []SimAccountFilter{SimAccountFilterFn(func(a SimAccount) bool { return a.AddressBech32 == accs[2].AddressBech32 })},
			assert: func(t *testing.T, got SimAccount, reporter SimulationReporter) { //nolint:thelper // not a helper
				assert.Equal(t, accs[2].AddressBech32, got.AddressBech32)
				assert.False(t, reporter.IsSkipped())
			},
		},
		"no match": {
			filters: []SimAccountFilter{SimAccountFilterFn(func(a SimAccount) bool { return false })},
			assert: func(t *testing.T, got SimAccount, reporter SimulationReporter) { //nolint:thelper // not a helper
				assert.Empty(t, got.AddressBech32)
				assert.True(t, reporter.IsSkipped())
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			reporter := NewBasicSimulationReporter()
			c := NewChainDataSource(sdk.Context{}, r, nil, nil, codec, accs...)
			a := c.AnyAccount(reporter, spec.filters...)
			spec.assert(t, a, reporter)
		})
	}
}

func TestChainDataSourceGetHasAccount(t *testing.T) {
	codec := txConfig().SigningContext().AddressCodec()
	r := rand.New(rand.NewSource(1))
	accs := simtypes.RandomAccounts(r, 3)
	reporter := NewBasicSimulationReporter()
	c := NewChainDataSource(sdk.Context{}, r, nil, nil, codec, accs...)
	exisingAddr := accs[0].AddressBech32
	assert.Equal(t, exisingAddr, c.GetAccount(reporter, exisingAddr).AddressBech32)
	assert.False(t, reporter.IsSkipped())
	assert.True(t, c.HasAccount(exisingAddr))
	// and non-existing account
	reporter = NewBasicSimulationReporter()
	assert.Empty(t, c.GetAccount(reporter, "non-existing").AddressBech32)
	assert.False(t, c.HasAccount("non-existing"))
	assert.True(t, reporter.IsSkipped())
}
