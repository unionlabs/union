// Copyright 2017 The go-ethereum Authors
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

// This file contains the implementation for interacting with the Trezor hardware
// wallets. The wire protocol spec can be found on the SatoshiLabs website:
// https://wiki.trezor.io/Developers_guide-Message_Workflows

// !!! STAHP !!!
//
// Before you touch the protocol files, you need to be aware of a breaking change
// that occurred between firmware versions 1.7.3->1.8.0 (Model One) and 2.0.10->
// 2.1.0 (Model T). The Ethereum address representation was changed from the 20
// byte binary blob to a 42 byte hex string. The upstream protocol buffer files
// only support the new format, so blindly pulling in a new spec will break old
// devices!
//
// The Trezor devs had the foresight to add the string version as a new message
// code instead of replacing the binary one. This means that the proto file can
// actually define both the old and the new versions as optional. Please ensure
// that you add back the old addresses everywhere (to avoid name clash. use the
// addressBin and addressHex names).
//
// If in doubt, reach out to @karalabe.

// To regenerate the protocol files in this package:
//   - Download the latest protoc https://github.com/protocolbuffers/protobuf/releases
//   - Build with the usual `./configure && make` and ensure it's on your $PATH
//   - Delete all the .proto and .pb.go files, pull in fresh ones from Trezor
//   - Grab the latest Go plugin `go get -u github.com/golang/protobuf/protoc-gen-go`
//   - Vendor in the latest Go plugin `govendor fetch github.com/golang/protobuf/...`

//go:generate protoc -I/usr/local/include:. --go_out=import_path=trezor:. messages.proto messages-common.proto messages-management.proto messages-ethereum.proto

// Package trezor contains the wire protocol.
package trezor

import (
	"reflect"

	"github.com/golang/protobuf/proto"
)

// Type returns the protocol buffer type number of a specific message. If the
// message is nil, this method panics!
func Type(msg proto.Message) uint16 {
	return uint16(MessageType_value["MessageType_"+reflect.TypeOf(msg).Elem().Name()])
}

// Name returns the friendly message type name of a specific protocol buffer
// type number.
func Name(kind uint16) string {
	name := MessageType_name[int32(kind)]
	if len(name) < 12 {
		return name
	}
	return name[12:]
}
