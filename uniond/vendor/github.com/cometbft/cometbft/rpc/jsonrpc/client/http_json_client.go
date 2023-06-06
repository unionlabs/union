package client

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"strings"

	cmtsync "github.com/cometbft/cometbft/libs/sync"
	types "github.com/cometbft/cometbft/rpc/jsonrpc/types"
)

const (
	protoHTTP  = "http"
	protoHTTPS = "https"
	protoWSS   = "wss"
	protoWS    = "ws"
	protoTCP   = "tcp"
	protoUNIX  = "unix"
)

//-------------------------------------------------------------

// Parsed URL structure
type parsedURL struct {
	url.URL

	isUnixSocket bool
}

// Parse URL and set defaults
func newParsedURL(remoteAddr string) (*parsedURL, error) {
	u, err := url.Parse(remoteAddr)
	if err != nil {
		return nil, err
	}

	// default to tcp if nothing specified
	if u.Scheme == "" {
		u.Scheme = protoTCP
	}

	pu := &parsedURL{
		URL:          *u,
		isUnixSocket: false,
	}

	if u.Scheme == protoUNIX {
		pu.isUnixSocket = true
	}

	return pu, nil
}

// Change protocol to HTTP for unknown protocols and TCP protocol - useful for RPC connections
func (u *parsedURL) SetDefaultSchemeHTTP() {
	// protocol to use for http operations, to support both http and https
	switch u.Scheme {
	case protoHTTP, protoHTTPS, protoWS, protoWSS:
		// known protocols not changed
	default:
		// default to http for unknown protocols (ex. tcp)
		u.Scheme = protoHTTP
	}
}

// Get full address without the protocol - useful for Dialer connections
func (u parsedURL) GetHostWithPath() string {
	// Remove protocol, userinfo and # fragment, assume opaque is empty
	return u.Host + u.EscapedPath()
}

// Get a trimmed address - useful for WS connections
func (u parsedURL) GetTrimmedHostWithPath() string {
	// if it's not an unix socket we return the normal URL
	if !u.isUnixSocket {
		return u.GetHostWithPath()
	}
	// if it's a unix socket we replace the host slashes with a period
	// this is because otherwise the http.Client would think that the
	// domain is invalid.
	return strings.ReplaceAll(u.GetHostWithPath(), "/", ".")
}

// GetDialAddress returns the endpoint to dial for the parsed URL
func (u parsedURL) GetDialAddress() string {
	// if it's not a unix socket we return the host, example: localhost:443
	if !u.isUnixSocket {
		return u.Host
	}
	// otherwise we return the path of the unix socket, ex /tmp/socket
	return u.GetHostWithPath()
}

// Get a trimmed address with protocol - useful as address in RPC connections
func (u parsedURL) GetTrimmedURL() string {
	return u.Scheme + "://" + u.GetTrimmedHostWithPath()
}

//-------------------------------------------------------------

// HTTPClient is a common interface for JSON-RPC HTTP clients.
type HTTPClient interface {
	// Call calls the given method with the params and returns a result.
	Call(ctx context.Context, method string, params map[string]interface{}, result interface{}) (interface{}, error)
}

// Caller implementers can facilitate calling the JSON-RPC endpoint.
type Caller interface {
	Call(ctx context.Context, method string, params map[string]interface{}, result interface{}) (interface{}, error)
}

//-------------------------------------------------------------

// Client is a JSON-RPC client, which sends POST HTTP requests to the
// remote server.
//
// Client is safe for concurrent use by multiple goroutines.
type Client struct {
	address  string
	username string
	password string

	client *http.Client

	mtx       cmtsync.Mutex
	nextReqID int
}

var _ HTTPClient = (*Client)(nil)

// Both Client and RequestBatch can facilitate calls to the JSON
// RPC endpoint.
var _ Caller = (*Client)(nil)
var _ Caller = (*RequestBatch)(nil)

// New returns a Client pointed at the given address.
// An error is returned on invalid remote. The function panics when remote is nil.
func New(remote string) (*Client, error) {
	httpClient, err := DefaultHTTPClient(remote)
	if err != nil {
		return nil, err
	}
	return NewWithHTTPClient(remote, httpClient)
}

// NewWithHTTPClient returns a Client pointed at the given
// address using a custom http client. An error is returned on invalid remote.
// The function panics when remote is nil.
func NewWithHTTPClient(remote string, client *http.Client) (*Client, error) {
	if client == nil {
		panic("nil http.Client provided")
	}

	parsedURL, err := newParsedURL(remote)
	if err != nil {
		return nil, fmt.Errorf("invalid remote %s: %s", remote, err)
	}

	parsedURL.SetDefaultSchemeHTTP()

	address := parsedURL.GetTrimmedURL()
	username := parsedURL.User.Username()
	password, _ := parsedURL.User.Password()

	rpcClient := &Client{
		address:  address,
		username: username,
		password: password,
		client:   client,
	}

	return rpcClient, nil
}

// Call issues a POST HTTP request. Requests are JSON encoded. Content-Type:
// application/json.
func (c *Client) Call(
	ctx context.Context,
	method string,
	params map[string]interface{},
	result interface{},
) (interface{}, error) {
	id := c.nextRequestID()

	request, err := types.MapToRequest(id, method, params)
	if err != nil {
		return nil, fmt.Errorf("failed to encode params: %w", err)
	}

	requestBytes, err := json.Marshal(request)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	requestBuf := bytes.NewBuffer(requestBytes)
	httpRequest, err := http.NewRequestWithContext(ctx, http.MethodPost, c.address, requestBuf)
	if err != nil {
		return nil, fmt.Errorf("request failed: %w", err)
	}

	httpRequest.Header.Set("Content-Type", "application/json")

	if c.username != "" || c.password != "" {
		httpRequest.SetBasicAuth(c.username, c.password)
	}

	httpResponse, err := c.client.Do(httpRequest)
	if err != nil {
		return nil, fmt.Errorf("post failed: %w", err)
	}
	defer httpResponse.Body.Close()

	responseBytes, err := io.ReadAll(httpResponse.Body)
	if err != nil {
		return nil, fmt.Errorf("%s. Failed to read response body: %w", getHTTPRespErrPrefix(httpResponse), err)
	}

	res, err := unmarshalResponseBytes(responseBytes, id, result)
	if err != nil {
		return nil, fmt.Errorf("%s. %w", getHTTPRespErrPrefix(httpResponse), err)
	}
	return res, nil
}

func getHTTPRespErrPrefix(resp *http.Response) string {
	return fmt.Sprintf("error in json rpc client, with http response metadata: (Status: %s, Protocol %s)", resp.Status, resp.Proto)
}

// NewRequestBatch starts a batch of requests for this client.
func (c *Client) NewRequestBatch() *RequestBatch {
	return &RequestBatch{
		requests: make([]*jsonRPCBufferedRequest, 0),
		client:   c,
	}
}

func (c *Client) sendBatch(ctx context.Context, requests []*jsonRPCBufferedRequest) ([]interface{}, error) {
	reqs := make([]types.RPCRequest, 0, len(requests))
	results := make([]interface{}, 0, len(requests))
	for _, req := range requests {
		reqs = append(reqs, req.request)
		results = append(results, req.result)
	}

	// serialize the array of requests into a single JSON object
	requestBytes, err := json.Marshal(reqs)
	if err != nil {
		return nil, fmt.Errorf("json marshal: %w", err)
	}

	httpRequest, err := http.NewRequestWithContext(ctx, http.MethodPost, c.address, bytes.NewBuffer(requestBytes))
	if err != nil {
		return nil, fmt.Errorf("new request: %w", err)
	}

	httpRequest.Header.Set("Content-Type", "application/json")

	if c.username != "" || c.password != "" {
		httpRequest.SetBasicAuth(c.username, c.password)
	}

	httpResponse, err := c.client.Do(httpRequest)
	if err != nil {
		return nil, fmt.Errorf("post: %w", err)
	}

	defer httpResponse.Body.Close()

	responseBytes, err := io.ReadAll(httpResponse.Body)
	if err != nil {
		return nil, fmt.Errorf("read response body: %w", err)
	}

	// collect ids to check responses IDs in unmarshalResponseBytesArray
	ids := make([]types.JSONRPCIntID, len(requests))
	for i, req := range requests {
		ids[i] = req.request.ID.(types.JSONRPCIntID)
	}

	return unmarshalResponseBytesArray(responseBytes, ids, results)
}

func (c *Client) nextRequestID() types.JSONRPCIntID {
	c.mtx.Lock()
	id := c.nextReqID
	c.nextReqID++
	c.mtx.Unlock()
	return types.JSONRPCIntID(id)
}

//------------------------------------------------------------------------------------

// jsonRPCBufferedRequest encapsulates a single buffered request, as well as its
// anticipated response structure.
type jsonRPCBufferedRequest struct {
	request types.RPCRequest
	result  interface{} // The result will be deserialized into this object.
}

// RequestBatch allows us to buffer multiple request/response structures
// into a single batch request. Note that this batch acts like a FIFO queue, and
// is thread-safe.
type RequestBatch struct {
	client *Client

	mtx      cmtsync.Mutex
	requests []*jsonRPCBufferedRequest
}

// Count returns the number of enqueued requests waiting to be sent.
func (b *RequestBatch) Count() int {
	b.mtx.Lock()
	defer b.mtx.Unlock()
	return len(b.requests)
}

func (b *RequestBatch) enqueue(req *jsonRPCBufferedRequest) {
	b.mtx.Lock()
	defer b.mtx.Unlock()
	b.requests = append(b.requests, req)
}

// Clear empties out the request batch.
func (b *RequestBatch) Clear() int {
	b.mtx.Lock()
	defer b.mtx.Unlock()
	return b.clear()
}

func (b *RequestBatch) clear() int {
	count := len(b.requests)
	b.requests = make([]*jsonRPCBufferedRequest, 0)
	return count
}

// Send will attempt to send the current batch of enqueued requests, and then
// will clear out the requests once done. On success, this returns the
// deserialized list of results from each of the enqueued requests.
func (b *RequestBatch) Send(ctx context.Context) ([]interface{}, error) {
	b.mtx.Lock()
	defer func() {
		b.clear()
		b.mtx.Unlock()
	}()
	return b.client.sendBatch(ctx, b.requests)
}

// Call enqueues a request to call the given RPC method with the specified
// parameters, in the same way that the `Client.Call` function would.
func (b *RequestBatch) Call(
	_ context.Context,
	method string,
	params map[string]interface{},
	result interface{},
) (interface{}, error) {
	id := b.client.nextRequestID()
	request, err := types.MapToRequest(id, method, params)
	if err != nil {
		return nil, err
	}
	b.enqueue(&jsonRPCBufferedRequest{request: request, result: result})
	return result, nil
}

//-------------------------------------------------------------

func makeHTTPDialer(remoteAddr string) (func(string, string) (net.Conn, error), error) {
	u, err := newParsedURL(remoteAddr)
	if err != nil {
		return nil, err
	}

	protocol := u.Scheme

	// accept http(s) as an alias for tcp
	switch protocol {
	case protoHTTP, protoHTTPS:
		protocol = protoTCP
	}

	dialFn := func(proto, addr string) (net.Conn, error) {
		return net.Dial(protocol, u.GetDialAddress())
	}

	return dialFn, nil
}

// DefaultHTTPClient is used to create an http client with some default parameters.
// We overwrite the http.Client.Dial so we can do http over tcp or unix.
// remoteAddr should be fully featured (eg. with tcp:// or unix://).
// An error will be returned in case of invalid remoteAddr.
func DefaultHTTPClient(remoteAddr string) (*http.Client, error) {
	dialFn, err := makeHTTPDialer(remoteAddr)
	if err != nil {
		return nil, err
	}

	client := &http.Client{
		Transport: &http.Transport{
			// Set to true to prevent GZIP-bomb DoS attacks
			DisableCompression: true,
			Dial:               dialFn,
		},
	}

	return client, nil
}
