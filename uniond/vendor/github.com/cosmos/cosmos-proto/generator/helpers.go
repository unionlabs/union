// Copyright (c) 2021 PlanetScale Inc. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package generator

import (
	"google.golang.org/protobuf/encoding/protowire"
	"google.golang.org/protobuf/reflect/protoreflect"
)

const ProtoPkg = "google.golang.org/protobuf/proto"

func KeySize(fieldNumber protoreflect.FieldNumber, wireType protowire.Type) int {
	x := uint32(fieldNumber)<<3 | uint32(wireType)
	size := 0
	for size = 0; x > 127; size++ {
		x >>= 7
	}
	size++
	return size
}

var wireTypes = map[protoreflect.Kind]protowire.Type{
	protoreflect.BoolKind:     protowire.VarintType,
	protoreflect.EnumKind:     protowire.VarintType,
	protoreflect.Int32Kind:    protowire.VarintType,
	protoreflect.Sint32Kind:   protowire.VarintType,
	protoreflect.Uint32Kind:   protowire.VarintType,
	protoreflect.Int64Kind:    protowire.VarintType,
	protoreflect.Sint64Kind:   protowire.VarintType,
	protoreflect.Uint64Kind:   protowire.VarintType,
	protoreflect.Sfixed32Kind: protowire.Fixed32Type,
	protoreflect.Fixed32Kind:  protowire.Fixed32Type,
	protoreflect.FloatKind:    protowire.Fixed32Type,
	protoreflect.Sfixed64Kind: protowire.Fixed64Type,
	protoreflect.Fixed64Kind:  protowire.Fixed64Type,
	protoreflect.DoubleKind:   protowire.Fixed64Type,
	protoreflect.StringKind:   protowire.BytesType,
	protoreflect.BytesKind:    protowire.BytesType,
	protoreflect.MessageKind:  protowire.BytesType,
	protoreflect.GroupKind:    protowire.StartGroupType,
}

func ProtoWireType(k protoreflect.Kind) protowire.Type {
	return wireTypes[k]
}
