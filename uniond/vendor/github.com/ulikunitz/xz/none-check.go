// Copyright 2014-2022 Ulrich Kunitz. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package xz

import "hash"

type noneHash struct{}

func (h noneHash) Write(p []byte) (n int, err error) { return len(p), nil }

func (h noneHash) Sum(b []byte) []byte { return b }

func (h noneHash) Reset() {}

func (h noneHash) Size() int { return 0 }

func (h noneHash) BlockSize() int { return 0 }

func newNoneHash() hash.Hash {
	return &noneHash{}
}
