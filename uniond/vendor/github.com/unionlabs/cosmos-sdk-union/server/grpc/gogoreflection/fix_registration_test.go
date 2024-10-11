package gogoreflection

import (
	"testing"

	"google.golang.org/protobuf/runtime/protoimpl"
)

func TestRegistrationFix(t *testing.T) {
	res := getFileDescriptor("gogoproto/gogo.proto")
	rawDesc, err := decompress(res)
	if err != nil {
		t.Fatal(err)
	}
	fd := protoimpl.DescBuilder{
		RawDescriptor: rawDesc,
	}.Build()

	if fd.File.Extensions().Len() == 0 {
		t.Fatal("unexpected parsing")
	}
}
