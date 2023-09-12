// Copyright 2020 The Cockroach Authors.
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

package rfmt

import (
	"reflect"

	i "github.com/cockroachdb/redact/interfaces"
)

// RegisterSafeType registers a data type to always be considered safe
// during the production of redactable strings.
func RegisterSafeType(t reflect.Type) {
	safeTypeRegistry[t] = true
}

// safeTypeRegistry registers Go data types that are to be always
// considered safe, even when they don't implement SafeValue.
var safeTypeRegistry = map[reflect.Type]bool{}

func isSafeValue(a interface{}) bool {
	return safeTypeRegistry[reflect.TypeOf(a)]
}

// redactErrorFn can be injected from an error library
// to render error objects safely.
var redactErrorFn func(err error, p i.SafePrinter, verb rune)

// RegisterRedactErrorFn registers an error redaction function for use
// during automatic redaction by this package.
// Provided e.g. by cockroachdb/errors.
func RegisterRedactErrorFn(fn func(err error, p i.SafePrinter, verb rune)) {
	redactErrorFn = fn
}
