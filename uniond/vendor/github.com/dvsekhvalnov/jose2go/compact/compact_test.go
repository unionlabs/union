package compact

import (
	"fmt"
	. "gopkg.in/check.v1"
	"testing"
)

func Test(t *testing.T) { TestingT(t) }

type TestSuite struct{}

var _ = Suite(&TestSuite{})

func (s *TestSuite) TestParseThreeParts(c *C) {
	//when
	test, err := Parse("eyJhbGciOiJIUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.chIoYWrQMA8XL5nFz6oLDJyvgHk2KA4BrFGrKymjC8E")

	//then
	c.Assert(err, IsNil)
	c.Assert(test, HasLen, 3)
	c.Assert(test[0], DeepEquals, []byte{123, 34, 97, 108, 103, 34, 58, 34, 72, 83, 50, 53, 54, 34, 44, 34, 99, 116, 121, 34, 58, 34, 116, 101, 120, 116, 92, 47, 112, 108, 97, 105, 110, 34, 125})
	c.Assert(test[1], DeepEquals, []byte{123, 34, 104, 101, 108, 108, 111, 34, 58, 32, 34, 119, 111, 114, 108, 100, 34, 125})
	c.Assert(test[2], DeepEquals, []byte{114, 18, 40, 97, 106, 208, 48, 15, 23, 47, 153, 197, 207, 170, 11, 12, 156, 175, 128, 121, 54, 40, 14, 1, 172, 81, 171, 43, 41, 163, 11, 193})
}

func (s *TestSuite) TestParseEmptyTrailingPart(c *C) {
	//when
	test, err := Parse("eyJhbGciOiJub25lIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.")

	//then
	c.Assert(err, IsNil)
	c.Assert(test, HasLen, 3)
	c.Assert(test[0], DeepEquals, []byte{123, 34, 97, 108, 103, 34, 58, 34, 110, 111, 110, 101, 34, 125})
	c.Assert(test[1], DeepEquals, []byte{123, 34, 104, 101, 108, 108, 111, 34, 58, 32, 34, 119, 111, 114, 108, 100, 34, 125})
	c.Assert(test[2], DeepEquals, []byte{})
}

func (s *TestSuite) TestParseFiveParts(c *C) {
	//when
	test, err := Parse("eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMTI4R0NNIn0.FojyyzygtFOyNBjzqTRfr9HVHPrvtqbVUt9sXSuU59ZhLlzk7FrirryFnFGtj8YC9lx-IX156Ro9rBaJTCU_dfERd05DhPMffT40rdcDiLxfCLOY0E2PfsMyGQPhI6YtNBtf_sQjXWEBC59zH_VoswFAUstkvXY9eVVecoM-W9HFlIxwUXMVpEPtS96xZX5LMksDgJ9sYDTNa6EQOA0hfzw07fD_FFJShcueqJuoJjILYbad-AHbpnLTV4oTbFTYjskRxpEYQr9plFZsT4_xKiCU89slT9EFhmuaiUI_-NGdX-kNDyQZj2Vtid4LSOVv5kGxyygThuQb6wjr1AGe1g.O92pf8iqwlBIQmXA.YdGjkN7lzeKYIv743XlPRYTd3x4VA0xwa5WVoGf1hiHlhQuXGEg4Jv3elk4JoFJzgVuMMQMex8fpFFL3t5I4H9bH18pbrEo7wLXvGOsP971cuOOaXPxhX6qClkwx5qkWhcTbO_2AuJxzIaU9qBwtwWaxJm9axofAPYgYbdaMZkU4F5sFdaFY8IOe94wUA1Ocn_gxC_DYp9IEAyZut0j5RImmthPgiRO_0pK9OvusE_Xg3iGfdxu70x0KpoItuNwlEf0LUA.uP5jOGMxtDUiT6E3ubucBw")

	//then
	c.Assert(err, IsNil)
	c.Assert(test, HasLen, 5)
	c.Assert(test[0], DeepEquals, []byte{123, 34, 97, 108, 103, 34, 58, 34, 82, 83, 65, 49, 95, 53, 34, 44, 34, 101, 110, 99, 34, 58, 34, 65, 49, 50, 56, 71, 67, 77, 34, 125})
	c.Assert(test[1], DeepEquals, []byte{22, 136, 242, 203, 60, 160, 180, 83, 178, 52, 24, 243, 169, 52, 95, 175, 209, 213, 28, 250, 239, 182, 166, 213, 82, 223, 108, 93, 43, 148, 231, 214, 97, 46, 92, 228, 236, 90, 226, 174, 188, 133, 156, 81, 173, 143, 198, 2, 246, 92, 126, 33, 125, 121, 233, 26, 61, 172, 22, 137, 76, 37, 63, 117, 241, 17, 119, 78, 67, 132, 243, 31, 125, 62, 52, 173, 215, 3, 136, 188, 95, 8, 179, 152, 208, 77, 143, 126, 195, 50, 25, 3, 225, 35, 166, 45, 52, 27, 95, 254, 196, 35, 93, 97, 1, 11, 159, 115, 31, 245, 104, 179, 1, 64, 82, 203, 100, 189, 118, 61, 121, 85, 94, 114, 131, 62, 91, 209, 197, 148, 140, 112, 81, 115, 21, 164, 67, 237, 75, 222, 177, 101, 126, 75, 50, 75, 3, 128, 159, 108, 96, 52, 205, 107, 161, 16, 56, 13, 33, 127, 60, 52, 237, 240, 255, 20, 82, 82, 133, 203, 158, 168, 155, 168, 38, 50, 11, 97, 182, 157, 248, 1, 219, 166, 114, 211, 87, 138, 19, 108, 84, 216, 142, 201, 17, 198, 145, 24, 66, 191, 105, 148, 86, 108, 79, 143, 241, 42, 32, 148, 243, 219, 37, 79, 209, 5, 134, 107, 154, 137, 66, 63, 248, 209, 157, 95, 233, 13, 15, 36, 25, 143, 101, 109, 137, 222, 11, 72, 229, 111, 230, 65, 177, 203, 40, 19, 134, 228, 27, 235, 8, 235, 212, 1, 158, 214})
	c.Assert(test[2], DeepEquals, []byte{59, 221, 169, 127, 200, 170, 194, 80, 72, 66, 101, 192})
	c.Assert(test[3], DeepEquals, []byte{97, 209, 163, 144, 222, 229, 205, 226, 152, 34, 254, 248, 221, 121, 79, 69, 132, 221, 223, 30, 21, 3, 76, 112, 107, 149, 149, 160, 103, 245, 134, 33, 229, 133, 11, 151, 24, 72, 56, 38, 253, 222, 150, 78, 9, 160, 82, 115, 129, 91, 140, 49, 3, 30, 199, 199, 233, 20, 82, 247, 183, 146, 56, 31, 214, 199, 215, 202, 91, 172, 74, 59, 192, 181, 239, 24, 235, 15, 247, 189, 92, 184, 227, 154, 92, 252, 97, 95, 170, 130, 150, 76, 49, 230, 169, 22, 133, 196, 219, 59, 253, 128, 184, 156, 115, 33, 165, 61, 168, 28, 45, 193, 102, 177, 38, 111, 90, 198, 135, 192, 61, 136, 24, 109, 214, 140, 102, 69, 56, 23, 155, 5, 117, 161, 88, 240, 131, 158, 247, 140, 20, 3, 83, 156, 159, 248, 49, 11, 240, 216, 167, 210, 4, 3, 38, 110, 183, 72, 249, 68, 137, 166, 182, 19, 224, 137, 19, 191, 210, 146, 189, 58, 251, 172, 19, 245, 224, 222, 33, 159, 119, 27, 187, 211, 29, 10, 166, 130, 45, 184, 220, 37, 17, 253, 11, 80})
	c.Assert(test[4], DeepEquals, []byte{184, 254, 99, 56, 99, 49, 180, 53, 34, 79, 161, 55, 185, 187, 156, 7})
}

func (s *TestSuite) TestParseInvalidBase64Encoded(c *C) {
	//when
	test, err := Parse("eyJhbGciOiJub25lIn0.eyJo@#xsbyI6ICJ3b3JsZCJ9.")

	//then
	c.Assert(err, NotNil)
	fmt.Printf("\nerr=%v\n", err)
	c.Assert(test, IsNil)
}

func (s *TestSuite) TestParseEmptyMiddlePart(c *C) {
	//when
	test, err := Parse("eyJhbGciOiJkaXIiLCJlbmMiOiJBMTI4R0NNIn0..yVi-LdQQngN0C5WS.1McwSmhZzAtmmLp9y-OdnJwaJFo1nj_4ashmzl2LhubGf0Jl1OTEVJzsHZb7bkup7cGTkuxh6Vfv10ljHsjWf_URXoxP3stQqQeViVcuPV0y2Q_WHYzTNGZpmHGe-hM6gjDhyZyvu3yeXGFSvfPQmp9pWVOgDjI4RC0MQ83rzzn-rRdnZkznWjbmOPxwPrR72Qng0BISsEwbkPn4oO8-vlHkVmPpuDTaYzCT2ZR5K9JnIU8d8QdxEAGb7-s8GEJ1yqtd_w._umbK59DAKA3O89h15VoKQ")

	//then
	c.Assert(err, IsNil)
	c.Assert(test, HasLen, 5)
	c.Assert(test[0], DeepEquals, []byte{123, 34, 97, 108, 103, 34, 58, 34, 100, 105, 114, 34, 44, 34, 101, 110, 99, 34, 58, 34, 65, 49, 50, 56, 71, 67, 77, 34, 125})
	c.Assert(test[1], DeepEquals, []byte{})
	c.Assert(test[2], DeepEquals, []byte{201, 88, 190, 45, 212, 16, 158, 3, 116, 11, 149, 146})
	c.Assert(test[3], DeepEquals, []byte{212, 199, 48, 74, 104, 89, 204, 11, 102, 152, 186, 125, 203, 227, 157, 156, 156, 26, 36, 90, 53, 158, 63, 248, 106, 200, 102, 206, 93, 139, 134, 230, 198, 127, 66, 101, 212, 228, 196, 84, 156, 236, 29, 150, 251, 110, 75, 169, 237, 193, 147, 146, 236, 97, 233, 87, 239, 215, 73, 99, 30, 200, 214, 127, 245, 17, 94, 140, 79, 222, 203, 80, 169, 7, 149, 137, 87, 46, 61, 93, 50, 217, 15, 214, 29, 140, 211, 52, 102, 105, 152, 113, 158, 250, 19, 58, 130, 48, 225, 201, 156, 175, 187, 124, 158, 92, 97, 82, 189, 243, 208, 154, 159, 105, 89, 83, 160, 14, 50, 56, 68, 45, 12, 67, 205, 235, 207, 57, 254, 173, 23, 103, 102, 76, 231, 90, 54, 230, 56, 252, 112, 62, 180, 123, 217, 9, 224, 208, 18, 18, 176, 76, 27, 144, 249, 248, 160, 239, 62, 190, 81, 228, 86, 99, 233, 184, 52, 218, 99, 48, 147, 217, 148, 121, 43, 210, 103, 33, 79, 29, 241, 7, 113, 16, 1, 155, 239, 235, 60, 24, 66, 117, 202, 171, 93, 255})
	c.Assert(test[4], DeepEquals, []byte{254, 233, 155, 43, 159, 67, 0, 160, 55, 59, 207, 97, 215, 149, 104, 41})
}

func (s *TestSuite) TestSerialize(c *C) {
	//when
	test := Serialize([]byte{123, 34, 97, 108, 103, 34, 58, 34, 72, 83, 50, 53, 54, 34, 44, 34, 99, 116, 121, 34, 58, 34, 116, 101, 120, 116, 92, 47, 112, 108, 97, 105, 110, 34, 125},
		[]byte{123, 34, 104, 101, 108, 108, 111, 34, 58, 32, 34, 119, 111, 114, 108, 100, 34, 125},
		[]byte{114, 18, 40, 97, 106, 208, 48, 15, 23, 47, 153, 197, 207, 170, 11, 12, 156, 175, 128, 121, 54, 40, 14, 1, 172, 81, 171, 43, 41, 163, 11, 193})

	//then
	c.Assert(test, Equals, "eyJhbGciOiJIUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.chIoYWrQMA8XL5nFz6oLDJyvgHk2KA4BrFGrKymjC8E")
}

func (s *TestSuite) TestSerializeEmptyTrailingPart(c *C) {
	//when
	test := Serialize([]byte{123, 34, 97, 108, 103, 34, 58, 34, 72, 83, 50, 53, 54, 34, 44, 34, 99, 116, 121, 34, 58, 34, 116, 101, 120, 116, 92, 47, 112, 108, 97, 105, 110, 34, 125},
		[]byte{123, 34, 104, 101, 108, 108, 111, 34, 58, 32, 34, 119, 111, 114, 108, 100, 34, 125},
		[]byte{})

	//then
	c.Assert(test, Equals, "eyJhbGciOiJIUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.")
}

func (s *TestSuite) TestSerializeEmptyMiddlePart(c *C) {
	//when
	test := Serialize([]byte{123, 34, 97, 108, 103, 34, 58, 34, 72, 83, 50, 53, 54, 34, 44, 34, 99, 116, 121, 34, 58, 34, 116, 101, 120, 116, 92, 47, 112, 108, 97, 105, 110, 34, 125},
		[]byte{},
		[]byte{114, 18, 40, 97, 106, 208, 48, 15, 23, 47, 153, 197, 207, 170, 11, 12, 156, 175, 128, 121, 54, 40, 14, 1, 172, 81, 171, 43, 41, 163, 11, 193})

	//then
	c.Assert(test, Equals, "eyJhbGciOiJIUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0..chIoYWrQMA8XL5nFz6oLDJyvgHk2KA4BrFGrKymjC8E")
}
