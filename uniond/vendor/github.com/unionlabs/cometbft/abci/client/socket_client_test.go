package abcicli_test

import (
	"context"
	"fmt"
	"math/rand"
	"os"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	abcicli "github.com/cometbft/cometbft/abci/client"
	"github.com/cometbft/cometbft/abci/server"
	"github.com/cometbft/cometbft/abci/types"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/libs/service"
)

func TestCalls(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	app := types.BaseApplication{}

	_, c := setupClientServer(t, app)

	resp := make(chan error, 1)
	go func() {
		res, err := c.Echo(ctx, "hello")
		require.NoError(t, err)
		require.NotNil(t, res)
		resp <- c.Error()
	}()

	select {
	case <-time.After(time.Second):
		require.Fail(t, "No response arrived")
	case err, ok := <-resp:
		require.True(t, ok, "Must not close channel")
		require.NoError(t, err)
	}
}

func TestHangingAsyncCalls(t *testing.T) {
	app := slowApp{}

	s, c := setupClientServer(t, app)

	resp := make(chan error, 1)
	go func() {
		// Call CheckTx
		reqres, err := c.CheckTxAsync(context.Background(), &types.CheckTxRequest{
			Type: types.CHECK_TX_TYPE_CHECK,
		})
		require.NoError(t, err)
		// wait 50 ms for all events to travel socket, but
		// no response yet from server
		time.Sleep(50 * time.Millisecond)
		// kill the server, so the connections break
		err = s.Stop()
		require.NoError(t, err)

		// wait for the response from CheckTx
		reqres.Wait()
		resp <- c.Error()
	}()

	select {
	case <-time.After(time.Second):
		require.Fail(t, "No response arrived")
	case err, ok := <-resp:
		require.True(t, ok, "Must not close channel")
		require.Error(t, err, "We should get EOF error")
	}
}

func TestBulk(t *testing.T) {
	const numTxs = 700000
	// use a socket instead of a port
	socketFile := fmt.Sprintf("test-%08x.sock", rand.Int31n(1<<30))
	defer os.Remove(socketFile)
	socket := fmt.Sprintf("unix://%v", socketFile)
	app := types.NewBaseApplication()
	// Start the listener
	server := server.NewSocketServer(socket, app)
	t.Cleanup(func() {
		if err := server.Stop(); err != nil {
			t.Log(err)
		}
	})
	err := server.Start()
	require.NoError(t, err)

	// Connect to the socket
	client := abcicli.NewSocketClient(socket, false)

	t.Cleanup(func() {
		if err := client.Stop(); err != nil {
			t.Log(err)
		}
	})

	err = client.Start()
	require.NoError(t, err)

	// Construct request
	rfb := &types.FinalizeBlockRequest{Txs: make([][]byte, numTxs)}
	for counter := 0; counter < numTxs; counter++ {
		rfb.Txs[counter] = []byte("test")
	}
	// Send bulk request
	res, err := client.FinalizeBlock(context.Background(), rfb)
	require.NoError(t, err)
	require.Len(t, res.TxResults, numTxs, "Number of txs doesn't match")
	for _, tx := range res.TxResults {
		require.Equal(t, uint32(0), tx.Code, "Tx failed")
	}

	// Send final flush message
	err = client.Flush(context.Background())
	require.NoError(t, err)
}

func setupClientServer(t *testing.T, app types.Application) (
	service.Service, abcicli.Client,
) {
	t.Helper()

	// some port between 20k and 30k
	port := 20000 + cmtrand.Int32()%10000
	addr := fmt.Sprintf("localhost:%d", port)

	s := server.NewSocketServer(addr, app)
	err := s.Start()
	require.NoError(t, err)

	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Log(err)
		}
	})

	c := abcicli.NewSocketClient(addr, true)
	err = c.Start()
	require.NoError(t, err)

	t.Cleanup(func() {
		if err := c.Stop(); err != nil {
			t.Log(err)
		}
	})

	return s, c
}

type slowApp struct {
	types.BaseApplication
}

func (slowApp) CheckTx(context.Context, *types.CheckTxRequest) (*types.CheckTxResponse, error) {
	time.Sleep(time.Second)
	return &types.CheckTxResponse{}, nil
}

// TestCallbackInvokedWhenSetLaet ensures that the callback is invoked when
// set after the client completes the call into the app. Currently this
// test relies on the callback being allowed to be invoked twice if set multiple
// times, once when set early and once when set late.
func TestCallbackInvokedWhenSetLate(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	wg := &sync.WaitGroup{}
	wg.Add(1)
	app := blockedABCIApplication{
		wg: wg,
	}
	_, c := setupClientServer(t, app)
	reqRes, err := c.CheckTxAsync(ctx, &types.CheckTxRequest{
		Type: types.CHECK_TX_TYPE_CHECK,
	})
	require.NoError(t, err)

	done := make(chan struct{})
	cb := func(_ *types.Response) {
		close(done)
	}
	reqRes.SetCallback(cb)
	app.wg.Done()
	<-done

	var called bool
	cb = func(_ *types.Response) {
		called = true
	}
	reqRes.SetCallback(cb)
	require.True(t, called)
}

type blockedABCIApplication struct {
	wg *sync.WaitGroup
	types.BaseApplication
}

func (b blockedABCIApplication) CheckTxAsync(ctx context.Context, r *types.CheckTxRequest) (*types.CheckTxResponse, error) {
	b.wg.Wait()
	return b.BaseApplication.CheckTx(ctx, r)
}

// TestCallbackInvokedWhenSetEarly ensures that the callback is invoked when
// set before the client completes the call into the app.
func TestCallbackInvokedWhenSetEarly(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	wg := &sync.WaitGroup{}
	wg.Add(1)
	app := blockedABCIApplication{
		wg: wg,
	}
	_, c := setupClientServer(t, app)
	reqRes, err := c.CheckTxAsync(ctx, &types.CheckTxRequest{
		Type: types.CHECK_TX_TYPE_CHECK,
	})
	require.NoError(t, err)

	done := make(chan struct{})
	cb := func(_ *types.Response) {
		close(done)
	}
	reqRes.SetCallback(cb)
	app.wg.Done()

	called := func() bool {
		select {
		case <-done:
			return true
		default:
			return false
		}
	}
	require.Eventually(t, called, time.Second, time.Millisecond*25)
}
