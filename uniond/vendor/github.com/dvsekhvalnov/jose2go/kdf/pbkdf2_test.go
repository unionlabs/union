package kdf

import (
	"crypto/sha256"
	"crypto/sha512"
	. "gopkg.in/check.v1"
	"testing"
)

func Test(t *testing.T) { TestingT(t) }

type TestSuite struct{}

var _ = Suite(&TestSuite{})

var password = []byte("password")
var salt = []byte("salt")

func (s *TestSuite) TestDerivePbkdf2Sha256Count1(c *C) {
	//when
	test := DerivePBKDF2(password, salt, 1, 256, sha256.New)

	//then
	c.Assert(test, DeepEquals, []byte{18, 15, 182, 207, 252, 248, 179, 44, 67, 231, 34, 82, 86, 196, 248, 55, 168, 101, 72, 201, 44, 204, 53, 72, 8, 5, 152, 124, 183, 11, 225, 123})
}

func (s *TestSuite) TestDerivePbkdf2Sha256Count2(c *C) {
	//when
	test := DerivePBKDF2(password, salt, 2, 256, sha256.New)

	//then
	c.Assert(test, DeepEquals, []byte{174, 77, 12, 149, 175, 107, 70, 211, 45, 10, 223, 249, 40, 240, 109, 208, 42, 48, 63, 142, 243, 194, 81, 223, 214, 226, 216, 90, 149, 71, 76, 67})
}

func (s *TestSuite) TestDerivePbkdf2Sha256Count4096(c *C) {
	//when
	test := DerivePBKDF2(password, salt, 4096, 256, sha256.New)

	//then
	c.Assert(test, DeepEquals, []byte{197, 228, 120, 213, 146, 136, 200, 65, 170, 83, 13, 182, 132, 92, 76, 141, 150, 40, 147, 160, 1, 206, 78, 17, 164, 150, 56, 115, 170, 152, 19, 74})
}

func (s *TestSuite) TestDerivePbkdf2Sha256Count4096Len320(c *C) {
	//when
	test := DerivePBKDF2([]byte("passwordPASSWORDpassword"), []byte("saltSALTsaltSALTsaltSALTsaltSALTsalt"), 4096, 320, sha256.New)

	//then
	c.Assert(test, DeepEquals, []byte{52, 140, 137, 219, 203, 211, 43, 47, 50, 216, 20, 184, 17, 110, 132, 207, 43, 23, 52, 126, 188, 24, 0, 24, 28, 78, 42, 31, 184, 221, 83, 225, 198, 53, 81, 140, 125, 172, 71, 233})
}

func (s *TestSuite) TestDerivePbkdf2Sha384Count1Len384(c *C) {
	//when
	test := DerivePBKDF2(password, salt, 1, 384, sha512.New384)

	//then
	c.Assert(test, DeepEquals, []byte{192, 225, 79, 6, 228, 158, 50, 215, 63, 159, 82, 221, 241, 208, 197, 199, 25, 22, 9, 35, 54, 49, 218, 221, 118, 165, 103, 219, 66, 183, 134, 118, 179, 143, 200, 0, 204, 83, 221, 182, 66, 245, 199, 68, 66, 230, 43, 228})
}

func (s *TestSuite) TestDerivePbkdf2Sha384Count2Len384(c *C) {
	//when
	test := DerivePBKDF2(password, salt, 2, 384, sha512.New384)

	//then
	c.Assert(test, DeepEquals, []byte{84, 247, 117, 198, 215, 144, 242, 25, 48, 69, 145, 98, 252, 83, 93, 191, 4, 169, 57, 24, 81, 39, 1, 106, 4, 23, 106, 7, 48, 198, 241, 244, 251, 72, 131, 42, 209, 38, 27, 170, 221, 44, 237, 213, 8, 20, 177, 200})
}

func (s *TestSuite) TestDerivePbkdf2Sha384Count4096Len384(c *C) {
	//when
	test := DerivePBKDF2(password, salt, 4096, 384, sha512.New384)

	//then
	c.Assert(test, DeepEquals, []byte{85, 151, 38, 190, 56, 219, 18, 91, 200, 94, 215, 137, 95, 110, 60, 245, 116, 199, 160, 28, 8, 12, 52, 71, 219, 30, 138, 118, 118, 77, 235, 60, 48, 123, 148, 133, 63, 190, 66, 79, 100, 136, 197, 244, 241, 40, 150, 38})
}

func (s *TestSuite) TestDerivePbkdf2Sha384Count4096Len768(c *C) {
	//when
	test := DerivePBKDF2([]byte("passwordPASSWORDpassword"), []byte("saltSALTsaltSALTsaltSALTsaltSALTsalt"), 4096, 768, sha512.New384)

	//then
	c.Assert(test, DeepEquals, []byte{129, 145, 67, 173, 102, 223, 154, 85, 37, 89, 185, 225, 49, 197, 42, 230, 197, 193, 176, 238, 209, 143, 77, 40, 59, 140, 92, 158, 174, 185, 43, 57, 44, 20, 124, 194, 210, 134, 157, 88, 255, 226, 247, 218, 19, 209, 95, 141, 146, 87, 33, 240, 237, 26, 250, 250, 36, 72, 13, 85, 207, 96, 96, 177, 127, 17, 42, 61, 231, 76, 174, 37, 253, 243, 86, 158, 36, 127, 41, 228, 219, 184, 68, 33, 132, 120, 34, 234, 153, 189, 32, 40, 60, 58, 37, 166})
}

func (s *TestSuite) TestDerivePbkdf2Sha512Count1Len512(c *C) {
	//when
	test := DerivePBKDF2(password, salt, 1, 512, sha512.New)

	//then
	c.Assert(test, DeepEquals, []byte{134, 127, 112, 207, 26, 222, 2, 207, 243, 117, 37, 153, 163, 165, 61, 196, 175, 52, 199, 166, 105, 129, 90, 229, 213, 19, 85, 78, 28, 140, 242, 82, 192, 45, 71, 10, 40, 90, 5, 1, 186, 217, 153, 191, 233, 67, 192, 143, 5, 2, 53, 215, 214, 139, 29, 165, 94, 99, 247, 59, 96, 165, 127, 206})
}

func (s *TestSuite) TestDerivePbkdf2Sha512Count2Len512(c *C) {
	//when
	test := DerivePBKDF2(password, salt, 2, 512, sha512.New)

	//then
	c.Assert(test, DeepEquals, []byte{225, 217, 193, 106, 166, 129, 112, 138, 69, 245, 199, 196, 226, 21, 206, 182, 110, 1, 26, 46, 159, 0, 64, 113, 63, 24, 174, 253, 184, 102, 213, 60, 247, 108, 171, 40, 104, 163, 155, 159, 120, 64, 237, 206, 79, 239, 90, 130, 190, 103, 51, 92, 119, 166, 6, 142, 4, 17, 39, 84, 242, 124, 207, 78})
}

func (s *TestSuite) TestDerivePbkdf2Sha512Count4096Len512(c *C) {
	//when
	test := DerivePBKDF2(password, salt, 4096, 512, sha512.New)

	//then
	c.Assert(test, DeepEquals, []byte{209, 151, 177, 179, 61, 176, 20, 62, 1, 139, 18, 243, 209, 209, 71, 158, 108, 222, 189, 204, 151, 197, 192, 248, 127, 105, 2, 224, 114, 244, 87, 181, 20, 63, 48, 96, 38, 65, 179, 213, 92, 211, 53, 152, 140, 179, 107, 132, 55, 96, 96, 236, 213, 50, 224, 57, 183, 66, 162, 57, 67, 74, 242, 213})
}

func (s *TestSuite) TestDerivePbkdf2Sha256Count4096Len1024(c *C) {
	//when
	test := DerivePBKDF2([]byte("passwordPASSWORDpassword"), []byte("saltSALTsaltSALTsaltSALTsaltSALTsalt"), 4096, 1024, sha512.New)

	//then
	c.Assert(test, DeepEquals, []byte{140, 5, 17, 244, 198, 229, 151, 198, 172, 99, 21, 216, 240, 54, 46, 34, 95, 60, 80, 20, 149, 186, 35, 184, 104, 192, 5, 23, 77, 196, 238, 113, 17, 91, 89, 249, 230, 12, 217, 83, 47, 163, 62, 15, 117, 174, 254, 48, 34, 92, 88, 58, 24, 108, 216, 43, 212, 218, 234, 151, 36, 163, 211, 184, 4, 247, 91, 221, 65, 73, 79, 163, 36, 202, 178, 75, 204, 104, 15, 179, 185, 106, 48, 207, 93, 33, 250, 195, 194, 135, 89, 19, 145, 159, 51, 153, 177, 217, 206, 126, 181, 76, 149, 186, 73, 17, 133, 150, 207, 116, 101, 113, 155, 190, 2, 196, 236, 171, 27, 21, 65, 41, 140, 50, 29, 19, 198, 246})
}
