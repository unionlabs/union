// Copyright 2021 The Cockroach Authors.
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

// +build !go1.16

package fmttests

import "strings"

func fakeGo116(s string) string {
	// In Go 1.16, the canonical type for os.PathError is io/fs/*fs.PathError.
	// So when running the tests with a pre-1.16 runtime, the strings
	// emitted by printing out the error objects don't match the output
	// expected in the tests, which was generated with go 1.16.
	s = strings.ReplaceAll(s, "os/*os.PathError (*::)", "io/fs/*fs.PathError (os/*os.PathError::)")
	s = strings.ReplaceAll(s, " *os.PathError", " *fs.PathError")
	s = strings.ReplaceAll(s, "\n*os.PathError", "\n*fs.PathError")
	s = strings.ReplaceAll(s, "&os.PathError", "&fs.PathError")
	return s
}
