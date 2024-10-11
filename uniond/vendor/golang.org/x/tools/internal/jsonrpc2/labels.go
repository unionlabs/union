// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package jsonrpc2

import "golang.org/x/tools/internal/event/keys"

// These keys are used for creating labels to instrument jsonrpc2 events.
var (
	Method        = keys.NewString("method", "")
	RPCID         = keys.NewString("id", "")
	RPCDirection  = keys.NewString("direction", "")
	Started       = keys.NewInt64("started", "Count of started RPCs.")
	SentBytes     = keys.NewInt64("sent_bytes", "Bytes sent.")         //, unit.Bytes)
	ReceivedBytes = keys.NewInt64("received_bytes", "Bytes received.") //, unit.Bytes)
	StatusCode    = keys.NewString("status.code", "")
	Latency       = keys.NewFloat64("latency_ms", "Elapsed time in milliseconds") //, unit.Milliseconds)
)

const (
	Inbound  = "in"
	Outbound = "out"
)
