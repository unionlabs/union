package amino_test

import (
	"fmt"
	"testing"

	"time"

	"github.com/stretchr/testify/assert"
	"github.com/tendermint/go-amino"
)

func TestNilSliceEmptySlice(t *testing.T) {
	var cdc = amino.NewCodec()

	type TestStruct struct {
		A []byte
		B []int
		C [][]byte
		D [][]int
		E []*[]byte
		F []*[]int
	}
	nnb, nni := []byte(nil), []int(nil)
	eeb, eei := []byte{}, []int{}

	a := TestStruct{
		A: nnb,
		B: nni,
		C: [][]byte{nnb},
		D: [][]int{nni},
		E: []*[]byte{nil},
		F: []*[]int{nil},
	}
	b := TestStruct{
		A: eeb,
		B: eei,
		C: [][]byte{eeb},
		D: [][]int{eei},
		E: []*[]byte{&nnb},
		F: []*[]int{&nni},
	}
	c := TestStruct{
		A: eeb,
		B: eei,
		C: [][]byte{eeb},
		D: [][]int{eei},
		E: []*[]byte{&eeb},
		F: []*[]int{&eei},
	}

	abz := cdc.MustMarshalBinaryLengthPrefixed(a)
	bbz := cdc.MustMarshalBinaryLengthPrefixed(b)
	cbz := cdc.MustMarshalBinaryLengthPrefixed(c)

	assert.Equal(t, abz, bbz, "a != b")
	assert.Equal(t, abz, cbz, "a != c")
}

func TestNewFieldBackwardsCompatibility(t *testing.T) {
	type V1 struct {
		String  string
		String2 string
	}

	type V2 struct {
		String  string
		String2 string
		// new fields in V2:
		Time time.Time
		Int  int
	}

	type SomeStruct struct {
		Sth int
	}

	type V3 struct {
		String string
		// different from V1 starting here:
		Int  int
		Some SomeStruct
	}

	cdc := amino.NewCodec()
	notNow, _ := time.Parse("2006-01-02", "1934-11-09")
	v2 := V2{String: "hi", String2: "cosmos", Time: notNow, Int: 4}
	bz, err := cdc.MarshalBinaryBare(v2)
	assert.Nil(t, err, "unexpected error while encoding V2: %v", err)

	var v1 V1
	err = cdc.UnmarshalBinaryBare(bz, &v1)
	assert.Nil(t, err, "unexpected error %v", err)
	assert.Equal(t, v1, V1{"hi", "cosmos"},
		"backwards compatibility failed: didn't yield expected result ...")

	v3 := V3{String: "tender", Int: 2014, Some: SomeStruct{Sth: 84}}
	bz2, err := cdc.MarshalBinaryBare(v3)
	assert.Nil(t, err, "unexpected error")

	err = cdc.UnmarshalBinaryBare(bz2, &v1)
	// this might change later but we include this case to document the current behaviour:
	assert.NotNil(t, err, "expected an error here because of changed order of fields")

	// we still expect that decoding worked to some extend (until above error occurred):
	assert.Equal(t, v1, V1{"tender", "cosmos"})
}

func TestWriteEmpty(t *testing.T) {
	type Inner struct {
		Val int
	}
	type SomeStruct struct {
		Inner Inner
	}

	cdc := amino.NewCodec()
	b, err := cdc.MarshalBinaryBare(Inner{})
	assert.NoError(t, err)
	assert.Equal(t, b, []byte(nil), "empty struct should be encoded as empty bytes")
	var inner Inner
	cdc.UnmarshalBinaryBare(b, &inner)
	assert.Equal(t, Inner{}, inner, "")

	b, err = cdc.MarshalBinaryBare(SomeStruct{})
	assert.NoError(t, err)
	assert.Equal(t, b, []byte(nil), "empty structs should be encoded as empty bytes")
	var outer SomeStruct
	cdc.UnmarshalBinaryBare(b, &outer)
	assert.Equal(t, SomeStruct{}, outer, "")
}

func TestForceWriteEmpty(t *testing.T) {
	type InnerWriteEmpty struct {
		// sth. that isn't zero-len if default, e.g. fixed32:
		ValIn int32 `amino:"write_empty" binary:"fixed32"`
	}

	type OuterWriteEmpty struct {
		In  InnerWriteEmpty `amino:"write_empty"`
		Val int             `amino:"write_empty" binary:"fixed32"`
	}

	cdc := amino.NewCodec()

	b, err := cdc.MarshalBinaryBare(OuterWriteEmpty{})
	assert.NoError(t, err)
	assert.NotZero(t, len(b), "amino:\"write_empty\" did not work")

	b, err = cdc.MarshalBinaryBare(InnerWriteEmpty{})
	assert.NoError(t, err)
	t.Log(b)
	// TODO(ismail): this alone won't be encoded:
	//assert.NotZero(t, len(b), "amino:\"write_empty\" did not work")
}

func TestStructSlice(t *testing.T) {
	type Foo struct {
		A int
		B int
	}

	type Foos []Foo

	f := Foos{Foo{100, 101}, Foo{102, 103}}

	cdc := amino.NewCodec()

	bz, err := cdc.MarshalBinaryBare(f)
	assert.NoError(t, err)
	assert.Equal(t, "0A04086410650A0408661067", fmt.Sprintf("%X", bz))
	t.Log(bz)
	var f2 Foos
	cdc.UnmarshalBinaryBare(bz, &f2)
	assert.Equal(t, f, f2)
}

func TestStructPointerSlice1(t *testing.T) {
	cdc := amino.NewCodec()

	type Foo struct {
		A string
		B int
		C []*Foo
		D string // exposed
	}

	var f = Foo{
		A: "k",
		B: 2,
		C: []*Foo{nil, nil, nil},
		D: "j",
	}
	bz, err := cdc.MarshalBinaryLengthPrefixed(f)
	assert.NoError(t, err)

	var f2 Foo
	err = cdc.UnmarshalBinaryLengthPrefixed(bz, &f2)
	assert.Nil(t, err)

	assert.Equal(t, f, f2)
	assert.Nil(t, f2.C[0])

	var f3 = Foo{
		A: "k",
		B: 2,
		C: []*Foo{{}, {}, {}},
		D: "j",
	}
	bz2, err := cdc.MarshalBinaryLengthPrefixed(f3)
	assert.NoError(t, err)
	assert.Equal(t, bz, bz2, "empty slices should be decoded to nil unless empty_elements")
}

// Like TestStructPointerSlice2, but with EmptyElements.
func TestStructPointerSlice2(t *testing.T) {
	cdc := amino.NewCodec()

	type Foo struct {
		A string
		B int
		C []*Foo `amino:"empty_elements"`
		D string // exposed
	}

	var f = Foo{
		A: "k",
		B: 2,
		C: []*Foo{nil, nil, nil},
		D: "j",
	}
	bz, err := cdc.MarshalBinaryLengthPrefixed(f)
	assert.Error(t, err, "nil elements of a slice/array not supported when empty_elements field tag set.")

	f.C = []*Foo{{}, {}, {}}
	bz, err = cdc.MarshalBinaryLengthPrefixed(f)
	assert.NoError(t, err)

	var f2 Foo
	err = cdc.UnmarshalBinaryLengthPrefixed(bz, &f2)
	assert.Nil(t, err)

	assert.Equal(t, f, f2)
	assert.NotNil(t, f2.C[0])
}
