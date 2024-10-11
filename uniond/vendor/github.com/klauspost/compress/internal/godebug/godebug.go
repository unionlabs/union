// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package godebug makes the simplified settings in the $GODEBUG environment variable
// available to packages.
// Needed since internal/godebug is not available here.
package godebug

import "os"

func Get(key string) string {
	s := os.Getenv("GODEBUG")
	if s == "" {
		return ""
	}
	// Scan the string backward so that later settings are used
	// and earlier settings are ignored.
	// Note that a forward scan would cause cached values
	// to temporarily use the ignored value before being
	// updated to the "correct" one.
	end := len(s)
	eq := -1
	for i := end - 1; i >= -1; i-- {
		if i == -1 || s[i] == ',' {
			if eq >= 0 {
				name, arg := s[i+1:eq], s[eq+1:end]
				if name == key {
					for j := 0; j < len(arg); j++ {
						if arg[j] == '#' {
							return arg[:j]
						}
					}
					return arg
				}
			}
			eq = -1
			end = i
		} else if s[i] == '=' {
			eq = i
		}
	}
	return ""
}
