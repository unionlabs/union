package mock

import (
	"context"
	"math/rand"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/log"

	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
)

// SetupApp initializes a new application,
// failing t if initialization fails.
func SetupApp(t *testing.T) servertypes.ABCI {
	t.Helper()

	logger := log.NewTestLogger(t)

	rootDir := t.TempDir()

	app, err := NewApp(rootDir, logger)
	require.NoError(t, err)

	return app
}

func TestInitApp(t *testing.T) {
	app := SetupApp(t)

	appState, err := AppGenState(nil, genutiltypes.AppGenesis{}, nil)
	require.NoError(t, err)

	req := abci.InitChainRequest{
		AppStateBytes: appState,
	}
	res, err := app.InitChain(&req)
	require.NoError(t, err)
	_, err = app.FinalizeBlock(&abci.FinalizeBlockRequest{
		Hash:   res.AppHash,
		Height: 1,
	})
	require.NoError(t, err)

	_, err = app.Commit()
	require.NoError(t, err)

	// make sure we can query these values
	query := abci.QueryRequest{
		Path: "/store/main/key",
		Data: []byte("foo"),
	}

	qres, err := app.Query(context.Background(), &query)
	require.NoError(t, err)
	require.Equal(t, uint32(0), qres.Code, qres.Log)
	require.Equal(t, []byte("bar"), qres.Value)
}

func TestDeliverTx(t *testing.T) {
	app := SetupApp(t)

	key := "my-special-key"
	value := "top-secret-data!!"

	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	randomAccounts := simtypes.RandomAccounts(r, 1)

	tx := NewTx(key, value, randomAccounts[0].Address)
	txBytes := tx.GetSignBytes()

	res, err := app.FinalizeBlock(&abci.FinalizeBlockRequest{
		Hash:   []byte("apphash"),
		Height: 1,
		Txs:    [][]byte{txBytes},
	})
	require.NoError(t, err)
	require.NotEmpty(t, res.AppHash)

	_, err = app.Commit()
	require.NoError(t, err)

	// make sure we can query these values
	query := abci.QueryRequest{
		Path: "/store/main/key",
		Data: []byte(key),
	}

	qres, err := app.Query(context.Background(), &query)
	require.NoError(t, err)
	require.Equal(t, uint32(0), qres.Code, qres.Log)
	require.Equal(t, []byte(value), qres.Value)
}
