// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package impl_test

import (
	"testing"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/testing/protopack"

	lazytestpb "google.golang.org/protobuf/internal/testprotos/lazy"
)

// Constructs a message encoded in denormalized (non-minimal) wire format, but
// using two levels of nesting: A top-level message with a child message which
// in turn has a grandchild message.
func denormalizedTwoLevel(t *testing.T) ([]byte, *lazytestpb.Top, error) {
	// Construct a message with denormalized (non-minimal) wire format:
	// 1. Encode a top-level message with submessage B (ext) + C (field)
	// 2. Replace the encoding of submessage C (field) with
	//    another instance of submessage B (ext)
	//
	// This modification of the wire format is spec'd in Protobuf:
	// https://github.com/protocolbuffers/protobuf/issues/9257
	grandchild := &lazytestpb.Sub{}
	proto.SetExtension(grandchild, lazytestpb.E_Ext_B, &lazytestpb.Ext{
		SomeFlag: proto.Bool(true),
	})
	expectedMessage := &lazytestpb.Top{
		Child: &lazytestpb.Sub{
			Grandchild: grandchild,
		},
		A: proto.Uint32(2342),
	}

	fullMessage := protopack.Message{
		protopack.Tag{1, protopack.VarintType}, protopack.Varint(2342),
		// Child
		protopack.Tag{2, protopack.BytesType}, protopack.LengthPrefix(protopack.Message{
			// Grandchild
			protopack.Tag{4, protopack.BytesType}, protopack.LengthPrefix(protopack.Message{
				// The first occurrence of B matches expectedMessage:
				protopack.Tag{2, protopack.BytesType}, protopack.LengthPrefix(protopack.Message{
					protopack.Tag{1, protopack.VarintType}, protopack.Varint(1),
				}),
				// This second duplicative occurrence of B is spec'd in Protobuf:
				// https://github.com/protocolbuffers/protobuf/issues/9257
				protopack.Tag{2, protopack.BytesType}, protopack.LengthPrefix(protopack.Message{
					protopack.Tag{1, protopack.VarintType}, protopack.Varint(1),
				}),
			}),
		}),
	}.Marshal()

	return fullMessage, expectedMessage, nil
}

func TestNoInvalidWireFormatWithDeterministicLazy(t *testing.T) {
	fullMessage, _, err := denormalizedTwoLevel(t)
	if err != nil {
		t.Fatal(err)
	}

	top := &lazytestpb.Top{}
	if err := proto.Unmarshal(fullMessage, top); err != nil {
		t.Fatal(err)
	}

	// Requesting deterministic marshaling should result in unmarshaling (and
	// thereby normalizing the non-minimal encoding) when sizing.
	//
	// If the deterministic flag is dropped (like before cl/624951104), the size
	// cache is populated with the non-minimal size. The Marshal call below
	// lazily unmarshals (due to the Deterministic flag), which includes
	// normalization, and will then report a size mismatch error (instead of
	// producing invalid wire format).
	proto.MarshalOptions{Deterministic: true}.Size(top)

	_, err = proto.MarshalOptions{
		Deterministic: true,
		UseCachedSize: true,
	}.Marshal(top)
	if err != nil {
		t.Fatal(err)
	}
}
