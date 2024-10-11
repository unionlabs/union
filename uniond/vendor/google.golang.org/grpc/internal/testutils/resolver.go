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
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/serviceconfig"
)

// Logger wraps the logging methods from testing.T.
type Logger interface {
	Log(args ...any)
	Logf(format string, args ...any)
	Errorf(format string, args ...any)
}

// ResolverClientConn is a fake implementation of the resolver.ClientConn
// interface to be used in tests.
type ResolverClientConn struct {
	resolver.ClientConn // Embedding the interface to avoid implementing deprecated methods.

	Logger       Logger                     // Tests should pass testing.T for this.
	UpdateStateF func(resolver.State) error // Invoked when resolver pushes a state update.
	ReportErrorF func(err error)            // Invoked when resolver pushes an error.
}

// UpdateState invokes the test specified callback with the update received from
// the resolver. If the callback returns a non-nil error, the same will be
// propagated to the resolver.
func (t *ResolverClientConn) UpdateState(s resolver.State) error {
	t.Logger.Logf("testutils.ResolverClientConn: UpdateState(%s)", pretty.ToJSON(s))

	if t.UpdateStateF != nil {
		return t.UpdateStateF(s)
	}
	return nil
}

// ReportError pushes the error received from the resolver on to ErrorCh.
func (t *ResolverClientConn) ReportError(err error) {
	t.Logger.Logf("testutils.ResolverClientConn: ReportError(%v)", err)

	if t.ReportErrorF != nil {
		t.ReportErrorF(err)
	}
}

// ParseServiceConfig parses the provided service by delegating the work to the
// implementation in the grpc package.
func (t *ResolverClientConn) ParseServiceConfig(jsonSC string) *serviceconfig.ParseResult {
	return internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
}
