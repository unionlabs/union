package arrays

import (
	. "gopkg.in/check.v1"
	"testing"
)

func Test(t *testing.T) { TestingT(t) }

type TestSuite struct{}

var _ = Suite(&TestSuite{})

func (s *TestSuite) TestXor(c *C) {
	//given
	data := []byte{0xFF, 0x00, 0xF0, 0x0F, 0x55, 0xAA, 0xBB, 0xCC}

	//when
	test_1 := Xor(data, []byte{0x00, 0xFF, 0x0F, 0xF0, 0xAA, 0x55, 0x44, 0x33})
	test_2 := Xor(data, []byte{0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF})

	//then
	c.Assert(test_1, DeepEquals, []byte{0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF})
	c.Assert(test_2, DeepEquals, []byte{0x00, 0xFF, 0x0F, 0xF0, 0xAA, 0x55, 0x44, 0x33})
}

func (s *TestSuite) TestSlice(c *C) {
	//given
	data := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8}

	//when
	test := Slice(data, 3)

	//then
	c.Assert(len(test), Equals, 3)
	c.Assert(test[0], DeepEquals, []byte{0, 1, 2})
	c.Assert(test[1], DeepEquals, []byte{3, 4, 5})
	c.Assert(test[2], DeepEquals, []byte{6, 7, 8})
}

func (s *TestSuite) TestConcat(c *C) {
	//given
	a := []byte{1, 2, 3}
	b := []byte{4, 5}
	d := []byte{6}
	e := []byte{}
	f := []byte{7, 8, 9, 10}

	//when
	test := Concat(a, b, d, e, f)

	//then
	c.Assert(test, DeepEquals, []byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10})
}

func (s *TestSuite) TestUnwrap(c *C) {
	//given
	a := []byte{1, 2, 3}
	b := []byte{4, 5}
	d := []byte{6}
	e := []byte{}
	f := []byte{7, 8, 9, 10}

	//when
	test := Unwrap([][]byte{a, b, d, e, f})

	//then
	c.Assert(test, DeepEquals, []byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10})
}

func (s *TestSuite) TestUInt32ToBytes(c *C) {
	//then
	c.Assert(UInt32ToBytes(0xFF), DeepEquals, []byte{0x00, 0x00, 0x00, 0xFF})
	c.Assert(UInt32ToBytes(0xFFFFFFFE), DeepEquals, []byte{0xff, 0xff, 0xff, 0xfe})
}

func (s *TestSuite) TestUInt64ToBytes(c *C) {
	//then
	c.Assert(UInt64ToBytes(0xFF), DeepEquals, []byte{0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF})
	c.Assert(UInt64ToBytes(0xFFFFFFFFFFFFFFFE), DeepEquals, []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe})
}
