package server

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"reflect"
	"sort"

	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/libs/log"
	types "github.com/cometbft/cometbft/rpc/jsonrpc/types"
)

// HTTP + JSON handler

// jsonrpc calls grab the given method's function info and runs reflect.Call
func makeJSONRPCHandler(funcMap map[string]*RPCFunc, logger log.Logger) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		b, err := io.ReadAll(r.Body)
		if err != nil {
			res := types.RPCInvalidRequestError(nil,
				fmt.Errorf("error reading request body: %w", err),
			)
			if wErr := WriteRPCResponseHTTPError(w, http.StatusBadRequest, res); wErr != nil {
				logger.Error("failed to write response", "err", wErr)
			}
			return
		}

		// if its an empty request (like from a browser), just display a list of
		// functions
		if len(b) == 0 {
			writeListOfEndpoints(w, r, funcMap)
			return
		}

		// first try to unmarshal the incoming request as an array of RPC requests
		var (
			requests  []types.RPCRequest
			responses []types.RPCResponse
		)
		if err := json.Unmarshal(b, &requests); err != nil {
			// next, try to unmarshal as a single request
			var request types.RPCRequest
			if err := json.Unmarshal(b, &request); err != nil {
				res := types.RPCParseError(fmt.Errorf("error unmarshaling request: %w", err))
				if wErr := WriteRPCResponseHTTPError(w, http.StatusInternalServerError, res); wErr != nil {
					logger.Error("failed to write response", "err", wErr)
				}
				return
			}
			requests = []types.RPCRequest{request}
		}

		// Set the default response cache to true unless
		// 1. Any RPC request error.
		// 2. Any RPC request doesn't allow to be cached.
		// 3. Any RPC request has the height argument and the value is 0 (the default).
		cache := true
		for _, request := range requests {
			request := request

			// A Notification is a Request object without an "id" member.
			// The Server MUST NOT reply to a Notification, including those that are within a batch request.
			if request.ID == nil {
				logger.Debug(
					"HTTPJSONRPC received a notification, skipping... (please send a non-empty ID if you want to call a method)",
					"req", request,
				)
				continue
			}
			if len(r.URL.Path) > 1 {
				responses = append(
					responses,
					types.RPCInvalidRequestError(request.ID, fmt.Errorf("path %s is invalid", r.URL.Path)),
				)
				cache = false
				continue
			}
			rpcFunc, ok := funcMap[request.Method]
			if !ok || (rpcFunc.ws) {
				responses = append(responses, types.RPCMethodNotFoundError(request.ID))
				cache = false
				continue
			}
			ctx := &types.Context{JSONReq: &request, HTTPReq: r}
			args := []reflect.Value{reflect.ValueOf(ctx)}
			if len(request.Params) > 0 {
				fnArgs, err := jsonParamsToArgs(rpcFunc, request.Params)
				if err != nil {
					responses = append(
						responses,
						types.RPCInvalidParamsError(request.ID, fmt.Errorf("error converting json params to arguments: %w", err)),
					)
					cache = false
					continue
				}
				args = append(args, fnArgs...)
			}

			if cache && !rpcFunc.cacheableWithArgs(args) {
				cache = false
			}

			returns := rpcFunc.f.Call(args)
			result, err := unreflectResult(returns)
			if err != nil {
				responses = append(responses, types.RPCInternalError(request.ID, err))
				continue
			}
			responses = append(responses, types.NewRPCSuccessResponse(request.ID, result))
		}

		if len(responses) > 0 {
			var wErr error
			if cache {
				wErr = WriteCacheableRPCResponseHTTP(w, responses...)
			} else {
				wErr = WriteRPCResponseHTTP(w, responses...)
			}
			if wErr != nil {
				logger.Error("failed to write responses", "err", wErr)
			}
		}
	}
}

func handleInvalidJSONRPCPaths(next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		// Since the pattern "/" matches all paths not matched by other registered patterns,
		//  we check whether the path is indeed "/", otherwise return a 404 error
		if r.URL.Path != "/" {
			http.NotFound(w, r)
			return
		}

		next(w, r)
	}
}

func mapParamsToArgs(
	rpcFunc *RPCFunc,
	params map[string]json.RawMessage,
	argsOffset int,
) ([]reflect.Value, error) {
	values := make([]reflect.Value, len(rpcFunc.argNames))
	for i, argName := range rpcFunc.argNames {
		argType := rpcFunc.args[i+argsOffset]

		if p, ok := params[argName]; ok && p != nil && len(p) > 0 {
			val := reflect.New(argType)
			err := cmtjson.Unmarshal(p, val.Interface())
			if err != nil {
				return nil, err
			}
			values[i] = val.Elem()
		} else { // use default for that type
			values[i] = reflect.Zero(argType)
		}
	}

	return values, nil
}

func arrayParamsToArgs(
	rpcFunc *RPCFunc,
	params []json.RawMessage,
	argsOffset int,
) ([]reflect.Value, error) {
	if len(rpcFunc.argNames) != len(params) {
		return nil, fmt.Errorf("expected %v parameters (%v), got %v (%v)",
			len(rpcFunc.argNames), rpcFunc.argNames, len(params), params)
	}

	values := make([]reflect.Value, len(params))
	for i, p := range params {
		argType := rpcFunc.args[i+argsOffset]
		val := reflect.New(argType)
		err := cmtjson.Unmarshal(p, val.Interface())
		if err != nil {
			return nil, err
		}
		values[i] = val.Elem()
	}
	return values, nil
}

// raw is unparsed json (from json.RawMessage) encoding either a map or an
// array.
//
// Example:
//
//	rpcFunc.args = [rpctypes.Context string]
//	rpcFunc.argNames = ["arg"]
func jsonParamsToArgs(rpcFunc *RPCFunc, raw []byte) ([]reflect.Value, error) {
	const argsOffset = 1

	// TODO: Make more efficient, perhaps by checking the first character for '{' or '['?
	// First, try to get the map.
	var m map[string]json.RawMessage
	err := json.Unmarshal(raw, &m)
	if err == nil {
		return mapParamsToArgs(rpcFunc, m, argsOffset)
	}

	// Otherwise, try an array.
	var a []json.RawMessage
	err = json.Unmarshal(raw, &a)
	if err == nil {
		return arrayParamsToArgs(rpcFunc, a, argsOffset)
	}

	// Otherwise, bad format, we cannot parse
	return nil, fmt.Errorf("unknown type for JSON params: %v. Expected map or array", err)
}

// writes a list of available rpc endpoints as an html page
func writeListOfEndpoints(w http.ResponseWriter, r *http.Request, funcMap map[string]*RPCFunc) {
	noArgNames := []string{}
	argNames := []string{}
	for name, funcData := range funcMap {
		if len(funcData.args) == 0 {
			noArgNames = append(noArgNames, name)
		} else {
			argNames = append(argNames, name)
		}
	}
	sort.Strings(noArgNames)
	sort.Strings(argNames)
	buf := new(bytes.Buffer)
	buf.WriteString("<html><body>")
	buf.WriteString("<br>Available endpoints:<br>")

	for _, name := range noArgNames {
		link := fmt.Sprintf("//%s/%s", r.Host, name)
		buf.WriteString(fmt.Sprintf("<a href=\"%s\">%s</a></br>", link, link))
	}

	buf.WriteString("<br>Endpoints that require arguments:<br>")
	for _, name := range argNames {
		link := fmt.Sprintf("//%s/%s?", r.Host, name)
		funcData := funcMap[name]
		for i, argName := range funcData.argNames {
			link += argName + "=_"
			if i < len(funcData.argNames)-1 {
				link += "&"
			}
		}
		buf.WriteString(fmt.Sprintf("<a href=\"%s\">%s</a></br>", link, link))
	}
	buf.WriteString("</body></html>")
	w.Header().Set("Content-Type", "text/html")
	w.WriteHeader(200)
	w.Write(buf.Bytes()) //nolint: errcheck
}
