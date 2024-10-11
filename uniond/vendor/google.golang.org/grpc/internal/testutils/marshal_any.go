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

package testutils

import (
	"testing"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
)

// MarshalAny is a convenience function to marshal protobuf messages into any
// protos. function will fail the test with a fatal error if the marshaling fails.
func MarshalAny(t *testing.T, m proto.Message) *anypb.Any {
	t.Helper()

	a, err := anypb.New(m)
	if err != nil {
		t.Fatalf("Failed to marshal proto %+v into an Any: %v", m, err)
	}
	return a
}
