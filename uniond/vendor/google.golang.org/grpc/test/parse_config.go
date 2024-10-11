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
 *
 */

package test

import (
	"testing"

	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"
)

// parseServiceConfig is a test helper which uses the manual resolver to parse
// the given service config. It calls t.Fatal() if service config parsing fails.
func parseServiceConfig(t *testing.T, r *manual.Resolver, sc string) *serviceconfig.ParseResult {
	t.Helper()

	scpr := r.CC.ParseServiceConfig(sc)
	if scpr.Err != nil {
		t.Fatalf("Failed to parse service config %q: %v", sc, scpr.Err)
	}
	return scpr
}
