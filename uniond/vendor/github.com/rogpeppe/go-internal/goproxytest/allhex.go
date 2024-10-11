// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package goproxytest

// This code was taken from src/cmd/go/internal/modfetch/codehost.

// allHex reports whether the revision rev is entirely lower-case hexadecimal digits.
func allHex(rev string) bool {
	for i := 0; i < len(rev); i++ {
		c := rev[i]
		if '0' <= c && c <= '9' || 'a' <= c && c <= 'f' {
			continue
		}
		return false
	}
	return true
}
