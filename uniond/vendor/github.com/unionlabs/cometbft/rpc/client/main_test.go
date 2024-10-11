package client_test

import (
	"os"
	"testing"

	"github.com/cometbft/cometbft/abci/example/kvstore"
	nm "github.com/cometbft/cometbft/node"
	rpctest "github.com/cometbft/cometbft/rpc/test"
)

var node *nm.Node

func TestMain(m *testing.M) {
	// start a CometBFT node (and kvstore) in the background to test against
	dir, err := os.MkdirTemp("/tmp", "rpc-client-test")
	if err != nil {
		panic(err)
	}

	app := kvstore.NewPersistentApplication(dir)
	// If testing block event generation
	// app.SetGenBlockEvents() // needs to be called here (see TestBlockSearch in rpc_test.go)
	node = rpctest.StartCometBFT(app)

	code := m.Run()

	// and shut down proper at the end
	rpctest.StopCometBFT(node)
	_ = os.RemoveAll(dir)
	os.Exit(code)
}
