// Copyright 2021 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package rlpx

import (
	"bytes"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/stretchr/testify/assert"
)

func TestReadBufferReset(t *testing.T) {
	reader := bytes.NewReader(hexutil.MustDecode("0x010202030303040505"))
	var b readBuffer

	s1, _ := b.read(reader, 1)
	s2, _ := b.read(reader, 2)
	s3, _ := b.read(reader, 3)

	assert.Equal(t, []byte{1}, s1)
	assert.Equal(t, []byte{2, 2}, s2)
	assert.Equal(t, []byte{3, 3, 3}, s3)

	b.reset()

	s4, _ := b.read(reader, 1)
	s5, _ := b.read(reader, 2)

	assert.Equal(t, []byte{4}, s4)
	assert.Equal(t, []byte{5, 5}, s5)

	s6, err := b.read(reader, 2)

	assert.EqualError(t, err, "EOF")
	assert.Nil(t, s6)
}
