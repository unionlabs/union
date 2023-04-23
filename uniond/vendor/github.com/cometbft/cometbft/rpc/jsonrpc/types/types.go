package types

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"reflect"
	"strings"

	cmtjson "github.com/cometbft/cometbft/libs/json"
)

// a wrapper to emulate a sum type: jsonrpcid = string | int
// TODO: refactor when Go 2.0 arrives https://github.com/golang/go/issues/19412
type jsonrpcid interface {
	isJSONRPCID()
}

// JSONRPCStringID a wrapper for JSON-RPC string IDs
type JSONRPCStringID string

func (JSONRPCStringID) isJSONRPCID()      {}
func (id JSONRPCStringID) String() string { return string(id) }

// JSONRPCIntID a wrapper for JSON-RPC integer IDs
type JSONRPCIntID int

func (JSONRPCIntID) isJSONRPCID()      {}
func (id JSONRPCIntID) String() string { return fmt.Sprintf("%d", id) }

func idFromInterface(idInterface interface{}) (jsonrpcid, error) {
	switch id := idInterface.(type) {
	case string:
		return JSONRPCStringID(id), nil
	case float64:
		// json.Unmarshal uses float64 for all numbers
		// (https://golang.org/pkg/encoding/json/#Unmarshal),
		// but the JSONRPC2.0 spec says the id SHOULD NOT contain
		// decimals - so we truncate the decimals here.
		return JSONRPCIntID(int(id)), nil
	default:
		typ := reflect.TypeOf(id)
		return nil, fmt.Errorf("json-rpc ID (%v) is of unknown type (%v)", id, typ)
	}
}

//----------------------------------------
// REQUEST

type RPCRequest struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      jsonrpcid       `json:"id,omitempty"`
	Method  string          `json:"method"`
	Params  json.RawMessage `json:"params"` // must be map[string]interface{} or []interface{}
}

// UnmarshalJSON custom JSON unmarshalling due to jsonrpcid being string or int
func (req *RPCRequest) UnmarshalJSON(data []byte) error {
	unsafeReq := struct {
		JSONRPC string          `json:"jsonrpc"`
		ID      interface{}     `json:"id,omitempty"`
		Method  string          `json:"method"`
		Params  json.RawMessage `json:"params"` // must be map[string]interface{} or []interface{}
	}{}

	err := json.Unmarshal(data, &unsafeReq)
	if err != nil {
		return err
	}

	if unsafeReq.ID == nil { // notification
		return nil
	}

	req.JSONRPC = unsafeReq.JSONRPC
	req.Method = unsafeReq.Method
	req.Params = unsafeReq.Params
	id, err := idFromInterface(unsafeReq.ID)
	if err != nil {
		return err
	}
	req.ID = id

	return nil
}

func NewRPCRequest(id jsonrpcid, method string, params json.RawMessage) RPCRequest {
	return RPCRequest{
		JSONRPC: "2.0",
		ID:      id,
		Method:  method,
		Params:  params,
	}
}

func (req RPCRequest) String() string {
	return fmt.Sprintf("RPCRequest{%s %s/%X}", req.ID, req.Method, req.Params)
}

func MapToRequest(id jsonrpcid, method string, params map[string]interface{}) (RPCRequest, error) {
	var paramsMap = make(map[string]json.RawMessage, len(params))
	for name, value := range params {
		valueJSON, err := cmtjson.Marshal(value)
		if err != nil {
			return RPCRequest{}, err
		}
		paramsMap[name] = valueJSON
	}

	payload, err := json.Marshal(paramsMap)
	if err != nil {
		return RPCRequest{}, err
	}

	return NewRPCRequest(id, method, payload), nil
}

func ArrayToRequest(id jsonrpcid, method string, params []interface{}) (RPCRequest, error) {
	var paramsMap = make([]json.RawMessage, len(params))
	for i, value := range params {
		valueJSON, err := cmtjson.Marshal(value)
		if err != nil {
			return RPCRequest{}, err
		}
		paramsMap[i] = valueJSON
	}

	payload, err := json.Marshal(paramsMap)
	if err != nil {
		return RPCRequest{}, err
	}

	return NewRPCRequest(id, method, payload), nil
}

//----------------------------------------
// RESPONSE

type RPCError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
	Data    string `json:"data,omitempty"`
}

func (err RPCError) Error() string {
	const baseFormat = "RPC error %v - %s"
	if err.Data != "" {
		return fmt.Sprintf(baseFormat+": %s", err.Code, err.Message, err.Data)
	}
	return fmt.Sprintf(baseFormat, err.Code, err.Message)
}

type RPCResponse struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      jsonrpcid       `json:"id,omitempty"`
	Result  json.RawMessage `json:"result,omitempty"`
	Error   *RPCError       `json:"error,omitempty"`
}

// UnmarshalJSON custom JSON unmarshalling due to jsonrpcid being string or int
func (resp *RPCResponse) UnmarshalJSON(data []byte) error {
	unsafeResp := &struct {
		JSONRPC string          `json:"jsonrpc"`
		ID      interface{}     `json:"id,omitempty"`
		Result  json.RawMessage `json:"result,omitempty"`
		Error   *RPCError       `json:"error,omitempty"`
	}{}
	err := json.Unmarshal(data, &unsafeResp)
	if err != nil {
		return err
	}
	resp.JSONRPC = unsafeResp.JSONRPC
	resp.Error = unsafeResp.Error
	resp.Result = unsafeResp.Result
	if unsafeResp.ID == nil {
		return nil
	}
	id, err := idFromInterface(unsafeResp.ID)
	if err != nil {
		return err
	}
	resp.ID = id
	return nil
}

func NewRPCSuccessResponse(id jsonrpcid, res interface{}) RPCResponse {
	var rawMsg json.RawMessage

	if res != nil {
		var js []byte
		js, err := cmtjson.Marshal(res)
		if err != nil {
			return RPCInternalError(id, fmt.Errorf("error marshaling response: %w", err))
		}
		rawMsg = json.RawMessage(js)
	}

	return RPCResponse{JSONRPC: "2.0", ID: id, Result: rawMsg}
}

func NewRPCErrorResponse(id jsonrpcid, code int, msg string, data string) RPCResponse {
	return RPCResponse{
		JSONRPC: "2.0",
		ID:      id,
		Error:   &RPCError{Code: code, Message: msg, Data: data},
	}
}

func (resp RPCResponse) String() string {
	if resp.Error == nil {
		return fmt.Sprintf("RPCResponse{%s %X}", resp.ID, resp.Result)
	}
	return fmt.Sprintf("RPCResponse{%s %v}", resp.ID, resp.Error)
}

// From the JSON-RPC 2.0 spec:
//
//	If there was an error in detecting the id in the Request object (e.g. Parse
//	error/Invalid Request), it MUST be Null.
func RPCParseError(err error) RPCResponse {
	return NewRPCErrorResponse(nil, -32700, "Parse error. Invalid JSON", err.Error())
}

// From the JSON-RPC 2.0 spec:
//
//	If there was an error in detecting the id in the Request object (e.g. Parse
//	error/Invalid Request), it MUST be Null.
func RPCInvalidRequestError(id jsonrpcid, err error) RPCResponse {
	return NewRPCErrorResponse(id, -32600, "Invalid Request", err.Error())
}

func RPCMethodNotFoundError(id jsonrpcid) RPCResponse {
	return NewRPCErrorResponse(id, -32601, "Method not found", "")
}

func RPCInvalidParamsError(id jsonrpcid, err error) RPCResponse {
	return NewRPCErrorResponse(id, -32602, "Invalid params", err.Error())
}

func RPCInternalError(id jsonrpcid, err error) RPCResponse {
	return NewRPCErrorResponse(id, -32603, "Internal error", err.Error())
}

func RPCServerError(id jsonrpcid, err error) RPCResponse {
	return NewRPCErrorResponse(id, -32000, "Server error", err.Error())
}

//----------------------------------------

// WSRPCConnection represents a websocket connection.
type WSRPCConnection interface {
	// GetRemoteAddr returns a remote address of the connection.
	GetRemoteAddr() string
	// WriteRPCResponse writes the response onto connection (BLOCKING).
	WriteRPCResponse(context.Context, RPCResponse) error
	// TryWriteRPCResponse tries to write the response onto connection (NON-BLOCKING).
	TryWriteRPCResponse(RPCResponse) bool
	// Context returns the connection's context.
	Context() context.Context
}

// Context is the first parameter for all functions. It carries a json-rpc
// request, http request and websocket connection.
//
// - JSONReq is non-nil when JSONRPC is called over websocket or HTTP.
// - WSConn is non-nil when we're connected via a websocket.
// - HTTPReq is non-nil when URI or JSONRPC is called over HTTP.
type Context struct {
	// json-rpc request
	JSONReq *RPCRequest
	// websocket connection
	WSConn WSRPCConnection
	// http request
	HTTPReq *http.Request
}

// RemoteAddr returns the remote address (usually a string "IP:port").
// If neither HTTPReq nor WSConn is set, an empty string is returned.
// HTTP:
//
//	http.Request#RemoteAddr
//
// WS:
//
//	result of GetRemoteAddr
func (ctx *Context) RemoteAddr() string {
	if ctx.HTTPReq != nil {
		return ctx.HTTPReq.RemoteAddr
	} else if ctx.WSConn != nil {
		return ctx.WSConn.GetRemoteAddr()
	}
	return ""
}

// Context returns the request's context.
// The returned context is always non-nil; it defaults to the background context.
// HTTP:
//
//	The context is canceled when the client's connection closes, the request
//	is canceled (with HTTP/2), or when the ServeHTTP method returns.
//
// WS:
//
//	The context is canceled when the client's connections closes.
func (ctx *Context) Context() context.Context {
	if ctx.HTTPReq != nil {
		return ctx.HTTPReq.Context()
	} else if ctx.WSConn != nil {
		return ctx.WSConn.Context()
	}
	return context.Background()
}

//----------------------------------------
// SOCKETS

// Determine if its a unix or tcp socket.
// If tcp, must specify the port; `0.0.0.0` will return incorrectly as "unix" since there's no port
// TODO: deprecate
func SocketType(listenAddr string) string {
	socketType := "unix"
	if len(strings.Split(listenAddr, ":")) >= 2 {
		socketType = "tcp"
	}
	return socketType
}
