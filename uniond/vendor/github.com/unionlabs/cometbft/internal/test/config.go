package test

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/cometbft/cometbft/config"
	cmtos "github.com/cometbft/cometbft/internal/os"
)

func ResetTestRoot(testName string) *config.Config {
	return resetTestRoot(testName, "", true)
}

func ResetTestRootWithChainID(testName string, chainID string) *config.Config {
	return resetTestRoot(testName, chainID, true)
}

func ResetTestRootWithChainIDNoOverwritePrivval(testName string, chainID string) *config.Config {
	return resetTestRoot(testName, chainID, false)
}

func resetTestRoot(testName string, chainID string, overwritePrivKey bool) *config.Config {
	// create a unique, concurrency-safe test directory under os.TempDir()
	rootDir, err := os.MkdirTemp("", fmt.Sprintf("%s-%s_", chainID, testName))
	if err != nil {
		panic(err)
	}

	config.EnsureRoot(rootDir)

	baseConfig := config.DefaultBaseConfig()
	genesisFilePath := filepath.Join(rootDir, baseConfig.Genesis)
	privKeyFilePath := filepath.Join(rootDir, baseConfig.PrivValidatorKey)
	privStateFilePath := filepath.Join(rootDir, baseConfig.PrivValidatorState)

	if !cmtos.FileExists(genesisFilePath) {
		if chainID == "" {
			chainID = DefaultTestChainID
		}
		testGenesis := fmt.Sprintf(testGenesisFmt, chainID)
		cmtos.MustWriteFile(genesisFilePath, []byte(testGenesis), 0o644)
	}
	if overwritePrivKey {
		cmtos.MustWriteFile(privKeyFilePath, []byte(testPrivValidatorKey), 0o644)
	}
	cmtos.MustWriteFile(privStateFilePath, []byte(testPrivValidatorState), 0o644)

	config := config.TestConfig().SetRoot(rootDir)
	return config
}

var testGenesisFmt = `{
  "genesis_time": "2018-10-10T08:20:13.695936996Z",
  "chain_id": "%s",
  "initial_height": "1",
  "consensus_params": {
		"block": {
			"max_bytes": "22020096",
			"max_gas": "-1",
			"time_iota_ms": "10"
		},
		"synchrony": {
			"message_delay": "500000000",
			"precision": "10000000"
		},
		"evidence": {
			"max_age_num_blocks": "100000",
			"max_age_duration": "172800000000000",
			"max_bytes": "1048576"
		},
		"validator": {
			"pub_key_types": [
				"ed25519"
			]
		},
		"abci": {
			"vote_extensions_enable_height": "0"
		},
		"version": {},
		"feature": {
			"vote_extensions_enable_height": "0",
			"pbts_enable_height": "1"
		}
  },
  "validators": [
    {
      "pub_key": {
        "type": "tendermint/PubKeyEd25519",
        "value":"AT/+aaL1eB0477Mud9JMm8Sh8BIvOYlPGC9KkIUmFaE="
      },
      "power": "10",
      "name": ""
    }
  ],
  "app_hash": ""
}`

var testPrivValidatorKey = `{
  "address": "A3258DCBF45DCA0DF052981870F2D1441A36D145",
  "pub_key": {
    "type": "tendermint/PubKeyEd25519",
    "value": "AT/+aaL1eB0477Mud9JMm8Sh8BIvOYlPGC9KkIUmFaE="
  },
  "priv_key": {
    "type": "tendermint/PrivKeyEd25519",
    "value": "EVkqJO/jIXp3rkASXfh9YnyToYXRXhBr6g9cQVxPFnQBP/5povV4HTjvsy530kybxKHwEi85iU8YL0qQhSYVoQ=="
  }
}`

var testPrivValidatorState = `{
  "height": "0",
  "round": 0,
  "step": 0
}`
