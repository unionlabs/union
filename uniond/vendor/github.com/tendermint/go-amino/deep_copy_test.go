package amino_test

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	amino "github.com/tendermint/go-amino"
)

type DCFoo1 struct{ a string }

func newDCFoo1(a string) *DCFoo1                  { return &DCFoo1{a: a} }
func (dcf *DCFoo1) MarshalAmino() (string, error) { return dcf.a, nil }
func (dcf *DCFoo1) UnmarshalAmino(s string) error { dcf.a = s; return nil }

func TestDeepCopyFoo1(t *testing.T) {
	dcf1 := newDCFoo1("foobar")
	dcf2 := amino.DeepCopy(dcf1).(*DCFoo1)
	assert.Equal(t, "foobar", dcf2.a)
}

type DCFoo2 struct{ a string }

func newDCFoo2(a string) *DCFoo2                  { return &DCFoo2{a: a} }
func (dcf DCFoo2) MarshalAmino() (string, error)  { return dcf.a, nil } // non-pointer receiver
func (dcf *DCFoo2) UnmarshalAmino(s string) error { dcf.a = s; return nil }

func TestDeepCopyFoo2(t *testing.T) {
	dcf1 := newDCFoo2("foobar")
	dcf2 := amino.DeepCopy(dcf1).(*DCFoo2)
	assert.Equal(t, "foobar", dcf2.a)
}

type DCFoo3 struct{ a string }

func newDCFoo3(a string) *DCFoo3                  { return &DCFoo3{a: a} }
func (dcf DCFoo3) MarshalAmino() (string, error)  { return dcf.a, nil }
func (dcf *DCFoo3) UnmarshalAmino(s []byte) error { dcf.a = string(s); return nil } // mismatch type

func TestDeepCopyFoo3(t *testing.T) {
	dcf1 := newDCFoo3("foobar")
	dcf2 := amino.DeepCopy(dcf1).(*DCFoo3)
	assert.Equal(t, "", dcf2.a)
}

type DCFoo4 struct{ a string }

func newDCFoo4(a string) *DCFoo4                  { return &DCFoo4{a: a} }
func (dcf *DCFoo4) DeepCopy() *DCFoo4             { return &DCFoo4{"good"} }
func (dcf DCFoo4) MarshalAmino() (string, error)  { return dcf.a, nil }
func (dcf *DCFoo4) UnmarshalAmino(s string) error { dcf.a = string(s); return nil } // mismatch type

func TestDeepCopyFoo4(t *testing.T) {
	dcf1 := newDCFoo4("foobar")
	dcf2 := amino.DeepCopy(dcf1).(*DCFoo4)
	assert.Equal(t, "good", dcf2.a)
}

type DCFoo5 struct{ a string }

func newDCFoo5(a string) *DCFoo5                  { return &DCFoo5{a: a} }
func (dcf DCFoo5) DeepCopy() DCFoo5               { return DCFoo5{"good"} }
func (dcf DCFoo5) MarshalAmino() (string, error)  { return dcf.a, nil }
func (dcf *DCFoo5) UnmarshalAmino(s string) error { dcf.a = string(s); return nil } // mismatch type

func TestDeepCopyFoo5(t *testing.T) {
	dcf1 := newDCFoo5("foobar")
	dcf2 := amino.DeepCopy(dcf1).(*DCFoo5)
	assert.Equal(t, "good", dcf2.a)
}

type DCFoo6 struct{ a string }

func newDCFoo6(a string) *DCFoo6     { return &DCFoo6{a: a} }
func (dcf *DCFoo6) DeepCopy() DCFoo6 { return DCFoo6{"good"} }

func TestDeepCopyFoo6(t *testing.T) {
	dcf1 := newDCFoo6("foobar")
	dcf2 := amino.DeepCopy(dcf1).(*DCFoo6)
	assert.Equal(t, "good", dcf2.a)
}

type DCFoo7 struct{ a string }

func newDCFoo7(a string) *DCFoo7     { return &DCFoo7{a: a} }
func (dcf DCFoo7) DeepCopy() *DCFoo7 { return &DCFoo7{"good"} }

func TestDeepCopyFoo7(t *testing.T) {
	dcf1 := newDCFoo7("foobar")
	dcf2 := amino.DeepCopy(dcf1).(*DCFoo7)
	assert.Equal(t, "good", dcf2.a)
}

type DCFoo8 struct{ a string }

func newDCFoo8(a string) *DCFoo8                  { return &DCFoo8{a: a} }
func (dcf DCFoo8) MarshalAmino() (string, error)  { return "", errors.New("uh oh") } // error
func (dcf *DCFoo8) UnmarshalAmino(s string) error { dcf.a = string(s); return nil }

func TestDeepCopyFoo8(t *testing.T) {
	dcf1 := newDCFoo8("foobar")
	assert.Panics(t, func() { amino.DeepCopy(dcf1) })
}

type DCFoo9 struct{ a string }

func newDCFoo9(a string) *DCFoo9                  { return &DCFoo9{a: a} }
func (dcf DCFoo9) MarshalAmino() (string, error)  { return dcf.a, nil }
func (dcf *DCFoo9) UnmarshalAmino(s string) error { return errors.New("uh oh") } // error

func TestDeepCopyFoo9(t *testing.T) {
	dcf1 := newDCFoo9("foobar")
	assert.Panics(t, func() { amino.DeepCopy(dcf1) })
}

type DCInterface1 struct {
	Foo interface{}
}

func TestDeepCopyInterface1(t *testing.T) {
	dci1 := DCInterface1{Foo: nil}
	dci2 := amino.DeepCopy(dci1).(DCInterface1)
	assert.Nil(t, dci2.Foo)
}

type DCInterface2 struct {
	Foo interface{}
}

func TestDeepCopyInterface2(t *testing.T) {
	dci1 := DCInterface1{Foo: "foo"}
	dci2 := amino.DeepCopy(dci1).(DCInterface1)
	assert.Equal(t, "foo", dci2.Foo)
}
