/*
 *
 * Copyright 2018 gRPC authors.
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

// Package dns implements a dns resolver to be installed as the default resolver
// in grpc.
//
// Deprecated: this package is imported by grpc and should not need to be
// imported directly by users.
package dns

import (
	"time"

	"google.golang.org/grpc/internal/resolver/dns"
	"google.golang.org/grpc/resolver"
)

// SetResolvingTimeout sets the maximum duration for DNS resolution requests.
//
// This function affects the global timeout used by all channels using the DNS
// name resolver scheme.
//
// It must be called only at application startup, before any gRPC calls are
// made. Modifying this value after initialization is not thread-safe.
//
// The default value is 30 seconds. Setting the timeout too low may result in
// premature timeouts during resolution, while setting it too high may lead to
// unnecessary delays in service discovery. Choose a value appropriate for your
// specific needs and network environment.
func SetResolvingTimeout(timeout time.Duration) {
	dns.ResolvingTimeout = timeout
}

// NewBuilder creates a dnsBuilder which is used to factory DNS resolvers.
//
// Deprecated: import grpc and use resolver.Get("dns") instead.
func NewBuilder() resolver.Builder {
	return dns.NewBuilder()
}
