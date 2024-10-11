package light_test

import (
	"context"
	"fmt"
	stdlog "log"
	"os"
	"testing"
	"time"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/abci/example/kvstore"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/light"
	"github.com/cometbft/cometbft/light/provider"
	httpp "github.com/cometbft/cometbft/light/provider/http"
	dbs "github.com/cometbft/cometbft/light/store/db"
	rpctest "github.com/cometbft/cometbft/rpc/test"
	cmttime "github.com/cometbft/cometbft/types/time"
)

// Automatically getting new headers and verifying them.
func ExampleClient_Update() {
	// give CometBFT time to generate some blocks
	time.Sleep(5 * time.Second)

	dbDir, err := os.MkdirTemp("", "light-client-example")
	if err != nil {
		stdlog.Fatal(err)
	}
	defer os.RemoveAll(dbDir)

	config := rpctest.GetConfig()

	primary, err := httpp.New(chainID, config.RPC.ListenAddress)
	if err != nil {
		stdlog.Fatal(err)
	}

	block, err := primary.LightBlock(context.Background(), 2)
	if err != nil {
		stdlog.Fatal(err)
	}

	db, err := dbm.NewGoLevelDB("light-client-db", dbDir)
	if err != nil {
		stdlog.Fatal(err)
	}

	c, err := light.NewClient(
		context.Background(),
		chainID,
		light.TrustOptions{
			Period: 504 * time.Hour, // 21 days
			Height: 2,
			Hash:   block.Hash(),
		},
		primary,
		[]provider.Provider{primary}, // NOTE: primary should not be used here
		dbs.New(db, chainID),
		light.Logger(log.TestingLogger()),
	)
	if err != nil {
		stdlog.Fatal(err)
	}
	defer func() {
		if err := c.Cleanup(); err != nil {
			stdlog.Fatal(err)
		}
	}()

	time.Sleep(2 * time.Second)

	h, err := c.Update(context.Background(), cmttime.Now())
	if err != nil {
		stdlog.Fatal(err)
	}

	if h != nil && h.Height > 2 {
		fmt.Println("successful update")
	} else {
		fmt.Println("update failed")
	}
	// Output: successful update
}

// Manually getting light blocks and verifying them.
func ExampleClient_VerifyLightBlockAtHeight() {
	// give CometBFT time to generate some blocks
	time.Sleep(5 * time.Second)

	dbDir, err := os.MkdirTemp("", "light-client-example")
	if err != nil {
		stdlog.Fatal(err)
	}
	defer os.RemoveAll(dbDir)

	config := rpctest.GetConfig()

	primary, err := httpp.New(chainID, config.RPC.ListenAddress)
	if err != nil {
		stdlog.Fatal(err)
	}

	block, err := primary.LightBlock(context.Background(), 2)
	if err != nil {
		stdlog.Fatal(err)
	}

	db, err := dbm.NewGoLevelDB("light-client-db", dbDir)
	if err != nil {
		stdlog.Fatal(err)
	}

	c, err := light.NewClient(
		context.Background(),
		chainID,
		light.TrustOptions{
			Period: 504 * time.Hour, // 21 days
			Height: 2,
			Hash:   block.Hash(),
		},
		primary,
		[]provider.Provider{primary}, // NOTE: primary should not be used here
		dbs.New(db, chainID),
		light.Logger(log.TestingLogger()),
	)
	if err != nil {
		stdlog.Fatal(err)
	}
	defer func() {
		if err := c.Cleanup(); err != nil {
			stdlog.Fatal(err)
		}
	}()

	_, err = c.VerifyLightBlockAtHeight(context.Background(), 3, cmttime.Now())
	if err != nil {
		stdlog.Fatal(err)
	}

	h, err := c.TrustedLightBlock(3)
	if err != nil {
		stdlog.Fatal(err)
	}

	fmt.Println("got header", h.Height)
	// Output: got header 3
}

func TestMain(m *testing.M) {
	// start a CometBFT node (and kvstore) in the background to test against
	app := kvstore.NewInMemoryApplication()
	node := rpctest.StartCometBFT(app, rpctest.SuppressStdout)

	code := m.Run()

	// and shut down proper at the end
	rpctest.StopCometBFT(node)
	os.Exit(code)
}
