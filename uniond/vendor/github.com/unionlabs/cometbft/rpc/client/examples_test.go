package client_test

import (
	"bytes"
	"context"
	"fmt"
	"log"
	"strings"

	"github.com/cometbft/cometbft/abci/example/kvstore"
	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
	ctypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/cometbft/cometbft/rpc/jsonrpc/types"
	rpctest "github.com/cometbft/cometbft/rpc/test"
)

func ExampleHTTP_simple() {
	// Start a CometBFT node (and kvstore) in the background to test against
	app := kvstore.NewInMemoryApplication()
	node := rpctest.StartCometBFT(app, rpctest.SuppressStdout, rpctest.RecreateConfig)
	defer rpctest.StopCometBFT(node)

	// Create our RPC client
	rpcAddr := rpctest.GetConfig().RPC.ListenAddress
	c, err := rpchttp.New(rpcAddr)
	if err != nil {
		log.Fatal(err)
	}

	// Create a transaction
	k := []byte("name")
	v := []byte("satoshi")
	tx := append(k, append([]byte("="), v...)...)

	// Broadcast the transaction and wait for it to commit (rather use
	// c.BroadcastTxSync though in production).
	bres, err := c.BroadcastTxCommit(context.Background(), tx)
	if err != nil {
		log.Fatal(err)
	}
	if bres.CheckTx.IsErr() || bres.TxResult.IsErr() {
		log.Fatal("BroadcastTxCommit transaction failed")
	}

	// Now try to fetch the value for the key
	qres, err := c.ABCIQuery(context.Background(), "/key", k)
	if err != nil {
		log.Fatal(err)
	}
	if qres.Response.IsErr() {
		log.Fatal("ABCIQuery failed")
	}
	if !bytes.Equal(qres.Response.Key, k) {
		log.Fatal("returned key does not match queried key")
	}
	if !bytes.Equal(qres.Response.Value, v) {
		log.Fatal("returned value does not match sent value")
	}

	fmt.Println("Sent tx     :", string(tx))
	fmt.Println("Queried for :", string(qres.Response.Key))
	fmt.Println("Got value   :", string(qres.Response.Value))

	// Output:
	// Sent tx     : name=satoshi
	// Queried for : name
	// Got value   : satoshi
}

func ExampleHTTP_batching() {
	// Start a CometBFT node (and kvstore) in the background to test against
	app := kvstore.NewInMemoryApplication()
	node := rpctest.StartCometBFT(app, rpctest.SuppressStdout, rpctest.RecreateConfig)

	// Create our RPC client
	rpcAddr := rpctest.GetConfig().RPC.ListenAddress
	c, err := rpchttp.New(rpcAddr)
	if err != nil {
		log.Fatal(err)
	}

	defer rpctest.StopCometBFT(node)

	// Create our two transactions
	k1 := []byte("firstName")
	v1 := []byte("satoshi")
	tx1 := append(k1, append([]byte("="), v1...)...)

	k2 := []byte("lastName")
	v2 := []byte("nakamoto")
	tx2 := append(k2, append([]byte("="), v2...)...)

	txs := [][]byte{tx1, tx2}

	// Create a new batch
	batch := c.NewBatch()

	// Queue up our transactions
	for _, tx := range txs {
		// Broadcast the transaction and wait for it to commit (rather use
		// c.BroadcastTxSync though in production).
		if _, err := batch.BroadcastTxCommit(context.Background(), tx); err != nil {
			log.Fatal(err)
		}
	}

	// Send the batch of 2 transactions
	if _, err := batch.Send(context.Background()); err != nil {
		log.Fatal(err)
	}

	// Now let's query for the original results as a batch
	keys := [][]byte{k1, k2}
	for _, key := range keys {
		if _, err := batch.ABCIQuery(context.Background(), "/key", key); err != nil {
			log.Fatal(err)
		}
	}

	// Send the 2 queries and keep the results
	results, err := batch.Send(context.Background())
	if err != nil {
		log.Fatal(err)
	}

	// Each result in the returned list is the deserialized result of each
	// respective ABCIQuery response
	for _, result := range results {
		qr, ok := result.(*ctypes.ResultABCIQuery)
		if !ok {
			log.Fatal("invalid result type from ABCIQuery request")
		}
		fmt.Println(string(qr.Response.Key), "=", string(qr.Response.Value))
	}

	// Output:
	// firstName = satoshi
	// lastName = nakamoto
}

// Test the maximum batch request size middleware.
func ExampleHTTP_maxBatchSize() {
	// Start a CometBFT node (and kvstore) in the background to test against
	app := kvstore.NewInMemoryApplication()
	node := rpctest.StartCometBFT(app, rpctest.RecreateConfig, rpctest.SuppressStdout, rpctest.MaxReqBatchSize)

	// Change the max_request_batch_size
	node.Config().RPC.MaxRequestBatchSize = 2

	// Create our RPC client
	rpcAddr := rpctest.GetConfig().RPC.ListenAddress
	c, err := rpchttp.New(rpcAddr)
	if err != nil {
		log.Fatal(err)
	}

	defer rpctest.StopCometBFT(node)

	// Create a new batch
	batch := c.NewBatch()

	for i := 1; i <= 5; i++ {
		if _, err := batch.Health(context.Background()); err != nil {
			log.Fatal(err)
		}
	}

	// Send the requests
	results, err := batch.Send(context.Background())
	if err != nil {
		log.Fatal(err)
	}

	// Each result in the returned list is the deserialized result of each
	// respective status response
	for _, result := range results {
		rpcError, ok := result.(*types.RPCError)
		if !ok {
			log.Fatal("invalid result type")
		}
		if !strings.Contains(rpcError.Data, "batch request exceeds maximum") {
			fmt.Println("Error message does not contain 'Max Request Batch Exceeded'")
		} else {
			// The max request batch size rpcError has been returned
			fmt.Println("Max Request Batch Exceeded")
		}
	}

	// Output:
	// Max Request Batch Exceeded
}
