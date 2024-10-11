package padding

import (
	"testing"
	. "gopkg.in/check.v1"
)

func Test(t *testing.T) { TestingT(t) }
type TestSuite struct{}
var _ = Suite(&TestSuite{})

func (s *TestSuite) TestRemovePkcs7NoPadding(c *C) {
	//given
	padded:=[]byte{1,2,3,4,5,6,7,8}
	
	//when	
	test:=RemovePkcs7(padded,8)
	
	//then
	c.Assert(test, DeepEquals, []byte{1,2,3,4,5,6,7,8})	
}

func (s *TestSuite) TestRemovePkcs7(c *C) {
	//given
	padded:=[]byte{1,2,3,4,5,3,3,3}
	
	//when	
	test:=RemovePkcs7(padded,8)
	
	//then
	c.Assert(test, DeepEquals, []byte{1,2,3,4,5})
}

func (s *TestSuite) TestRemovePkcs7OneBytePadding(c *C) {
	//given
	padded:=[]byte{1,2,3,4,5,1}
	
	//when	
	test:=RemovePkcs7(padded,6)
	
	//then
	c.Assert(test, DeepEquals, []byte{1,2,3,4,5})
}

func (s *TestSuite) TestRemovePkcs7TrailingZeroByte(c *C) {
	//given
	padded:=[]byte{1,2,3,4,5,0}
	
	//when	
	test:=RemovePkcs7(padded,6)
	
	//then
	c.Assert(test, DeepEquals, []byte{1,2,3,4,5,0})
}

func (s *TestSuite) TestRemovePkcs7ExtraBlockPadding(c *C) {
	//given
	padded:=[]byte{1,2,3,4,5,5,5,5,5,5}
	
	//when	
	test:=RemovePkcs7(padded,5)
	
	//then
	c.Assert(test, DeepEquals, []byte{1,2,3,4,5})
}

func (s *TestSuite) TestRemovePkcs7TrailingByteGreaterBlockSize(c *C) {
	//given
	padded:=[]byte{1,2,3,4,5,10}
	
	//when	
	test:=RemovePkcs7(padded,6)
	
	//then
	c.Assert(test, DeepEquals, []byte{1,2,3,4,5,10})
}

func (s *TestSuite) TestAddPkcs7(c *C) {
	//given
	in:=[]byte{1,2,3,4,5}
	
	//when	
	test := AddPkcs7(in,8)
	
	//then
	c.Assert(test, DeepEquals, []byte{1,2,3,4,5,3,3,3})
}

func (s *TestSuite) TestAddPkcs7OneBytePadding(c *C) {
	//given
	in:=[]byte{1,2,3,4,5}
	
	//when	
	test := AddPkcs7(in,6)
	
	//then
	c.Assert(test, DeepEquals, []byte{1,2,3,4,5,1})
}

func (s *TestSuite) TestAddPkcs7ExtraBlockPadding(c *C) {
	//given
	in:=[]byte{1,2,3,4,5,6,7,8}
	
	//when	
	test := AddPkcs7(in,8)
	
	//then
	c.Assert(test, DeepEquals, []byte{1,2,3,4,5,6,7,8,8,8,8,8,8,8,8,8})
}