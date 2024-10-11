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

package metadata

import (
	"errors"
	"reflect"
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/attributes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/resolver"
)

func TestGet(t *testing.T) {
	tests := []struct {
		name string
		addr resolver.Address
		want metadata.MD
	}{
		{
			name: "not set",
			addr: resolver.Address{},
			want: nil,
		},
		{
			name: "not set",
			addr: resolver.Address{
				Attributes: attributes.New(mdKey, mdValue(metadata.Pairs("k", "v"))),
			},
			want: metadata.Pairs("k", "v"),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := Get(tt.addr); !cmp.Equal(got, tt.want) {
				t.Errorf("Get() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestSet(t *testing.T) {
	tests := []struct {
		name string
		addr resolver.Address
		md   metadata.MD
	}{
		{
			name: "unset before",
			addr: resolver.Address{},
			md:   metadata.Pairs("k", "v"),
		},
		{
			name: "set before",
			addr: resolver.Address{
				Attributes: attributes.New(mdKey, mdValue(metadata.Pairs("bef", "ore"))),
			},
			md: metadata.Pairs("k", "v"),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			newAddr := Set(tt.addr, tt.md)
			newMD := Get(newAddr)
			if !cmp.Equal(newMD, tt.md) {
				t.Errorf("md after Set() = %v, want %v", newMD, tt.md)
			}
		})
	}
}

func TestValidate(t *testing.T) {
	for _, test := range []struct {
		md   metadata.MD
		want error
	}{
		{
			md:   map[string][]string{string(rune(0x19)): {"testVal"}},
			want: errors.New("header key \"\\x19\" contains illegal characters not in [0-9a-z-_.]"),
		},
		{
			md:   map[string][]string{"test": {string(rune(0x19))}},
			want: errors.New("header key \"test\" contains value with non-printable ASCII characters"),
		},
		{
			md:   map[string][]string{"": {"valid"}},
			want: errors.New("there is an empty key in the header"),
		},
		{
			md:   map[string][]string{"test-bin": {string(rune(0x19))}},
			want: nil,
		},
	} {
		err := Validate(test.md)
		if !reflect.DeepEqual(err, test.want) {
			t.Errorf("validating metadata which is %v got err :%v, want err :%v", test.md, err, test.want)
		}
	}
}
