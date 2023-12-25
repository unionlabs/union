// Protocol Buffers - Google's data interchange format
// Copyright 2008 Google Inc.  All rights reserved.
// https://developers.google.com/protocol-buffers/
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
//     * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
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

package anyutil

import (
	"fmt"
	"strings"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	protoimpl "google.golang.org/protobuf/runtime/protoimpl"
	"google.golang.org/protobuf/types/dynamicpb"
	"google.golang.org/protobuf/types/known/anypb"
)

// New marshals src into a new Any instance.
func New(src proto.Message) (*anypb.Any, error) {
	dst := new(anypb.Any)
	if err := MarshalFrom(dst, src, proto.MarshalOptions{}); err != nil {
		return nil, err
	}
	return dst, nil
}

// MarshalFrom marshals src into dst as the underlying message
// using the provided marshal options.
//
// If no options are specified, call dst.MarshalFrom instead.
func MarshalFrom(dst *anypb.Any, src proto.Message, opts proto.MarshalOptions) error {
	if src == nil {
		return protoimpl.X.NewError("invalid nil source message")
	}
	b, err := opts.Marshal(src)
	if err != nil {
		return err
	}
	dst.TypeUrl = "/" + string(src.ProtoReflect().Descriptor().FullName())
	dst.Value = b
	return nil
}

// Unpack unpacks the message inside an any, first using the provided
// typeResolver (defaults to protoregistry.GlobalTypes), and if that fails,
// then using the provided fileResolver (defaults to protoregistry.GlobalFiles)
// with dynamicpb.
func Unpack(any *anypb.Any, fileResolver protodesc.Resolver, typeResolver protoregistry.MessageTypeResolver) (proto.Message, error) {
	if typeResolver == nil {
		typeResolver = protoregistry.GlobalTypes
	}

	url := any.TypeUrl
	typ, err := typeResolver.FindMessageByURL(url)
	if err == protoregistry.NotFound {
		if fileResolver == nil {
			fileResolver = protoregistry.GlobalFiles
		}

		// If the proto v2 registry doesn't have this message, then we use
		// protoFiles (which can e.g. be initialized to gogo's MergedRegistry)
		// to retrieve the message descriptor, and then use dynamicpb on that
		// message descriptor to create a proto.Message
		typeURL := strings.TrimPrefix(any.TypeUrl, "/")

		msgDesc, err := fileResolver.FindDescriptorByName(protoreflect.FullName(typeURL))
		if err != nil {
			return nil, fmt.Errorf("protoFiles does not have descriptor %s: %w", any.TypeUrl, err)
		}

		typ = dynamicpb.NewMessageType(msgDesc.(protoreflect.MessageDescriptor))

	} else if err != nil {
		return nil, err
	}

	packedMsg := typ.New().Interface()
	err = any.UnmarshalTo(packedMsg)
	if err != nil {
		return nil, fmt.Errorf("cannot unmarshal msg %s: %w", any.TypeUrl, err)
	}

	return packedMsg, nil
}
