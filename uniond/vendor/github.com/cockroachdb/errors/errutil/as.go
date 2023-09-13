// Copyright 2018 The Go Authors. All rights reserved.
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

package errutil

import (
	"reflect"

	"github.com/cockroachdb/errors/errbase"
)

// As finds the first error in err's chain that matches the type to which target
// points, and if so, sets the target to its value and returns true. An error
// matches a type if it is assignable to the target type, or if it has a method
// As(interface{}) bool such that As(target) returns true. As will panic if target
// is not a non-nil pointer to a type which implements error or is of interface type.
//
// The As method should set the target to its value and return true if err
// matches the type to which target points.
//
// Note: this implementation differs from that of xerrors as follows:
// - it also supports recursing through causes with Cause().
// - if it detects an API use error, its panic object is a valid error.
func As(err error, target interface{}) bool {
	if target == nil {
		panic(AssertionFailedf("errors.As: target cannot be nil"))
	}

	// We use introspection for now, of course when/if Go gets generics
	// all this can go away.
	val := reflect.ValueOf(target)
	typ := val.Type()
	if typ.Kind() != reflect.Ptr || val.IsNil() {
		panic(AssertionFailedf("errors.As: target must be a non-nil pointer, found %T", target))
	}
	if e := typ.Elem(); e.Kind() != reflect.Interface && !e.Implements(errorType) {
		panic(AssertionFailedf("errors.As: *target must be interface or implement error, found %T", target))
	}

	targetType := typ.Elem()
	for c := err; c != nil; c = errbase.UnwrapOnce(c) {
		if reflect.TypeOf(c).AssignableTo(targetType) {
			val.Elem().Set(reflect.ValueOf(c))
			return true
		}
		if x, ok := c.(interface{ As(interface{}) bool }); ok && x.As(target) {
			return true
		}
	}
	return false
}

var errorType = reflect.TypeOf((*error)(nil)).Elem()
