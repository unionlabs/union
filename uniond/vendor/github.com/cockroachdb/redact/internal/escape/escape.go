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

package escape

import (
	"bytes"
	"unicode/utf8"

	m "github.com/cockroachdb/redact/internal/markers"
)

// InternalEscapeBytes escapes redaction markers in the provided buf
// starting at the location startLoc.
// The bytes before startLoc are considered safe (already escaped).
//
// If breakNewLines is set, a closing redaction marker
// is placed before sequences of one or more newline characters,
// and an open redaction marker is placed afterwards.
//
// If strip is set, final newlines and spaces are trimmed from the
// output.
func InternalEscapeBytes(b []byte, startLoc int, breakNewLines, strip bool) (res []byte) {
	// Note: we use len(...RedactableS) and not len(...RedactableBytes)
	// because the ...S variant is a compile-time constant so this
	// accelerates the loops below.
	start, ls := m.StartBytes, len(m.StartS)
	end, le := m.EndBytes, len(m.EndS)
	escape := m.EscapeMarkBytes

	// Trim final newlines/spaces, for convenience.
	if strip {
		end := len(b)
		for i := end - 1; i >= startLoc; i-- {
			if b[i] == '\n' || b[i] == ' ' {
				end = i
			} else {
				break
			}
		}
		b = b[:end]
	}

	// res is the output slice. In the common case where there is
	// nothing to escape, the input slice is returned directly
	// and no allocation takes place.
	res = b
	// copied is true if and only if `res` is a copy of `b`.  It only
	// turns to true if the loop below finds something to escape.
	copied := false
	// k is the index in b up to (and excluding) the byte which we've
	// already copied into res (if copied=true).
	k := 0

	for i := startLoc; i < len(b); i++ {
		if breakNewLines && b[i] == '\n' {
			if !copied {
				// We only allocate an output slice when we know we definitely
				// need it.
				res = make([]byte, 0, len(b))
				copied = true
			}
			res = append(res, b[k:i]...)
			// Either add an end marker, or elide a start marker immediately prior.
			if bytes.HasSuffix(res, start) {
				res = res[:len(res)-ls]
			} else {
				res = append(res, end...)
			}
			// Advance to the last newline character. We want to forward
			// them all in a single call to doWrite, for performance.
			lastNewLine := i
			for lastNewLine < len(b) && b[lastNewLine] == '\n' {
				lastNewLine++
			}
			res = append(res, b[i:lastNewLine]...)
			res = append(res, start...)
			k = lastNewLine
			i = lastNewLine - 1
		} else
		// Ensure that occurrences of the delimiter inside the string get
		// escaped.
		// Reminder: ls and le are likely greater than 1, as we are scanning
		// utf-8 encoded delimiters (the utf-8 encoding is multibyte).
		if i+ls <= len(b) && bytes.Equal(b[i:i+ls], start) {
			if !copied {
				// We only allocate an output slice when we know we definitely
				// need it.
				res = make([]byte, 0, len(b)+len(escape))
				copied = true
			}
			res = append(res, b[k:i]...)
			res = append(res, escape...)
			// Advance the counters by the length (in bytes) of the delimiter.
			k = i + ls
			i += ls - 1 /* -1 because we have i++ at the end of every iteration */
		} else if i+le <= len(b) && bytes.Equal(b[i:i+le], end) {
			if !copied {
				// See the comment above about res allocation.
				res = make([]byte, 0, len(b)+len(escape))
				copied = true
			}
			res = append(res, b[k:i]...)
			res = append(res, escape...)
			// Advance the counters by the length (in bytes) of the delimiter.
			k = i + le
			i += le - 1 /* -1 because we have i++ at the end of every iteration */
		}
	}
	// If the string terminates with an invalid utf-8 sequence, we
	// want to avoid a run-in with a subsequent redaction marker.
	if r, s := utf8.DecodeLastRune(b); s == 1 && r == utf8.RuneError {
		if !copied {
			// See the comment above about res allocation.
			res = make([]byte, 0, len(b)+len(escape))
			copied = true
		}
		res = append(res, b[k:]...)
		res = append(res, escape...)
		k = len(b)
	}
	if copied {
		res = append(res, b[k:]...)
	}
	return
}
