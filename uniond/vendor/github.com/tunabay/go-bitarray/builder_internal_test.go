// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

// A calls the unexported append() method.
func (b *Builder) A(buf []byte, off, nBits int, zf bool) {
	b.append(buf, off, nBits, zf)
}
