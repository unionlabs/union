package main

import (
	"fmt"
	"math/rand"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
)

// TestGenerator tests that only valid manifests are generated.
func TestGenerator(t *testing.T) {
	cfg := &generateConfig{
		randSource: rand.New(rand.NewSource(randomSeed)),
	}
	manifests, err := Generate(cfg)
	require.NoError(t, err)

	for idx, m := range manifests {
		t.Run(fmt.Sprintf("Case%04d", idx), func(t *testing.T) {
			infra, err := e2e.NewDockerInfrastructureData(m)
			require.NoError(t, err)
			_, err = e2e.NewTestnetFromManifest(m, filepath.Join(t.TempDir(), fmt.Sprintf("Case%04d", idx)), infra, "")
			require.NoError(t, err)
		})
	}
}

func TestVersionFinder(t *testing.T) {
	testCases := []struct {
		baseVer        string
		tags           []string
		expectedLatest string
	}{
		{
			baseVer:        "v0.34.0",
			tags:           []string{"v0.34.0", "v0.34.1", "v0.34.2", "v0.34.3-rc1", "v0.34.3", "v0.35.0", "v0.35.1", "v0.36.0-rc1"},
			expectedLatest: "v0.34.3",
		},
		{
			baseVer:        "v0.38.0-dev",
			tags:           []string{"v0.34.0", "v0.34.1", "v0.34.2", "v0.37.0-rc2", "dev-v0.38.0"},
			expectedLatest: "",
		},
		{
			baseVer:        "v0.37.1-rc1",
			tags:           []string{"v0.36.0", "v0.37.0-rc1", "v0.37.0"},
			expectedLatest: "v0.37.0",
		},
		{
			baseVer:        "v1.0.0",
			tags:           []string{"v0.34.0", "v0.35.0", "v1.0.0", "v1.0.1"},
			expectedLatest: "v1.0.1",
		},
		{
			baseVer:        "v1.1.5",
			tags:           []string{"v0.35.0", "v1.0.0", "v1.0.1", "v1.1.1", "v1.1.2", "v1.1.3", "v1.1.4"},
			expectedLatest: "v1.1.4",
		},
	}
	for _, tc := range testCases {
		actualLatest, err := findLatestReleaseTag(tc.baseVer, tc.tags)
		require.NoError(t, err)
		assert.Equal(t, tc.expectedLatest, actualLatest)
	}
}
