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

package redact

import (
	origFmt "fmt"

	i "github.com/cockroachdb/redact/interfaces"
	"github.com/cockroachdb/redact/internal/fmtforward"
)

// Unsafe turns any value that would otherwise be considered safe,
// into an unsafe value.
func Unsafe(a interface{}) interface{} {
	return unsafeWrap{a}
}

// UnsafeWrap is the type of wrapper produced by Unsafe.
// This is exported only for use by the rfmt package.
// Client packages should not make assumptions about
// the concrete return type of Unsafe().
type UnsafeWrap = unsafeWrap

type unsafeWrap struct {
	a interface{}
}

func (w unsafeWrap) GetValue() interface{} { return w.a }

func (w unsafeWrap) Format(s origFmt.State, verb rune) {
	fmtforward.ReproducePrintf(s, s, verb, w.a)
}

// Safe turns any value into an object that is considered as safe by
// the formatter.
//
// This is provided as an “escape hatch” for cases where the other
// interfaces and conventions fail. Increased usage of this mechanism
// should be taken as a signal that a new abstraction is missing.
// The implementation is also slow.
func Safe(a interface{}) i.SafeValue {
	return safeWrapper{a}
}

// SafeWrapper is the type of wrapper produced by Safe.
// This is exported only for use by the rfmt package.
// Client packages should not make assumptions about
// the concrete return type of Safe().
type SafeWrapper = safeWrapper

type safeWrapper struct {
	a interface{}
}

var _ i.SafeValue = safeWrapper{}
var _ origFmt.Formatter = safeWrapper{}
var _ i.SafeMessager = safeWrapper{}

func (w safeWrapper) GetValue() interface{} { return w.a }

// SafeValue implements the SafeValue interface.
func (w safeWrapper) SafeValue() {}

// Format implements the fmt.Formatter interface.
func (w safeWrapper) Format(s origFmt.State, verb rune) {
	fmtforward.ReproducePrintf(s, s, verb, w.a)
}

// SafeMessage implements SafeMessager.
func (w safeWrapper) SafeMessage() string {
	return origFmt.Sprintf("%v", w.a)
}
