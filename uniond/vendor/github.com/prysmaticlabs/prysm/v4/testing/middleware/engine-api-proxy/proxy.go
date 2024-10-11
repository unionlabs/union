// Package proxy provides a proxy middleware for engine API requests between Ethereum
// consensus clients and execution clients accordingly. Allows for customizing
// in-flight requests or responses using custom triggers. Useful for end-to-end testing.
package proxy

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"
	"net"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/network"
	"github.com/sirupsen/logrus"
)

var (
	defaultProxyHost = "127.0.0.1"
	defaultProxyPort = 8545
)

type jsonRPCObject struct {
	Jsonrpc string        `json:"jsonrpc"`
	Method  string        `json:"method"`
	Params  []interface{} `json:"params"`
	ID      uint64        `json:"id"`
	Result  interface{}   `json:"result"`
}

type interceptorConfig struct {
	responseGen func() interface{}
	trigger     func() bool
}

// Proxy server that sits as a middleware between an Ethereum consensus client and an execution client,
// allowing us to modify in-flight requests and responses for testing purposes.
type Proxy struct {
	cfg              *config
	address          string
	srv              *http.Server
	lock             sync.RWMutex
	interceptors     map[string]*interceptorConfig
	backedUpRequests map[string][]*http.Request
}

// New creates a proxy server forwarding requests from a consensus client to an execution client.
func New(opts ...Option) (*Proxy, error) {
	p := &Proxy{
		cfg: &config{
			proxyHost: defaultProxyHost,
			proxyPort: defaultProxyPort,
			logger:    logrus.New(),
		},
		interceptors:     make(map[string]*interceptorConfig),
		backedUpRequests: map[string][]*http.Request{},
	}
	for _, o := range opts {
		if err := o(p); err != nil {
			return nil, err
		}
	}
	if p.cfg.destinationUrl == nil {
		return nil, errors.New("must provide a destination address for request proxying")
	}
	mux := http.NewServeMux()
	mux.Handle("/", p)
	addr := fmt.Sprintf("%s:%d", p.cfg.proxyHost, p.cfg.proxyPort)
	srv := &http.Server{
		Handler:           mux,
		Addr:              addr,
		ReadHeaderTimeout: time.Second,
	}
	p.address = addr
	p.srv = srv
	return p, nil
}

// Address for the proxy server.
func (p *Proxy) Address() string {
	return p.address
}

// Start a proxy server.
func (p *Proxy) Start(ctx context.Context) error {
	p.srv.BaseContext = func(listener net.Listener) context.Context {
		return ctx
	}
	p.cfg.logger.WithFields(logrus.Fields{
		"forwardingAddress": p.cfg.destinationUrl.String(),
	}).Infof("Engine proxy now listening on address %s", p.address)
	go func() {
		if err := p.srv.ListenAndServe(); err != nil {
			p.cfg.logger.Error(err)
		}
	}()
	for {
		<-ctx.Done()
		return p.srv.Shutdown(context.Background())
	}
}

// ServeHTTP requests from a consensus client to an execution client, modifying in-flight requests
// and/or responses as desired. It also processes any backed-up requests.
func (p *Proxy) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	requestBytes, err := parseRequestBytes(r)
	if err != nil {
		p.cfg.logger.WithError(err).Error("Could not parse request")
		return
	}
	// Check if we need to intercept the request with a custom response.
	hasIntercepted, err := p.interceptIfNeeded(requestBytes, w, r)
	if err != nil {
		p.cfg.logger.WithError(err).Error("Could not intercept request")
		return
	}
	if hasIntercepted {
		return
	}
	// If we are not intercepting the request, we proxy as normal.
	p.proxyRequest(requestBytes, w, r)
}

// AddRequestInterceptor for a desired json-rpc method by specifying a custom response
// and a function that checks if the interceptor should be triggered.
func (p *Proxy) AddRequestInterceptor(rpcMethodName string, response func() interface{}, trigger func() bool) {
	p.lock.Lock()
	defer p.lock.Unlock()
	p.cfg.logger.Infof("Adding in interceptor for method %s", rpcMethodName)
	p.interceptors[rpcMethodName] = &interceptorConfig{
		response,
		trigger,
	}
}

// RemoveRequestInterceptor removes the request interceptor for the provided method.
func (p *Proxy) RemoveRequestInterceptor(rpcMethodName string) {
	p.lock.Lock()
	defer p.lock.Unlock()
	p.cfg.logger.Infof("Removing interceptor for method %s", rpcMethodName)
	delete(p.interceptors, rpcMethodName)
}

// ReleaseBackedUpRequests releases backed up http requests which
// were previously ignored due to our interceptors.
func (p *Proxy) ReleaseBackedUpRequests(rpcMethodName string) {
	p.lock.Lock()
	defer p.lock.Unlock()
	reqs := p.backedUpRequests[rpcMethodName]
	for _, r := range reqs {
		p.cfg.logger.Infof("Sending backed up request for method %s", rpcMethodName)
		rBytes, err := parseRequestBytes(r)
		if err != nil {
			p.cfg.logger.Error(err)
			continue
		}
		res, err := p.sendHttpRequest(r, rBytes)
		if err != nil {
			p.cfg.logger.Error(err)
			continue
		}
		p.cfg.logger.Infof("Received response %s for backed up request for method %s", http.StatusText(res.StatusCode), rpcMethodName)
	}
	delete(p.backedUpRequests, rpcMethodName)
}

// Checks if there is a custom interceptor hook on the request, check if it can be
// triggered, and then write the custom response to the writer.
func (p *Proxy) interceptIfNeeded(requestBytes []byte, w http.ResponseWriter, r *http.Request) (hasIntercepted bool, err error) {
	if !isEngineAPICall(requestBytes) {
		return
	}
	var jreq *jsonRPCObject
	jreq, err = unmarshalRPCObject(requestBytes)
	if err != nil {
		return
	}
	p.lock.RLock()
	defer p.lock.RUnlock()
	interceptor, shouldIntercept := p.interceptors[jreq.Method]
	if !shouldIntercept {
		return
	}
	if !interceptor.trigger() {
		return
	}
	jResp := &jsonRPCObject{
		Method: jreq.Method,
		ID:     jreq.ID,
		Result: interceptor.responseGen(),
	}
	if err = json.NewEncoder(w).Encode(jResp); err != nil {
		return
	}
	hasIntercepted = true
	p.backedUpRequests[jreq.Method] = append(p.backedUpRequests[jreq.Method], r)
	return
}

// Create a new proxy request to the execution client.
func (p *Proxy) proxyRequest(requestBytes []byte, w http.ResponseWriter, r *http.Request) {
	jreq, err := unmarshalRPCObject(requestBytes)
	if err != nil {
		p.cfg.logger.WithError(err).Error("Could not unmarshal request")
		// Continue and mark it as unknown.
		jreq = &jsonRPCObject{Method: "unknown"}
	}
	p.cfg.logger.Infof("Forwarding %s request for method %s to %s", r.Method, jreq.Method, p.cfg.destinationUrl.String())
	proxyRes, err := p.sendHttpRequest(r, requestBytes)
	if err != nil {
		p.cfg.logger.WithError(err).Error("Could create new request")
		return
	}
	p.cfg.logger.Infof("Received response for %s request with method %s from %s", r.Method, jreq.Method, p.cfg.destinationUrl.String())

	defer func() {
		if err = proxyRes.Body.Close(); err != nil {
			p.cfg.logger.WithError(err).Error("Could not do close proxy responseGen body")
		}
	}()

	// Pipe the proxy responseGen to the original caller.
	if _, err = io.Copy(w, proxyRes.Body); err != nil {
		p.cfg.logger.WithError(err).Error("Could not copy proxy request body")
		return
	}
}

func (p *Proxy) sendHttpRequest(req *http.Request, requestBytes []byte) (*http.Response, error) {
	proxyReq, err := http.NewRequest(req.Method, p.cfg.destinationUrl.String(), req.Body)
	if err != nil {
		p.cfg.logger.WithError(err).Error("Could not create new request")
		return nil, err
	}

	// Set the modified request as the proxy request body.
	proxyReq.Body = ioutil.NopCloser(bytes.NewBuffer(requestBytes))

	// Required proxy headers for forwarding JSON-RPC requests to the execution client.
	proxyReq.Header.Set("Host", req.Host)
	proxyReq.Header.Set("X-Forwarded-For", req.RemoteAddr)
	proxyReq.Header.Set("Content-Type", "application/json")

	client := &http.Client{}
	if p.cfg.secret != "" {
		client = network.NewHttpClientWithSecret(p.cfg.secret, "")
	}
	proxyRes, err := client.Do(proxyReq)
	if err != nil {
		p.cfg.logger.WithError(err).Error("Could not forward request to destination server")
		return nil, err
	}
	return proxyRes, nil
}

// Peek into the bytes of an HTTP request's body.
func parseRequestBytes(req *http.Request) ([]byte, error) {
	requestBytes, err := ioutil.ReadAll(req.Body)
	if err != nil {
		return nil, err
	}
	if err = req.Body.Close(); err != nil {
		return nil, err
	}
	req.Body = ioutil.NopCloser(bytes.NewBuffer(requestBytes))
	return requestBytes, nil
}

// Checks whether the JSON-RPC request is for the Ethereum engine API.
func isEngineAPICall(reqBytes []byte) bool {
	jsonRequest, err := unmarshalRPCObject(reqBytes)
	if err != nil {
		switch {
		case strings.Contains(err.Error(), "cannot unmarshal array"):
			return false
		default:
			return false
		}
	}
	return strings.Contains(jsonRequest.Method, "engine_")
}

func unmarshalRPCObject(b []byte) (*jsonRPCObject, error) {
	r := &jsonRPCObject{}
	if err := json.Unmarshal(b, r); err != nil {
		return nil, err
	}
	return r, nil
}
