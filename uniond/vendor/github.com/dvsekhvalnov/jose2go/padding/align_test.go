package	padding

import (
	. "gopkg.in/check.v1"
)

func (s *TestSuite) TestAlignOneByte(c *C) {
	//given
	data:=[]byte{1,2,3}
	
	//when	
	test:=Align(data,27)
	
	//then
	c.Assert(test, DeepEquals, []byte{0,1,2,3})
	
}

func (s *TestSuite) TestAlignMultiByte(c *C) {
	//given
	data:=[]byte{1,2,3}
	
	//when	
	test:=Align(data,40)
	
	//then
	c.Assert(test, DeepEquals, []byte{0,0,1,2,3})
	
}

func (s *TestSuite) TestAlignMultiBytePartial(c *C) {
	//given
	data:=[]byte{1,2,3}
	
	//when	
	test:=Align(data,43)
	
	//then
	c.Assert(test, DeepEquals, []byte{0,0,0,1,2,3})
	
}

func (s *TestSuite) TestAlignedArray(c *C) {
	//given
	data:=[]byte{1,2,3}
	
	//when	
	test:=Align(data,24)
	
	//then
	c.Assert(test, DeepEquals, []byte{1,2,3})
}