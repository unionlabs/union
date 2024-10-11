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

// Package weightedroundrobin provides an implementation of the weighted round
// robin LB policy, as defined in [gRFC A58].
//
// # Experimental
//
// Notice: This package is EXPERIMENTAL and may be changed or removed in a
// later release.
//
// [gRFC A58]: https://github.com/grpc/proposal/blob/master/A58-client-side-weighted-round-robin-lb-policy.md
package weightedroundrobin

import (
	"fmt"

	"google.golang.org/grpc/resolver"
)

// attributeKey is the type used as the key to store AddrInfo in the
// BalancerAttributes field of resolver.Address.
type attributeKey struct{}

// AddrInfo will be stored in the BalancerAttributes field of Address in order
// to use weighted roundrobin balancer.
type AddrInfo struct {
	Weight uint32
}

// Equal allows the values to be compared by Attributes.Equal.
func (a AddrInfo) Equal(o any) bool {
	oa, ok := o.(AddrInfo)
	return ok && oa.Weight == a.Weight
}

// SetAddrInfo returns a copy of addr in which the BalancerAttributes field is
// updated with addrInfo.
func SetAddrInfo(addr resolver.Address, addrInfo AddrInfo) resolver.Address {
	addr.BalancerAttributes = addr.BalancerAttributes.WithValue(attributeKey{}, addrInfo)
	return addr
}

// GetAddrInfo returns the AddrInfo stored in the BalancerAttributes field of
// addr.
func GetAddrInfo(addr resolver.Address) AddrInfo {
	v := addr.BalancerAttributes.Value(attributeKey{})
	ai, _ := v.(AddrInfo)
	return ai
}

func (a AddrInfo) String() string {
	return fmt.Sprintf("Weight: %d", a.Weight)
}
