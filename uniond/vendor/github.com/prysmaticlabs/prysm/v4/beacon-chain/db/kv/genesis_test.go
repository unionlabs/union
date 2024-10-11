package kv

import (
	"context"
	"encoding/hex"
	"os"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/iface"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestStore_SaveGenesisData(t *testing.T) {
	ctx := context.Background()
	db := setupDB(t)

	gs, err := util.NewBeaconState()
	assert.NoError(t, err)

	assert.NoError(t, db.SaveGenesisData(ctx, gs))

	testGenesisDataSaved(t, db)
}

func testGenesisDataSaved(t *testing.T, db iface.Database) {
	ctx := context.Background()

	gb, err := db.GenesisBlock(ctx)
	require.NoError(t, err)
	require.NotNil(t, gb)

	gbHTR, err := gb.Block().HashTreeRoot()
	require.NoError(t, err)

	gss, err := db.StateSummary(ctx, gbHTR)
	require.NoError(t, err)
	require.NotNil(t, gss)

	head, err := db.HeadBlock(ctx)
	require.NoError(t, err)
	require.NotNil(t, head)

	headHTR, err := head.Block().HashTreeRoot()
	require.NoError(t, err)
	require.Equal(t, gbHTR, headHTR, "head block does not match genesis block")
}

func TestLoadCapellaFromFile(t *testing.T) {
	cfg, err := params.ByName(params.MainnetName)
	require.NoError(t, err)
	// This state fixture is from a hive testnet, `0a` is the suffix they are using in their fork versions.
	suffix, err := hex.DecodeString("0a")
	require.NoError(t, err)
	require.Equal(t, 1, len(suffix))
	reversioned := cfg.Copy()
	params.FillTestVersions(reversioned, suffix[0])
	reversioned.CapellaForkEpoch = 0
	require.Equal(t, [4]byte{3, 0, 0, 10}, bytesutil.ToBytes4(reversioned.CapellaForkVersion))
	reversioned.ConfigName = "capella-genesis-test"
	undo, err := params.SetActiveWithUndo(reversioned)
	require.NoError(t, err)
	defer func() {
		require.NoError(t, undo())
	}()

	fp := "testdata/capella_genesis.ssz"
	rfp, err := bazel.Runfile(fp)
	if err == nil {
		fp = rfp
	}
	sb, err := os.ReadFile(fp)
	require.NoError(t, err)

	db := setupDB(t)
	require.NoError(t, db.LoadGenesis(context.Background(), sb))
	testGenesisDataSaved(t, db)
}

func TestLoadGenesisFromFile(t *testing.T) {
	// for this test to work, we need the active config to have these properties:
	// - fork version schedule that matches mainnnet.genesis.ssz
	// - name that does not match params.MainnetName - otherwise we'll trigger the codepath that loads the state
	//   from the compiled binary.
	// to do that, first we need to rewrite the mainnet fork schedule so it won't conflict with a renamed config that
	// uses the mainnet fork schedule. construct the differently named mainnet config and set it active.
	// finally, revert all this at the end of the test.

	// first get the real mainnet out of the way by overwriting its schedule.
	cfg, err := params.ByName(params.MainnetName)
	require.NoError(t, err)
	cfg = cfg.Copy()
	reversioned := cfg.Copy()
	params.FillTestVersions(reversioned, 127)
	undo, err := params.SetActiveWithUndo(reversioned)
	require.NoError(t, err)
	defer func() {
		require.NoError(t, undo())
	}()

	// then set up a new config, which uses the real mainnet schedule, and activate it
	cfg.ConfigName = "genesis-test"
	undo2, err := params.SetActiveWithUndo(cfg)
	require.NoError(t, err)
	defer func() {
		require.NoError(t, undo2())
	}()

	fp := "testdata/mainnet.genesis.ssz"
	rfp, err := bazel.Runfile(fp)
	if err == nil {
		fp = rfp
	}
	sb, err := os.ReadFile(fp)
	require.NoError(t, err)

	db := setupDB(t)
	require.NoError(t, db.LoadGenesis(context.Background(), sb))
	testGenesisDataSaved(t, db)

	// Loading the same genesis again should not throw an error
	require.NoError(t, err)
	require.NoError(t, db.LoadGenesis(context.Background(), sb))
	testGenesisDataSaved(t, db)
}

func TestLoadGenesisFromFile_mismatchedForkVersion(t *testing.T) {
	fp := "testdata/altona.genesis.ssz"
	rfp, err := bazel.Runfile(fp)
	if err == nil {
		fp = rfp
	}
	sb, err := os.ReadFile(fp)
	assert.NoError(t, err)

	// Loading a genesis with the wrong fork version as beacon config should throw an error.
	db := setupDB(t)
	assert.ErrorContains(t, "not found in any known fork choice schedule", db.LoadGenesis(context.Background(), sb))
}

func TestEnsureEmbeddedGenesis(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	// Embedded Genesis works with Mainnet config
	cfg := params.MainnetConfig().Copy()
	cfg.SecondsPerSlot = 1
	undo, err := params.SetActiveWithUndo(cfg)
	require.NoError(t, err)
	defer func() {
		require.NoError(t, undo())
	}()

	ctx := context.Background()
	db := setupDB(t)

	gb, err := db.GenesisBlock(ctx)
	assert.NoError(t, err)
	if gb != nil && !gb.IsNil() {
		t.Fatal("Genesis block exists already")
	}

	gs, err := db.GenesisState(ctx)
	assert.NoError(t, err)
	assert.NotNil(t, gs, "an embedded genesis state does not exist")

	assert.NoError(t, db.EnsureEmbeddedGenesis(ctx))

	gb, err = db.GenesisBlock(ctx)
	assert.NoError(t, err)
	assert.NotNil(t, gb)

	testGenesisDataSaved(t, db)
}
