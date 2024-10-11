package primitives_test

import (
	"reflect"
	"strings"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

func TestSSZUint64_Limit(t *testing.T) {
	sszType := primitives.SSZUint64(0)
	var serializedObj [7]byte
	err := sszType.UnmarshalSSZ(serializedObj[:])
	if err == nil || !strings.Contains(err.Error(), "expected buffer of length") {
		t.Errorf("Expected Error = %s, got: %v", "expected buffer of length", err)
	}
}

func TestSSZUint64_RoundTrip(t *testing.T) {
	fixedVal := uint64(8)
	sszVal := primitives.SSZUint64(fixedVal)

	marshalledObj, err := sszVal.MarshalSSZ()
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	newVal := primitives.SSZUint64(0)

	err = newVal.UnmarshalSSZ(marshalledObj)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	if fixedVal != uint64(newVal) {
		t.Errorf("Unequal: %v = %v", fixedVal, uint64(newVal))
	}
}

func TestSSZUint64(t *testing.T) {
	tests := []struct {
		name            string
		serializedBytes []byte
		actualValue     uint64
		root            []byte
		wantErr         bool
	}{
		{
			name:            "max",
			serializedBytes: hexDecodeOrDie(t, "ffffffffffffffff"),
			actualValue:     18446744073709551615,
			root:            hexDecodeOrDie(t, "ffffffffffffffff000000000000000000000000000000000000000000000000"),
			wantErr:         false,
		},
		{
			name:            "random",
			serializedBytes: hexDecodeOrDie(t, "357c8de9d7204577"),
			actualValue:     8594311575614880821,
			root:            hexDecodeOrDie(t, "357c8de9d7204577000000000000000000000000000000000000000000000000"),
			wantErr:         false,
		},
		{
			name:            "zero",
			serializedBytes: hexDecodeOrDie(t, "0000000000000000"),
			actualValue:     0,
			root:            hexDecodeOrDie(t, "0000000000000000000000000000000000000000000000000000000000000000"),
			wantErr:         false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var s primitives.SSZUint64
			if err := s.UnmarshalSSZ(tt.serializedBytes); (err != nil) != tt.wantErr {
				t.Errorf("SSZUint64.UnmarshalSSZ() error = %v, wantErr %v", err, tt.wantErr)
			}
			if tt.actualValue != uint64(s) {
				t.Errorf("SSZUint64.UnmarshalSSZ() = %v, want %v", uint64(s), tt.actualValue)
			}

			serializedBytes, err := s.MarshalSSZ()
			if err != nil {
				t.Errorf("SSZUint64.MarshalSSZ() unexpected error = %v", err)
			}
			if !reflect.DeepEqual(tt.serializedBytes, serializedBytes) {
				t.Errorf("SSZUint64.MarshalSSZ() = %v, want %v", serializedBytes, tt.serializedBytes)
			}

			htr, err := s.HashTreeRoot()
			if err != nil {
				t.Errorf("SSZUint64.HashTreeRoot() unexpected error = %v", err)
			}
			if !reflect.DeepEqual(tt.root, htr[:]) {
				t.Errorf("SSZUint64.HashTreeRoot() = %v, want %v", htr[:], tt.root)
			}
		})
	}
}
