package proto_test

import (
	"testing"

	"github.com/golang/protobuf/jsonpb"

	"github.com/cosmos/gogoproto/proto"
	descriptorpb "github.com/cosmos/gogoproto/protoc-gen-gogo/descriptor"
)

// This tests for compatibility with google.golang.org types.
func TestV2Support(t *testing.T) {
	// Test that we can marshal and unmarshal a timestamp.
	fd := &descriptorpb.FieldDescriptorProto{
		Name:   proto.String("test"),
		Number: proto.Int32(1),
	}
	bz, err := proto.Marshal(fd)
	if err != nil {
		t.Fatalf("unexpected error marshaling: %v", err)
	}

	fd2 := &descriptorpb.FieldDescriptorProto{}
	if err := proto.Unmarshal(bz, fd2); err != nil {
		t.Fatalf("unexpected error unmarshaling: %v", err)
	}

	// Test json marshaling
	str, err := (&jsonpb.Marshaler{}).MarshalToString(fd)
	if err != nil {
		t.Fatalf("unexpected error marshaling to json: %v", err)
	}

	fd3 := &descriptorpb.FieldDescriptorProto{}
	if err := jsonpb.UnmarshalString(str, fd3); err != nil {
		t.Fatalf("unexpected error unmarshaling from json: %v", err)
	}

	// Test that we get the right message name
	if proto.MessageName(fd) != "google.protobuf.FieldDescriptorProto" {
		t.Fatalf("unexpected message name: %v", proto.MessageName(fd))
	}
}
