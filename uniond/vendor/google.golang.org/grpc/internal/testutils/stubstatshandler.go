/*
 *
 * Copyright 2023 gRPC authors.
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

package testutils

import (
	"context"

	"google.golang.org/grpc/stats"
)

// StubStatsHandler is a stats handler that is easy to customize within
// individual test cases. It is a stubbable implementation of
// google.golang.org/grpc/stats.Handler for testing purposes.
type StubStatsHandler struct {
	TagRPCF     func(ctx context.Context, info *stats.RPCTagInfo) context.Context
	HandleRPCF  func(ctx context.Context, info stats.RPCStats)
	TagConnF    func(ctx context.Context, info *stats.ConnTagInfo) context.Context
	HandleConnF func(ctx context.Context, info stats.ConnStats)
}

// TagRPC calls the StubStatsHandler's TagRPCF, if set.
func (ssh *StubStatsHandler) TagRPC(ctx context.Context, info *stats.RPCTagInfo) context.Context {
	if ssh.TagRPCF != nil {
		return ssh.TagRPCF(ctx, info)
	}
	return ctx
}

// HandleRPC calls the StubStatsHandler's HandleRPCF, if set.
func (ssh *StubStatsHandler) HandleRPC(ctx context.Context, rs stats.RPCStats) {
	if ssh.HandleRPCF != nil {
		ssh.HandleRPCF(ctx, rs)
	}
}

// TagConn calls the StubStatsHandler's TagConnF, if set.
func (ssh *StubStatsHandler) TagConn(ctx context.Context, info *stats.ConnTagInfo) context.Context {
	if ssh.TagConnF != nil {
		return ssh.TagConnF(ctx, info)
	}
	return ctx
}

// HandleConn calls the StubStatsHandler's HandleConnF, if set.
func (ssh *StubStatsHandler) HandleConn(ctx context.Context, cs stats.ConnStats) {
	if ssh.HandleConnF != nil {
		ssh.HandleConnF(ctx, cs)
	}
}
