package descriptor_test

import (
	"testing"

	"github.com/cosmos/gogoproto/protoc-gen-gogo/descriptor"
)

func TestMessage(t *testing.T) {
	var msg *descriptor.DescriptorProto
	fd, md := descriptor.ForMessage(msg)
	if pkg, want := fd.GetPackage(), "google.protobuf"; pkg != want {
		t.Errorf("descriptor.ForMessage(%T).GetPackage() = %q; want %q", msg, pkg, want)
	}
	if name, want := md.GetName(), "DescriptorProto"; name != want {
		t.Fatalf("descriptor.ForMessage(%T).GetName() = %q; want %q", msg, name, want)
	}
}
