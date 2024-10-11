// Protocol Buffers for Go with Gadgets
//
// Copyright (c) 2013, The GoGo Authors. All rights reserved.
// http://github.com/gogo/protobuf
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package protoio_test

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"math/rand"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/cosmos/gogoproto/test"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/libs/protoio"
)

func iotest(writer protoio.WriteCloser, reader protoio.ReadCloser) error {
	varint := make([]byte, binary.MaxVarintLen64)
	size := 1000
	msgs := make([]*test.NinOptNative, size)
	lens := make([]int, size)
	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	for i := range msgs {
		msgs[i] = test.NewPopulatedNinOptNative(r, true)
		// issue 31
		if i == 5 {
			msgs[i] = &test.NinOptNative{}
		}
		// issue 31
		if i == 999 {
			msgs[i] = &test.NinOptNative{}
		}
		// FIXME Check size
		bz, err := proto.Marshal(msgs[i])
		if err != nil {
			return err
		}
		visize := binary.PutUvarint(varint, uint64(len(bz)))
		n, err := writer.WriteMsg(msgs[i])
		if err != nil {
			return err
		}
		if n != len(bz)+visize {
			return fmt.Errorf("WriteMsg() wrote %v bytes, expected %v", n, len(bz)+visize)
		}
		lens[i] = n
	}
	if err := writer.Close(); err != nil {
		return err
	}
	i := 0
	for {
		msg := &test.NinOptNative{}
		if n, err := reader.ReadMsg(msg); err != nil {
			if err == io.EOF {
				break
			}
			return err
		} else if n != lens[i] {
			return fmt.Errorf("read %v bytes, expected %v", n, lens[i])
		}
		if err := msg.VerboseEqual(msgs[i]); err != nil {
			return err
		}
		i++
	}
	if i != size {
		panic("not enough messages read")
	}
	return reader.Close()
}

type buffer struct {
	*bytes.Buffer
	closed bool
}

func (b *buffer) Close() error {
	b.closed = true
	return nil
}

func newBuffer() *buffer {
	return &buffer{bytes.NewBuffer(nil), false}
}

func TestVarintNormal(t *testing.T) {
	buf := newBuffer()
	writer := protoio.NewDelimitedWriter(buf)
	reader := protoio.NewDelimitedReader(buf, 1024*1024)
	err := iotest(writer, reader)
	require.NoError(t, err)
	require.True(t, buf.closed, "did not close buffer")
}

func TestVarintNoClose(t *testing.T) {
	buf := bytes.NewBuffer(nil)
	writer := protoio.NewDelimitedWriter(buf)
	reader := protoio.NewDelimitedReader(buf, 1024*1024)
	err := iotest(writer, reader)
	require.NoError(t, err)
}

// issue 32.
func TestVarintMaxSize(t *testing.T) {
	buf := newBuffer()
	writer := protoio.NewDelimitedWriter(buf)
	reader := protoio.NewDelimitedReader(buf, 20)
	err := iotest(writer, reader)
	require.Error(t, err)
}

func TestVarintError(t *testing.T) {
	buf := newBuffer()
	buf.Write([]byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f})
	reader := protoio.NewDelimitedReader(buf, 1024*1024)
	msg := &test.NinOptNative{}
	n, err := reader.ReadMsg(msg)
	require.Error(t, err)
	require.Equal(t, 10, n)
}

func TestVarintTruncated(t *testing.T) {
	buf := newBuffer()
	buf.Write([]byte{0xff, 0xff})
	reader := protoio.NewDelimitedReader(buf, 1024*1024)
	msg := &test.NinOptNative{}
	n, err := reader.ReadMsg(msg)
	require.Error(t, err)
	require.Equal(t, 2, n)
}

func TestShort(t *testing.T) {
	buf := newBuffer()

	varintBuf := make([]byte, binary.MaxVarintLen64)
	varintLen := binary.PutUvarint(varintBuf, 100)
	_, err := buf.Write(varintBuf[:varintLen])
	require.NoError(t, err)

	bz, err := proto.Marshal(&test.NinOptNative{Field15: []byte{0x01, 0x02, 0x03}})
	require.NoError(t, err)
	buf.Write(bz)

	reader := protoio.NewDelimitedReader(buf, 1024*1024)
	require.NoError(t, err)
	msg := &test.NinOptNative{}
	n, err := reader.ReadMsg(msg)
	require.Error(t, err)
	require.Equal(t, varintLen+len(bz), n)
}
