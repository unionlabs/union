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
//
// Modified to return number of bytes written by Writer.WriteMsg(), and added byteReader.

package protoio

import (
	"io"

	"github.com/cosmos/gogoproto/proto"
)

type Writer interface {
	WriteMsg(proto.Message) (int, error)
}

type WriteCloser interface {
	Writer
	io.Closer
}

type Reader interface {
	ReadMsg(msg proto.Message) (int, error)
}

type ReadCloser interface {
	Reader
	io.Closer
}

type marshaler interface {
	MarshalTo(data []byte) (n int, err error)
}

func getSize(v interface{}) (int, bool) {
	if sz, ok := v.(interface {
		Size() (n int)
	}); ok {
		return sz.Size(), true
	} else if sz, ok := v.(interface {
		ProtoSize() (n int)
	}); ok {
		return sz.ProtoSize(), true
	}
	return 0, false
}

// byteReader wraps an io.Reader and implements io.ByteReader, required by
// binary.ReadUvarint(). Reading one byte at a time is extremely slow, but this
// is what Amino did previously anyway, and the caller can wrap the underlying
// reader in a bufio.Reader if appropriate.
type byteReader struct {
	reader    io.Reader
	buf       []byte
	bytesRead int // keeps track of bytes read via ReadByte()
}

func newByteReader(r io.Reader) *byteReader {
	return &byteReader{
		reader: r,
		buf:    make([]byte, 1),
	}
}

func (r *byteReader) ReadByte() (byte, error) {
	n, err := r.reader.Read(r.buf)
	r.bytesRead += n
	if err != nil {
		return 0x00, err
	}
	return r.buf[0], nil
}
