// HTTP RPC server supporting calls via uri params, jsonrpc over HTTP, and jsonrpc over
// websockets
//
// # Client Requests
//
// Suppose we want to expose the rpc function `HelloWorld(name string, num int)`.
//
// GET (URI)
//
// As a GET request, it would have URI encoded parameters, and look like:
//
//	curl 'http://localhost:8008/hello_world?name="my_world"&num=5'
//
// Note the `'` around the url, which is just so bash doesn't ignore the quotes in `"my_world"`.
// This should also work:
//
//	curl http://localhost:8008/hello_world?name=\"my_world\"&num=5
//
// A GET request to `/` returns a list of available endpoints.
// For those which take arguments, the arguments will be listed in order, with `_` where the actual value should be.
//
// POST (JSONRPC)
//
// As a POST request, we use JSONRPC. For instance, the same request would have this as the body:
//
//	{
//	  "jsonrpc": "2.0",
//	  "id": "anything",
//	  "method": "hello_world",
//	  "params": {
//	    "name": "my_world",
//	    "num": 5
//	  }
//	}
//
// With the above saved in file `data.json`, we can make the request with
//
//	curl --data @data.json http://localhost:8008
//
// WebSocket (JSONRPC)
//
// All requests are exposed over websocket in the same form as the POST JSONRPC.
// Websocket connections are available at their own endpoint, typically `/websocket`,
// though this is configurable when starting the server.
//
// # Server Definition
//
// Define some types and routes:
//
//	type ResultStatus struct {
//		    Value string
//	}
//
// Define some routes
//
//	  var Routes = map[string]*rpcserver.RPCFunc{
//		    "status": rpcserver.NewRPCFunc(Status, "arg"),
//	  }
//
// An rpc function:
//
//	  func Status(v string) (*ResultStatus, error) {
//		    return &ResultStatus{v}, nil
//	  }
//
// Now start the server:
//
//	mux := http.NewServeMux()
//	rpcserver.RegisterRPCFuncs(mux, Routes)
//	wm := rpcserver.NewWebsocketManager(Routes)
//	mux.HandleFunc("/websocket", wm.WebsocketHandler)
//	logger := log.NewTMLogger(log.NewSyncWriter(os.Stdout))
//	listener, err := rpc.Listen("0.0.0.0:8080", rpcserver.Config{})
//	if err != nil { panic(err) }
//	go rpcserver.Serve(listener, mux, logger)
//
// Note that unix sockets are supported as well (eg. `/path/to/socket` instead of `0.0.0.0:8008`)
// Now see all available endpoints by sending a GET request to `0.0.0.0:8008`.
// Each route is available as a GET request, as a JSONRPCv2 POST request, and via JSONRPCv2 over websockets.
//
// # Examples
//
// - [CometBFT](https://github.com/cometbft/cometbft/blob/main/rpc/core/routes.go)
package jsonrpc
