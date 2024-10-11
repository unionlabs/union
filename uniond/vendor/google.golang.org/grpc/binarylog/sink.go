/*
 *
 * Copyright 2020 gRPC authors.
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

// Package binarylog implementation binary logging as defined in
// https://github.com/grpc/proposal/blob/master/A16-binary-logging.md.
//
// Notice: All APIs in this package are experimental.
package binarylog

import (
	"fmt"
	"os"

	binlogpb "google.golang.org/grpc/binarylog/grpc_binarylog_v1"
	iblog "google.golang.org/grpc/internal/binarylog"
)

// SetSink sets the destination for the binary log entries.
//
// NOTE: this function must only be called during initialization time (i.e. in
// an init() function), and is not thread-safe.
func SetSink(s Sink) {
	if iblog.DefaultSink != nil {
		iblog.DefaultSink.Close()
	}
	iblog.DefaultSink = s
}

// Sink represents the destination for the binary log entries.
type Sink interface {
	// Write marshals the log entry and writes it to the destination. The format
	// is not specified, but should have sufficient information to rebuild the
	// entry. Some options are: proto bytes, or proto json.
	//
	// Note this function needs to be thread-safe.
	Write(*binlogpb.GrpcLogEntry) error
	// Close closes this sink and cleans up resources (e.g. the flushing
	// goroutine).
	Close() error
}

// NewTempFileSink creates a temp file and returns a Sink that writes to this
// file.
func NewTempFileSink() (Sink, error) {
	// Two other options to replace this function:
	// 1. take filename as input.
	// 2. export NewBufferedSink().
	tempFile, err := os.CreateTemp("/tmp", "grpcgo_binarylog_*.txt")
	if err != nil {
		return nil, fmt.Errorf("failed to create temp file: %v", err)
	}
	return iblog.NewBufferedSink(tempFile), nil
}
