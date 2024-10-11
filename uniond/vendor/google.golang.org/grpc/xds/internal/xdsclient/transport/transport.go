/*
 *
 * Copyright 2022 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// Package transport implements the xDS transport protocol functionality
// required by the xdsclient.
package transport

import (
	"context"
	"errors"
	"fmt"
	"sync"
	"sync/atomic"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/backoff"
	"google.golang.org/grpc/internal/buffer"
	"google.golang.org/grpc/internal/grpclog"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/keepalive"
	xdsclientinternal "google.golang.org/grpc/xds/internal/xdsclient/internal"
	"google.golang.org/grpc/xds/internal/xdsclient/load"
	transportinternal "google.golang.org/grpc/xds/internal/xdsclient/transport/internal"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
	"google.golang.org/protobuf/types/known/anypb"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3adsgrpc "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	statuspb "google.golang.org/genproto/googleapis/rpc/status"
)

type adsStream = v3adsgrpc.AggregatedDiscoveryService_StreamAggregatedResourcesClient

func init() {
	transportinternal.GRPCNewClient = grpc.NewClient
	xdsclientinternal.NewADSStream = func(ctx context.Context, cc *grpc.ClientConn) (adsStream, error) {
		return v3adsgrpc.NewAggregatedDiscoveryServiceClient(cc).StreamAggregatedResources(ctx)
	}
}

// Any per-RPC level logs which print complete request or response messages
// should be gated at this verbosity level. Other per-RPC level logs which print
// terse output should be at `INFO` and verbosity 2.
const perRPCVerbosityLevel = 9

// Transport provides a resource-type agnostic implementation of the xDS
// transport protocol. At this layer, resource contents are supposed to be
// opaque blobs which should be meaningful only to the xDS data model layer
// which is implemented by the `xdsresource` package.
//
// Under the hood, it owns the gRPC connection to a single management server and
// manages the lifecycle of ADS/LRS streams. It uses the xDS v3 transport
// protocol version.
type Transport struct {
	// These fields are initialized at creation time and are read-only afterwards.
	cc              *grpc.ClientConn        // ClientConn to the management server.
	serverURI       string                  // URI of the management server.
	onRecvHandler   OnRecvHandlerFunc       // Resource update handler. xDS data model layer.
	onErrorHandler  func(error)             // To report underlying stream errors.
	onSendHandler   OnSendHandlerFunc       // To report resources requested on ADS stream.
	lrsStore        *load.Store             // Store returned to user for pushing loads.
	backoff         func(int) time.Duration // Backoff after stream failures.
	nodeProto       *v3corepb.Node          // Identifies the gRPC application.
	logger          *grpclog.PrefixLogger   // Prefix logger for transport logs.
	adsRunnerCancel context.CancelFunc      // CancelFunc for the ADS goroutine.
	adsRunnerDoneCh chan struct{}           // To notify exit of ADS goroutine.
	lrsRunnerDoneCh chan struct{}           // To notify exit of LRS goroutine.

	// These channels enable synchronization amongst the different goroutines
	// spawned by the transport, and between asynchronous events resulting from
	// receipt of responses from the management server.
	adsStreamCh  chan adsStream    // New ADS streams are pushed here.
	adsRequestCh *buffer.Unbounded // Resource and ack requests are pushed here.

	// mu guards the following runtime state maintained by the transport.
	mu sync.Mutex
	// resources is map from resource type URL to the set of resource names
	// being requested for that type. When the ADS stream is restarted, the
	// transport requests all these resources again from the management server.
	resources map[string]map[string]bool
	// versions is a map from resource type URL to the most recently ACKed
	// version for that resource. Resource versions are a property of the
	// resource type and not the stream, and will not be reset upon stream
	// restarts.
	versions map[string]string
	// nonces is a map from resource type URL to the most recently received
	// nonce for that resource type. Nonces are a property of the ADS stream and
	// will be reset upon stream restarts.
	nonces map[string]string

	lrsMu           sync.Mutex         // Protects all LRS state.
	lrsCancelStream context.CancelFunc // CancelFunc for the LRS stream.
	lrsRefCount     int                // Reference count on the load store.
}

// OnRecvHandlerFunc is the implementation at the xDS data model layer, which
// determines if the configuration received from the management server can be
// applied locally or not.
//
// A nil error is returned from this function when the data model layer believes
// that the received configuration is good and can be applied locally. This will
// cause the transport layer to send an ACK to the management server. A non-nil
// error is returned from this function when the data model layer believes
// otherwise, and this will cause the transport layer to send a NACK.
//
// The implementation is expected to invoke onDone when local processing of the
// update is complete, i.e. it is consumed by all watchers.
type OnRecvHandlerFunc func(update ResourceUpdate, onDone func()) error

// OnSendHandlerFunc is the implementation at the authority, which handles state
// changes for the resource watch and stop watch timers accordingly.
type OnSendHandlerFunc func(update *ResourceSendInfo)

// ResourceUpdate is a representation of the configuration update received from
// the management server. It only contains fields which are useful to the data
// model layer, and layers above it.
type ResourceUpdate struct {
	// Resources is the list of resources received from the management server.
	Resources []*anypb.Any
	// URL is the resource type URL for the above resources.
	URL string
	// Version is the resource version, for the above resources, as specified by
	// the management server.
	Version string
}

// Options specifies configuration knobs used when creating a new Transport.
type Options struct {
	// ServerCfg contains all the configuration required to connect to the xDS
	// management server.
	ServerCfg *bootstrap.ServerConfig
	// OnRecvHandler is the component which makes ACK/NACK decisions based on
	// the received resources.
	//
	// Invoked inline and implementations must not block.
	OnRecvHandler OnRecvHandlerFunc
	// OnErrorHandler provides a way for the transport layer to report
	// underlying stream errors. These can be bubbled all the way up to the user
	// of the xdsClient.
	//
	// Invoked inline and implementations must not block.
	OnErrorHandler func(error)
	// OnSendHandler provides a way for the transport layer to report underlying
	// resource requests sent on the stream. However, Send() on the ADS stream will
	// return successfully as long as:
	//   1. there is enough flow control quota to send the message.
	//   2. the message is added to the send buffer.
	// However, the connection may fail after the callback is invoked and before
	// the message is actually sent on the wire. This is accepted.
	//
	// Invoked inline and implementations must not block.
	OnSendHandler func(*ResourceSendInfo)
	// Backoff controls the amount of time to backoff before recreating failed
	// ADS streams. If unspecified, a default exponential backoff implementation
	// is used. For more details, see:
	// https://github.com/grpc/grpc/blob/master/doc/connection-backoff.md.
	Backoff func(retries int) time.Duration
	// Logger does logging with a prefix.
	Logger *grpclog.PrefixLogger
	// NodeProto contains the Node proto to be used in xDS requests. This will be
	// of type *v3corepb.Node.
	NodeProto *v3corepb.Node
}

// New creates a new Transport.
func New(opts Options) (*Transport, error) {
	switch {
	case opts.OnRecvHandler == nil:
		return nil, errors.New("missing OnRecv callback handler when creating a new transport")
	case opts.OnErrorHandler == nil:
		return nil, errors.New("missing OnError callback handler when creating a new transport")
	case opts.OnSendHandler == nil:
		return nil, errors.New("missing OnSend callback handler when creating a new transport")
	}

	// Dial the xDS management with the passed in credentials.
	dopts := []grpc.DialOption{
		opts.ServerCfg.CredsDialOption(),
		grpc.WithKeepaliveParams(keepalive.ClientParameters{
			// We decided to use these sane defaults in all languages, and
			// kicked the can down the road as far making these configurable.
			Time:    5 * time.Minute,
			Timeout: 20 * time.Second,
		}),
	}
	grpcNewClient := transportinternal.GRPCNewClient.(func(string, ...grpc.DialOption) (*grpc.ClientConn, error))
	cc, err := grpcNewClient(opts.ServerCfg.ServerURI(), dopts...)
	if err != nil {
		// An error from a non-blocking dial indicates something serious.
		return nil, fmt.Errorf("failed to create a transport to the management server %q: %v", opts.ServerCfg.ServerURI(), err)
	}
	cc.Connect()

	boff := opts.Backoff
	if boff == nil {
		boff = backoff.DefaultExponential.Backoff
	}
	ret := &Transport{
		cc:             cc,
		serverURI:      opts.ServerCfg.ServerURI(),
		onRecvHandler:  opts.OnRecvHandler,
		onErrorHandler: opts.OnErrorHandler,
		onSendHandler:  opts.OnSendHandler,
		lrsStore:       load.NewStore(),
		backoff:        boff,
		nodeProto:      opts.NodeProto,
		logger:         opts.Logger,

		adsStreamCh:     make(chan adsStream, 1),
		adsRequestCh:    buffer.NewUnbounded(),
		resources:       make(map[string]map[string]bool),
		versions:        make(map[string]string),
		nonces:          make(map[string]string),
		adsRunnerDoneCh: make(chan struct{}),
	}

	// This context is used for sending and receiving RPC requests and
	// responses. It is also used by all the goroutines spawned by this
	// Transport. Therefore, cancelling this context when the transport is
	// closed will essentially cancel any pending RPCs, and cause the goroutines
	// to terminate.
	ctx, cancel := context.WithCancel(context.Background())
	ret.adsRunnerCancel = cancel
	go ret.adsRunner(ctx)

	ret.logger.Infof("Created transport to server %q", ret.serverURI)
	return ret, nil
}

// resourceRequest wraps the resource type url and the resource names requested
// by the user of this transport.
type resourceRequest struct {
	resources []string
	url       string
}

// SendRequest sends out an ADS request for the provided resources of the
// specified resource type.
//
// The request is sent out asynchronously. If no valid stream exists at the time
// of processing this request, it is queued and will be sent out once a valid
// stream exists.
//
// If a successful response is received, the update handler callback provided at
// creation time is invoked. If an error is encountered, the stream error
// handler callback provided at creation time is invoked.
func (t *Transport) SendRequest(url string, resources []string) {
	t.adsRequestCh.Put(&resourceRequest{
		url:       url,
		resources: resources,
	})
}

// ResourceSendInfo wraps the names and url of resources sent to the management
// server. This is used by the `authority` type to start/stop the watch timer
// associated with every resource in the update.
type ResourceSendInfo struct {
	ResourceNames []string
	URL           string
}

func (t *Transport) sendAggregatedDiscoveryServiceRequest(stream adsStream, sendNodeProto bool, resourceNames []string, resourceURL, version, nonce string, nackErr error) error {
	req := &v3discoverypb.DiscoveryRequest{
		TypeUrl:       resourceURL,
		ResourceNames: resourceNames,
		VersionInfo:   version,
		ResponseNonce: nonce,
	}
	if sendNodeProto {
		req.Node = t.nodeProto
	}
	if nackErr != nil {
		req.ErrorDetail = &statuspb.Status{
			Code: int32(codes.InvalidArgument), Message: nackErr.Error(),
		}
	}
	if err := stream.Send(req); err != nil {
		return err
	}
	if t.logger.V(perRPCVerbosityLevel) {
		t.logger.Infof("ADS request sent: %v", pretty.ToJSON(req))
	} else {
		if t.logger.V(2) {
			t.logger.Infof("ADS request sent for type %q, resources: %v, version %q, nonce %q", resourceURL, resourceNames, version, nonce)
		}
	}
	t.onSendHandler(&ResourceSendInfo{URL: resourceURL, ResourceNames: resourceNames})
	return nil
}

func (t *Transport) recvAggregatedDiscoveryServiceResponse(stream adsStream) (resources []*anypb.Any, resourceURL, version, nonce string, err error) {
	resp, err := stream.Recv()
	if err != nil {
		return nil, "", "", "", err
	}
	if t.logger.V(perRPCVerbosityLevel) {
		t.logger.Infof("ADS response received: %v", pretty.ToJSON(resp))
	} else if t.logger.V(2) {
		t.logger.Infof("ADS response received for type %q, version %q, nonce %q", resp.GetTypeUrl(), resp.GetVersionInfo(), resp.GetNonce())
	}
	return resp.GetResources(), resp.GetTypeUrl(), resp.GetVersionInfo(), resp.GetNonce(), nil
}

// adsRunner starts an ADS stream (and backs off exponentially, if the previous
// stream failed without receiving a single reply) and runs the sender and
// receiver routines to send and receive data from the stream respectively.
func (t *Transport) adsRunner(ctx context.Context) {
	defer close(t.adsRunnerDoneCh)

	go t.send(ctx)

	// We reset backoff state when we successfully receive at least one
	// message from the server.
	runStreamWithBackoff := func() error {
		newStream := xdsclientinternal.NewADSStream.(func(context.Context, *grpc.ClientConn) (adsStream, error))
		stream, err := newStream(ctx, t.cc)
		if err != nil {
			t.onErrorHandler(err)
			t.logger.Warningf("Creating new ADS stream failed: %v", err)
			return nil
		}
		t.logger.Infof("ADS stream created")

		select {
		case <-t.adsStreamCh:
		default:
		}
		t.adsStreamCh <- stream
		msgReceived := t.recv(ctx, stream)
		if msgReceived {
			return backoff.ErrResetBackoff
		}
		return nil
	}
	backoff.RunF(ctx, runStreamWithBackoff, t.backoff)
}

// send is a separate goroutine for sending resource requests on the ADS stream.
//
// For every new stream received on the stream channel, all existing resources
// are re-requested from the management server.
//
// For every new resource request received on the resources channel, the
// resources map is updated (this ensures that resend will pick them up when
// there are new streams) and the appropriate request is sent out.
func (t *Transport) send(ctx context.Context) {
	var stream adsStream
	// The xDS protocol only requires that we send the node proto in the first
	// discovery request on every stream. Sending the node proto in every
	// request message wastes CPU resources on the client and the server.
	sentNodeProto := false
	for {
		select {
		case <-ctx.Done():
			return
		case stream = <-t.adsStreamCh:
			// We have a new stream and we've to ensure that the node proto gets
			// sent out in the first request on the stream.
			var err error
			if sentNodeProto, err = t.sendExisting(stream); err != nil {
				// Send failed, clear the current stream. Attempt to resend will
				// only be made after a new stream is created.
				stream = nil
				continue
			}
		case u, ok := <-t.adsRequestCh.Get():
			if !ok {
				// No requests will be sent after the adsRequestCh buffer is closed.
				return
			}
			t.adsRequestCh.Load()

			var (
				resources           []string
				url, version, nonce string
				send                bool
				nackErr             error
			)
			switch update := u.(type) {
			case *resourceRequest:
				resources, url, version, nonce = t.processResourceRequest(update)
			case *ackRequest:
				resources, url, version, nonce, send = t.processAckRequest(update, stream)
				if !send {
					continue
				}
				nackErr = update.nackErr
			}
			if stream == nil {
				// There's no stream yet. Skip the request. This request
				// will be resent to the new streams. If no stream is
				// created, the watcher will timeout (same as server not
				// sending response back).
				continue
			}
			if err := t.sendAggregatedDiscoveryServiceRequest(stream, !sentNodeProto, resources, url, version, nonce, nackErr); err != nil {
				t.logger.Warningf("Sending ADS request for resources: %q, url: %q, version: %q, nonce: %q failed: %v", resources, url, version, nonce, err)
				// Send failed, clear the current stream.
				stream = nil
			}
			sentNodeProto = true
		}
	}
}

// sendExisting sends out xDS requests for existing resources when recovering
// from a broken stream.
//
// We call stream.Send() here with the lock being held. It should be OK to do
// that here because the stream has just started and Send() usually returns
// quickly (once it pushes the message onto the transport layer) and is only
// ever blocked if we don't have enough flow control quota.
//
// Returns true if the node proto was sent.
func (t *Transport) sendExisting(stream adsStream) (sentNodeProto bool, err error) {
	t.mu.Lock()
	defer t.mu.Unlock()

	// Reset only the nonces map when the stream restarts.
	//
	// xDS spec says the following. See section:
	// https://www.envoyproxy.io/docs/envoy/latest/api-docs/xds_protocol#ack-nack-and-resource-type-instance-version
	//
	// Note that the version for a resource type is not a property of an
	// individual xDS stream but rather a property of the resources themselves. If
	// the stream becomes broken and the client creates a new stream, the client’s
	// initial request on the new stream should indicate the most recent version
	// seen by the client on the previous stream
	t.nonces = make(map[string]string)

	// Send node proto only in the first request on the stream.
	for url, resources := range t.resources {
		if len(resources) == 0 {
			continue
		}
		if err := t.sendAggregatedDiscoveryServiceRequest(stream, !sentNodeProto, mapToSlice(resources), url, t.versions[url], "", nil); err != nil {
			t.logger.Warningf("Sending ADS request for resources: %q, url: %q, version: %q, nonce: %q failed: %v", resources, url, t.versions[url], "", err)
			return false, err
		}
		sentNodeProto = true
	}

	return sentNodeProto, nil
}

// recv receives xDS responses on the provided ADS stream and branches out to
// message specific handlers. Returns true if at least one message was
// successfully received.
func (t *Transport) recv(ctx context.Context, stream adsStream) bool {
	// Initialize the flow control quota for the stream. This helps to block the
	// next read until the previous one is consumed by all watchers.
	fc := newADSFlowControl()

	msgReceived := false
	for {
		// Wait for ADS stream level flow control to be available.
		if !fc.wait(ctx) {
			if t.logger.V(2) {
				t.logger.Infof("ADS stream context canceled")
			}
			return msgReceived
		}

		resources, url, rVersion, nonce, err := t.recvAggregatedDiscoveryServiceResponse(stream)
		if err != nil {
			// Note that we do not consider it an error if the ADS stream was closed
			// after having received a response on the stream. This is because there
			// are legitimate reasons why the server may need to close the stream during
			// normal operations, such as needing to rebalance load or the underlying
			// connection hitting its max connection age limit.
			// (see [gRFC A9](https://github.com/grpc/proposal/blob/master/A9-server-side-conn-mgt.md)).
			if msgReceived {
				err = xdsresource.NewErrorf(xdsresource.ErrTypeStreamFailedAfterRecv, err.Error())
			}
			t.onErrorHandler(err)
			t.logger.Warningf("ADS stream closed: %v", err)
			return msgReceived
		}
		msgReceived = true

		u := ResourceUpdate{
			Resources: resources,
			URL:       url,
			Version:   rVersion,
		}
		fc.setPending()
		if err = t.onRecvHandler(u, fc.onDone); xdsresource.ErrType(err) == xdsresource.ErrorTypeResourceTypeUnsupported {
			t.logger.Warningf("%v", err)
			continue
		}
		// If the data model layer returned an error, we need to NACK the
		// response in which case we need to set the version to the most
		// recently accepted version of this resource type.
		if err != nil {
			t.mu.Lock()
			t.adsRequestCh.Put(&ackRequest{
				url:     url,
				nonce:   nonce,
				stream:  stream,
				version: t.versions[url],
				nackErr: err,
			})
			t.mu.Unlock()
			t.logger.Warningf("Sending NACK for resource type: %q, version: %q, nonce: %q, reason: %v", url, rVersion, nonce, err)
			continue
		}
		t.adsRequestCh.Put(&ackRequest{
			url:     url,
			nonce:   nonce,
			stream:  stream,
			version: rVersion,
		})
		if t.logger.V(2) {
			t.logger.Infof("Sending ACK for resource type: %q, version: %q, nonce: %q", url, rVersion, nonce)
		}
	}
}

func mapToSlice(m map[string]bool) []string {
	ret := make([]string, 0, len(m))
	for i := range m {
		ret = append(ret, i)
	}
	return ret
}

func sliceToMap(ss []string) map[string]bool {
	ret := make(map[string]bool, len(ss))
	for _, s := range ss {
		ret[s] = true
	}
	return ret
}

// processResourceRequest pulls the fields needed to send out an ADS request.
// The resource type and the list of resources to request are provided by the
// user, while the version and nonce are maintained internally.
//
// The resources map, which keeps track of the resources being requested, is
// updated here. Any subsequent stream failure will re-request resources stored
// in this map.
//
// Returns the list of resources, resource type url, version and nonce.
func (t *Transport) processResourceRequest(req *resourceRequest) ([]string, string, string, string) {
	t.mu.Lock()
	defer t.mu.Unlock()

	resources := sliceToMap(req.resources)
	t.resources[req.url] = resources
	return req.resources, req.url, t.versions[req.url], t.nonces[req.url]
}

type ackRequest struct {
	url     string // Resource type URL.
	version string // NACK if version is an empty string.
	nonce   string
	nackErr error // nil for ACK, non-nil for NACK.
	// ACK/NACK are tagged with the stream it's for. When the stream is down,
	// all the ACK/NACK for this stream will be dropped, and the version/nonce
	// won't be updated.
	stream grpc.ClientStream
}

// processAckRequest pulls the fields needed to send out an ADS ACK. The nonces
// and versions map is updated.
//
// Returns the list of resources, resource type url, version, nonce, and an
// indication of whether an ACK should be sent on the wire or not.
func (t *Transport) processAckRequest(ack *ackRequest, stream grpc.ClientStream) ([]string, string, string, string, bool) {
	if ack.stream != stream {
		// If ACK's stream isn't the current sending stream, this means the ACK
		// was pushed to queue before the old stream broke, and a new stream has
		// been started since. Return immediately here so we don't update the
		// nonce for the new stream.
		return nil, "", "", "", false
	}

	t.mu.Lock()
	defer t.mu.Unlock()

	// Update the nonce irrespective of whether we send the ACK request on wire.
	// An up-to-date nonce is required for the next request.
	nonce := ack.nonce
	t.nonces[ack.url] = nonce

	s, ok := t.resources[ack.url]
	if !ok || len(s) == 0 {
		// We don't send the ACK request if there are no resources of this type
		// in our resources map. This can be either when the server sends
		// responses before any request, or the resources are removed while the
		// ackRequest was in queue). If we send a request with an empty
		// resource name list, the server may treat it as a wild card and send
		// us everything.
		return nil, "", "", "", false
	}
	resources := mapToSlice(s)

	// Update the versions map only when we plan to send an ACK.
	if ack.nackErr == nil {
		t.versions[ack.url] = ack.version
	}

	return resources, ack.url, ack.version, nonce, true
}

// Close closes the Transport and frees any associated resources.
func (t *Transport) Close() {
	t.adsRunnerCancel()
	<-t.adsRunnerDoneCh
	t.adsRequestCh.Close()
	t.cc.Close()
}

// ChannelConnectivityStateForTesting returns the connectivity state of the gRPC
// channel to the management server.
//
// Only for testing purposes.
func (t *Transport) ChannelConnectivityStateForTesting() connectivity.State {
	return t.cc.GetState()
}

// adsFlowControl implements ADS stream level flow control that enables the
// transport to block the reading of the next message off of the stream until
// the previous update is consumed by all watchers.
//
// The lifetime of the flow control is tied to the lifetime of the stream.
type adsFlowControl struct {
	logger *grpclog.PrefixLogger

	// Whether the most recent update is pending consumption by all watchers.
	pending atomic.Bool
	// Channel used to notify when all the watchers have consumed the most
	// recent update. Wait() blocks on reading a value from this channel.
	readyCh chan struct{}
}

// newADSFlowControl returns a new adsFlowControl.
func newADSFlowControl() *adsFlowControl {
	return &adsFlowControl{readyCh: make(chan struct{}, 1)}
}

// setPending changes the internal state to indicate that there is an update
// pending consumption by all watchers.
func (fc *adsFlowControl) setPending() {
	fc.pending.Store(true)
}

// wait blocks until all the watchers have consumed the most recent update and
// returns true. If the context expires before that, it returns false.
func (fc *adsFlowControl) wait(ctx context.Context) bool {
	// If there is no pending update, there is no need to block.
	if !fc.pending.Load() {
		// If all watchers finished processing the most recent update before the
		// `recv` goroutine made the next call to `Wait()`, there would be an
		// entry in the readyCh channel that needs to be drained to ensure that
		// the next call to `Wait()` doesn't unblock before it actually should.
		select {
		case <-fc.readyCh:
		default:
		}
		return true
	}

	select {
	case <-ctx.Done():
		return false
	case <-fc.readyCh:
		return true
	}
}

// onDone indicates that all watchers have consumed the most recent update.
func (fc *adsFlowControl) onDone() {
	fc.pending.Store(false)

	select {
	// Writes to the readyCh channel should not block ideally. The default
	// branch here is to appease the paranoid mind.
	case fc.readyCh <- struct{}{}:
	default:
		if fc.logger.V(2) {
			fc.logger.Infof("ADS stream flow control readyCh is full")
		}
	}
}
