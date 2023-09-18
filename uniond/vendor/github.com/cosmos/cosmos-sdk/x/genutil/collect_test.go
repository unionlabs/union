package genutil_test

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"

	"github.com/cosmos/gogoproto/proto"

	tmtypes "github.com/cometbft/cometbft/types"

	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/server"
	"github.com/cosmos/cosmos-sdk/types"
	bankexported "github.com/cosmos/cosmos-sdk/x/bank/exported"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	gtypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
)

type doNothingUnmarshalJSON struct {
	codec.JSONCodec
}

func (dnj *doNothingUnmarshalJSON) UnmarshalJSON(_ []byte, _ proto.Message) error {
	return nil
}

type doNothingIterator struct {
	gtypes.GenesisBalancesIterator
}

func (dni *doNothingIterator) IterateGenesisBalances(_ codec.JSONCodec, _ map[string]json.RawMessage, _ func(bankexported.GenesisBalance) bool) {
}

// Ensures that CollectTx correctly traverses directories and won't error out on encountering
// a directory during traversal of the first level. See issue https://github.com/cosmos/cosmos-sdk/issues/6788.
func TestCollectTxsHandlesDirectories(t *testing.T) {
	testDir, err := os.MkdirTemp(os.TempDir(), "testCollectTxs")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(testDir)

	// 1. We'll insert a directory as the first element before JSON file.
	subDirPath := filepath.Join(testDir, "_adir")
	if err := os.MkdirAll(subDirPath, 0o755); err != nil {
		t.Fatal(err)
	}

	txDecoder := types.TxDecoder(func(txBytes []byte) (types.Tx, error) {
		return nil, nil
	})

	// 2. Ensure that we don't encounter any error traversing the directory.
	srvCtx := server.NewDefaultContext()
	_ = srvCtx
	cdc := codec.NewProtoCodec(cdctypes.NewInterfaceRegistry())
	gdoc := tmtypes.GenesisDoc{AppState: []byte("{}")}
	balItr := new(doNothingIterator)

	dnc := &doNothingUnmarshalJSON{cdc}
	if _, _, err := genutil.CollectTxs(dnc, txDecoder, "foo", testDir, gdoc, balItr, gtypes.DefaultMessageValidator); err != nil {
		t.Fatal(err)
	}
}
