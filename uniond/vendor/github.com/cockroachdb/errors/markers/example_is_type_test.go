// Copyright 2019 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package markers_test

import (
	"fmt"
	"net"

	"github.com/cockroachdb/errors/markers"
	"github.com/pkg/errors"
)

type ExampleError struct{ msg string }

func (e *ExampleError) Error() string { return e.msg }

func ExampleIsType() {
	base := &ExampleError{"world"}
	err := errors.Wrap(base, "hello")
	fmt.Println(markers.HasType(err, (*ExampleError)(nil)))
	fmt.Println(markers.HasType(err, nil))
	fmt.Println(markers.HasType(err, (*net.AddrError)(nil)))

	// Output:
	//
	// true
	// false
	// false
}

func ExampleIsInterface() {
	base := &net.AddrError{
		Addr: "ndn",
		Err:  "ndn doesn't really exists :(",
	}
	err := errors.Wrap(base, "bummer")
	fmt.Println(markers.HasInterface(err, (*net.Error)(nil)))
	func() {
		defer func() {
			if r := recover(); r != nil {
				fmt.Println("*net.AddrError is not a pointer to an interface type so the call panics")
			}
		}()
		fmt.Println(markers.HasInterface(err, (*net.AddrError)(nil)))
	}()

	// Output:
	//
	// true
	// *net.AddrError is not a pointer to an interface type so the call panics
}
