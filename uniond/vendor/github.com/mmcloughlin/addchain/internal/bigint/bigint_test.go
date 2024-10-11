package bigint

import (
	"math/big"
	"reflect"
	"testing"
)

func TestHex(t *testing.T) {
	cases := []struct {
		Input  string
		Expect int64
	}{
		{"0", 0},
		{"1", 1},
		{"f_4", 0xf4},
		{"abcd_ef", 0xabcdef},
	}
	for _, c := range cases {
		if got := MustHex(c.Input); !EqualInt64(got, c.Expect) {
			t.FailNow()
		}
	}
}

func TestBinary(t *testing.T) {
	cases := []struct {
		Input  string
		Expect int64
	}{
		{"0", 0},
		{"1", 1},
		{"1111_0100", 0xf4},
		{"1_1_____1_1_010______0", 0xf4},
	}
	for _, c := range cases {
		if got := MustBinary(c.Input); !EqualInt64(got, c.Expect) {
			t.FailNow()
		}
	}
}

func TestMask(t *testing.T) {
	if Mask(4, 16).Uint64() != 0xfff0 {
		t.Fail()
	}
}

func TestOnes(t *testing.T) {
	if Ones(8).Uint64() != 0xff {
		t.Fail()
	}
}

func TestBitsSet(t *testing.T) {
	x := big.NewInt(0x130)
	got := BitsSet(x)
	expect := []int{4, 5, 8}
	if !reflect.DeepEqual(got, expect) {
		t.FailNow()
	}
}

func TestExtract(t *testing.T) {
	x := big.NewInt(0xbeefcafe)
	if Extract(x, 4, 16).Uint64() != 0xcaf {
		t.Fail()
	}
}

func TestUint64s(t *testing.T) {
	x := MustHex("deadbeef_fedcba98_76543210")
	got := Uint64s(x)
	expect := []uint64{
		0xfedcba9876543210,
		0x00000000deadbeef,
	}
	if !reflect.DeepEqual(expect, got) {
		t.Fail()
	}
}

func TestBytesLittleEndian(t *testing.T) {
	x := MustHex("fe_ed_be_ef")
	got := BytesLittleEndian(x)
	expect := []byte{0xef, 0xbe, 0xed, 0xfe}
	if !reflect.DeepEqual(expect, got) {
		t.Fail()
	}
}
