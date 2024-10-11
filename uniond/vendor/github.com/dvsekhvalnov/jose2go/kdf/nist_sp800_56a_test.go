package kdf

import (
	. "gopkg.in/check.v1"	
	"crypto/sha256"
	"github.com/dvsekhvalnov/jose2go/arrays"
	"github.com/dvsekhvalnov/jose2go/base64url"	
)

var z=[]byte{0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31}
var z2=[]byte{0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47}

var algId=prependDatalen([]byte("alg"))
var partyUInfo=datalenFormat("pui")
var partyVInfo=datalenFormat("pvi")

func (s *TestSuite) Test256Secret_256Key(c *C) {
	//when	
	test:=DeriveConcatKDF(256,z,algId,partyUInfo,partyVInfo,arrays.UInt32ToBytes(256),nil,sha256.New())
	
	//then
	c.Assert(test, DeepEquals, []byte{190, 69, 15, 62, 38, 64, 30, 141, 208, 163, 55, 202, 18, 71, 176, 174, 114, 221, 249, 255, 207, 131, 190, 77, 12, 115, 220, 144, 102, 149, 78, 28})
}

func (s *TestSuite) Test256Secret_384Key(c *C) {
	//when	
	test:=DeriveConcatKDF(384,z,algId,partyUInfo,partyVInfo,arrays.UInt32ToBytes(384),nil,sha256.New())
	
	//then
	c.Assert(test, DeepEquals, []byte{187, 16, 110, 253, 151, 199, 57, 235, 219, 8, 73, 191, 208, 108, 63, 241, 235, 137, 178, 149, 3, 199, 216, 99, 105, 217, 45, 114, 109, 213, 83, 198, 52, 101, 99, 146, 193, 31, 172, 157, 19, 140, 25, 202, 153, 92, 252, 192})
}

func (s *TestSuite) Test256Secret_521Key(c *C) {
	//when	
	test:=DeriveConcatKDF(521,z,algId,partyUInfo,partyVInfo,arrays.UInt32ToBytes(521),nil,sha256.New())
	
	//then
	c.Assert(test, DeepEquals, []byte{245, 40, 159, 13, 25, 70, 170, 74, 210, 240, 242, 224, 37, 215, 23, 201, 126, 90, 108, 103, 205, 180, 48, 193, 131, 40, 86, 183, 97, 144, 1, 150, 154, 186, 196, 127, 6, 19, 17, 230, 75, 144, 229, 195, 61, 240, 20, 173, 167, 159, 50, 103, 133, 177, 241, 145, 134, 84, 50, 246, 157, 252, 51, 24, 35})
}

func (s *TestSuite) Test384Secret_256Key(c *C) {
	//when	
	test:=DeriveConcatKDF(256,z2,algId,partyUInfo,partyVInfo,arrays.UInt32ToBytes(256),nil,sha256.New())
	
	//then
	c.Assert(test, DeepEquals, []byte{27, 55, 33, 99, 20, 191, 202, 69, 84, 176, 250, 108, 99, 7, 91, 49, 200, 47, 219, 142, 190, 216, 197, 154, 235, 17, 76, 12, 165, 75, 201, 108})
}

//test utils
func datalenFormat(str string) []byte {
	bytes,_:=base64url.Decode(str)
	return prependDatalen(bytes)
}

func prependDatalen(bytes []byte) []byte {
	return arrays.Concat(arrays.UInt32ToBytes(uint32(len(bytes))),bytes)
}