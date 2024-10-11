package eth1

import (
	"context"
	"encoding/hex"
	"fmt"
	"os"
	"path"
	"strconv"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/helpers"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	e2etypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	proxy "github.com/prysmaticlabs/prysm/v4/testing/middleware/engine-api-proxy"
	log "github.com/sirupsen/logrus"
)

// ProxySet represents a set of proxies for the engine-api.
type ProxySet struct {
	e2etypes.ComponentRunner
	started chan struct{}
	proxies []e2etypes.ComponentRunner
}

// NewProxySet creates and returns a set of engine-api proxies.
func NewProxySet() *ProxySet {
	return &ProxySet{
		started: make(chan struct{}, 1),
	}
}

// Start starts all the proxies in set.
func (s *ProxySet) Start(ctx context.Context) error {
	totalNodeCount := e2e.TestParams.BeaconNodeCount + e2e.TestParams.LighthouseBeaconNodeCount
	nodes := make([]e2etypes.ComponentRunner, totalNodeCount)
	for i := 0; i < totalNodeCount; i++ {
		nodes[i] = NewProxy(i)
	}
	s.proxies = nodes

	// Wait for all nodes to finish their job (blocking).
	// Once nodes are ready passed in handler function will be called.
	return helpers.WaitOnNodes(ctx, nodes, func() {
		// All nodes started, close channel, so that all services waiting on a set, can proceed.
		close(s.started)
	})
}

// Started checks whether proxy set is started and all proxies are ready to be queried.
func (s *ProxySet) Started() <-chan struct{} {
	return s.started
}

// Pause pauses the component and its underlying process.
func (s *ProxySet) Pause() error {
	for _, n := range s.proxies {
		if err := n.Pause(); err != nil {
			return err
		}
	}
	return nil
}

// Resume resumes the component and its underlying process.
func (s *ProxySet) Resume() error {
	for _, n := range s.proxies {
		if err := n.Resume(); err != nil {
			return err
		}
	}
	return nil
}

// Stop stops the component and its underlying process.
func (s *ProxySet) Stop() error {
	for _, n := range s.proxies {
		if err := n.Stop(); err != nil {
			return err
		}
	}
	return nil
}

// PauseAtIndex pauses the component and its underlying process at the desired index.
func (s *ProxySet) PauseAtIndex(i int) error {
	if i >= len(s.proxies) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.proxies))
	}
	return s.proxies[i].Pause()
}

// ResumeAtIndex resumes the component and its underlying process at the desired index.
func (s *ProxySet) ResumeAtIndex(i int) error {
	if i >= len(s.proxies) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.proxies))
	}
	return s.proxies[i].Resume()
}

// StopAtIndex stops the component and its underlying process at the desired index.
func (s *ProxySet) StopAtIndex(i int) error {
	if i >= len(s.proxies) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.proxies))
	}
	return s.proxies[i].Stop()
}

// ComponentAtIndex returns the component at the provided index.
func (s *ProxySet) ComponentAtIndex(i int) (e2etypes.ComponentRunner, error) {
	if i >= len(s.proxies) {
		return nil, errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.proxies))
	}
	return s.proxies[i], nil
}

// Proxy represents an engine-api proxy.
type Proxy struct {
	e2etypes.ComponentRunner
	started     chan struct{}
	index       int
	engineProxy *proxy.Proxy
	cancel      func()
}

// NewProxy creates and returns an engine-api proxy.
func NewProxy(index int) *Proxy {
	return &Proxy{
		started: make(chan struct{}, 1),
		index:   index,
	}
}

// Start runs a proxy.
func (node *Proxy) Start(ctx context.Context) error {
	f, err := os.Create(path.Join(e2e.TestParams.LogPath, "eth1_proxy_"+strconv.Itoa(node.index)+".log"))
	if err != nil {
		return err
	}
	jwtPath := path.Join(e2e.TestParams.TestPath, "eth1data/"+strconv.Itoa(node.index)+"/")
	if node.index == 0 {
		jwtPath = path.Join(e2e.TestParams.TestPath, "eth1data/miner/")
	}
	jwtPath = path.Join(jwtPath, "geth/jwtsecret")
	secret, err := parseJWTSecretFromFile(jwtPath)
	if err != nil {
		return err
	}
	opts := []proxy.Option{
		proxy.WithDestinationAddress(fmt.Sprintf("http://127.0.0.1:%d", e2e.TestParams.Ports.Eth1AuthRPCPort+node.index)),
		proxy.WithPort(e2e.TestParams.Ports.Eth1ProxyPort + node.index),
		proxy.WithLogger(log.New()),
		proxy.WithLogFile(f),
		proxy.WithJwtSecret(string(secret)),
	}
	nProxy, err := proxy.New(opts...)
	if err != nil {
		return err
	}
	log.Infof("Starting eth1 proxy %d with port: %d and file %s", node.index, e2e.TestParams.Ports.Eth1ProxyPort+node.index, f.Name())

	// Set cancel into context.
	ctx, cancel := context.WithCancel(ctx)
	node.cancel = cancel
	node.engineProxy = nProxy
	// Mark node as ready.
	close(node.started)
	return nProxy.Start(ctx)
}

// Started checks whether the eth1 proxy is started and ready to be queried.
func (node *Proxy) Started() <-chan struct{} {
	return node.started
}

// Pause pauses the component and its underlying process.
func (node *Proxy) Pause() error {
	// no-op
	return nil
}

// Resume resumes the component and its underlying process.
func (node *Proxy) Resume() error {
	// no-op
	return nil
}

// Stop kills the component and its underlying process.
func (node *Proxy) Stop() error {
	node.cancel()
	return nil
}

// AddRequestInterceptor adds in a json-rpc request interceptor.
func (node *Proxy) AddRequestInterceptor(rpcMethodName string, responseGen func() interface{}, trigger func() bool) {
	node.engineProxy.AddRequestInterceptor(rpcMethodName, responseGen, trigger)
}

// RemoveRequestInterceptor removes the request interceptor for the provided method.
func (node *Proxy) RemoveRequestInterceptor(rpcMethodName string) {
	node.engineProxy.RemoveRequestInterceptor(rpcMethodName)
}

// ReleaseBackedUpRequests releases backed up http requests which
// were previously ignored due to our interceptors.
func (node *Proxy) ReleaseBackedUpRequests(rpcMethodName string) {
	node.engineProxy.ReleaseBackedUpRequests(rpcMethodName)
}

func parseJWTSecretFromFile(jwtSecretFile string) ([]byte, error) {
	enc, err := file.ReadFileAsBytes(jwtSecretFile)
	if err != nil {
		return nil, err
	}
	strData := strings.TrimSpace(string(enc))
	if len(strData) == 0 {
		return nil, fmt.Errorf("provided JWT secret in file %s cannot be empty", jwtSecretFile)
	}
	secret, err := hex.DecodeString(strings.TrimPrefix(strData, "0x"))
	if err != nil {
		return nil, err
	}
	if len(secret) < 32 {
		return nil, errors.New("provided JWT secret should be a hex string of at least 32 bytes")
	}
	return secret, nil
}
