/*
 *
 * Copyright 2021 gRPC authors.
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

// Package e2e implements xds e2e tests using go-control-plane.
package e2e

import (
	"context"
	"fmt"
	"io"
	"os"
	"os/exec"

	"google.golang.org/grpc"
	channelzgrpc "google.golang.org/grpc/channelz/grpc_channelz_v1"
	channelzpb "google.golang.org/grpc/channelz/grpc_channelz_v1"
	"google.golang.org/grpc/credentials/insecure"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

func cmd(path string, logger io.Writer, args []string, env []string) *exec.Cmd {
	cmd := exec.Command(path, args...)
	cmd.Env = append(os.Environ(), env...)
	cmd.Stdout = logger
	cmd.Stderr = logger
	return cmd
}

const (
	clientStatsPort = 60363 // TODO: make this different per-test, only needed for parallel tests.
)

type client struct {
	cmd *exec.Cmd

	target  string
	statsCC *grpc.ClientConn
}

// newClient create a client with the given target and bootstrap content.
func newClient(target, binaryPath, bootstrap string, logger io.Writer, flags ...string) (*client, error) {
	cmd := cmd(
		binaryPath,
		logger,
		append([]string{
			"--server=" + target,
			"--print_response=true",
			"--qps=100",
			fmt.Sprintf("--stats_port=%d", clientStatsPort),
		}, flags...), // Append any flags from caller.
		[]string{
			"GRPC_GO_LOG_VERBOSITY_LEVEL=99",
			"GRPC_GO_LOG_SEVERITY_LEVEL=info",
			"GRPC_XDS_BOOTSTRAP_CONFIG=" + bootstrap, // The bootstrap content doesn't need to be quoted.
		},
	)
	cmd.Start()

	cc, err := grpc.Dial(fmt.Sprintf("localhost:%d", clientStatsPort), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithDefaultCallOptions(grpc.WaitForReady(true)))
	if err != nil {
		return nil, err
	}
	return &client{
		cmd:     cmd,
		target:  target,
		statsCC: cc,
	}, nil
}

func (c *client) clientStats(ctx context.Context) (*testpb.LoadBalancerStatsResponse, error) {
	ccc := testgrpc.NewLoadBalancerStatsServiceClient(c.statsCC)
	return ccc.GetClientStats(ctx, &testpb.LoadBalancerStatsRequest{
		NumRpcs:    100,
		TimeoutSec: 10,
	})
}

func (c *client) configRPCs(ctx context.Context, req *testpb.ClientConfigureRequest) error {
	ccc := testgrpc.NewXdsUpdateClientConfigureServiceClient(c.statsCC)
	_, err := ccc.Configure(ctx, req)
	return err
}

func (c *client) channelzSubChannels(ctx context.Context) ([]*channelzpb.Subchannel, error) {
	ccc := channelzgrpc.NewChannelzClient(c.statsCC)
	r, err := ccc.GetTopChannels(ctx, &channelzpb.GetTopChannelsRequest{})
	if err != nil {
		return nil, err
	}

	var ret []*channelzpb.Subchannel
	for _, cc := range r.Channel {
		if cc.Data.Target != c.target {
			continue
		}
		for _, sc := range cc.SubchannelRef {
			rr, err := ccc.GetSubchannel(ctx, &channelzpb.GetSubchannelRequest{SubchannelId: sc.SubchannelId})
			if err != nil {
				return nil, err
			}
			ret = append(ret, rr.Subchannel)
		}
	}
	return ret, nil
}

func (c *client) stop() {
	c.cmd.Process.Kill()
	c.cmd.Wait()
}

const (
	serverPort = 50051 // TODO: make this different per-test, only needed for parallel tests.
)

type server struct {
	cmd  *exec.Cmd
	port int
}

// newServer creates multiple servers with the given bootstrap content.
//
// Each server gets a different hostname, in the format of
// <hostnamePrefix>-<index>.
func newServers(hostnamePrefix, binaryPath, bootstrap string, logger io.Writer, count int) (_ []*server, err error) {
	var ret []*server
	defer func() {
		if err != nil {
			for _, s := range ret {
				s.stop()
			}
		}
	}()
	for i := 0; i < count; i++ {
		port := serverPort + i
		cmd := cmd(
			binaryPath,
			logger,
			[]string{
				fmt.Sprintf("--port=%d", port),
				fmt.Sprintf("--host_name_override=%s-%d", hostnamePrefix, i),
			},
			[]string{
				"GRPC_GO_LOG_VERBOSITY_LEVEL=99",
				"GRPC_GO_LOG_SEVERITY_LEVEL=info",
				"GRPC_XDS_BOOTSTRAP_CONFIG=" + bootstrap, // The bootstrap content doesn't need to be quoted.,
			},
		)
		cmd.Start()
		ret = append(ret, &server{cmd: cmd, port: port})
	}
	return ret, nil
}

func (s *server) stop() {
	s.cmd.Process.Kill()
	s.cmd.Wait()
}
