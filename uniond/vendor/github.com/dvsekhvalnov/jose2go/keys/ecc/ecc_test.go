package ecc

import (
	"testing"
	. "gopkg.in/check.v1"
	"io/ioutil"
	"math/big"
	"crypto/elliptic"
)

func Test(t *testing.T) { TestingT(t) }
type TestSuite struct{}
var _ = Suite(&TestSuite{})

func (s *TestSuite) TestNewPublic(c *C) {
	//given
	x:=[]byte {4, 114, 29, 223, 58, 3, 191, 170, 67, 128, 229, 33, 242, 178, 157, 150, 133, 25, 209, 139, 166, 69, 55, 26, 84, 48, 169, 165, 67, 232, 98, 9}
	y:=[]byte {131, 116, 8, 14, 22, 150, 18, 75, 24, 181, 159, 78, 90, 51, 71, 159, 214, 186, 250, 47, 207, 246, 142, 127, 54, 183, 72, 72, 253, 21, 88, 53}
	
	//when	
	test:=NewPublic(x,y)
	
	//then
	c.Assert(test.X, DeepEquals, bigInt("2010878128539620107131539503221291822343443356718189500659356750794038206985"))
	c.Assert(test.Y, DeepEquals, bigInt("59457993017710823357637488495120101390437944162821778556218889662829000218677"))
	c.Assert(test.Curve, Equals, elliptic.P256())
}

func (s *TestSuite) TestNewPrivate(c *C) {
	//given
	x:=[]byte {4, 114, 29, 223, 58, 3, 191, 170, 67, 128, 229, 33, 242, 178, 157, 150, 133, 25, 209, 139, 166, 69, 55, 26, 84, 48, 169, 165, 67, 232, 98, 9}
	y:=[]byte {131, 116, 8, 14, 22, 150, 18, 75, 24, 181, 159, 78, 90, 51, 71, 159, 214, 186, 250, 47, 207, 246, 142, 127, 54, 183, 72, 72, 253, 21, 88, 53}
	d:=[]byte{ 42, 148, 231, 48, 225, 196, 166, 201, 23, 190, 229, 199, 20, 39, 226, 70, 209, 148, 29, 70, 125, 14, 174, 66, 9, 198, 80, 251, 95, 107, 98, 206 }
	
	//when	
	test:=NewPrivate(x,y,d)
	
	//then
	c.Assert(test.X, DeepEquals, bigInt("2010878128539620107131539503221291822343443356718189500659356750794038206985"))
	c.Assert(test.Y, DeepEquals, bigInt("59457993017710823357637488495120101390437944162821778556218889662829000218677"))
	c.Assert(test.D, DeepEquals, bigInt("19260228627344101198652694952536756709538941185117188878548538012226554651342"))
	c.Assert(test.Curve, Equals, elliptic.P256())
}

func (s *TestSuite) TestReadPublicPKIX(c *C) {
	//given
	keyBytes, _ := ioutil.ReadFile("./ec_public.key")
	
	//when	
	test,e := ReadPublic(keyBytes)
	
	//then
	c.Assert(e, IsNil)
	
	c.Assert(test.X, DeepEquals, bigInt("76939435694210362824363841832595476784225842365248086547769733757874741672069"))
	c.Assert(test.Y, DeepEquals, bigInt("80047042001812490693675653292813886154388201612539715595028491948003157744818"))
	c.Assert(test.Curve, Equals, elliptic.P256())
}

func (s *TestSuite) TestReadPublicPKCS1(c *C) {
	//given
	keyBytes, _ := ioutil.ReadFile("./ec_cert.pem")
	
	//when	
	test,e := ReadPublic(keyBytes)
	
	//then
	c.Assert(e, IsNil)
	
	c.Assert(test.X, DeepEquals, bigInt("76939435694210362824363841832595476784225842365248086547769733757874741672069"))
	c.Assert(test.Y, DeepEquals, bigInt("80047042001812490693675653292813886154388201612539715595028491948003157744818"))
	c.Assert(test.Curve, Equals, elliptic.P256())
}

func (s *TestSuite) TestReadPrivatePKCS1(c *C) {
	//given
	keyBytes, _ := ioutil.ReadFile("./ec_private.key")
	
	//when	
	test,e := ReadPrivate(keyBytes)
	
	//then
	c.Assert(e, IsNil)
	
	c.Assert(test.X, DeepEquals, bigInt("76939435694210362824363841832595476784225842365248086547769733757874741672069"))
	c.Assert(test.Y, DeepEquals, bigInt("80047042001812490693675653292813886154388201612539715595028491948003157744818"))
	c.Assert(test.D, DeepEquals, bigInt("7222604869653061109880849859470152714201198955914263913554931724612175399644"))
	c.Assert(test.Curve, Equals, elliptic.P256())
}

func (s *TestSuite) TestReadPrivatePKCS8(c *C) {
	//given
	keyBytes, _ := ioutil.ReadFile("./ec_private.pem")
	
	//when	
	test,e := ReadPrivate(keyBytes)
	
	//then
	c.Assert(e, IsNil)
	
	c.Assert(test.X, DeepEquals, bigInt("76939435694210362824363841832595476784225842365248086547769733757874741672069"))
	c.Assert(test.Y, DeepEquals, bigInt("80047042001812490693675653292813886154388201612539715595028491948003157744818"))
	c.Assert(test.D, DeepEquals, bigInt("7222604869653061109880849859470152714201198955914263913554931724612175399644"))
	c.Assert(test.Curve, Equals, elliptic.P256())
}

//utils
func bigInt(value string) *big.Int {
	i:=new (big.Int)
	i.SetString(value,10)
	
	return i	
}