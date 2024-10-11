package abcicli_test

import (
	"fmt"
	"math/rand"
	"net"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"golang.org/x/net/context"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	abciserver "github.com/cometbft/cometbft/abci/server"
	"github.com/cometbft/cometbft/abci/types"
	cmtnet "github.com/cometbft/cometbft/internal/net"
	"github.com/cometbft/cometbft/libs/log"
)

func TestGRPC(t *testing.T) {
	app := types.NewBaseApplication()
	numCheckTxs := 2000
	socketFile := fmt.Sprintf("/tmp/test-%08x.sock", rand.Int31n(1<<30))
	defer os.Remove(socketFile)
	socket := fmt.Sprintf("unix://%v", socketFile)

	// Start the listener
	server := abciserver.NewGRPCServer(socket, app)
	server.SetLogger(log.TestingLogger().With("module", "abci-server"))
	err := server.Start()
	require.NoError(t, err)

	t.Cleanup(func() {
		if err := server.Stop(); err != nil {
			t.Error(err)
		}
	})

	// Connect to the socket
	conn, err := grpc.NewClient(socket, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithContextDialer(dialerFunc))
	require.NoError(t, err)

	t.Cleanup(func() {
		if err := conn.Close(); err != nil {
			t.Error(err)
		}
	})

	client := types.NewABCIClient(conn)

	// Write requests
	for counter := 0; counter < numCheckTxs; counter++ {
		// Send request
		response, err := client.CheckTx(
			context.Background(),
			&types.CheckTxRequest{
				Tx:   []byte("test"),
				Type: types.CHECK_TX_TYPE_CHECK,
			})
		require.NoError(t, err)
		counter++
		if response.Code != 0 {
			t.Error("CheckTx failed with ret_code", response.Code)
		}
		if counter > numCheckTxs {
			t.Fatal("Too many CheckTx responses")
		}
		t.Log("response", counter)
		if counter == numCheckTxs {
			go func() {
				time.Sleep(time.Second * 1) // Wait for a bit to allow counter overflow
			}()
		}
	}
}

func dialerFunc(_ context.Context, addr string) (net.Conn, error) {
	return cmtnet.Connect(addr)
}
