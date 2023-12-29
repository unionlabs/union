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

package fmtforward

import (
	"fmt"
	"io"
	"strconv"
	"strings"
)

// ReproducePrintf calls fmt.Print/fmt.Printf on the argument
// based on the details present in the fmt.State and
// writes to the io.Writer.
func ReproducePrintf(w io.Writer, s fmt.State, verb rune, arg interface{}) {
	justV, revFmt := MakeFormat(s, verb)
	if justV {
		// Common case, avoids generating then parsing the format again.
		fmt.Fprint(w, arg)
	} else {
		fmt.Fprintf(w, revFmt, arg)
	}
}

// MakeFormat reproduces the format currently active
// in fmt.State and verb. This is provided because Go's standard
// fmt.State does not make the original format string available to us.
//
// If the return value justV is true, then the current state
// was found to be %v exactly; in that case the caller
// can avoid a full-blown Printf call and use just Print instead
// to take a shortcut.
func MakeFormat(s fmt.State, verb rune) (justV bool, format string) {
	plus, minus, hash, sp, z := s.Flag('+'), s.Flag('-'), s.Flag('#'), s.Flag(' '), s.Flag('0')
	w, wp := s.Width()
	p, pp := s.Precision()

	if !plus && !minus && !hash && !sp && !z && !wp && !pp {
		switch verb {
		case 'v':
			return true, "%v"
		case 's':
			return false, "%s"
		case 'd':
			return false, "%d"
		}
		// Other cases handled in the slow path below.
	}

	var f strings.Builder
	f.WriteByte('%')
	if plus {
		f.WriteByte('+')
	}
	if minus {
		f.WriteByte('-')
	}
	if hash {
		f.WriteByte('#')
	}
	if sp {
		f.WriteByte(' ')
	}
	if z {
		f.WriteByte('0')
	}
	if wp {
		f.WriteString(strconv.Itoa(w))
	}
	if pp {
		f.WriteByte('.')
		f.WriteString(strconv.Itoa(p))
	}
	f.WriteRune(verb)
	return false, f.String()
}
