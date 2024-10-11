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
 */

// Package internal contains functions/structs shared by xds
// balancers/resolvers.
package internal

import (
	"encoding/json"
	"fmt"

	"google.golang.org/grpc/resolver"
)

// LocalityID is xds.Locality without XXX fields, so it can be used as map
// keys.
//
// xds.Locality cannot be map keys because one of the XXX fields is a slice.
type LocalityID struct {
	Region  string `json:"region,omitempty"`
	Zone    string `json:"zone,omitempty"`
	SubZone string `json:"subZone,omitempty"`
}

// ToString generates a string representation of LocalityID by marshalling it into
// json. Not calling it String() so printf won't call it.
func (l LocalityID) ToString() (string, error) {
	b, err := json.Marshal(l)
	if err != nil {
		return "", err
	}
	return string(b), nil
}

// Equal allows the values to be compared by Attributes.Equal.
func (l LocalityID) Equal(o any) bool {
	ol, ok := o.(LocalityID)
	if !ok {
		return false
	}
	return l.Region == ol.Region && l.Zone == ol.Zone && l.SubZone == ol.SubZone
}

// Empty returns whether or not the locality ID is empty.
func (l LocalityID) Empty() bool {
	return l.Region == "" && l.Zone == "" && l.SubZone == ""
}

// LocalityIDFromString converts a json representation of locality, into a
// LocalityID struct.
func LocalityIDFromString(s string) (ret LocalityID, _ error) {
	err := json.Unmarshal([]byte(s), &ret)
	if err != nil {
		return LocalityID{}, fmt.Errorf("%s is not a well formatted locality ID, error: %v", s, err)
	}
	return ret, nil
}

type localityKeyType string

const localityKey = localityKeyType("grpc.xds.internal.address.locality")

// GetLocalityID returns the locality ID of addr.
func GetLocalityID(addr resolver.Address) LocalityID {
	path, _ := addr.BalancerAttributes.Value(localityKey).(LocalityID)
	return path
}

// SetLocalityID sets locality ID in addr to l.
func SetLocalityID(addr resolver.Address, l LocalityID) resolver.Address {
	addr.BalancerAttributes = addr.BalancerAttributes.WithValue(localityKey, l)
	return addr
}

// ResourceTypeMapForTesting maps TypeUrl to corresponding ResourceType.
var ResourceTypeMapForTesting map[string]any

// UnknownCSMLabels are TelemetryLabels emitted from CDS if CSM Telemetry Label
// data is not present in the CDS Resource.
var UnknownCSMLabels = map[string]string{
	"csm.service_name":           "unknown",
	"csm.service_namespace_name": "unknown",
}
