package jsonrpc

import (
	"bytes"
	"context"
	"encoding/json"
	"io/ioutil"
	"net/http"
	"net/url"
	"sync/atomic"

	"github.com/go-kit/kit/endpoint"
	httptransport "github.com/go-kit/kit/transport/http"
)

// Client wraps a JSON RPC method and provides a method that implements endpoint.Endpoint.
type Client struct {
	client httptransport.HTTPClient

	// JSON RPC endpoint URL
	tgt *url.URL

	// JSON RPC method name.
	method string

	enc            EncodeRequestFunc
	dec            DecodeResponseFunc
	before         []httptransport.RequestFunc
	after          []httptransport.ClientResponseFunc
	finalizer      httptransport.ClientFinalizerFunc
	requestID      RequestIDGenerator
	bufferedStream bool
}

type clientRequest struct {
	JSONRPC string          `json:"jsonrpc"`
	Method  string          `json:"method"`
	Params  json.RawMessage `json:"params"`
	ID      interface{}     `json:"id"`
}

// NewClient constructs a usable Client for a single remote method.
func NewClient(
	tgt *url.URL,
	method string,
	options ...ClientOption,
) *Client {
	c := &Client{
		client:         http.DefaultClient,
		method:         method,
		tgt:            tgt,
		enc:            DefaultRequestEncoder,
		dec:            DefaultResponseDecoder,
		before:         []httptransport.RequestFunc{},
		after:          []httptransport.ClientResponseFunc{},
		requestID:      NewAutoIncrementID(0),
		bufferedStream: false,
	}
	for _, option := range options {
		option(c)
	}
	return c
}

// DefaultRequestEncoder marshals the given request to JSON.
func DefaultRequestEncoder(_ context.Context, req interface{}) (json.RawMessage, error) {
	return json.Marshal(req)
}

// DefaultResponseDecoder unmarshals the result to interface{}, or returns an
// error, if found.
func DefaultResponseDecoder(_ context.Context, res Response) (interface{}, error) {
	if res.Error != nil {
		return nil, *res.Error
	}
	var result interface{}
	err := json.Unmarshal(res.Result, &result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

// ClientOption sets an optional parameter for clients.
type ClientOption func(*Client)

// SetClient sets the underlying HTTP client used for requests.
// By default, http.DefaultClient is used.
func SetClient(client httptransport.HTTPClient) ClientOption {
	return func(c *Client) { c.client = client }
}

// ClientBefore sets the RequestFuncs that are applied to the outgoing HTTP
// request before it's invoked.
func ClientBefore(before ...httptransport.RequestFunc) ClientOption {
	return func(c *Client) { c.before = append(c.before, before...) }
}

// ClientAfter sets the ClientResponseFuncs applied to the server's HTTP
// response prior to it being decoded. This is useful for obtaining anything
// from the response and adding onto the context prior to decoding.
func ClientAfter(after ...httptransport.ClientResponseFunc) ClientOption {
	return func(c *Client) { c.after = append(c.after, after...) }
}

// ClientFinalizer is executed at the end of every HTTP request.
// By default, no finalizer is registered.
func ClientFinalizer(f httptransport.ClientFinalizerFunc) ClientOption {
	return func(c *Client) { c.finalizer = f }
}

// ClientRequestEncoder sets the func used to encode the request params to JSON.
// If not set, DefaultRequestEncoder is used.
func ClientRequestEncoder(enc EncodeRequestFunc) ClientOption {
	return func(c *Client) { c.enc = enc }
}

// ClientResponseDecoder sets the func used to decode the response params from
// JSON. If not set, DefaultResponseDecoder is used.
func ClientResponseDecoder(dec DecodeResponseFunc) ClientOption {
	return func(c *Client) { c.dec = dec }
}

// RequestIDGenerator returns an ID for the request.
type RequestIDGenerator interface {
	Generate() interface{}
}

// ClientRequestIDGenerator is executed before each request to generate an ID
// for the request.
// By default, AutoIncrementRequestID is used.
func ClientRequestIDGenerator(g RequestIDGenerator) ClientOption {
	return func(c *Client) { c.requestID = g }
}

// BufferedStream sets whether the Response.Body is left open, allowing it
// to be read from later. Useful for transporting a file as a buffered stream.
func BufferedStream(buffered bool) ClientOption {
	return func(c *Client) { c.bufferedStream = buffered }
}

// Endpoint returns a usable endpoint that invokes the remote endpoint.
func (c Client) Endpoint() endpoint.Endpoint {
	return func(ctx context.Context, request interface{}) (interface{}, error) {
		ctx, cancel := context.WithCancel(ctx)
		defer cancel()

		var (
			resp *http.Response
			err  error
		)
		if c.finalizer != nil {
			defer func() {
				if resp != nil {
					ctx = context.WithValue(ctx, httptransport.ContextKeyResponseHeaders, resp.Header)
					ctx = context.WithValue(ctx, httptransport.ContextKeyResponseSize, resp.ContentLength)
				}
				c.finalizer(ctx, err)
			}()
		}

		ctx = context.WithValue(ctx, ContextKeyRequestMethod, c.method)

		var params json.RawMessage
		if params, err = c.enc(ctx, request); err != nil {
			return nil, err
		}
		rpcReq := clientRequest{
			JSONRPC: Version,
			Method:  c.method,
			Params:  params,
			ID:      c.requestID.Generate(),
		}

		req, err := http.NewRequest("POST", c.tgt.String(), nil)
		if err != nil {
			return nil, err
		}

		req.Header.Set("Content-Type", "application/json; charset=utf-8")
		var b bytes.Buffer
		req.Body = ioutil.NopCloser(&b)
		err = json.NewEncoder(&b).Encode(rpcReq)
		if err != nil {
			return nil, err
		}

		for _, f := range c.before {
			ctx = f(ctx, req)
		}

		resp, err = c.client.Do(req.WithContext(ctx))
		if err != nil {
			return nil, err
		}

		if !c.bufferedStream {
			defer resp.Body.Close()
		}

		for _, f := range c.after {
			ctx = f(ctx, resp)
		}

		// Decode the body into an object
		var rpcRes Response
		err = json.NewDecoder(resp.Body).Decode(&rpcRes)
		if err != nil {
			return nil, err
		}

		response, err := c.dec(ctx, rpcRes)
		if err != nil {
			return nil, err
		}

		return response, nil
	}
}

// ClientFinalizerFunc can be used to perform work at the end of a client HTTP
// request, after the response is returned. The principal
// intended use is for error logging. Additional response parameters are
// provided in the context under keys with the ContextKeyResponse prefix.
// Note: err may be nil. There maybe also no additional response parameters
// depending on when an error occurs.
type ClientFinalizerFunc func(ctx context.Context, err error)

// autoIncrementID is a RequestIDGenerator that generates
// auto-incrementing integer IDs.
type autoIncrementID struct {
	v *uint64
}

// NewAutoIncrementID returns an auto-incrementing request ID generator,
// initialised with the given value.
func NewAutoIncrementID(init uint64) RequestIDGenerator {
	// Offset by one so that the first generated value = init.
	v := init - 1
	return &autoIncrementID{v: &v}
}

// Generate satisfies RequestIDGenerator
func (i *autoIncrementID) Generate() interface{} {
	id := atomic.AddUint64(i.v, 1)
	return id
}
