package aes

import (
	"testing"	
	// "fmt"
	. "gopkg.in/check.v1"
	// "github.com/dvsekhvalnov/jose2go/arrays"
)

func Test(t *testing.T) { TestingT(t) }
type TestSuite struct{}
var _ = Suite(&TestSuite{})

func (s *TestSuite) TestWrap_128Key_128Kek(c *C) {
	//given (Section 4.1)

    //000102030405060708090A0B0C0D0E0F
    kek := []byte{0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15}

    //00112233445566778899AABBCCDDEEFF
    key := []byte{0,17,34,51,68,85,102,119,136,153,170,187,204,221,238,255}

    //1FA68B0A8112B447AEF34BD8FB5A7B829D3E862371D2CFE5
    expected := []byte{ 31, 166, 139, 10, 129, 18, 180, 71, 174, 243, 75, 216, 251, 90, 123, 130, 157, 62, 134,35, 113, 210, 207, 229}

    //when
    test,_ := KeyWrap(key, kek);

    //then
    c.Assert(test, DeepEquals, expected)
}

func (s *TestSuite) Test_Unwrap128Key_128Kek(c *C) {
    //given (Section 4.1)

    //000102030405060708090A0B0C0D0E0F
    kek := []byte{ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 }

    //00112233445566778899AABBCCDDEEFF
    expected := []byte{ 0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255 }

    //1FA68B0A8112B447AEF34BD8FB5A7B829D3E862371D2CFE5
    key := []byte{ 31, 166, 139, 10, 129, 18, 180, 71, 174, 243, 75, 216, 251, 90, 123, 130, 157, 62, 134, 35, 113, 210, 207, 229 }

    //when
    test,_ := KeyUnwrap(key, kek)

    //then
    c.Assert(test, DeepEquals, expected)
}

func (s *TestSuite) Test_Wrap_128Key_192Kek(c *C) {
	//given (Section 4.2)

	//000102030405060708090A0B0C0D0E0F1011121314151617
    kek := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23}

	//00112233445566778899AABBCCDDEEFF
    key := []byte{0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255}

	//96778B25AE6CA435F92B5B97C050AED2468AB8A17AD84E5D
    expected := []byte{150, 119, 139, 37, 174, 108, 164, 53, 249, 43, 91, 151, 192, 80, 174, 210, 70, 138, 184, 161, 122, 216, 78, 93}

    //when
    test,_ := KeyWrap(key, kek);

    //then
    c.Assert(test, DeepEquals, expected)
}

func (s *TestSuite) Test_Unwrap128Key_192Kek(c *C) {
    //given (Section 4.2)

    //000102030405060708090A0B0C0D0E0F1011121314151617
    kek := []byte{ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23 }

    //00112233445566778899AABBCCDDEEFF
    expected := []byte{ 0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255 }

    //96778B25AE6CA435F92B5B97C050AED2468AB8A17AD84E5D
    key := []byte{ 150, 119, 139, 37, 174, 108, 164, 53, 249, 43, 91, 151, 192, 80, 174, 210, 70, 138, 184, 161, 122, 216, 78, 93 }

    //when
    test,_ := KeyUnwrap(key, kek)

    //then
    c.Assert(test, DeepEquals, expected)
}

func (s *TestSuite) Test_Wrap_128Key_256Kek(c *C) {
	//given (Section 4.3)

	//000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F
    kek := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31}

	//00112233445566778899AABBCCDDEEFF
    key := []byte{0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255}

	//64E8C3F9CE0F5BA263E9777905818A2A93C8191E7D6E8AE7
    expected := []byte{100, 232, 195, 249, 206, 15, 91, 162, 99, 233, 119, 121, 5, 129, 138, 42, 147, 200, 25, 30, 125, 110, 138, 231}

    //when
    test,_ := KeyWrap(key, kek);

    //then
    c.Assert(test, DeepEquals, expected)
}

func (s *TestSuite) Test_Unwrap128Key_256Kek(c *C) {
    //given (Section 4.3)

    //000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F
    kek := []byte{ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31 }

    //00112233445566778899AABBCCDDEEFF
    expected := []byte{ 0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255 }

    //64E8C3F9CE0F5BA263E9777905818A2A93C8191E7D6E8AE7
    key := []byte{ 100, 232, 195, 249, 206, 15, 91, 162, 99, 233, 119, 121, 5, 129, 138, 42, 147, 200, 25, 30, 125, 110, 138, 231 }

    //when
    test,_ := KeyUnwrap(key, kek)

    //then
    c.Assert(test, DeepEquals, expected)
}

func (s *TestSuite) Test_Wrap_192Key_192Kek(c *C) {
	//given (Section 4.4)

	//000102030405060708090A0B0C0D0E0F1011121314151617
    kek := []byte{ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23 }

	//00112233445566778899AABBCCDDEEFF0001020304050607
    key := []byte{ 0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255, 0, 1, 2, 3, 4, 5, 6, 7 }

	//031D33264E15D33268F24EC260743EDCE1C6C7DDEE725A936BA814915C6762D2
    expected := []byte{ 3, 29, 51, 38, 78, 21, 211, 50, 104, 242, 78, 194, 96, 116, 62, 220, 225, 198, 199, 221, 238, 114, 90, 147, 107, 168, 20, 145, 92, 103, 98, 210 }

    //when
    test,_ := KeyWrap(key, kek);

    //then
    c.Assert(test, DeepEquals, expected)
}

func (s *TestSuite) Test_Unwrap192Key_192Kek(c *C) {
    //given (Section 4.4)

    //000102030405060708090A0B0C0D0E0F1011121314151617
    kek := []byte{ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23 }

    //00112233445566778899AABBCCDDEEFF0001020304050607
    expected := []byte{ 0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255, 0, 1, 2, 3, 4, 5, 6, 7 }

    //031D33264E15D33268F24EC260743EDCE1C6C7DDEE725A936BA814915C6762D2
    key := []byte{ 3, 29, 51, 38, 78, 21, 211, 50, 104, 242, 78, 194, 96, 116, 62, 220, 225, 198, 199, 221, 238, 114, 90, 147, 107, 168, 20, 145, 92, 103, 98, 210 }

    //when
    test,_ := KeyUnwrap(key, kek)

    //then
    c.Assert(test, DeepEquals, expected)
}

func (s *TestSuite) Test_Wrap_192Key_256Kek(c *C) {
	//given (Section 4.5)

	//000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F
    kek := []byte{ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31 }

	//00112233445566778899AABBCCDDEEFF0001020304050607
    key := []byte{ 0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255, 0, 1, 2, 3, 4, 5, 6, 7 }

	//A8F9BC1612C68B3FF6E6F4FBE30E71E4769C8B80A32CB8958CD5D17D6B254DA1
    expected := []byte{ 168, 249, 188, 22, 18, 198, 139, 63, 246, 230, 244, 251, 227, 14, 113, 228, 118, 156, 139, 128, 163, 44, 184, 149, 140, 213, 209, 125, 107, 37, 77, 161 }

    //when
    test,_ := KeyWrap(key, kek);

    //then
    c.Assert(test, DeepEquals, expected)
}

func (s *TestSuite) Test_Unwrap192Key_256Kek(c *C) {
    //given (Section 4.5)

    //000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F
    kek := []byte{ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31 }

    //00112233445566778899AABBCCDDEEFF0001020304050607
    expected := []byte{ 0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255, 0, 1, 2, 3, 4, 5, 6, 7 }

    //A8F9BC1612C68B3FF6E6F4FBE30E71E4769C8B80A32CB8958CD5D17D6B254DA1
    key := []byte{ 168, 249, 188, 22, 18, 198, 139, 63, 246, 230, 244, 251, 227, 14, 113, 228, 118, 156, 139, 128, 163, 44, 184, 149, 140, 213, 209, 125, 107, 37, 77, 161 }

    //when
    test,_ := KeyUnwrap(key, kek)

    //then
    c.Assert(test, DeepEquals, expected)
}

func (s *TestSuite) Test_Wrap_256Key_256Kek(c *C) {
	//given (Section 4.6)

	//000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F
    kek := []byte{ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31 }

	//00112233445566778899AABBCCDDEEFF000102030405060708090A0B0C0D0E0F
    key := []byte{ 0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 }

	//28C9F404C4B810F4CBCCB35CFB87F8263F5786E2D80ED326CBC7F0E71A99F43BFB988B9B7A02DD21
    expected := []byte{ 40, 201, 244, 4, 196, 184, 16, 244, 203, 204, 179, 92, 251, 135, 248, 38, 63, 87, 134, 226, 216, 14, 211, 38, 203, 199, 240, 231, 26, 153, 244, 59, 251, 152, 139, 155, 122, 2, 221, 33 }

    //when
    test,_ := KeyWrap(key, kek);

    //then
    c.Assert(test, DeepEquals, expected)
}

func (s *TestSuite) Test_Unwrap256Key_256Kek(c *C) {
    //given (Section 4.6)

    //000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F
    kek := []byte{ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31 }

    //00112233445566778899AABBCCDDEEFF000102030405060708090A0B0C0D0E0F
    expected := []byte{ 0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 }

    //28C9F404C4B810F4CBCCB35CFB87F8263F5786E2D80ED326CBC7F0E71A99F43BFB988B9B7A02DD21
    key := []byte{ 40, 201, 244, 4, 196, 184, 16, 244, 203, 204, 179, 92, 251, 135, 248, 38, 63, 87, 134, 226, 216, 14, 211, 38, 203, 199, 240, 231, 26, 153, 244, 59, 251, 152, 139, 155, 122, 2, 221, 33 }

    //when
    test,_ := KeyUnwrap(key, kek)

    //then
    c.Assert(test, DeepEquals, expected)
}