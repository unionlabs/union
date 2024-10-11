package primitives

import (
	"reflect"
	"testing"
)

func TestDomain_Casting(t *testing.T) {
	t.Run("empty byte slice", func(t *testing.T) {
		b := make([]byte, 0)
		d := Domain(b)
		if !reflect.DeepEqual([]byte(d), b) {
			t.Errorf("Unequal: %v = %v", d, b)
		}
	})

	t.Run("non-empty byte slice", func(t *testing.T) {
		b := make([]byte, 2)
		b[0] = byte('a')
		b[1] = byte('b')
		d := Domain(b)
		if !reflect.DeepEqual([]byte(d), b) {
			t.Errorf("Unequal: %v = %v", d, b)
		}
	})

	t.Run("byte array", func(t *testing.T) {
		var b [2]byte
		b[0] = byte('a')
		b[1] = byte('b')
		d := Domain(b[:])
		if !reflect.DeepEqual([]byte(d), b[:]) {
			t.Errorf("Unequal: %v = %v", d, b)
		}
	})
}

func TestDomain_UnmarshalSSZ(t *testing.T) {
	t.Run("Ok", func(t *testing.T) {
		d := Domain{}
		var b = [32]byte{'f', 'o', 'o'}
		err := d.UnmarshalSSZ(b[:])
		if err != nil {
			t.Fatalf("Unexpected error: %v", err)
		}
		if !reflect.DeepEqual(b[:], []byte(d)) {
			t.Errorf("Unequal: %v = %v", b, []byte(d))
		}
	})

	t.Run("Wrong slice length", func(t *testing.T) {
		d := Domain{}
		var b = [16]byte{'f', 'o', 'o'}
		err := d.UnmarshalSSZ(b[:])
		if err == nil {
			t.Error("Expected error")
		}
	})
}

func TestDomain_MarshalSSZTo(t *testing.T) {
	d := Domain("foo")
	dst := []byte("bar")
	b, err := d.MarshalSSZTo(dst)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	expected := []byte("barfoo")
	if !reflect.DeepEqual(expected, b) {
		t.Errorf("Unequal: %v = %v", expected, b)
	}
}

func TestDomain_MarshalSSZ(t *testing.T) {
	d := Domain("foo")
	b, err := d.MarshalSSZ()
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}
	if !reflect.DeepEqual(b, []byte(d)) {
		t.Errorf("Unequal: %v = %v", b, []byte(d))
	}
}

func TestDomain_SizeSSZ(t *testing.T) {
	d := Domain{}
	if d.SizeSSZ() != 32 {
		t.Errorf("Wrong SSZ size. Expected %v vs actual %v", 32, d.SizeSSZ())
	}
}
