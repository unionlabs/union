package server

import (
	"fmt"
	"net/http"
	"reflect"
	"strings"

	"github.com/cometbft/cometbft/libs/log"
)

// RegisterRPCFuncs adds a route for each function in the funcMap, as well as
// general jsonrpc and websocket handlers for all functions. "result" is the
// interface on which the result objects are registered, and is popualted with
// every RPCResponse
func RegisterRPCFuncs(mux *http.ServeMux, funcMap map[string]*RPCFunc, logger log.Logger) {
	// HTTP endpoints
	for funcName, rpcFunc := range funcMap {
		mux.HandleFunc("/"+funcName, makeHTTPHandler(rpcFunc, logger))
	}

	// JSONRPC endpoints
	mux.HandleFunc("/", handleInvalidJSONRPCPaths(makeJSONRPCHandler(funcMap, logger)))
}

type Option func(*RPCFunc)

// Cacheable enables returning a cache control header from RPC functions to
// which it is applied.
//
// `noCacheDefArgs` is a list of argument names that, if omitted or set to
// their defaults when calling the RPC function, will skip the response
// caching.
func Cacheable(noCacheDefArgs ...string) Option {
	return func(r *RPCFunc) {
		r.cacheable = true
		r.noCacheDefArgs = make(map[string]interface{})
		for _, arg := range noCacheDefArgs {
			r.noCacheDefArgs[arg] = nil
		}
	}
}

// Ws enables WebSocket communication.
func Ws() Option {
	return func(r *RPCFunc) {
		r.ws = true
	}
}

// RPCFunc contains the introspected type information for a function
type RPCFunc struct {
	f              reflect.Value          // underlying rpc function
	args           []reflect.Type         // type of each function arg
	returns        []reflect.Type         // type of each return arg
	argNames       []string               // name of each argument
	cacheable      bool                   // enable cache control
	ws             bool                   // enable websocket communication
	noCacheDefArgs map[string]interface{} // a lookup table of args that, if not supplied or are set to default values, cause us to not cache
}

// NewRPCFunc wraps a function for introspection.
// f is the function, args are comma separated argument names
func NewRPCFunc(f interface{}, args string, options ...Option) *RPCFunc {
	return newRPCFunc(f, args, options...)
}

// NewWSRPCFunc wraps a function for introspection and use in the websockets.
func NewWSRPCFunc(f interface{}, args string, options ...Option) *RPCFunc {
	options = append(options, Ws())
	return newRPCFunc(f, args, options...)
}

// cacheableWithArgs returns whether or not a call to this function is cacheable,
// given the specified arguments.
func (f *RPCFunc) cacheableWithArgs(args []reflect.Value) bool {
	if !f.cacheable {
		return false
	}
	// Skip the context variable common to all RPC functions
	for i := 1; i < len(f.args); i++ {
		// f.argNames does not include the context variable
		argName := f.argNames[i-1]
		if _, hasDefault := f.noCacheDefArgs[argName]; hasDefault {
			// Argument with default value was not supplied
			if i >= len(args) {
				return false
			}
			// Argument with default value is set to its zero value
			if args[i].IsZero() {
				return false
			}
		}
	}
	return true
}

func newRPCFunc(f interface{}, args string, options ...Option) *RPCFunc {
	var argNames []string
	if args != "" {
		argNames = strings.Split(args, ",")
	}

	r := &RPCFunc{
		f:        reflect.ValueOf(f),
		args:     funcArgTypes(f),
		returns:  funcReturnTypes(f),
		argNames: argNames,
	}

	for _, opt := range options {
		opt(r)
	}

	return r
}

// return a function's argument types
func funcArgTypes(f interface{}) []reflect.Type {
	t := reflect.TypeOf(f)
	n := t.NumIn()
	typez := make([]reflect.Type, n)
	for i := 0; i < n; i++ {
		typez[i] = t.In(i)
	}
	return typez
}

// return a function's return types
func funcReturnTypes(f interface{}) []reflect.Type {
	t := reflect.TypeOf(f)
	n := t.NumOut()
	typez := make([]reflect.Type, n)
	for i := 0; i < n; i++ {
		typez[i] = t.Out(i)
	}
	return typez
}

//-------------------------------------------------------------

// NOTE: assume returns is result struct and error. If error is not nil, return it
func unreflectResult(returns []reflect.Value) (interface{}, error) {
	errV := returns[1]
	if errV.Interface() != nil {
		return nil, fmt.Errorf("%v", errV.Interface())
	}
	rv := returns[0]
	// the result is a registered interface,
	// we need a pointer to it so we can marshal with type byte
	rvp := reflect.New(rv.Type())
	rvp.Elem().Set(rv)
	return rvp.Interface(), nil
}
