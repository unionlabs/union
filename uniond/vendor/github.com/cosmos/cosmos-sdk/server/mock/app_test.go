package mock

import (
	"math/rand"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cometbft/cometbft/types"
	"github.com/stretchr/testify/require"

	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

func TestInitApp(t *testing.T) {
	app, closer, err := SetupApp()
	// closer may need to be run, even when error in later stage
	if closer != nil {
		defer closer()
	}
	require.NoError(t, err)

	appState, err := AppGenState(nil, types.GenesisDoc{}, nil)
	require.NoError(t, err)

	req := abci.RequestInitChain{
		AppStateBytes: appState,
	}
	app.InitChain(req)
	app.Commit()

	// make sure we can query these values
	query := abci.RequestQuery{
		Path: "/store/main/key",
		Data: []byte("foo"),
	}

	qres := app.Query(query)
	require.Equal(t, uint32(0), qres.Code, qres.Log)
	require.Equal(t, []byte("bar"), qres.Value)
}

func TestDeliverTx(t *testing.T) {
	app, closer, err := SetupApp()
	// closer may need to be run, even when error in later stage
	if closer != nil {
		defer closer()
	}
	require.NoError(t, err)

	key := "my-special-key"
	value := "top-secret-data!!"

	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	randomAccounts := simtypes.RandomAccounts(r, 1)

	tx := NewTx(key, value, randomAccounts[0].Address)
	txBytes := tx.GetSignBytes()

	app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{
		AppHash: []byte("apphash"),
		Height:  1,
	}})

	dres := app.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
	require.Equal(t, uint32(0), dres.Code, dres.Log)

	app.EndBlock(abci.RequestEndBlock{})

	cres := app.Commit()
	require.NotEmpty(t, cres.Data)

	// make sure we can query these values
	query := abci.RequestQuery{
		Path: "/store/main/key",
		Data: []byte(key),
	}

	qres := app.Query(query)
	require.Equal(t, uint32(0), qres.Code, qres.Log)
	require.Equal(t, []byte(value), qres.Value)
}
