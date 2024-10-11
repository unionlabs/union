package genutil

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/cometbft/cometbft/config"
	tmed25519 "github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/privval"
	"github.com/stretchr/testify/require"
)

func TestExportGenesisFileWithTime(t *testing.T) {
	t.Parallel()

	fname := filepath.Join(t.TempDir(), "genesis.json")

	require.NoError(t, ExportGenesisFileWithTime(fname, "test", nil, json.RawMessage(`{"account_owner": "Bob"}`), time.Now()))
}

func TestInitializeNodeValidatorFilesFromMnemonic(t *testing.T) {
	t.Parallel()

	cfg := config.TestConfig()
	cfg.RootDir = t.TempDir()
	require.NoError(t, os.MkdirAll(filepath.Join(cfg.RootDir, "config"), 0o755))

	tests := []struct {
		name     string
		mnemonic string
		expError bool
	}{
		{
			name:     "invalid mnemonic returns error",
			mnemonic: "side video kiss hotel essence",
			expError: true,
		},
		{
			name:     "empty mnemonic does not return error",
			mnemonic: "",
			expError: false,
		},
		{
			name:     "valid mnemonic does not return error",
			mnemonic: "side video kiss hotel essence door angle student degree during vague adjust submit trick globe muscle frozen vacuum artwork million shield bind useful wave",
			expError: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, _, err := InitializeNodeValidatorFilesFromMnemonic(cfg, tt.mnemonic, "ed25519")

			if tt.expError {
				require.Error(t, err)
			} else {
				require.NoError(t, err)

				if tt.mnemonic != "" {
					actualPVFile := privval.LoadFilePV(cfg.PrivValidatorKeyFile(), cfg.PrivValidatorStateFile())
					expectedPrivateKey := tmed25519.GenPrivKeyFromSecret([]byte(tt.mnemonic))
					expectedFile := privval.NewFilePV(expectedPrivateKey, cfg.PrivValidatorKeyFile(), cfg.PrivValidatorStateFile())
					require.Equal(t, expectedFile, actualPVFile)
				}
			}
		})
	}
}
