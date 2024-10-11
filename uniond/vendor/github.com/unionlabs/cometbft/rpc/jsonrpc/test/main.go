package main

import (
	"fmt"
	"net/http"
	"os"

	cmtos "github.com/cometbft/cometbft/internal/os"
	"github.com/cometbft/cometbft/libs/log"
	rpcserver "github.com/cometbft/cometbft/rpc/jsonrpc/server"
	rpctypes "github.com/cometbft/cometbft/rpc/jsonrpc/types"
)

var routes = map[string]*rpcserver.RPCFunc{
	"hello_world": rpcserver.NewRPCFunc(HelloWorld, "name,num"),
}

func HelloWorld(_ *rpctypes.Context, name string, num int) (Result, error) {
	return Result{fmt.Sprintf("hi %s %d", name, num)}, nil
}

type Result struct {
	Result string
}

func main() {
	var (
		mux    = http.NewServeMux()
		logger = log.NewTMLogger(log.NewSyncWriter(os.Stdout))
	)

	// Stop upon receiving SIGTERM or CTRL-C.
	cmtos.TrapSignal(logger, func() {})

	rpcserver.RegisterRPCFuncs(mux, routes, logger)
	config := rpcserver.DefaultConfig()
	listener, err := rpcserver.Listen("tcp://127.0.0.1:8008", config.MaxOpenConnections)
	if err != nil {
		cmtos.Exit(err.Error())
	}

	if err = rpcserver.Serve(listener, mux, logger, config); err != nil {
		cmtos.Exit(err.Error())
	}
}
