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

// Package experimental is a collection of experimental features that might
// have some rough edges to them. Housing experimental features in this package
// results in a user accessing these APIs as `experimental.Foo`, thereby making
// it explicit that the feature is experimental and using them in production
// code is at their own risk.
//
// All APIs in this package are experimental.
package experimental

import (
	"google.golang.org/grpc"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/mem"
)

// WithBufferPool returns a grpc.DialOption that configures the use of bufferPool
// for parsing incoming messages on a grpc.ClientConn, and for temporary buffers
// when marshaling outgoing messages. By default, mem.DefaultBufferPool is used,
// and this option only exists to provide alternative buffer pool implementations
// to the client, such as more optimized size allocations etc. However, the
// default buffer pool is already tuned to account for many different use-cases.
//
// Note: The following options will interfere with the buffer pool because they
// require a fully materialized buffer instead of a sequence of buffers:
// EnableTracing, and binary logging. In such cases, materializing the buffer
// will generate a lot of garbage, reducing the overall benefit from using a
// pool.
func WithBufferPool(bufferPool mem.BufferPool) grpc.DialOption {
	return internal.WithBufferPool.(func(mem.BufferPool) grpc.DialOption)(bufferPool)
}

// BufferPool returns a grpc.ServerOption that configures the server to use the
// provided buffer pool for parsing incoming messages and for temporary buffers
// when marshaling outgoing messages. By default, mem.DefaultBufferPool is used,
// and this option only exists to provide alternative buffer pool implementations
// to the server, such as more optimized size allocations etc. However, the
// default buffer pool is already tuned to account for many different use-cases.
//
// Note: The following options will interfere with the buffer pool because they
// require a fully materialized buffer instead of a sequence of buffers:
// EnableTracing, and binary logging. In such cases, materializing the buffer
// will generate a lot of garbage, reducing the overall benefit from using a
// pool.
func BufferPool(bufferPool mem.BufferPool) grpc.ServerOption {
	return internal.BufferPool.(func(mem.BufferPool) grpc.ServerOption)(bufferPool)
}
