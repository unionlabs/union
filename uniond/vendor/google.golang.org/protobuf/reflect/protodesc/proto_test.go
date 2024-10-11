// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package protodesc

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/protobuf/internal/filedesc"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/descriptorpb"
)

func TestEditionsRequired(t *testing.T) {
	fd := new(filedesc.Field)
	fd.L0.ParentFile = filedesc.SurrogateEdition2023
	fd.L0.FullName = "foo_field"
	fd.L1.Number = 1337
	fd.L1.Cardinality = protoreflect.Required
	fd.L1.Kind = protoreflect.BytesKind

	want := &descriptorpb.FieldDescriptorProto{
		Name:   proto.String("foo_field"),
		Number: proto.Int32(1337),
		Label:  descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
		Type:   descriptorpb.FieldDescriptorProto_TYPE_BYTES.Enum(),
	}

	got := ToFieldDescriptorProto(fd)
	if diff := cmp.Diff(want, got, protocmp.Transform()); diff != "" {
		t.Errorf("ToFieldDescriptor: unexpected diff (-want +got):\n%s", diff)
	}
}

func TestProto2Required(t *testing.T) {
	fd := new(filedesc.Field)
	fd.L0.ParentFile = filedesc.SurrogateProto2
	fd.L0.FullName = "foo_field"
	fd.L1.Number = 1337
	fd.L1.Cardinality = protoreflect.Required
	fd.L1.Kind = protoreflect.BytesKind

	want := &descriptorpb.FieldDescriptorProto{
		Name:   proto.String("foo_field"),
		Number: proto.Int32(1337),
		Label:  descriptorpb.FieldDescriptorProto_LABEL_REQUIRED.Enum(),
		Type:   descriptorpb.FieldDescriptorProto_TYPE_BYTES.Enum(),
	}

	got := ToFieldDescriptorProto(fd)
	if diff := cmp.Diff(want, got, protocmp.Transform()); diff != "" {
		t.Errorf("ToFieldDescriptor: unexpected diff (-want +got):\n%s", diff)
	}
}

func TestEditionsDelimited(t *testing.T) {
	md := new(filedesc.Message)
	md.L0.ParentFile = filedesc.SurrogateEdition2023
	md.L0.FullName = "foo_message"
	fd := new(filedesc.Field)
	fd.L0.ParentFile = filedesc.SurrogateEdition2023
	fd.L0.FullName = "foo_field"
	fd.L1.Number = 1337
	fd.L1.Cardinality = protoreflect.Optional
	fd.L1.Kind = protoreflect.GroupKind
	fd.L1.Message = md

	want := &descriptorpb.FieldDescriptorProto{
		Name:     proto.String("foo_field"),
		Number:   proto.Int32(1337),
		Label:    descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
		Type:     descriptorpb.FieldDescriptorProto_TYPE_MESSAGE.Enum(),
		TypeName: proto.String(".foo_message"),
	}

	got := ToFieldDescriptorProto(fd)
	if diff := cmp.Diff(want, got, protocmp.Transform()); diff != "" {
		t.Errorf("ToFieldDescriptor: unexpected diff (-want +got):\n%s", diff)
	}
}

func TestProto2Group(t *testing.T) {
	md := new(filedesc.Message)
	md.L0.ParentFile = filedesc.SurrogateProto2
	md.L0.FullName = "foo_message"
	fd := new(filedesc.Field)
	fd.L0.ParentFile = filedesc.SurrogateProto2
	fd.L0.FullName = "foo_field"
	fd.L1.Number = 1337
	fd.L1.Cardinality = protoreflect.Optional
	fd.L1.Kind = protoreflect.GroupKind
	fd.L1.Message = md

	want := &descriptorpb.FieldDescriptorProto{
		Name:     proto.String("foo_field"),
		Number:   proto.Int32(1337),
		Label:    descriptorpb.FieldDescriptorProto_LABEL_OPTIONAL.Enum(),
		Type:     descriptorpb.FieldDescriptorProto_TYPE_GROUP.Enum(),
		TypeName: proto.String(".foo_message"),
	}

	got := ToFieldDescriptorProto(fd)
	if diff := cmp.Diff(want, got, protocmp.Transform()); diff != "" {
		t.Errorf("ToFieldDescriptor: unexpected diff (-want +got):\n%s", diff)
	}
}
