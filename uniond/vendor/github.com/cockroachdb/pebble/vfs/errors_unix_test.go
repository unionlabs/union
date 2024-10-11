// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build darwin || dragonfly || freebsd || linux || openbsd || netbsd
// +build darwin dragonfly freebsd linux openbsd netbsd

package vfs

import (
	"testing"

	"github.com/cockroachdb/errors"
	"github.com/stretchr/testify/require"
	"golang.org/x/sys/unix"
)

func TestIsNoSpaceError(t *testing.T) {
	err := errors.WithStack(unix.ENOSPC)
	require.True(t, IsNoSpaceError(err))
}
