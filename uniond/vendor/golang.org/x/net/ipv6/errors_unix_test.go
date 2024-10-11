// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build aix || darwin || dragonfly || freebsd || linux || netbsd || openbsd || solaris

package ipv6_test

import (
	"errors"

	"golang.org/x/sys/unix"
)

// isENOBUFS reports whether err is unix.ENOBUFS.
// (Always false on non-Unix platforms.)
func isENOBUFS(err error) bool {
	return errors.Is(err, unix.ENOBUFS)
}
