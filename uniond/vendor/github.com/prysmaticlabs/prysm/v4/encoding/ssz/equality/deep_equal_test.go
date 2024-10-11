package equality_test

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/encoding/ssz/equality"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestDeepEqualBasicTypes(t *testing.T) {
	assert.Equal(t, true, equality.DeepEqual(true, true))
	assert.Equal(t, false, equality.DeepEqual(true, false))

	assert.Equal(t, true, equality.DeepEqual(byte(222), byte(222)))
	assert.Equal(t, false, equality.DeepEqual(byte(222), byte(111)))

	assert.Equal(t, true, equality.DeepEqual(uint64(1234567890), uint64(1234567890)))
	assert.Equal(t, false, equality.DeepEqual(uint64(1234567890), uint64(987653210)))

	assert.Equal(t, true, equality.DeepEqual("hello", "hello"))
	assert.Equal(t, false, equality.DeepEqual("hello", "world"))

	assert.Equal(t, true, equality.DeepEqual([3]byte{1, 2, 3}, [3]byte{1, 2, 3}))
	assert.Equal(t, false, equality.DeepEqual([3]byte{1, 2, 3}, [3]byte{1, 2, 4}))

	var nilSlice1, nilSlice2 []byte
	assert.Equal(t, true, equality.DeepEqual(nilSlice1, nilSlice2))
	assert.Equal(t, true, equality.DeepEqual(nilSlice1, []byte{}))
	assert.Equal(t, true, equality.DeepEqual([]byte{1, 2, 3}, []byte{1, 2, 3}))
	assert.Equal(t, false, equality.DeepEqual([]byte{1, 2, 3}, []byte{1, 2, 4}))
}

func TestDeepEqualStructs(t *testing.T) {
	type Store struct {
		V1 uint64
		V2 []byte
	}
	store1 := Store{uint64(1234), nil}
	store2 := Store{uint64(1234), []byte{}}
	store3 := Store{uint64(4321), []byte{}}
	assert.Equal(t, true, equality.DeepEqual(store1, store2))
	assert.Equal(t, false, equality.DeepEqual(store1, store3))
}

func TestDeepEqualStructs_Unexported(t *testing.T) {
	type Store struct {
		V1           uint64
		V2           []byte
		dontIgnoreMe string
	}
	store1 := Store{uint64(1234), nil, "hi there"}
	store2 := Store{uint64(1234), []byte{}, "hi there"}
	store3 := Store{uint64(4321), []byte{}, "wow"}
	store4 := Store{uint64(4321), []byte{}, "bow wow"}
	assert.Equal(t, true, equality.DeepEqual(store1, store2))
	assert.Equal(t, false, equality.DeepEqual(store1, store3))
	assert.Equal(t, false, equality.DeepEqual(store3, store4))
}

func TestDeepEqualProto(t *testing.T) {
	var fork1, fork2 *ethpb.Fork
	assert.Equal(t, true, equality.DeepEqual(fork1, fork2))

	fork1 = &ethpb.Fork{
		PreviousVersion: []byte{123},
		CurrentVersion:  []byte{124},
		Epoch:           1234567890,
	}
	fork2 = &ethpb.Fork{
		PreviousVersion: []byte{123},
		CurrentVersion:  []byte{125},
		Epoch:           1234567890,
	}
	assert.Equal(t, true, equality.DeepEqual(fork1, fork1))
	assert.Equal(t, false, equality.DeepEqual(fork1, fork2))

	checkpoint1 := &ethpb.Checkpoint{
		Epoch: 1234567890,
		Root:  []byte{},
	}
	checkpoint2 := &ethpb.Checkpoint{
		Epoch: 1234567890,
		Root:  nil,
	}
	assert.Equal(t, true, equality.DeepEqual(checkpoint1, checkpoint2))
}

func Test_IsProto(t *testing.T) {
	tests := []struct {
		name string
		item interface{}
		want bool
	}{
		{
			name: "uint64",
			item: 0,
			want: false,
		},
		{
			name: "string",
			item: "foobar cheese",
			want: false,
		},
		{
			name: "uint64 array",
			item: []uint64{1, 2, 3, 4, 5, 6},
			want: false,
		},
		{
			name: "Attestation",
			item: &ethpb.Attestation{},
			want: true,
		},
		{
			name: "Array of attestations",
			item: []*ethpb.Attestation{},
			want: true,
		},
		{
			name: "Map of attestations",
			item: make(map[uint64]*ethpb.Attestation),
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := equality.IsProto(tt.item); got != tt.want {
				t.Errorf("isProtoSlice() = %v, want %v", got, tt.want)
			}
		})
	}
}
