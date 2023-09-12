// Copyright 2019 The Cockroach Authors.
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

// +build !plan9

package errbase

import (
	"context"
	"os"
	"runtime"
	"syscall"

	"github.com/cockroachdb/errors/errorspb"
	"github.com/gogo/protobuf/proto"
)

const thisArch = runtime.GOOS + ":" + runtime.GOARCH

func encodeErrno(_ context.Context, err error) (msg string, safe []string, payload proto.Message) {
	e := err.(syscall.Errno)
	payload = &errorspb.ErrnoPayload{
		OrigErrno:    int64(e),
		Arch:         thisArch,
		IsPermission: e.Is(os.ErrPermission),
		IsExist:      e.Is(os.ErrExist),
		IsNotExist:   e.Is(os.ErrNotExist),
		IsTimeout:    e.Timeout(),
		IsTemporary:  e.Temporary(),
	}
	return e.Error(), []string{e.Error()}, payload
}

func decodeErrno(_ context.Context, msg string, _ []string, payload proto.Message) error {
	m, ok := payload.(*errorspb.ErrnoPayload)
	if !ok {
		// If this ever happens, this means some version of the library
		// (presumably future) changed the payload type, and we're
		// receiving this here. In this case, give up and let
		// DecodeError use the opaque type.
		return nil
	}
	if m.Arch != thisArch {
		// The errno object is coming from a different platform. We'll
		// keep it opaque here.
		return &OpaqueErrno{msg: msg, details: m}
	}
	return syscall.Errno(m.OrigErrno)
}

func init() {
	pKey := GetTypeKey(syscall.Errno(0))
	RegisterLeafEncoder(pKey, encodeErrno)
	RegisterLeafDecoder(pKey, decodeErrno)
}
