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

package withstack

import (
	"fmt"
	"runtime"

	"github.com/cockroachdb/errors/errbase"
)

// stack represents a stack of program counters.
// This mirrors the (non-exported) type of the same name in github.com/pkg/errors.
type stack []uintptr

// Format mirrors the code in github.com/pkg/errors.
func (s *stack) Format(st fmt.State, verb rune) {
	switch verb {
	case 'v':
		switch {
		case st.Flag('+'):
			for _, pc := range *s {
				f := errbase.StackFrame(pc)
				fmt.Fprintf(st, "\n%+v", f)
			}
		}
	}
}

// StackTrace mirrors the code in github.com/pkg/errors.
func (s *stack) StackTrace() errbase.StackTrace {
	f := make([]errbase.StackFrame, len(*s))
	for i := 0; i < len(f); i++ {
		f[i] = errbase.StackFrame((*s)[i])
	}
	return f
}

// callers mirrors the code in github.com/pkg/errors,
// but makes the depth customizable.
func callers(depth int) *stack {
	const numFrames = 32
	var pcs [numFrames]uintptr
	n := runtime.Callers(2+depth, pcs[:])
	var st stack = pcs[0:n]
	return &st
}
