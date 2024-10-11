package types_test

import (
	"encoding/json"
	"os"
	"testing"

	cmttypes "github.com/cometbft/cometbft/types"
	"gotest.tools/v3/assert"
	"gotest.tools/v3/golden"

	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

func TestAppGenesis_Marshal(t *testing.T) {
	genesis := types.AppGenesis{
		AppName:    "simapp",
		AppVersion: "0.1.0",
		ChainID:    "test",
	}

	out, err := json.Marshal(&genesis)
	assert.NilError(t, err)
	assert.Equal(t, string(out), `{"app_name":"simapp","app_version":"0.1.0","genesis_time":"0001-01-01T00:00:00Z","chain_id":"test","initial_height":0,"app_hash":null}`)
}

func TestAppGenesis_Unmarshal(t *testing.T) {
	jsonBlob, err := os.ReadFile("testdata/app_genesis.json")
	assert.NilError(t, err)

	var genesis types.AppGenesis
	err = json.Unmarshal(jsonBlob, &genesis)
	assert.NilError(t, err)

	assert.DeepEqual(t, genesis.ChainID, "demo")
	assert.DeepEqual(t, genesis.Consensus.Params.Block.MaxBytes, int64(22020096))
}

func TestAppGenesis_ValidGenesis(t *testing.T) {
	// validate can read cometbft genesis file
	genesis, err := types.AppGenesisFromFile("testdata/cmt_genesis.json")
	assert.NilError(t, err)

	assert.DeepEqual(t, genesis.ChainID, "demo")
	assert.DeepEqual(t, genesis.Consensus.Validators[0].Name, "test")

	// validate the app genesis can be translated properly to cometbft genesis
	cmtGenesis, err := genesis.ToGenesisDoc()
	assert.NilError(t, err)
	rawCmtGenesis, err := cmttypes.GenesisDocFromFile("testdata/cmt_genesis.json")
	assert.NilError(t, err)
	assert.DeepEqual(t, cmtGenesis, rawCmtGenesis)

	// validate can properly marshal to app genesis file
	rawAppGenesis, err := json.Marshal(&genesis)
	assert.NilError(t, err)
	golden.Assert(t, string(rawAppGenesis), "app_genesis.json")

	// validate the app genesis can be unmarshalled properly
	var appGenesis types.AppGenesis
	err = json.Unmarshal(rawAppGenesis, &appGenesis)
	assert.NilError(t, err)
	assert.DeepEqual(t, appGenesis.Consensus.Params, genesis.Consensus.Params)

	// validate marshaling of app genesis
	rawAppGenesis, err = json.Marshal(&appGenesis)
	assert.NilError(t, err)
	golden.Assert(t, string(rawAppGenesis), "app_genesis.json")
}
