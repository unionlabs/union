package base64url

import (
	"testing"	
	"fmt"
	. "gopkg.in/check.v1"
)

func Test(t *testing.T) { TestingT(t) }
type TestSuite struct{}
var _ = Suite(&TestSuite{})

func (s *TestSuite) TestEncode(c *C) {
	//given
	in:=[]byte{72,101,108,108,111,32,66,97,115,101,54,52,85,114,108,32,101,110,99,111,100,105,110,103,33}
	
	//when
	test:=Encode(in)
	
	//then	
	c.Assert(test, Equals, "SGVsbG8gQmFzZTY0VXJsIGVuY29kaW5nIQ")
}

func  (s *TestSuite) TestDecode(c *C) {
	//when
	test,err := Decode("SGVsbG8gQmFzZTY0VXJsIGVuY29kaW5nIQ")
	
	//then	
	c.Assert(err, IsNil)	
	c.Assert(test, DeepEquals, []byte{72,101,108,108,111,32,66,97,115,101,54,52,85,114,108,32,101,110,99,111,100,105,110,103,33})		
}

func (s *TestSuite) TestDecodeIllegalBase64String(c *C) {
	//when
	_,err := Decode("SGVsbG8gQmFzZTY0VXJsIGVuY29kaW5nQ")
	
	//then
	c.Assert(err,NotNil)
	fmt.Printf("err = %v\n",err)	
}