package gateway

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/api/gateway"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestDefaultConfig(t *testing.T) {
	t.Run("Without Prysm API", func(t *testing.T) {
		cfg := DefaultConfig(true, "eth")
		assert.NotNil(t, cfg.EthPbMux.Mux)
		require.Equal(t, 2, len(cfg.EthPbMux.Patterns))
		assert.Equal(t, "/internal/eth/v1/", cfg.EthPbMux.Patterns[0])
		assert.Equal(t, 0, len(cfg.EthPbMux.Registrations))
		assert.Equal(t, (*gateway.PbMux)(nil), cfg.V1AlphaPbMux)
	})
	t.Run("Without Eth API", func(t *testing.T) {
		cfg := DefaultConfig(true, "prysm")
		assert.Equal(t, (*gateway.PbMux)(nil), cfg.EthPbMux)
		assert.NotNil(t, cfg.V1AlphaPbMux.Mux)
		require.Equal(t, 2, len(cfg.V1AlphaPbMux.Patterns))
		assert.Equal(t, "/eth/v1alpha1/", cfg.V1AlphaPbMux.Patterns[0])
		assert.Equal(t, "/eth/v1alpha2/", cfg.V1AlphaPbMux.Patterns[1])
		assert.Equal(t, 5, len(cfg.V1AlphaPbMux.Registrations))
	})
}
