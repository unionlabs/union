/*
 *
 * Copyright 2019 gRPC authors.
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
 *
 */

// Package fakeclient provides a fake implementation of an xDS client.
package fakeclient

import (
	"context"

	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/load"
)

// Client is a fake implementation of an xds client. It exposes a bunch of
// channels to signal the occurrence of various events.
type Client struct {
	// Embed XDSClient so this fake client implements the interface, but it's
	// never set (it's always nil). This may cause nil panic since not all the
	// methods are implemented.
	xdsclient.XDSClient

	name         string
	loadReportCh *testutils.Channel
	lrsCancelCh  *testutils.Channel
	loadStore    *load.Store
	bootstrapCfg *bootstrap.Config
}

// ReportLoadArgs wraps the arguments passed to ReportLoad.
type ReportLoadArgs struct {
	// Server is the name of the server to which the load is reported.
	Server *bootstrap.ServerConfig
}

// ReportLoad starts reporting load about clusterName to server.
func (xdsC *Client) ReportLoad(server *bootstrap.ServerConfig) (loadStore *load.Store, cancel func()) {
	xdsC.loadReportCh.Send(ReportLoadArgs{Server: server})
	return xdsC.loadStore, func() {
		xdsC.lrsCancelCh.Send(nil)
	}
}

// WaitForCancelReportLoad waits for a load report to be cancelled and returns
// context.DeadlineExceeded otherwise.
func (xdsC *Client) WaitForCancelReportLoad(ctx context.Context) error {
	_, err := xdsC.lrsCancelCh.Receive(ctx)
	return err
}

// LoadStore returns the underlying load data store.
func (xdsC *Client) LoadStore() *load.Store {
	return xdsC.loadStore
}

// WaitForReportLoad waits for ReportLoad to be invoked on this client and
// returns the arguments passed to it.
func (xdsC *Client) WaitForReportLoad(ctx context.Context) (ReportLoadArgs, error) {
	val, err := xdsC.loadReportCh.Receive(ctx)
	if err != nil {
		return ReportLoadArgs{}, err
	}
	return val.(ReportLoadArgs), nil
}

// BootstrapConfig returns the bootstrap config.
func (xdsC *Client) BootstrapConfig() *bootstrap.Config {
	return xdsC.bootstrapCfg
}

// SetBootstrapConfig updates the bootstrap config.
func (xdsC *Client) SetBootstrapConfig(cfg *bootstrap.Config) {
	xdsC.bootstrapCfg = cfg
}

// Name returns the name of the xds client.
func (xdsC *Client) Name() string {
	return xdsC.name
}

// NewClient returns a new fake xds client.
func NewClient() *Client {
	return NewClientWithName("")
}

// NewClientWithName returns a new fake xds client with the provided name. This
// is used in cases where multiple clients are created in the tests and we need
// to make sure the client is created for the expected balancer name.
func NewClientWithName(name string) *Client {
	return &Client{
		name:         name,
		loadReportCh: testutils.NewChannel(),
		lrsCancelCh:  testutils.NewChannel(),
		loadStore:    load.NewStore(),
		bootstrapCfg: &bootstrap.Config{},
	}
}
