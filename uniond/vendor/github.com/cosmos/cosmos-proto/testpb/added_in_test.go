package testpb

import (
	"testing"

	cosmos_proto "github.com/cosmos/cosmos-proto"
	"google.golang.org/protobuf/proto"
)

func TestMessageAddedIn(t *testing.T) {
	resp := new(CounterQueryResponse)
	desc := resp.ProtoReflect().Descriptor().Options()

	if !proto.HasExtension(desc, cosmos_proto.E_MessageAddedIn) {
		t.Fatalf("message_added_in extension missing %v", resp)
	}

	expectedVersion := "github.com/cosmos/cosmos-proto v1.0.1"
	versionField := proto.GetExtension(desc, cosmos_proto.E_MessageAddedIn).(string)
	if versionField != expectedVersion {
		t.Fatalf("versionField: %s != expectedVersion: %s", versionField, expectedVersion)
	}
}

func TestFieldAddedIn(t *testing.T) {
	resp := new(CounterQueryResponse)
	desc := resp.ProtoReflect().Descriptor().Fields().ByName("counter").Options()

	if !proto.HasExtension(desc, cosmos_proto.E_FieldAddedIn) {
		t.Fatalf("field_added_in extension missing %v", resp)
	}

	expectedVersion := "github.com/cosmos/cosmos-proto v1.0.1"
	versionField := proto.GetExtension(desc, cosmos_proto.E_FieldAddedIn).(string)
	if versionField != expectedVersion {
		t.Fatalf("versionField: %s != expectedVersion: %s", versionField, expectedVersion)
	}
}
