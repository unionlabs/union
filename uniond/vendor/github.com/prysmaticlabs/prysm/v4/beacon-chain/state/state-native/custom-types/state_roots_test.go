package customtypes

import (
	"reflect"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestStateRoots_UnmarshalSSZ(t *testing.T) {
	t.Run("Ok", func(t *testing.T) {
		d := StateRoots{}
		var b [fieldparams.StateRootsLength][32]byte
		b[0] = [32]byte{'f', 'o', 'o'}
		b[1] = [32]byte{'b', 'a', 'r'}
		bb := make([]byte, fieldparams.StateRootsLength*32)
		for i, elem32 := range b {
			for j, elem := range elem32 {
				bb[i*32+j] = elem
			}
		}
		err := d.UnmarshalSSZ(bb)
		if err != nil {
			t.Fatalf("Unexpected error: %v", err)
		}
		if !reflect.DeepEqual(b, [fieldparams.StateRootsLength][32]byte(d)) {
			t.Errorf("Unequal: %v = %v", b, [fieldparams.StateRootsLength][32]byte(d))
		}
	})

	t.Run("Wrong slice length", func(t *testing.T) {
		d := StateRoots{}
		var b [fieldparams.StateRootsLength][16]byte
		b[0] = [16]byte{'f', 'o', 'o'}
		b[1] = [16]byte{'b', 'a', 'r'}
		bb := make([]byte, fieldparams.StateRootsLength*16)
		for i, elem16 := range b {
			for j, elem := range elem16 {
				bb[i*16+j] = elem
			}
		}
		err := d.UnmarshalSSZ(bb)
		if err == nil {
			t.Error("Expected error")
		}
	})
}

func TestStateRoots_MarshalSSZTo(t *testing.T) {
	var d StateRoots
	dst := []byte("foo")
	b, err := d.MarshalSSZTo(dst)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	expected := []byte{'f', 'o', 'o'}
	actual := []byte{b[0], b[1], b[2]}
	if !reflect.DeepEqual(expected, actual) {
		t.Errorf("Unequal: %v = %v", expected, actual)
	}
}

func TestStateRoots_MarshalSSZ(t *testing.T) {
	d := StateRoots(make([][32]byte, fieldparams.StateRootsLength))
	d[0] = [32]byte{'f', 'o', 'o'}
	d[1] = [32]byte{'b', 'a', 'r'}
	b, err := d.MarshalSSZ()
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	if !reflect.DeepEqual(d[0][:], b[0:32]) {
		t.Errorf("Unequal: %v = %v", d[0], b[0:32])
	}
	if !reflect.DeepEqual(d[1][:], b[32:64]) {
		t.Errorf("Unequal: %v = %v", d[0], b[32:64])
	}
}

func TestStateRoots_SizeSSZ(t *testing.T) {
	d := StateRoots{}
	if d.SizeSSZ() != fieldparams.StateRootsLength*32 {
		t.Errorf("Wrong SSZ size. Expected %v vs actual %v", fieldparams.StateRootsLength*32, d.SizeSSZ())
	}
}

func TestStateRoots_Slice(t *testing.T) {
	a, b, c := [32]byte{'a'}, [32]byte{'b'}, [32]byte{'c'}
	roots := StateRoots(make([][32]byte, fieldparams.StateRootsLength))
	roots[1] = a
	roots[10] = b
	roots[100] = c
	slice := roots.Slice()
	assert.DeepEqual(t, a[:], slice[1])
	assert.DeepEqual(t, b[:], slice[10])
	assert.DeepEqual(t, c[:], slice[100])
}
