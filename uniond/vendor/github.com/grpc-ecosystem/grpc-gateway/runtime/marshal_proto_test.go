package runtime_test

import (
	"bytes"
	"testing"

	"github.com/golang/protobuf/proto"
	"github.com/golang/protobuf/ptypes/timestamp"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/grpc-ecosystem/grpc-gateway/runtime/internal/examplepb"
)

var message = &examplepb.ABitOfEverything{
	SingleNested:        &examplepb.ABitOfEverything_Nested{},
	RepeatedStringValue: nil,
	MappedStringValue:   nil,
	MappedNestedValue:   nil,
	RepeatedEnumValue:   nil,
	TimestampValue:      &timestamp.Timestamp{},
	Uuid:                "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
	Nested: []*examplepb.ABitOfEverything_Nested{
		{
			Name:   "foo",
			Amount: 12345,
		},
	},
	Uint64Value: 0xFFFFFFFFFFFFFFFF,
	EnumValue:   examplepb.NumericEnum_ONE,
	OneofValue: &examplepb.ABitOfEverything_OneofString{
		OneofString: "bar",
	},
	MapValue: map[string]examplepb.NumericEnum{
		"a": examplepb.NumericEnum_ONE,
		"b": examplepb.NumericEnum_ZERO,
	},
}

func TestProtoMarshalUnmarshal(t *testing.T) {
	marshaller := runtime.ProtoMarshaller{}

	// Marshal
	buffer, err := marshaller.Marshal(message)
	if err != nil {
		t.Fatalf("Marshalling returned error: %s", err.Error())
	}

	// Unmarshal
	unmarshalled := &examplepb.ABitOfEverything{}
	err = marshaller.Unmarshal(buffer, unmarshalled)
	if err != nil {
		t.Fatalf("Unmarshalling returned error: %s", err.Error())
	}

	if !proto.Equal(unmarshalled, message) {
		t.Errorf(
			"Unmarshalled didn't match original message: (original = %v) != (unmarshalled = %v)",
			unmarshalled,
			message,
		)
	}
}

func TestProtoEncoderDecodert(t *testing.T) {
	marshaller := runtime.ProtoMarshaller{}

	var buf bytes.Buffer

	encoder := marshaller.NewEncoder(&buf)
	decoder := marshaller.NewDecoder(&buf)

	// Encode
	err := encoder.Encode(message)
	if err != nil {
		t.Fatalf("Encoding returned error: %s", err.Error())
	}

	// Decode
	unencoded := &examplepb.ABitOfEverything{}
	err = decoder.Decode(unencoded)
	if err != nil {
		t.Fatalf("Unmarshalling returned error: %s", err.Error())
	}

	if !proto.Equal(unencoded, message) {
		t.Errorf(
			"Unencoded didn't match original message: (original = %v) != (unencoded = %v)",
			unencoded,
			message,
		)
	}
}
