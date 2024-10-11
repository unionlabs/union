package jose

import (
	"crypto/ecdsa"
	"crypto/rsa"
	"errors"
	"fmt"
	"strings"
	"testing"

	"github.com/dvsekhvalnov/jose2go/base64url"
	"github.com/dvsekhvalnov/jose2go/keys/ecc"
	Rsa "github.com/dvsekhvalnov/jose2go/keys/rsa"

	. "gopkg.in/check.v1"
)

func Test(t *testing.T) { TestingT(t) }

type TestSuite struct{}

var _ = Suite(&TestSuite{})

var shaKey = []byte{97, 48, 97, 50, 97, 98, 100, 56, 45, 54, 49, 54, 50, 45, 52, 49, 99, 51, 45, 56, 51, 100, 54, 45, 49, 99, 102, 53, 53, 57, 98, 52, 54, 97, 102, 99}
var aes128Key = []byte{194, 164, 235, 6, 138, 248, 171, 239, 24, 216, 11, 22, 137, 199, 215, 133}
var aes192Key = []byte{139, 156, 136, 148, 17, 147, 27, 233, 145, 80, 115, 197, 223, 11, 100, 221, 5, 50, 155, 226, 136, 222, 216, 14}
var aes256Key = []byte{164, 60, 194, 0, 161, 189, 41, 38, 130, 89, 141, 164, 45, 170, 159, 209, 69, 137, 243, 216, 191, 131, 47, 250, 32, 107, 231, 117, 37, 158, 225, 234}
var aes384Key = []byte{185, 30, 233, 199, 32, 98, 209, 3, 114, 250, 30, 124, 207, 173, 227, 152, 243, 202, 238, 165, 227, 199, 202, 230, 218, 185, 216, 113, 13, 53, 40, 100, 100, 20, 59, 67, 88, 97, 191, 3, 161, 37, 147, 223, 149, 237, 190, 156}
var aes512Key = []byte{238, 71, 183, 66, 57, 207, 194, 93, 82, 80, 80, 152, 92, 242, 84, 206, 194, 46, 67, 43, 231, 118, 208, 168, 156, 212, 33, 105, 27, 45, 60, 160, 232, 63, 61, 235, 68, 171, 206, 35, 152, 11, 142, 121, 174, 165, 140, 11, 172, 212, 13, 101, 13, 190, 82, 244, 109, 113, 70, 150, 251, 82, 215, 226}

var pubKey = `-----BEGIN CERTIFICATE-----
MIICnTCCAYUCBEReYeAwDQYJKoZIhvcNAQEFBQAwEzERMA8GA1UEAxMIand0LTIw
NDgwHhcNMTQwMTI0MTMwOTE2WhcNMzQwMjIzMjAwMDAwWjATMREwDwYDVQQDEwhq
d3QtMjA0ODCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAKhWb9KXmv45
+TKOKhFJkrboZbpbKPJ9Yp12xKLXf8060KfStEStIX+7dCuAYylYWoqiGpuLVVUL
5JmHgXmK9TJpzv9Dfe3TAc/+35r8r9IYB2gXUOZkebty05R6PLY0RO/hs2ZhrOoz
HMo+x216Gwz0CWaajcuiY5Yg1V8VvJ1iQ3rcRgZapk49RNX69kQrGS63gzj0gyHn
Rtbqc/Ua2kobCA83nnznCom3AGinnlSN65AFPP5jmri0l79+4ZZNIerErSW96mUF
8jlJFZI1yJIbzbv73tL+y4i0+BvzsWBs6TkHAp4pinaI8zT+hrVQ2jD4fkJEiRN9
lAqLPUd8CNkCAwEAATANBgkqhkiG9w0BAQUFAAOCAQEAnqBw3UHOSSHtU7yMi1+H
E+9119tMh7X/fCpcpOnjYmhW8uy9SiPBZBl1z6vQYkMPcURnDMGHdA31kPKICZ6G
LWGkBLY3BfIQi064e8vWHW7zX6+2Wi1zFWdJlmgQzBhbr8pYh9xjZe6FjPwbSEuS
0uE8dWSWHJLdWsA4xNX9k3pr601R2vPVFCDKs3K1a8P/Xi59kYmKMjaX6vYT879y
gWt43yhtGTF48y85+eqLdFRFANTbBFSzdRlPQUYa5d9PZGxeBTcg7UBkK/G+d6D5
sd78T2ymwlLYrNi+cSDYD6S4hwZaLeEK6h7p/OoG02RBNuT4VqFRu5DJ6Po+C6Jh
qQ==
-----END CERTIFICATE-----`

var privKey = `-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAqFZv0pea/jn5Mo4qEUmStuhlulso8n1inXbEotd/zTrQp9K0
RK0hf7t0K4BjKVhaiqIam4tVVQvkmYeBeYr1MmnO/0N97dMBz/7fmvyv0hgHaBdQ
5mR5u3LTlHo8tjRE7+GzZmGs6jMcyj7HbXobDPQJZpqNy6JjliDVXxW8nWJDetxG
BlqmTj1E1fr2RCsZLreDOPSDIedG1upz9RraShsIDzeefOcKibcAaKeeVI3rkAU8
/mOauLSXv37hlk0h6sStJb3qZQXyOUkVkjXIkhvNu/ve0v7LiLT4G/OxYGzpOQcC
nimKdojzNP6GtVDaMPh+QkSJE32UCos9R3wI2QIDAQABAoIBAQCUmHBvSkqUHaK/
IMU7q2FqOi0KWswDefEiJKQhRu9Wv5NOgW2FrfqDIXrDp7pg1dBezgeExHLX9v6d
FAOTwbj9/m6t3+r6k6fm7gp+ao3dfD6VgPd12L2oXQ0t5NVQ1UUBJ4/QUWps9h90
3AP4vK/COG1P+CAw4DDeZi9TlwF/Pr7e492GXcLBAUJODA6538ED2nYw8xQcbzbA
wr+w07UjRNimObtOfA0HCIpsx/6LkIqe6iGChisQNgt4yDd/fZ4GWOUIU1hqgK1P
6avVl7Q5Mk0PTi9t8ui1X4EEq6Uils45J5WkobuAnFkea/uKfs8Tn9bNrEoVWgdb
fBHq/8bNAoGBANKmjpE9e+L0RtxP+u4FN5YDoKE+i96VR7ru8H6yBKMcnD2uf5mV
RueEoL0FKHxlGBBo0dJWr1AIwpcPbTs3Dgx1/EQMZLg57QBZ7QcYETPiMwMvEM3k
Zf3G4YFYwUwIQXMYPt1ckr+RncRcq0GiKPDsvzzyNS+BBSmR5onAXd7bAoGBAMyT
6ggyqmiR/UwBn87em+GjbfX6YqxHHaQBdWwnnRX0JlGTNCxt6zLTgCIYxF4AA7eR
gfGTStwUJfAScjJirOe6Cpm1XDgxEQrT6oxAl17MR/ms/Z88WrT73G+4phVvDpVr
JcK+CCESnRI8xGLOLMkCc+5NpLajqWCOf1H2J8NbAoGAKTWmTGmf092AA1euOmRQ
5IsfIIxQ5qGDn+FgsRh4acSOGE8L7WrTrTU4EOJyciuA0qz+50xIDbs4/j5pWx1B
JVTrnhBin9vNLrVo9mtR6jmFS0ko226kOUpwEVLgtdQjobWLjtiuaMW+/Iw4gKWN
ptxZ6T1lBD8UWHaPiEFW2+MCgYAmfSWoyS96YQ0QwbV5TDRzrTXA84yg8PhIpOWc
pY9OVBLpghJs0XlQpK4UvCglr0cDwGJ8OsP4x+mjUzUc+aeiKURZSt/Ayqp0KQ6V
uIlCEpjwBnXpAYfnSQNeGZVVrwFFZ1VBYFNTNZdLmRcxp6yRXN7G1ODKY9w4CFc3
6mHsxQKBgQCxEA+KAmmXxL++x/XOElOscz3vFHC4HbpHpOb4nywpE9vunnHE2WY4
EEW9aZbF22jx0ESU2XJ1JlqffvfIEvHNb5tmBWn4HZEpPUHdaFNhb9WjkMuFaLzh
cydwnEftq+3G0X3KSxp4p7R7afcnpNNqfneYODgoXxTQ4Q7ZyKo72A==
-----END RSA PRIVATE KEY-----`

func (s *TestSuite) TestDecodePlaintext(c *C) {
	//given
	token := "eyJhbGciOiJub25lIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9."

	//when
	test, _, err := Decode(token, nil)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestDecodePlaintextWithSignature(c *C) {
	//given
	token := "eyJ0eXAiOiJKV1QiLCJhbGciOiJub25lIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.KmLWPfxC3JGopWImDgYg9IUpgAi8gwimviUfr6eJyFI"

	//when
	test, _, err := Decode(token, nil)

	//then
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecodePlaintextWithKey(c *C) {
	//given
	token := "eyJ0eXAiOiJKV1QiLCJhbGciOiJub25lIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9."

	//when
	test, _, err := Decode(token, shaKey)

	//then
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecodeHS256(c *C) {
	//given
	token := "eyJhbGciOiJIUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.chIoYWrQMA8XL5nFz6oLDJyvgHk2KA4BrFGrKymjC8E"

	//when
	test, _, err := Decode(token, shaKey)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestDecodeHS384(c *C) {
	//given
	token := "eyJhbGciOiJIUzM4NCIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.McDgk0h4mRdhPM0yDUtFG_omRUwwqVS2_679Yeivj-a7l6bHs_ahWiKl1KoX_hU_"

	//when
	test, _, err := Decode(token, shaKey)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestDecodeHS512(c *C) {
	//given
	token := "eyJhbGciOiJIUzUxMiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.9KirTNe8IRwFCBLjO8BZuXf3U2ZVagdsg7F9ZsvMwG3FuqY9W0vqwjzPOjLqPN-GkjPm6C3qWPnINhpr5bEDJQ"

	//when
	test, _, err := Decode(token, shaKey)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestEncodePlaintext(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, NONE, nil)

	fmt.Printf("\nnone = %v\n", test)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "eyJhbGciOiJub25lIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.")

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, nil)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncodeHS256(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, HS256, shaKey)

	fmt.Printf("\nHS256 = %v\n", test)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "eyJhbGciOiJIUzI1NiJ9.eyJoZWxsbyI6ICJ3b3JsZCJ9.UAuHfK1tObqUzQj2tdMoKjKq1SWoI17hAg17VJ2l8j0")

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, shaKey)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncodeHS384(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, HS384, shaKey)

	fmt.Printf("\nHS384 = %v\n", test)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "eyJhbGciOiJIUzM4NCJ9.eyJoZWxsbyI6ICJ3b3JsZCJ9.c92ouMmuQ1EmFN6IQmCEKAv5JZoVIeo_ehQl79OskFw3_y5GE_OLiUEfcRikturb")

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, shaKey)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncodeHS512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, HS512, shaKey)

	fmt.Printf("\nHS512 = %v\n", test)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "eyJhbGciOiJIUzUxMiJ9.eyJoZWxsbyI6ICJ3b3JsZCJ9.tfZJJbT-Nht5_JkM0vylscOSwiuf9-X7mcBdxTIB_fdHBC-LylKQGPWLqjzDWFnx29f7U_s9c4EjA3WUahib7w")

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, shaKey)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecodeRS256(c *C) {
	//given
	token := "eyJhbGciOiJSUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.NL_dfVpZkhNn4bZpCyMq5TmnXbT4yiyecuB6Kax_lV8Yq2dG8wLfea-T4UKnrjLOwxlbwLwuKzffWcnWv3LVAWfeBxhGTa0c4_0TX_wzLnsgLuU6s9M2GBkAIuSMHY6UTFumJlEeRBeiqZNrlqvmAzQ9ppJHfWWkW4stcgLCLMAZbTqvRSppC1SMxnvPXnZSWn_Fk_q3oGKWw6Nf0-j-aOhK0S0Lcr0PV69ZE4xBYM9PUS1MpMe2zF5J3Tqlc1VBcJ94fjDj1F7y8twmMT3H1PI9RozO-21R0SiXZ_a93fxhE_l_dj5drgOek7jUN9uBDjkXUwJPAyp9YPehrjyLdw"

	//when
	test, _, err := Decode(token, PubKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestDecodeRS384(c *C) {
	//given
	token := "eyJhbGciOiJSUzM4NCIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.cOPca7YEOxnXVdIi7cJqfgRMmDFPCrZG1M7WCJ23U57rAWvCTaQgEFdLjs7aeRAPY5Su_MVWV7YixcawKKYOGVG9eMmjdGiKHVoRcfjwVywGIb-nuD1IBzGesrQe7mFQrcWKtYD9FurjCY1WuI2FzGPp5YhW5Zf4TwmBvOKz6j2D1vOFfGsogzAyH4lqaMpkHpUAXddQxzu8rmFhZ54Rg4T-jMGVlsdrlAAlGA-fdRZ-V3F2PJjHQYUcyS6n1ULcy6ljEOgT5fY-_8DDLLpI8jAIdIhcHUAynuwvvnDr9bJ4xIy4olFRqcUQIHbcb5-WDeWul_cSGzTJdxDZsnDuvg"

	//when
	test, _, err := Decode(token, PubKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestDecodeRS512(c *C) {
	//given
	token := "eyJhbGciOiJSUzUxMiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.KP_mwCVRIxcF6ErdrzNcXZQDFGcL-Hlyocc4tIl3tJfzSfc7rz7qOLPjHpZ6UFH1ncd5TlpRc1B_pgvY-l0BNtx_s7n_QA55X4c1oeD8csrIoXQ6A6mtvdVGoSlGu2JnP6N2aqlDmlcefKqjl_Z-8nwDMGTMkDNhHKfHlIb2_Dliwxeq8LmNMREEdvNH2XVp_ffxBjiaKv2Eqbwc6I17241GCEmjDCvnagSgjX_5uu-da2H7TK2gtPJYUo8r9nzC7uzZJ5SB8suZH0COSofsP-9wvH0FESO40evCyEBylqg3bh9M9dIzeq8_bdTiC5kG93Fal44OEY8_Zm88wB_VjQ"

	//when
	test, _, err := Decode(token, PubKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestEncodeRS256(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, RS256, PrivKey())

	fmt.Printf("\nRS256 = %v\n", test)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "eyJhbGciOiJSUzI1NiJ9.eyJoZWxsbyI6ICJ3b3JsZCJ9.fLVmWBDvVKZZPKNAaGrSitQpRGLoku1JPDafaTJhQUjfvrgvu8_RgTO4WmwQVngbAdT0i57EXt28WZHpAdBEJK4wi6sVzRADxO3aSiAxZn_aY_QwH7B4CGeeCvdvjUBX-wNj-4S3F9xCygq2VGtl8PknzAJLtcPmG9V-2hEfyLeL8lcOgK6SGaWWoGC5PjvVW46ilYSbySCr2c3B86TPITcx29_g4juuAM_AtHUdGimOVX1eKyHKSB8_U2IXWc2ujMMgphc0eZNY0W2nl_MwEwq-SvcdyLGfWVuEhgmfqlRHXu5eztHZsw1ejki0J-9r9reG1FbV6GzTMsU9HARW_Q")

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PubKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncodeRS384(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, RS384, PrivKey())

	fmt.Printf("\nRS384 = %v\n", test)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "eyJhbGciOiJSUzM4NCJ9.eyJoZWxsbyI6ICJ3b3JsZCJ9.BqDcZoC0pWa1k_gFRRSOJnDrU_0c78SdRphUikML36JrZs6fVwcP2NtdVUpDl40CwRdeHFjJF9skXI0KmMbj2bRTFyxL-At8WOVAMVRY0N9jFOjK67nwdafSDCKBqPAjw-Vv1f9yKwNvDoObI2isvhKqTF_EK_wEj-AwQIsmJNqUNF3yD3IgB6PCLS8mW5zXEp6xwf8hMguco6aHV7WBU-ZhdMFjJRydyrwBinUFig68QCdcAUI7KFVbsjani4W87jHgJhinv2GyVyKw5QpIDdvcMQKeoTZWLKQg9BBW4Ar9-YhQIuuO-7chTn2KCMpFCq3hrdrZTqVtX1nEDvBYpA")

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PubKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncodeRS512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, RS512, PrivKey())

	fmt.Printf("\nRS512 = %v\n", test)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "eyJhbGciOiJSUzUxMiJ9.eyJoZWxsbyI6ICJ3b3JsZCJ9.AqwiRtgwP0th1K8mvmL1oW8bbf8EuqDiUGLIfSQLY7Z9eXzct3Zcd3diri_-v6WczQWKtQ-NipCSEl8CWJQSZY8cRqDhFjiDnLEImY5SzC-xqYxDVb5zu_iQZNd18nlmSw504W3IvxIwk1_avZ6Q12OvRNbG3gLGZtcWi3VLdTa-SGEemEicnlRfLpTAaCk3kkJq6St5NLbiVdQMpfhDBeGCOTFMptLZ6Tw1rWaHy6y5b7_PRwWG1Cb8QEl8du-dxmhDgbenreq5EiybOL05qZRFxJ-1861j5xduG-AY6xum6HtWKPGTSsIZNxepK3DTYfovN5HidcubV747w_6PXw")

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PubKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecodePS256(c *C) {
	//given
	token := "eyJhbGciOiJQUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.S9xuR-IGfXEj5qsHcMtK-jcj1lezvVstw1AISp8dEQVRNgwOMZhUQnSCx9i1CA-pMucxR-lv4e7zd6h3cYCfMnyv7iuxraxNiNAgREhOT-bkBCZMNgb5t15xEtDSJ3MuBlK3YBtXyVcDDIdKH_Bwj-u363y6LuvZ8FEOGmIK5WSFi18Xjg-ihhvH1C6UzH1G82wrRbX6DyJKqrUnHAg8yzUJVP1AdgjWRt5BKpuYbXSib-MKZZkaE4q_hCb-j25xCzn8Ez8a7PO7p0fDGvZuOk_yzSfvXSavg7iE0GLuUTNv3nQ_xW-rfbrpYeyXNtstoK3JPFpdtORTyH1iIh7VVA"

	//when
	test, _, err := Decode(token, PubKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestDecodePS384(c *C) {
	//given
	token := "eyJhbGciOiJQUzM4NCIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.EKqVLw6nLGNt1h7KNFZbzkKhf788VBYCfnigYc0dBZBa64MrfbIFHtJuFgIGkCVSDYH-qs-i4w9ke6mD8mxTZFniMgzFXXaCFIrv6QZeMbKh6VYtSEPp7l0B1zMZiQw6egZbZ6a8VBkCRipuZggSlUTg5tHMMTj_jNVxxlY4uUwXlz7vakpbqgXe19pCDJrzEoXE0cNKV13eRCNA1tXOHx0dFL7Jm9NUq7blvhJ8iTw1jMFzK8bV6g6L7GclHBMoJ3MIvRp71m6idir-QeW1KCUfVtBs3HRn3a822LW02vGqopSkaGdRzQZOI28136AMeW4679UXE852srA2v3mWHQ"

	//when
	test, _, err := Decode(token, PubKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestDecodePS512(c *C) {
	//given
	token := "eyJhbGciOiJQUzUxMiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.IvbnmxhKvM70C0n0grkF807wOQLyPOBwJOee-p7JHCQcSstNeml3Owdyw9C3HGHzOdK9db51yAkjJ2TCojxqHW4OR5Apna8tvafYgD2femn1V3GdkGj6ZvYdV3q4ldnmahVeO36vHYy5P0zFcEGU1_j3S3DwGmhw2ktZ4p5fLZ2up2qwhzlOjbtsQpWywHj7cLdeA32MLId9MTAPVGUHIZHw_W0xwjJRS6TgxD9vPQQnP70MY-q_2pVAhfRCM_pauPYO1XH5ldizrTvVr27q_-Uqtw-wV-UDUnyWYQUDDiMTpLBoX1EEXmsbvUGx0OH3yWEaNINoCsepgZvTKbiEQQ"

	//when
	test, _, err := Decode(token, PubKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestEncodePS256(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, PS256, PrivKey())

	fmt.Printf("\nPS256 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 3)
	c.Assert(parts[0], Equals, "eyJhbGciOiJQUzI1NiJ9")
	c.Assert(parts[1], Equals, "eyJoZWxsbyI6ICJ3b3JsZCJ9")
	c.Assert(len(parts[1]), Equals, 24)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PubKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncodePS384(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, PS384, PrivKey())

	fmt.Printf("\nPS384 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 3)
	c.Assert(parts[0], Equals, "eyJhbGciOiJQUzM4NCJ9")
	c.Assert(parts[1], Equals, "eyJoZWxsbyI6ICJ3b3JsZCJ9")
	c.Assert(len(parts[1]), Equals, 24)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PubKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncodePS512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, PS512, PrivKey())

	fmt.Printf("\nPS512 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 3)
	c.Assert(parts[0], Equals, "eyJhbGciOiJQUzUxMiJ9")
	c.Assert(parts[1], Equals, "eyJoZWxsbyI6ICJ3b3JsZCJ9")
	c.Assert(len(parts[1]), Equals, 24)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PubKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecodeES256(c *C) {
	//given
	token := "eyJhbGciOiJFUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.EVnmDMlz-oi05AQzts-R3aqWvaBlwVZddWkmaaHyMx5Phb2NSLgyI0kccpgjjAyo1S5KCB3LIMPfmxCX_obMKA"

	//when
	test, _, err := Decode(token, Ecc256Public())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestDecodeES384(c *C) {
	//given
	token := "eyJhbGciOiJFUzM4NCIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.jVTHd9T0fIQDJLNvAq3LPpgj_npXtWb64FfEK8Sm65Nr9q2goUWASrM9jv3h-71UrP4cBpM3on3yN--o6B-Tl6bscVUfpm1swPp94f7XD9VYLEjGMjQOaozr13iBZJCY"

	//when
	test, _, err := Decode(token, Ecc384Public())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestDecodeES512(c *C) {
	//given
	token := "eyJhbGciOiJFUzUxMiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.AHxJYFeTVpZmrfZsltpQKkkplmbkycQKFOFucD7hE4Sm3rCswUDi8hlSCfeYByugySYLFzogTQGk79PHP6vdl39sAUc9k2bhnv-NxRmJsN8ZxEx09qYKbc14qiNWZztLweQg0U-pU0DQ66rwJ0HikzSqgmyD1bJ6RxitJwceYLAovv0v"

	//when
	test, _, err := Decode(token, Ecc512Public())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestEncodeES256(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, ES256, Ecc256Private())

	fmt.Printf("\nES256 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 3)
	c.Assert(parts[0], Equals, "eyJhbGciOiJFUzI1NiJ9")
	c.Assert(parts[1], Equals, "eyJoZWxsbyI6ICJ3b3JsZCJ9")
	c.Assert(len(parts[2]), Equals, 86)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc256Public())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncodeES384(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, ES384, Ecc384Private())

	fmt.Printf("\nES384 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 3)
	c.Assert(parts[0], Equals, "eyJhbGciOiJFUzM4NCJ9")
	c.Assert(parts[1], Equals, "eyJoZWxsbyI6ICJ3b3JsZCJ9")
	c.Assert(len(parts[2]), Equals, 128)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc384Public())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncodeES512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, ES512, Ecc512Private())

	fmt.Printf("\nES512 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 3)
	c.Assert(parts[0], Equals, "eyJhbGciOiJFUzUxMiJ9")
	c.Assert(parts[1], Equals, "eyJoZWxsbyI6ICJ3b3JsZCJ9")
	c.Assert(len(parts[2]), Equals, 176)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc512Public())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecrypt_DIR_A128CBC_HS256(c *C) {
	//given
	token := "eyJhbGciOiJkaXIiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2In0..3lClLoerWhxIc811QXDLbg.iFd5MNk2eWDlW3hbq7vTFLPJlC0Od_MSyWGakEn5kfYbbPk7BM_SxUMptwcvDnZ5uBKwwPAYOsHIm5IjZ79LKZul9ZnOtJONRvxWLeS9WZiX4CghOLZL7dLypKn-mB22xsmSUbtizMuNSdgJwUCxEmms7vYOpL0Che-0_YrOu3NmBCLBiZzdWVtSSvYw6Ltzbch4OAaX2ye_IIemJoU1VnrdW0y-AjPgnAUA-GY7CAKJ70leS1LyjTW8H_ecB4sDCkLpxNOUsWZs3DN0vxxSQw.bxrZkcOeBgFAo3t0585ZdQ"

	//when
	test, _, err := Decode(token, aes256Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_DIR_A192CBC_HS384(c *C) {
	//given
	token := "eyJhbGciOiJkaXIiLCJlbmMiOiJBMTkyQ0JDLUhTMzg0In0..fX42Nn8ABHClA0UfbpkX_g.ClZzxQIzg40GpTETaLejGNhCN0mqSM1BNCIU5NldeF-hGS7_u_5uFsJoWK8BLCoWRtQ3cWIeaHgOa5njCftEK1AoHvechgNCQgme-fuF3f2v5DOphU-tveYzN-uvrUthS0LIrAYrwQW0c0DKcJZ-9vQmC__EzesZgUHiDB8SnoEROPTvJcsBKI4zhFT7wOgqnFS7P7_BQZj_UnbJkzTAiE5MURBBpCYR-OS3zn--QftbdGVJ2CWmwH3HuDO9-IE2IQ5cKYHnzSwu1vyME_SpZA.qd8ZGKzmOzzPhFV-Po8KgJ5jZb5xUQtU"

	//when
	test, _, err := Decode(token, aes384Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553372,"sub":"alice","nbf":1392552772,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"f81648e9-e9b3-4e37-a655-fcfacace0ef0","iat":1392552772}`)
}

func (s *TestSuite) TestDecrypt_DIR_A256CBC_HS512(c *C) {
	//given
	token := "eyJhbGciOiJkaXIiLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIn0..ZD93XtD7TOa2WMbqSuaY9g.1J5BAuxNRMWaw43s7hR82gqLiaZOHBmfD3_B9k4I2VIDKzS9oEF_NS2o7UIBa6t_fWHU7vDm9lNAN4rqq7OvtCBHJpFk31dcruQHxwYKn5xNefG7YP-o6QtpyNioNWJpaSD5VRcRO5ufRrw2bu4_nOth00yJU5jjN3O3n9f-0ewrN2UXDJIbZM-NiSuEDEgOVHImQXoOtOQd0BuaDx6xTJydw_rW5-_wtiOH2k-3YGlibfOWNu51kApGarRsAhhqKIPetYf5Mgmpv1bkUo6HJw.nVpOmg3Sxri0rh6nQXaIx5X0fBtCt7Kscg6c66NugHY"

	//when
	test, _, err := Decode(token, aes512Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553617,"sub":"alice","nbf":1392553017,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"029ea059-b8aa-44eb-a5ad-59458de678f8","iat":1392553017}`)
}

func (s *TestSuite) TestEncrypt_DIR_A128CBC_HS256(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, DIR, A128CBC_HS256, aes256Key)

	fmt.Printf("\nDIR A128CBC-HS256 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJkaXIiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2In0")
	c.Assert(len(parts[1]), Equals, 0)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes256Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_DIR_A192CBC_HS384(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, DIR, A192CBC_HS384, aes384Key)

	fmt.Printf("\nDIR A192CBC-HS384 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJkaXIiLCJlbmMiOiJBMTkyQ0JDLUhTMzg0In0")
	c.Assert(len(parts[1]), Equals, 0)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 32)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes384Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_DIR_A256CBC_HS512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, DIR, A256CBC_HS512, aes512Key)

	fmt.Printf("\nDIR A256CBC-HS512 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJkaXIiLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIn0")
	c.Assert(len(parts[1]), Equals, 0)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 43)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes512Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecrypt_DIR_A128GCM(c *C) {
	//given
	token := "eyJhbGciOiJkaXIiLCJlbmMiOiJBMTI4R0NNIn0..yVi-LdQQngN0C5WS.1McwSmhZzAtmmLp9y-OdnJwaJFo1nj_4ashmzl2LhubGf0Jl1OTEVJzsHZb7bkup7cGTkuxh6Vfv10ljHsjWf_URXoxP3stQqQeViVcuPV0y2Q_WHYzTNGZpmHGe-hM6gjDhyZyvu3yeXGFSvfPQmp9pWVOgDjI4RC0MQ83rzzn-rRdnZkznWjbmOPxwPrR72Qng0BISsEwbkPn4oO8-vlHkVmPpuDTaYzCT2ZR5K9JnIU8d8QdxEAGb7-s8GEJ1yqtd_w._umbK59DAKA3O89h15VoKQ"

	//when
	test, _, err := Decode(token, aes128Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392548520,"sub":"alice","nbf":1392547920,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"0e659a67-1cd3-438b-8888-217e72951ec9","iat":1392547920}`)
}

func (s *TestSuite) TestDecrypt_DIR_A128GCM_WRONG_NONCE_SIZE(c *C) {
	//given
	token := "eyJhbGciOiJkaXIiLCJlbmMiOiJBMTI4R0NNIn0..yVi-LQQngN0C5WS.1McwSmhZzAtmmLp9y-OdnJwaJFo1nj_4ashmzl2LhubGf0Jl1OTEVJzsHZb7bkup7cGTkuxh6Vfv10ljHsjWf_URXoxP3stQqQeViVcuPV0y2Q_WHYzTNGZpmHGe-hM6gjDhyZyvu3yeXGFSvfPQmp9pWVOgDjI4RC0MQ83rzzn-rRdnZkznWjbmOPxwPrR72Qng0BISsEwbkPn4oO8-vlHkVmPpuDTaYzCT2ZR5K9JnIU8d8QdxEAGb7-s8GEJ1yqtd_w._umbK59DAKA3O89h15VoKQ"

	//when
	_, _, err := Decode(token, aes128Key)

	//then
	c.Assert(err, NotNil)
}

func (s *TestSuite) TestDecrypt_DIR_A192GCM(c *C) {
	//given
	token := "eyJhbGciOiJkaXIiLCJlbmMiOiJBMTkyR0NNIn0..YW2WB0afVronbgSz.tfk1VADGjBnViYD7He5mbhxpbogoT1cmhKiDKzzoBV2AxfsgJ2Eq-vtEqPi9eY9H52FLLtht26rc5fPz9ZKOUH2hYeFdaRyKYXlpEnUR2cCT9_3TYcaFhpYBH4HCa59NruKlJHMBqM2ssWZLSEblFX9srUHFtu2OQz2ydMy1fr8ABDTdVYgaqyBoYRGykTkEsgayEyfAMz9u095N2J0JTCB5Q0IiXNdBzBSxZXG-i9f5HFEb6IliaTwFTNFnhDL66O4rsg._dh02z25W7HA6b1XiFVpUw"

	//when
	test, _, err := Decode(token, aes192Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392552631,"sub":"alice","nbf":1392552031,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"a3fea096-2e96-4d8b-b7cd-070e08b533fb","iat":1392552031}`)
}

func (s *TestSuite) TestDecrypt_DIR_A256GCM(c *C) {
	//given
	token := "eyJhbGciOiJkaXIiLCJlbmMiOiJBMjU2R0NNIn0..Fmz3PLVfv-ySl4IJ.LMZpXMDoBIll5yuEs81Bws2-iUUaBSpucJPL-GtDKXkPhFpJmES2T136Vd8xzvp-3JW-fvpRZtlhluqGHjywPctol71Zuz9uFQjuejIU4axA_XiAy-BadbRUm1-25FRT30WtrrxKltSkulmIS5N-Nsi_zmCz5xicB1ZnzneRXGaXY4B444_IHxGBIS_wdurPAN0OEGw4xIi2DAD1Ikc99a90L7rUZfbHNg_iTBr-OshZqDbR6C5KhmMgk5KqDJEN8Ik-Yw.Jbk8ZmO901fqECYVPKOAzg"

	//when
	test, _, err := Decode(token, aes256Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392552841,"sub":"alice","nbf":1392552241,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"efdfc02f-945e-4e1f-85a6-9f240f6cf153","iat":1392552241}`)
}

func (s *TestSuite) TestEncrypt_DIR_A128GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, DIR, A128GCM, aes128Key)

	fmt.Printf("\nDIR A128GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJkaXIiLCJlbmMiOiJBMTI4R0NNIn0")
	c.Assert(len(parts[1]), Equals, 0)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes128Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_DIR_A192GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, DIR, A192GCM, aes192Key)

	fmt.Printf("\nDIR A192GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJkaXIiLCJlbmMiOiJBMTkyR0NNIn0")
	c.Assert(len(parts[1]), Equals, 0)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes192Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_DIR_A256GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, DIR, A256GCM, aes256Key)

	fmt.Printf("\nDIR A256GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJkaXIiLCJlbmMiOiJBMjU2R0NNIn0")
	c.Assert(len(parts[1]), Equals, 0)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes256Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecrypt_RSA1_5_A128CBC_HS256(c *C) {
	//given
	token := "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2In0.Pi1BtHrCnYfqXB0jr9lDuUoZf5yVZQKm940dNrhyEtcTg5R5Nw8NIPepVYM0Lt_BKdUs-g9iwZ4FwO0NHnaLOysqmLiOkey1gO2dBnDySISG4-SDEhkbBxVmAK-schPxXcC9hR0SpoI202h5o4jMrEvoxfiT-D5wq8iTJAiipVMA2QFO46ODuDLQOxuDTf57_z0W_5Qg-LYce7C8_nyhpGnhqkONEAwFBVwX0gPX08bui7fxs2LctvB3YZLJ5o4QDb0dRJ9HjDJ6YMiuPt9lgH4a4TScD3wZFmeksZYnNiMHPsfL-xDexbabQb2WB0CEleRXCBAcoGYQdXy-cCr4Vg.S8T50xGdXg_PzORit9s3Hg.vtHSEqDBKRiLpmO3CVzANMNu0dKcm0r1VJ-3YkZD8LEEGx4K29Scq7hMKC1XgBUYkcY0F1e7itItEvpWvwXkNAzP8YPEj8-p_wHgrqSn0Z4x_8626vKxLjlOK30uleq_pJTc4lwjG5VTRW9ax1Jy8rVu6x464e9vwfjzdFdEcDMtfM2MYrhM9XFHqaJNRmUAqd0GKepvoPVQSOw3tVxeqblCEgpZt8CEHpKpNPQkzOiaF3Ec7a8Garm7FE0zI69qVh94-ViBp7_1uxxHw-AnSg.JVC_j28xJiA0iQOhL86v7w"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1408349987,\"sub\":\"alice\",\"nbf\":1408349387,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"e82d6f67-e3ed-4a1b-9f29-114575c37e6d\",\"iat\":1408349387}")
}

func (s *TestSuite) TestDecrypt_RSA1_5_A192CBC_HS384(c *C) {
	//given
	token := "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMTkyQ0JDLUhTMzg0In0.ApUpt1SGilnXuqvFSHdTV0K9QKSf0P6wEEOTrAqWMwyEOLlyb6VR8o6fdd4wXMTkkL5Bp9BH1x0oibTrVwVa50rxbPDlRJQe0yvBm0w02nkzl3Tt4fE3sGjEXGgI8w8ZxSVAN0EkaXLqzsG1rQ631ptzqyNzg9BWfy53cHhuzh9w00ZOXZtNc7GFBQ1LRvhK1EyLS2_my8KD091KwsjvXC-_J0eOp2W8NkycP_jCIrUzAOSwz--NZyRXt9V2o609HGItKajHplbE1PJVShaXO84MdJl3X6ef8ZXz7mCP3dRlsYfK-tlnFVeEKwC1Oy_zdFsdiY4j41Mj3usvG2j7xQ.GY4Em2zkSGMZsDLNr9pnDw.GZYJSpeQHmOtx34dk4WxEPCnt7l8R5oLKd3IyoMYbjZrWRtomyTufOKfiOVT-nY9ad0Vs5w5Imr2ysy6DnkAFoOnINV_Bzq1hQU4oFfUd_9bFfHZvGuW9H-NTUVBLDhok6NHosSBaY8xLdwHL_GiztRsX_rU4I88bmWBIFiu8T_IRskrX_kSKQ_iGpIJiDy5psIxY4il9dPihLJhcI_JqysW0pIMHB9ij_JSrCnVPs4ngXBHrQoxeDv3HiHFTGXziZ8k79LZ9LywanzC0-ZC5Q.1cmUwl7MnFl__CS9Y__a8t5aVyI9IKOY"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1391196668,\"sub\":\"alice\",\"nbf\":1391196068,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"c9d44ff8-ff1e-4490-8454-941e45766152\",\"iat\":1391196068}")
}

func (s *TestSuite) TestDecrypt_RSA1_5_A256CBC_HS512(c *C) {
	//given
	token := "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIn0.NPxfy2Oscg5bA_7pPpc8NCa_CpAu0DWCo8JHrr5acLQ4Fcn1veO0MucNLdHHXlTcolSudxLpjulfmTxafUIw0O1_rz9omsB577f3B0DA_ovnABLwIf1YUJTE-pPCn1qVU2noaWhq0OEnmzB3aJaDp1pp1Lb3UHt-X9dbXDD2zluiSyt4wAjLp7JJLPfNdypps0UwOjGPRG009C7Kbb3Gnx1YOSd_xVuRDf_CqwwF5-VoAQxbLxDU-38aCZL7VjMgiDDXs56SX2bqnpvW1Q_58wTA-7LIpiP4vk8th2jpg59W9k0vixAqszWeiw1nqVOJ5qZgusss-MXYlPHZGmWFpg.uo4pnWhJEa8Cd496i1aS8Q.buM4urwbq330U7HUM2uNExvhBHxneqy4MnU1_g3ar5aTiH0dOsKD2J8jJJPt3k2BTPOOm8YZh36M7sm5TUTdp2Suc9UyMeV-HgL5-DPGGPFaG-Kk1ZjZPBpGwf-RmpSq9yUhINx5TO3XoX1rXyJcNi8_3l1B3R5Fv3Kjl-cihbTejHz_q2IPQwDZJTaTIlmtkrflTN07aZ9Y_KFLc2qjG6KdP2CM-I1-Oq-gc-H0EdoBLj1zGvAwoPpPdviO8C8n4VBb80RV8ED-wt8J2JVc3A.6bGm8EKBxcyMvFcWXr5G1ziCD0e1Pv1TP0YOze11vZc"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1408349987,\"sub\":\"alice\",\"nbf\":1408349387,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"f6d4b905-3382-4207-be7a-611461e06c3d\",\"iat\":1408349387}")
}

func (s *TestSuite) TestDecrypt_RSA1_5_A128GCM(c *C) {
	//given
	token := "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMTI4R0NNIn0.SLM4JN2t0JHYBXQr2uJ-K3QHW4SRCWPTPL-Czo-9IJQWgBbIlBXjrXOpQ-HtLg3OeKlXGz4AIihdHpFRSpPAq-e4ht8O5DSK-O6qyl8_-PSJfinZCMeLLE9Q1L6qHY86Rv9NMFeVUZZVmBPIXUTer-jU4tAgi3Upwek0M_gMwzeH94FEhHQqCzdYsBfsqwHpuRcrjatdjxBoSBXIEvNg8fi31UYG5L6jEUt2evOVFYq1C0-R17wydC71-2ubb35xXKbsNt5pShiil0sDalLTkuVKMtDuuMWZeMWHPp81wGJSHj9Ctl_ISHHjPC99MbGq9ZXSOBX2DNbfVmUMVRfHcQ.bbsRwjtVRyEeovfu.jGoxrc-CRwigy9IXs6NwKUR2zMZL2gcj_qhEmZAL1uqhkusA4zcCvVNrt2hFL3ZprN7pjpJ0Nm56MngNgTWQZkxwt_oz-Dk7IHmemEkC3m2Xboyy9ZvfesHvJlZEs6xMmxM-7TEQrb7_FP0lR_wZVoIuMQUHl6rwxofjtPNdABzxDQyaW-PU7aOSGNO-Eg8lBGFJZFWTcpwOwsYZhSdHhtc4PnUmzfkwDMWdOAXwey4bRXih6D0LuQZZbVlsYOSZq3dpTg.Igb39yvfNHPGPVryD5a3WA"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1408349988,\"sub\":\"alice\",\"nbf\":1408349388,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"e7c97c7a-418c-440d-b3de-6f9ddc5d8f71\",\"iat\":1408349388}")
}

func (s *TestSuite) TestDecrypt_RSA1_5_A192GCM(c *C) {
	//given
	token := "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMTkyR0NNIn0.pqD1m66UeNppuOnP_H0WBQB3PCgYN0Zzf1DZraXM8XUJapw4LrwhvUXlYwcgrFdZ4v-asmzvABPc2sbGxzIJsPebGoEIlFDV3OvRRN1vUBOdst99v4q9En21FLa1MwNvCoDeg-TLK54t-NPShmsb4cLWFGS8KAKj2HEqXStw18Di4PTEfFfiJpeBUF8UHgqXfs5vqSJKmtvHPBNGUpCOPEhuNDOOKXvxMQotjoktKCyffJSgtr1fy0rOO4iR4EJjozclIAPVDCfQMVfVZBJEbUfszZ6cOCVsPLZKY3msWo3CFOQOcvXJOjkp0IXCaQxrvofetgmHP_ILsf4ty7_UIA.oIicSl5B8tBykEB-.Xlb-OZBpOdDfTEIjtYDFIm40CJeolfC7C4t2qgqi3Gd4kk-5oEbQzNuFMp8M4eUyDsSvNXdN5mlrOwAj9KsSLWB5FZf51Nvk5YJFN9scaQmdSzU7AlLhM2yMEpOmzZMo1GsiZxD8Rle0LyWmgFtMpGJk3PTEXwYnmNyyDAxhFAaWU2WOc1KofQsPbOjTA_0ijpVHG2FO-8jMPWNvl5GbZPAeId1yYt_mhTXbg6M7VRgazWQsq0n0IK8C3r0hu0UZ0DL0fg.xG8T-AWjAokmvAmBw-tiDA"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1408349988,\"sub\":\"alice\",\"nbf\":1408349388,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"0c8db172-5795-4033-badf-2e95fb8a24ac\",\"iat\":1408349388}")
}

func (s *TestSuite) TestDecrypt_RSA1_5_A256GCM(c *C) {
	//given
	token := "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMjU2R0NNIn0.atdPJ2O-zpVFKyT1_EY0OT8WfF78A6fgYabEtGUUg1KrlR-dDat8tAtj58__lu4TFrZ_J0kcWNVkPpRy0IiyCxKtuo2Gfeufiq0MCiOza0XjT87RJinvAQMJU5rjfOC9bnCgEmDdaePRYr2zbnq7cBU6xh4q9sUT6jDhOSQ49aXDZdzIsMz4lwHpEtUwpF8vFuK7CP1AyibVrrCp1PoVyx4kpT7sRP0V3cRrZutO1MGlI3BWHdFCO2Ak0PDtlqx5uh1atB4XlTD0nswLMoKRSSOuP4GviZFoX4v7REFVcNbuIVQz0bbszYSmJXNi0VBGSodrsM2Ok0ufXRxYWaBepg.hSe2EnGM-0CFZzy-.jplzmtN7z5cARKj9kpgFFUD2_4qpFyvfn4F2kUXEkgRSgRN7ej8S0GepH2qOksS0yrwD97NdVA-hqhD7EiixVHaBhyxT9hYMdJLmPygfiXG7WKvLuDR24zVUvD8q98b1lSP55JPKEIBj_y1YVmrybv_dl30wqt4cbDLhmqWScwJRSp53KX26F8-R903Zitr5dke_6G8wMcJTrszus0EDxsurSPkxkDlBmDl6h-J0XNgY7dzB4tXud8q7l6XPvbvSbNBpqg.4ppwz1DXn7ba80wmEKAwaQ"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1408349988,\"sub\":\"alice\",\"nbf\":1408349388,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"0b2f4ddf-8ed0-45d5-9e25-973212099e97\",\"iat\":1408349388}")
}

func (s *TestSuite) TestEncrypt_RSA1_5_A128CBC_HS256(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA1_5, A128CBC_HS256, PubKey())

	fmt.Printf("\nRSA1_5 A128CBC-HS256 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2In0")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA1_5_A192CBC_HS384(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA1_5, A192CBC_HS384, PubKey())

	fmt.Printf("\nRSA1_5 A192CBC-HS384 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMTkyQ0JDLUhTMzg0In0")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 32)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA1_5_A256CBC_HS512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA1_5, A256CBC_HS512, PubKey())

	fmt.Printf("\nRSA1_5 A256CBC-HS512 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIn0")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 43)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA1_5_A128GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA1_5, A128GCM, PubKey())

	fmt.Printf("\nRSA1_5 A128GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMTI4R0NNIn0")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA1_5_A192GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA1_5, A192GCM, PubKey())

	fmt.Printf("\nRSA1_5 A192GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMTkyR0NNIn0")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA1_5_A256GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA1_5, A256GCM, PubKey())

	fmt.Printf("\nRSA1_5 A256GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0ExXzUiLCJlbmMiOiJBMjU2R0NNIn0")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecrypt_RSA_OAEP_A128CBC_HS256(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkExMjhDQkMtSFMyNTYifQ.mlcD4YXAzTKgKsnkiPD8iBPkUkVijLCf9ODLs-ziHotR10iVPg2G2j1m-_BCgLB0SNWOcvkCTG1jzYJPr7ARs9GIBg63vtlqgk5TJhTko3J6TFwkrqMB_yeweuWvVvJ7R5CvlKRh_diUcB90RX1lPWSTx4wQHOjLssQ-xRG1iVZ2eroA1uhi8gW7Jdds2wmEnDqxfwmMaaK7IY6cpG_rUMYDqG3m9r3-B0j2MAYOtjGzPtwsJfTHsdasWz_pbypViOp35_BrUHRy6qfqR0uK3zfhe8sFfQDTDctMjissAUGo8rpqnVphoojjvxZVlkB2PmcctOzkAiaMusdOsAxhRg.yH5p6nQitVc9YxQJQP6ocw.XeHzeO9nlXJEmnKtEe_uyl72Aqyzje7Eiz96Zpz79Ot8udWPTmVpa902pGSEeLqHSmbvSpleGEGeuJP5HgOQ-VCrUd3FYL9PK88ieEwO2zsXZfGq4L-tVu7sgyYE8yz25dNRoMoXnrPRzv8dciq3XYxgIYeE0gMc9htVb42rXH7lXjbYeA1aCS2sjd-hRbh-5_afChrpDKYTavpd4EikOdz_-d7EddeUOnHmUsFBz6R6PVX4aOSUkIQug4RyXZLLaq0UkfBLfwezgO3t-dWY8Q.vjaIoaOKXR76eOuUZ1fang"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1408354731,\"sub\":\"alice\",\"nbf\":1408354131,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"bda732e5-8cbd-4f9d-8205-9e05a77a1f90\",\"iat\":1408354131}")
}

func (s *TestSuite) TestDecrypt_RSA_OAEP_A192CBC_HS384(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkExOTJDQkMtSFMzODQifQ.cT1VUi7KyfLuDL2JUVL5nVw7OcVUJTCwU2jPbIJVpPh2oR_89BYb4c5ff21A2F-69CqOgV-FD4zlmOkvjywiVq4BRlo8AO90qbe4dSDkIedXHTb0CM0o5HJCWPRNqlBfoPl8tTk7FFtCQyQIJmmTp88h8P4fEtHPe7v50oBjV6sVVdd0-vJa2Lp8id9ezKmlQ5IcgI6E3-81xIGfGvYERbtI7Tr-VQ8LjtylbgKPljfTtVQ_gn0UmbyLBIOUfQfLdpzF7Gdh9kfkEBy1nEnDuQ8V4BqkWfcb6yOWho_zSS4b9lj5qYisjo5KiOmsNNZcyl-EWbjq98k6gAKvAjWxlw.2yxe4VSlm_No0Ii5NTU3kQ.hN_a2DF0B02g7oBTs-Ft2_8nLPI1i6yzyys6a0TIjmfQZrzZTepORlQfHymBKwJOMDbThehvj4F3Yjjk_fziSN80eeuuvRdsi4wheH_jymcgoHJE1FCgpWpLKUfGlY9lMWeqaeL047W-qPdAmUDLUIiIsA4qguX55aPHFPQyjATg0-XaM1IfLySTxCNwV9Ha6FRtxBT01L9T2hbpucu708HgHCvdXn2jzOIC_Zs7SMdb3BbVeUaAyIobU6jDNd3WrgZxwUW0S0LZhovYip-sYg.xneenkPzCRAUQGO8G4YJMGNGYZzqjKoo"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1408439204,\"sub\":\"alice\",\"nbf\":1408438604,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"1d22fd82-84b5-4b24-96db-c13e41097a9f\",\"iat\":1408438604}")
}

func (s *TestSuite) TestDecrypt_RSA_OAEP_A256CBC_HS512(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkEyNTZDQkMtSFM1MTIifQ.JY7V0pIV5tFYju-WcdfO-KxYdV-d2tjKyuXnq3HwVjQLqttlwnbcxAtUApRuc1UuB6dqp8ggc9d3kgTTZVIveT5MaP57Jm6kU33puHxdujiz0CEFBpf0IWHetq175iFfZWFgAp4BWIK-FaG7_mbinSlNmSxZ4KOM8vgkfvfXQcPwBfsrehsxWRS6BIpJ2PKjOr61z0ry5s19pcMKWzrO-ygtDsusj9ioqH8H0ZahG323um-OtxIw1eVdBsa7rBg8n3zE-Lowgog6wZBOuIzN8bMdRVk5Y08xZRLxmcazJB5aMBB-sVZ5X695ACYTdG55oDpAet5uwyOJybkHF9MSRg.kiHWbMnhKCrSkjOXoIKpAw.xx44_UzBGn_kGOSPZI6QXQTGKq8OFHKyGM1ALZhk51UoiQQe6YzrtwHZcufT3yO0oFcw6TshkdyjKvwXecmSf-LOlUkgaR9o9zQ3PtcFLtgfHdTKBRDBEiubLHMOUTuHCi8laveJ0-RYuYcpwOnSCS_6hPoO86p3aAYzZOmJ3mFLgKQxdt7tgqV7DeRt3j9FyRXmS7vusA7g2oDJz-f7aRs0hWjtPiDN3TGwByacn6T_nlPCvTs6ZR-y9BVNYTaGCJEgujbXebtae_Y94_XZ3g.H2v7BssosabkZEEIk72pFhyYR5VUjQ9EEyQVtFyVvg8"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1408439204,\"sub\":\"alice\",\"nbf\":1408438604,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"ccb36bb8-7fe2-43b7-93ba-63c9331d0f47\",\"iat\":1408438604}")
}

func (s *TestSuite) TestDecrypt_RSA_OAEP_A128GCM(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkExMjhHQ00ifQ.Prr8uxpkVLvASjHK4n_TKrMNQWx9cMN4_t1FIMhXXW9aRD4uLXLTmTasspvbprwPbR47qkCu_KcHC6p5eBpZZIW0Ja96OyQLycnhZkueyelqxeGKYMuHEPQ-cAxoe1Vf47bNYBK-9J9G9cHBjngsISlgoxv2K8AjDnAvwlKRqz1Rjg_zdM6nVpFwrWMNEBg5M8HdLR_P0S_z8S9spOsOCWl_2hybwkmuViXdprwXBVdm7TItuhw0NZVP88FzIO7F3C23788bf4JU_6I7KECuWG9uNoPfORZjNGYCIvFWJvq3Adyk3r3DuQwiWc6tE2vHcMj9f7yFxWqWUsxWOtmfuA.sMJK64MRl7g4A_Qr.D3OvvJkI_D569j6G7o8l54iJUOZ36iqge7wvNkBGP0SksZEwhBB7HOJVH8avWY-CpA6Lhh_FZvvA3phgVIQRTMSzc3pXMnirXh0qGBSvdi4OKJsUKWC0dugGY1j5sgV8mEowepg3iUM_j9WS3FW1mSh65IBlyEsLL6eUiYQepeN4vr4cN3jn-vORYVu-KaRyAnab4IJgzhhyRD48_B9w9_dffcuN4_xXPc-O0IvBsFkJTtPRvWGJFkv0r-JMXJR-BZqBrg.pd51_IhdN5YRuzCB6XIHvw"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1408439204,\"sub\":\"alice\",\"nbf\":1408438604,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"3c039563-1d18-48df-8008-d113ac43cf2f\",\"iat\":1408438604}")
}

func (s *TestSuite) TestDecrypt_RSA_OAEP_A192GCM(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkExOTJHQ00ifQ.IuvEhYtqW3TXDfc44FyXtMgWL0urYpZktuo989Y0MhusaZjTsaIRnsl2wyaRUim6vfuRE5Svtfm98UYC5zKx_FOq9CTElI7t9Q3QdvimiIbswaWCA2LKx6aiJxA2eCEhVRMUlDDe49V1HY2_zOiPN87qE-P-htRdxSBuQCDplJ7t8agp9VPM73pRAxt9gGvmlXzdGbCpR1HXhIDK14urfcuEF31w5jk0LOR2TIyt8VBWEYMdFnHpYKAQN3KQrF6yNow7AC0jIgcBM4ep41F5d9gkVp2jZWUsRT1Epgv0UcHC0t6BDArLT6Gy2xemTbs81qmARO5v4iyBRBaIuSehuw.F7NhQ3VzsCGfpRI2.jQZz-Kc6n5jUZzPTdjlbnVk-ujFpTNTsEa3J3L222eLg6cPQ6WQH9kmnoSW9WJXbun1ZFBSESv6ti3fKxEUPqs4vwy8d5JIrjrhXcw8lf2AWfFytX92U-ZqSGaoiqGjp0AHnuCqpNnBvCk2-vIHTAabWnpdGvIszVnodEF299CHDgHS9XIerZTGkQCZrRaH89koVfsO__pXnXcjy5loOV9D3ACfj3bIVRUdmg8P1mvyZfNdDErCMuu1G1TK6yaqpcDbrOg.WwWMv-v1Czh67ltSsbI0-A"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1408439204,\"sub\":\"alice\",\"nbf\":1408438604,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"49ea8e2e-5de8-4fdf-a3d8-f8cd348c1289\",\"iat\":1408438604}")
}

func (s *TestSuite) TestDecrypt_RSA_OAEP_A256GCM(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkEyNTZHQ00ifQ.ClRqSlrZ-3uqe5g4iiAtg3KKqFY_28sFgAh_u-_jUDwU7JBtvDazKRMK0guPqXq0pImb-bCDA6QZhzWgrrz9RGOMEJOVmq_JFl2SF-ipTz6Xj8nDAfqDnYbxj2XRdqMdgdu0BM7xXIpc5GbXuutPFcu9aVxyafv5G4JQo5qCx9Yvo-zVuLUvdOom_cIHwkWK3bx4EPBifIPdPfTQXt8yCXdF36CpT_mmbc13Fe-Uw8JgWtoLQ2nfbXjIEsSgr8BZdxfhiCFYAIyHQ2CdSzEh6J34NtF50u0DVOnkSjAMGiukB0k1fW469EutkfrZJIvo_19Qk8n5Eif7gW1WUw9u9w.rmBi9peixHbDA8cX.o5Xt4SHLSKCYX28u4_rArpfC3KgWwiIcWQMZM1YJPrqX2hRvGqg8Tcl80vXEV-mfNntao0QCPA7hsz4gaoHKLTh1nU1ucY9UGICkwgv9D2q3soOwYRlKP1PuDrh6LQPcPcn2yoSMXTjFhcgPlNGEE-mFipQEnMB5Cn47fcXxpJrWyXSo_FHuOWB1_mfSs1n-HMIDE7Wrxtegp0VNU8AFMEIeQj3pDupZhPrvHzwwRKcfyXhccyYDdfAPrL4xkqFs5en_fw.z8Oazv9h_rLZyI0wQx-QiQ"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1408439204,\"sub\":\"alice\",\"nbf\":1408438604,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"7b032524-9b59-4469-b92a-fdf25f2c5a7c\",\"iat\":1408438604}")
}

func (s *TestSuite) TestEncrypt_RSA_OAEP_A128CBC_HS256(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA_OAEP, A128CBC_HS256, PubKey())

	fmt.Printf("\nRSA-OAEP A128CBC-HS256 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkExMjhDQkMtSFMyNTYifQ")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA_OAEP_A192CBC_HS384(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA_OAEP, A192CBC_HS384, PubKey())

	fmt.Printf("\nRSA-OAEP A192CBC-HS384 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkExOTJDQkMtSFMzODQifQ")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 32)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA_OAEP_A256CBC_HS512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA_OAEP, A256CBC_HS512, PubKey())

	fmt.Printf("\nRSA-OAEP A256CBC-HS512 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkEyNTZDQkMtSFM1MTIifQ")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 43)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA_OAEP_A128GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA_OAEP, A128GCM, PubKey())

	fmt.Printf("\nRSA-OAEP A128GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkExMjhHQ00ifQ")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA_OAEP_A192GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA_OAEP, A192GCM, PubKey())

	fmt.Printf("\nRSA-OAEP A192GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkExOTJHQ00ifQ")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA_OAEP_A256GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA_OAEP, A256GCM, PubKey())

	fmt.Printf("\nRSA-OAEP A256GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkEyNTZHQ00ifQ")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecrypt_A128KW_A128CBC_HS256(c *C) {
	//given
	token := "eyJhbGciOiJBMTI4S1ciLCJlbmMiOiJBMTI4Q0JDLUhTMjU2In0.DPRoUHQ3Ac8duyD32nUNH3eNUKzUIMYgEdf5GwJ8rW4MYQdl2PCIHA.B1dR6t93aUPcFC1c1aUjeA.lHPKTK0ehgzq70_Ihdh-svI2icUa9usgqP8sF5j50fsQAGizITZpTTXKOKd9-GSEVmJo07551hq9xscZj4vXsDEx-z-akxg0nlL5fFE24km7l4T3LfAeG17gmrMcJuLP55mFUg-F98j9duV2UCyKJPXP6RwOQ5X17VNw29c4k-_AxYM0EjTv3Fww1o3AGuVa07PfpLWE-GdJeJF9RLgaP_6Pua_mdVJud77bYXOsVxsweVtKIaBeLswMUUSU6PoC5oYURP_ybW76GOCjmgXpjA.avU8f5LK_tbJOyKW6-fRnw"

	//when
	test, _, err := Decode(token, aes128Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1392553211,\"sub\":\"alice\",\"nbf\":1392552611,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"586dd129-a29f-49c8-9de7-454af1155e27\",\"iat\":1392552611}")
}

func (s *TestSuite) TestDecrypt_A192KW_A192CBC_HS384(c *C) {
	//given
	token := "eyJhbGciOiJBMTkyS1ciLCJlbmMiOiJBMTkyQ0JDLUhTMzg0In0.OLwgc7EaQdvsf54GfU69qH143C79H_eETvM_yGBgJzEB5367k9tbw6qW4TlQ56GMj__5QDJBvAg.BvYY_v4_dxxsK4M8A0T_TA.V0jBe7o-OahMkqGDgWW0Lxq1eTKPJYix7hjKmmqaKlhdVcnT0cdOU0ahdg82Ls-Vg_NaWKas8MhahHspz18Gx2abDSwLIKbU0jcaf0LxWZkEuMmFJs5dodq0ZqQeaEldDsHe9De_V_TQwPFkcMOPYqWhx2XEb13bmFTPtxNST18Cwm_j263Y_Ouz2YNyC4uZENZDWeOXfJLy7c8jt_ToOvXEVpXj7oZN7Ik1S9bGAenTcvUDORP-gdFdJ3stLe9FmKulOlb94Y-KvP_meyIZ7Q.XPPqS5YVJu2utJcAIRTUxlBHlECGRaM5"

	//when
	test, _, err := Decode(token, aes192Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1392553211,\"sub\":\"alice\",\"nbf\":1392552611,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"586dd129-a29f-49c8-9de7-454af1155e27\",\"iat\":1392552611}")
}

func (s *TestSuite) TestDecrypt_A256KW_A256CBC_HS512(c *C) {
	//given
	token := "eyJhbGciOiJBMjU2S1ciLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIn0.91z9VM1VLIA_qyTbqeInFoit7c4PWVuQ5mHcDyNsfofDGXS1qUDdPCWRdLC8ybvJflqHej7SCjEUMxuzOtPOUOgo-8rcdeHi.rsx7FYNTunzditC8XTMJXg.k88BLb0qs8g0UnKjSq9rs2PcrhpafEaUEX2kT-wMdmviZ9UEJrECoQY7MmJgCyQYO30hnnay2psJcr_yaDhV-NpctBZ793Xf9tztLZZndIjz5omV9HjcFgheQZj4g1tbNcRLwxod5uYz-OLrKORzeROEM-wkLgHVEqs90wN98NAiyhGyVMw7CXVX5NdU2KFUacbflkJc5AcaiAZYAts1t9bo2877XLYSO1qBoI5k5QKv6ijjM8I03Uyr3H0p0tdF6EB-cdYNcxq68GvA5CTkOw.DBtOuSJTFu5AAIdcgymUR-JflpwfcXJ2AnZU8LNB3UA"

	//when
	test, _, err := Decode(token, aes256Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1392553211,\"sub\":\"alice\",\"nbf\":1392552611,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"586dd129-a29f-49c8-9de7-454af1155e27\",\"iat\":1392552611}")
}

func (s *TestSuite) TestDecrypt_A128KW_A128GCM(c *C) {
	//given
	token := "eyJhbGciOiJBMTI4S1ciLCJlbmMiOiJBMTI4R0NNIn0.T3p7Vi-P6jVWrvJIF3MYx6lyNtJOeFmL.IVVKIDU6Nlty559s.xFMzgqiPec95flp57O_TUrF8vDcZIz4zVMrnCQZSnGlLyS464A-whc7ORehjL-U8JLIHmnrD89GzXzC0h-0QG5x1QKv6_MzyeuGMvv4WQLapWtjhfU4s0xSyRswKPAKrjPWzBCi39OjIP0DDTjSmfTiQ6OsDf772Xy3RaZ5FmHHiwzhAs4X3J5foCKEZQXrMrE_H4y0f5eey_B5y_XrYjn2jlOHcBydSCW-cUi9q7f3WTQzlHgEzZMcGSg7imIuoSz6IQQ.xIRG_a_6eMNN4aBneKviEw"

	//when
	test, _, err := Decode(token, aes128Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1392553211,\"sub\":\"alice\",\"nbf\":1392552611,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"586dd129-a29f-49c8-9de7-454af1155e27\",\"iat\":1392552611}")
}

func (s *TestSuite) TestDecrypt_A192KW_A192GCM(c *C) {
	//given
	token := "eyJhbGciOiJBMTkyS1ciLCJlbmMiOiJBMTkyR0NNIn0.Rd2X_AyKle5d8IO4MQCxJ7WFiVTjzlQ-sDTpOU7dllk.i6-U-bPS1XU87QLu.vJ7Dgh1OhBjniKDOCzLNHsSvZBb6lyDZsIr2v3br6Zp7xhxhq8Cjwmyd2FGqMKpMtSzdg1Y7QSLH3qIfijjp7lH8AxSvCsE2PZTFUJKjCbYtWWmsa6WCA4LIJU8IHBGr0DVQIRh-77ru1cy5xFWGiU7yJMpZpQ62MvluqBSFNYYmzfy5BwXqYMf8n0P2BhxRl4SxvsOoyi7JZvzfCCMixPJm50u65aak83NEIInqN-76AkekWOP4KY57HyKSYs3Fk7lUgw.rN3AZ46DWMtVZa42lnnIHg"

	//when
	test, _, err := Decode(token, aes192Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1392553211,\"sub\":\"alice\",\"nbf\":1392552611,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"586dd129-a29f-49c8-9de7-454af1155e27\",\"iat\":1392552611}")
}

func (s *TestSuite) TestDecrypt_A256KW_A256GCM(c *C) {
	//given
	token := "eyJhbGciOiJBMjU2S1ciLCJlbmMiOiJBMjU2R0NNIn0.LY3h_i8x7z_Duc68B-DBrKrKqQpgBbiudtRzVkEPgEsASK1QO2ycVw.KOn1Sn5NOZNVFWje.-26uGTnqc72gaUg4GYEysV9P0lApakuMihLAp7MuWXM5Q267oOVjSd-YeFBvN94A8ZvhFPrNXRcALt3FDOVoEswYy9ryloJu-YbsPt_Gcpnvk8_6OJKDU6xKPWRMCCRDyP88hZ9sMGdV12bWqAQA7LZMGKa-KU0_DUeLeWVew_uoKmR1TqbWBw9_Cmqy5qFztP0zOTACzNQQb8xXij1EZfhPTBQXXDQc4tawGNysEif7SWtlDDQ0row38SsApFtU1saB5g.lPTSTfaMmpNbYp9Yg9YgpQ"

	//when
	test, _, err := Decode(token, aes256Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1392553211,\"sub\":\"alice\",\"nbf\":1392552611,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"586dd129-a29f-49c8-9de7-454af1155e27\",\"iat\":1392552611}")
}

func (s *TestSuite) TestEncrypt_A128KW_A128CBC_HS256(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, A128KW, A128CBC_HS256, aes128Key)

	fmt.Printf("\nA128KW A128CBC_HS256 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJBMTI4S1ciLCJlbmMiOiJBMTI4Q0JDLUhTMjU2In0")
	c.Assert(len(parts[1]), Equals, 54)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes128Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_A192KW_A192CBC_HS384(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, A192KW, A192CBC_HS384, aes192Key)

	fmt.Printf("\nA192KW A192CBC_HS384 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJBMTkyS1ciLCJlbmMiOiJBMTkyQ0JDLUhTMzg0In0")
	c.Assert(len(parts[1]), Equals, 75)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 32)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes192Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_A256KW_A256CBC_HS512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, A256KW, A256CBC_HS512, aes256Key)

	fmt.Printf("\nA256KW A256CBC_HS512 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJBMjU2S1ciLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIn0")
	c.Assert(len(parts[1]), Equals, 96)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 43)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes256Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_A128KW_A128GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, A128KW, A128GCM, aes128Key)

	fmt.Printf("\nA128KW A128GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJBMTI4S1ciLCJlbmMiOiJBMTI4R0NNIn0")
	c.Assert(len(parts[1]), Equals, 32)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes128Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_A192KW_A192GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, A192KW, A192GCM, aes192Key)

	fmt.Printf("\nA192KW A192GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJBMTkyS1ciLCJlbmMiOiJBMTkyR0NNIn0")
	c.Assert(len(parts[1]), Equals, 43)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes192Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_A256KW_A256GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, A256KW, A256GCM, aes256Key)

	fmt.Printf("\nA256KW A256GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJBMjU2S1ciLCJlbmMiOiJBMjU2R0NNIn0")
	c.Assert(len(parts[1]), Equals, 54)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes256Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecrypt_A128GCMKW_A128CBC_HS256(c *C) {
	//given
	token := "eyJhbGciOiJBMTI4R0NNS1ciLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiaXYiOiJ1SDVxVThlN2JVZXhGYWh3IiwidGFnIjoiamdxc2czdHoyUGo0QmhEWU1xTnBrdyJ9.peAzKiVO3_w2tAlSzRZdqqQpnUSpgPDHi_xgTd6VzP4.o8bhvYO_UTkrsxQmm__nIg.MSmgetpjXHWMs0TyuGgmWd-msfbQ7oVWC4WuCJcfAsbhLU9kLDLrd0naL5f_UkWBaM04bfcc31K4FRN20IiUxcHzLnMR-lY-HkvRFWYdur-kLWw1UXjIlPOb0nqCuyd2FRpxMdSfFnYr5Us9T45cF7DdK8p4iA7KqPToMHWBsvAcET_ycMIoERqJrBuiJzh-j7UtDzH6KtUfgD4tzZAm3iM6HWT2lq25Pqsu4qf19LYXxZaMIiFwFKboeexkJ5E0hc7P-wIeknzFJaZhkb5P4g.dTQAed1znLHX4cO-VDgxeA"

	//when
	test, _, err := Decode(token, aes128Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_A128GCMKW_A256CBC_HS512(c *C) {
	//given
	token := "eyJhbGciOiJBMTI4R0NNS1ciLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIiwiaXYiOiI5bUwxR1YzUUZIWGtVbEdUIiwidGFnIjoiU0xrTDVpdmhncy1HNjRBTS01bTBxdyJ9.S3_MudWEzKWCp8RRxIG5p6H2YOtMDCkOXXKM9J8J4lMX5N2CcUqsKkDQ4TE1rG7gD5qYgHsb8AiQFLbhjgDeAA.WiOHBPlws9hImQr6bZ8h5Q.jN9UbuvhTiS6uJi1jc0TsvpheXqHs8vdJzKOUVgFmVHZ_OG4vSNRLx408vSoAgSeqsRmj8C8i9Yi2R6kpgtRXZ-Rw7EQEjZ65kg2uwZuve1ObqK-uBm3UzDmcT_Jh6myp9Df1m28ng8ojfrY_JUz6oE5yEcJdlm7H8ahipJyznWOjFigOqhaiXosjW0kbGGpYE-njD5OX22vR5k0RxHlMCDAH2ONR69kaWbLQvDg7y4yMFSxi3ILUFSVz4uXo6qlb8RVCqMUWzlGho-5Cy9OPA.XQ0UmHH5btv14_km6CIlIUwzOFj-rQUYyEzF9VY0r70"

	//when
	test, _, err := Decode(token, aes128Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_A192GCMKW_A192CBC_HS384(c *C) {
	//given
	token := "eyJhbGciOiJBMTkyR0NNS1ciLCJlbmMiOiJBMTkyQ0JDLUhTMzg0IiwiaXYiOiJzRHRLdnRzZVk2UkFuU2twIiwidGFnIjoiZDFDS3dKWnlXSnlvcW5HTUFwbmR6dyJ9.2L9u7vV0P8bZddbkCKKe6_C5JTLf8wRZC8xzEe4gvmcGoF2K5AledhcqT6mIlaPx.1JY51r77jimrvKxts9EroQ.922BMD0HOscwZxn4pmYTRgV7oshegQ1dooU9njhonPcp46XbegdfsgeZAACVFpCc_CoY_XzOsM5trH1Z30QUDc7IGJmC0NKuPdK2KkrYQPXJAe6nuZMembGsyRkOHahtj7sew-ULZn9y0ztbntPqm5I9O716mv1Cu6_5_mBYu36c_VVd6jlzueUWun09yLDJLFuf5jRXDrqRrY4t6XIcqti8LF-QLowU_pa5DvRV_KzCtD_S8HvzJ217_TI9Y1qaApgvWr_BxDrfTXxO2xaZ2Q.0fnvCkg_ChWuf8F3KY8KUgbdIzifb_JT"

	//when
	test, _, err := Decode(token, aes192Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_A256GCMKW_A256CBC_HS512(c *C) {
	//given
	token := "eyJhbGciOiJBMjU2R0NNS1ciLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIiwiaXYiOiJvUV9xbDNJUHNibEVPaDBXIiwidGFnIjoieTllMEFfY1hZMnNDZ24tamxsNl9TdyJ9.K5BxtxcV0simNM-69RvjZuNBjxaavDVnBzP7EFXSjbWZi3NjZFoTcFljcu2TuzR_F9zdjjBbohEgaf4kUMVZfg.881rEerOD33OLCHKdTWDjQ.LvrzsNicH2slBjwERYFu-Fr4Bus2lcLTdFazEpsHc_0QH4NJ2tGrJJjByli6OaFOwtdWONEu_3Ax8xvEXWHc0WMhYKxaVLZI1HQwE0NnWyqfF9mtOkUCCXn9ljvSGSDQY5VUcupVUT6WQxAkaNe6mJ6qkJOxE4pBpiMskO0luW5PkPexk2N3bJVz-GwzMp3xVT6wtFimThucZm2V71594NPCKIkA0HvtBkW0gW0M66pSTfQTHkU0Uvm7WfRvr6TXpiuKntJUe7RX5pXFXbfN2g.aW8OWGfHFI5zTGfFyKuqeLFT5o0tleSYbpCb7kAv1Bs"

	//when
	test, _, err := Decode(token, aes256Key)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestEncrypt_A128GCMKW_A128CBC_HS256(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, A128GCMKW, A128CBC_HS256, aes128Key)

	fmt.Printf("\nA128GCMKW A128CBC_HS256 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 128)
	c.Assert(len(parts[1]), Equals, 43)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes128Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_A128GCMKW_A256CBC_HS512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, A128GCMKW, A256CBC_HS512, aes128Key)

	fmt.Printf("\nA128GCMKW A256CBC_HS512= %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 128)
	c.Assert(len(parts[1]), Equals, 86)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 43)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes128Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_A192GCMKW_A192CBC_HS384(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, A192GCMKW, A192CBC_HS384, aes192Key)

	fmt.Printf("\nA192GCMKW A192CBC_HS384= %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 128)
	c.Assert(len(parts[1]), Equals, 64)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 32)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes192Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_A256GCMKW_A256CBC_HS512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, A256GCMKW, A256CBC_HS512, aes256Key)

	fmt.Printf("\nA256GCMKW A256CBC_HS512= %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 128)
	c.Assert(len(parts[1]), Equals, 86)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 43)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, aes256Key)
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecrypt_PBSE2_HS256_A128KW_A128CBC_HS256(c *C) {
	//given
	token := "eyJhbGciOiJQQkVTMi1IUzI1NitBMTI4S1ciLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwicDJjIjo4MTkyLCJwMnMiOiJiMFlFVmxMemtaNW9UUjBMIn0.dhPAhJ9kmaEbP-02VtEoPOF2QSEYM5085V6zYt1U1qIlVNRcHTGDgQ.4QAAq0dVQT41dQKDG7dhRA.H9MgJmesbU1ow6GCa0lEMwv8A_sHvgaWKkaMcdoj_z6O8LaMSgquxA-G85R_5hEILnHUnFllNJ48oJY7VmAJw0BQW73dMnn58u161S6Ftq7Mjxxq7bcksWvFTVtG5RsqqYSol5BZz5xm8Fcj-y5BMYMvrsCyQhYdeGEHkAvwzRdvZ8pGMsU2XPzl6GqxGjjuRh2vApAeNrj6MwKuD-k6AR0MH46EiNkVCmMkd2w8CNAXjJe9z97zky93xbxlOLozaC3NBRO2Q4bmdGdRg5y4Ew.xNqRi0ouQd7uo5UrPraedg"

	//when
	test, _, err := Decode(token, "top secret")

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_PBSE2_HS384_A192KW_A192CBC_HS384(c *C) {
	//given
	token := "eyJhbGciOiJQQkVTMi1IUzM4NCtBMTkyS1ciLCJlbmMiOiJBMTkyQ0JDLUhTMzg0IiwicDJjIjo4MTkyLCJwMnMiOiIxZEdaODBpQTBqb3lGTzFqIn0.iElgf12HbQWt3enumKP_j3WDxGLfbwSePHYAbYEb_w3himk0swcdiTPo1Jm8MU7le7L_Z8rU2Uk.7LoW9-g7U8c3GNAYO3Z5Jw.guSjXuYN9deq6XIsbkbxAptU9Lp1jf9k11QdhsvjfUvaZRXKrWiE9vg3jEJRJnmF7lZq07cp2Ou8PztMg6R_ygT7gadmP_IYdgQwXD6HGQs__uzvFnqtjWALiwLWuL0V0INrKxBn3CivJ5Hg26nJwLACdVuO_k-fNTaphbox-nKefndS4UXaoe3hEuCzHFPgFivMlND4aZJb8pU8sQbGA29gx5U9qNBmWYOXwV2diYQ2q2SfUEbXoMV7uZyvfQ2juTcyqZBVnEfIYGf_8esALQ.QrgRr0TIlJDFkq2YWNXcoFoMpg4yMC6r"

	//when
	test, _, err := Decode(token, "top secret")

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_PBSE2_HS512_A256KW_A256CBC_HS512(c *C) {
	//given
	token := "eyJhbGciOiJQQkVTMi1IUzUxMitBMjU2S1ciLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIiwicDJjIjo4MTkyLCJwMnMiOiJCUlkxQ1M3VXNpaTZJNzhkIn0.ovjAL7yRnB_XdJbK8lAaUDRZ-CyVeio8f4pnqOt1FPj1PoQAdEX3S5x6DlzR8aqN_WR5LUwdqDSyUDYhSurnmq8VLfzd3AEe.YAjH6g_zekXJIlPN4Ooo5Q.tutaltxpeVyayXZ9pQovGXTWTf_GWWvtu25Jeg9jgoH0sUX9KCnL00A69e4GJR6EMxalmWsa45AItffbwjUBmwdyklC4ZbTgaovVRs-UwqsZFBO2fpEb7qLajjwra7o4OegzgXDD0jhrKrUusvRWGBvenvumb5euibUxmIfBUcVF1JbdfYxx7ztFeS-QKJpDkE00zyEkViq-QxfrMVl5p7LGmTz8hMrFL3LXLokypZSDgFBfsUzChJf3mlYzxiGaGUqhs7NksQJDoUYf6prPow.XwRVfVTTPogO74RnxZD_9Mse26fTSehna1pbWy4VHfY"

	//when
	test, _, err := Decode(token, "top secret")

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_PBSE2_HS512_A256KW_A256CBC_HS512_MinIterationViolation(c *C) {
	//given
	pbes2Hs512 := DeregisterJwa(PBES2_HS512_A256KW)
	RegisterJwa(NewPbse2HmacAesKWAlg(256, 300000, 10000))
	token := "eyJhbGciOiJQQkVTMi1IUzUxMitBMjU2S1ciLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIiwicDJjIjo4MTkyLCJwMnMiOiJCUlkxQ1M3VXNpaTZJNzhkIn0.ovjAL7yRnB_XdJbK8lAaUDRZ-CyVeio8f4pnqOt1FPj1PoQAdEX3S5x6DlzR8aqN_WR5LUwdqDSyUDYhSurnmq8VLfzd3AEe.YAjH6g_zekXJIlPN4Ooo5Q.tutaltxpeVyayXZ9pQovGXTWTf_GWWvtu25Jeg9jgoH0sUX9KCnL00A69e4GJR6EMxalmWsa45AItffbwjUBmwdyklC4ZbTgaovVRs-UwqsZFBO2fpEb7qLajjwra7o4OegzgXDD0jhrKrUusvRWGBvenvumb5euibUxmIfBUcVF1JbdfYxx7ztFeS-QKJpDkE00zyEkViq-QxfrMVl5p7LGmTz8hMrFL3LXLokypZSDgFBfsUzChJf3mlYzxiGaGUqhs7NksQJDoUYf6prPow.XwRVfVTTPogO74RnxZD_9Mse26fTSehna1pbWy4VHfY"

	//when
	test, headers, err := Decode(token, "top secret")
	fmt.Printf("\np2c min iteration err= %v\n", err)

	//then
	RegisterJwa(pbes2Hs512)
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
	c.Assert(headers, IsNil)
}

func (s *TestSuite) TestDecrypt_PBSE2_HS512_A256KW_A256CBC_HS512_MaxIterationViolation(c *C) {
	//given
	pbes2Hs512 := DeregisterJwa(PBES2_HS512_A256KW)
	RegisterJwa(NewPbse2HmacAesKWAlg(256, 8000, 0))
	token := "eyJhbGciOiJQQkVTMi1IUzUxMitBMjU2S1ciLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIiwicDJjIjo4MTkyLCJwMnMiOiJCUlkxQ1M3VXNpaTZJNzhkIn0.ovjAL7yRnB_XdJbK8lAaUDRZ-CyVeio8f4pnqOt1FPj1PoQAdEX3S5x6DlzR8aqN_WR5LUwdqDSyUDYhSurnmq8VLfzd3AEe.YAjH6g_zekXJIlPN4Ooo5Q.tutaltxpeVyayXZ9pQovGXTWTf_GWWvtu25Jeg9jgoH0sUX9KCnL00A69e4GJR6EMxalmWsa45AItffbwjUBmwdyklC4ZbTgaovVRs-UwqsZFBO2fpEb7qLajjwra7o4OegzgXDD0jhrKrUusvRWGBvenvumb5euibUxmIfBUcVF1JbdfYxx7ztFeS-QKJpDkE00zyEkViq-QxfrMVl5p7LGmTz8hMrFL3LXLokypZSDgFBfsUzChJf3mlYzxiGaGUqhs7NksQJDoUYf6prPow.XwRVfVTTPogO74RnxZD_9Mse26fTSehna1pbWy4VHfY"

	//when
	test, headers, err := Decode(token, "top secret")
	fmt.Printf("\np2c max iteration err= %v\n", err)

	//then
	RegisterJwa(pbes2Hs512)
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
	c.Assert(headers, IsNil)
}

func (s *TestSuite) TestEncrypt_PBSE2_HS256_A128KW_A128GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, PBES2_HS256_A128KW, A128GCM, "top secret")

	fmt.Printf("\nPBES2-HS256+A128KW A128GCM= %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 107)
	c.Assert(len(parts[1]), Equals, 32)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, "top secret")
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_PBSE2_HS384_A192KW_A192GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, PBES2_HS384_A192KW, A192GCM, "top secret")

	fmt.Printf("\nPBES2-HS384+A192KW A192GCM= %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 107)
	c.Assert(len(parts[1]), Equals, 43)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, "top secret")
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_PBSE2_HS512_A256KW_A256GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, PBES2_HS512_A256KW, A256GCM, "top secret")

	fmt.Printf("\nPBES2-HS512+A256KW A256GCM= %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 107)
	c.Assert(len(parts[1]), Equals, 54)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, "top secret")
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_PBSE2_HS256_A128KW_A256CBC_HS512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, PBES2_HS256_A128KW, A256CBC_HS512, "top secret")

	fmt.Printf("\nPBES2-HS256+A128KW A256CBC_HS512= %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 115)
	c.Assert(len(parts[1]), Equals, 96)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 43)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, "top secret")
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_PBES2_HS512_A256KW_A256CBC_HS512_Custom_p2c(c *C) {
	// given
	payload := `{"exp":1389189552,"sub":"alice","nbf":1389188952,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"e543edf6-edf0-4348-8940-c4e28614d463","iat":1389188952}"`

	// when
	test, err := Encrypt(payload, PBES2_HS256_A128KW, A256CBC_HS512, "top secret", Header("p2c", 10000))

	fmt.Printf("\nPBES2-HS256+A128KW A256CBC_HS512 (custom p2c=10000)= %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 116)
	c.Assert(len(parts[1]), Equals, 96)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 278)
	c.Assert(len(parts[4]), Equals, 43)

	// ensure custom 'p2c' header have been included
	v, _ := base64url.Decode(parts[0])
	c.Assert(string(v), Matches, `.*"p2c":10000.*`)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, "top secret")
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_PBES2_HS512_A256KW_A256CBC_HS512_MaxIterationViolation(c *C) {
	// given
	pbes2Hs512 := DeregisterJwa(PBES2_HS512_A256KW)
	RegisterJwa(NewPbse2HmacAesKWAlg(256, 8000, 0))

	payload := `{"hello": "world"}`

	// when
	test, err := Encrypt(payload, PBES2_HS512_A256KW, A256CBC_HS512, "top secret", Header("p2c", 10000))

	fmt.Printf("\nTestEncrypt_PBES2_HS512_A256KW_A256CBC_HS512_MaxIterationViolation, err = %v\n", err)

	//then
	RegisterJwa(pbes2Hs512)
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestEncrypt_PBES2_HS512_A256KW_A256CBC_HS512_MinIterationViolation(c *C) {
	// given
	pbes2Hs512 := DeregisterJwa(PBES2_HS512_A256KW)
	RegisterJwa(NewPbse2HmacAesKWAlg(256, 800000, 300000))

	payload := `{"hello": "world"}`

	// when
	test, err := Encrypt(payload, PBES2_HS512_A256KW, A256CBC_HS512, "top secret", Header("p2c", 10000))

	fmt.Printf("\nTestEncrypt_PBES2_HS512_A256KW_A256CBC_HS512_MinIterationViolation, err = %v\n", err)

	//then
	RegisterJwa(pbes2Hs512)
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecrypt_ECDH_ES_A128CBC_HS256(c *C) {
	//given
	token := "eyJhbGciOiJFQ0RILUVTIiwiZW5jIjoiQTEyOENCQy1IUzI1NiIsImVwayI6eyJrdHkiOiJFQyIsIngiOiItVk1LTG5NeW9IVHRGUlpGNnFXNndkRm5BN21KQkdiNzk4V3FVMFV3QVhZIiwieSI6ImhQQWNReTgzVS01Qjl1U21xbnNXcFZzbHVoZGJSZE1nbnZ0cGdmNVhXTjgiLCJjcnYiOiJQLTI1NiJ9fQ..UA3N2j-TbYKKD361AxlXUA.XxFur_nY1GauVp5W_KO2DEHfof5s7kUwvOgghiNNNmnB4Vxj5j8VRS8vMOb51nYy2wqmBb2gBf1IHDcKZdACkCOMqMIcpBvhyqbuKiZPLHiilwSgVV6ubIV88X0vK0C8ZPe5lEyRudbgFjdlTnf8TmsvuAsdtPn9dXwDjUR23bD2ocp8UGAV0lKqKzpAw528vTfD0gwMG8gt_op8yZAxqqLLljMuZdTnjofAfsW2Rq3Z6GyLUlxR51DAUlQKi6UpsKMJoXTrm1Jw8sXBHpsRqA.UHCYOtnqk4SfhAknCnymaQ"

	//when
	test, _, err := Decode(token, Ecc256Private())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_ECDH_ES_A192CBC_HS384(c *C) {
	//given
	token := "eyJhbGciOiJFQ0RILUVTIiwiZW5jIjoiQTE5MkNCQy1IUzM4NCIsImVwayI6eyJrdHkiOiJFQyIsIngiOiJPaFdQMHdYQy10YnotZEZPbzJUZGljZ2dBWnNPUm1CMTFwTHd0T1lGamhTS0diSEo2U05lRVU3akNzeXNRVzJZIiwieSI6IlNRdlFrbnJoRjk5RzJ2aHhkNE82Y1NQb2lOLVhTbm5LMS11UmM4LXZpS2lHTXJIQzkyXzZyeWRPeEtlN0JPdDgiLCJjcnYiOiJQLTM4NCJ9fQ..11aQFJ3lbGxS8hKGG0-qBw.8885t-_OenCjccLenOIsX2OiTe-xGejXJuKl4AyBRugk4VsBC16GQ24kGUtDPNASXcswgJhiCJde6BNte2aq9xyeuT6RSknua7lnl7njtnU19ssz9hhXJ_wuZr_XNGYmuyM0Pb_fmNK0mA0k2vo9rcfLktTFfprdKg7iLeCHt8576g_GGF577BOc5s4Pl0dVBGdiutSdczTf67hGBoTWIr6Q-9cipLinxr9DwbOpCtceX1gFlAxsWjqswICk6Vwah56GZ7_1XqwyBsO5flNyrQ.xzEn_kq5JvwoYpuPT1ePE4QRzje1qBCT"

	//when
	test, _, err := Decode(token, Ecc384Private())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_ECDH_ES_A256CBC_HS512(c *C) {
	//given
	token := "eyJhbGciOiJFQ0RILUVTIiwiZW5jIjoiQTI1NkNCQy1IUzUxMiIsImVwayI6eyJrdHkiOiJFQyIsIngiOiJBV3JGb092UkRnWWpaUmF4dXVWMk54c1U1NVNlT1QxUTJ4V3I1ektvbUtHS2hZVmxoVFRQWEtka2MtMzM1bDBWRHlXRTlQZl9SLUpCSThtdWpEaHlSYmVfIiwieSI6IkFhbjBIMEcwRVhWaHNpaDBicU1XeGZHb2JsM2FlUTM1eVZCaG5pSmhfMERNRHFUX3hxVHRWaEtGQWlOWEIwTGJKTjJnd0NSY182OE1vS1p5R3FrV3RIVDciLCJjcnYiOiJQLTUyMSJ9fQ..uA7eYAKVhOr6nnO1bQnhpQ.G5qlUuzVAWgN_szXjFa4eAdXVPEcjpStnw4YYFm2p_lsQs6VG5huSGseV3Wgr22gdc0xHk5XgwRyihB5tr4MiXf41mVKwmxT-u9z5N236fDZ6LYYBY1YrUAp_1pOlyO7RpxpHGrZTKTeuIxhTHhs0jkVC_RZlVNlsjEhU3ssWDCJsCxetLwbq8huL1zlzwyAYqV7uPYIrp2JdT7GkZYcxrsaP8dCkPIdfORRn_qhYIwDnum2rLiYMLc8H0KmpzZKYUVxI7PVPK4oRQpv6n801A.QL0cDhvNKWmMMF1fLUXhI80zE5w5Ghm1Z3_7hGOEesQ"

	//when
	test, _, err := Decode(token, Ecc512Private())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_ECDH_ES_A128GCM(c *C) {
	//given
	token := "eyJhbGciOiJFQ0RILUVTIiwiZW5jIjoiQTEyOEdDTSIsImVwayI6eyJrdHkiOiJFQyIsIngiOiJPbDdqSWk4SDFpRTFrcnZRTmFQeGp5LXEtY3pQME40RVdPM1I3NTg0aEdVIiwieSI6Ik1kU2V1OVNudWtwOWxLZGU5clVuYmp4a3ozbV9kTWpqQXc5NFd3Q0xaa3MiLCJjcnYiOiJQLTI1NiJ9fQ..E4XwpWZ2kO-Vg0xb.lP5LWPlabtmzS-m2EPGhlPGgllLNhI5OF2nAbbV9tVvtCckKpt358IQNRk-W8-JNL9SsLdWmVUMplrw-GO-KA2qwxEeh_8-muYCw3qfdhVVhLnOF-kL4mW9a00Xls_6nIZponGrqpHCwRQM5aSr365kqTNpfOnXgJTKG2459nqv8n4oSfmwV2iRUBlXEgTO-1Tvrq9doDwZCCHj__JKvbuPfyRBp5T7d-QJio0XRF1TO4QY36GtKMXWR264lS7g-T1xxtA.vFevA9zsyOnNA5RZanKqHA"

	//when
	test, _, err := Decode(token, Ecc256Private())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestEncrypt_ECDH_ES_A128CBC_HS256(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, ECDH_ES, A128CBC_HS256, Ecc256Public())

	fmt.Printf("\nECDH-ES A128CBC_HS256= %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 230)
	c.Assert(len(parts[1]), Equals, 0)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc256Private())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_ECDH_ES_A192CBC_HS384(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, ECDH_ES, A192CBC_HS384, Ecc384Public())

	fmt.Printf("\nECDH-ES A192CBC_H384= %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 286)
	c.Assert(len(parts[1]), Equals, 0)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 32)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc384Private())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_ECDH_ES_A256CBC_HS512(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, ECDH_ES, A256CBC_HS512, Ecc512Public())

	fmt.Printf("\nECDH-ES A256CBC_HS512 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 350)
	c.Assert(len(parts[1]), Equals, 0)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 43)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc512Private())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_ECDH_ES_A128GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, ECDH_ES, A128GCM, Ecc256Public())

	fmt.Printf("\nECDH-ES A128GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 222)
	c.Assert(len(parts[1]), Equals, 0)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc256Private())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecrypt_ECDH_ES_A128KW_A128GCM(c *C) {
	//given
	token := "eyJhbGciOiJFQ0RILUVTK0ExMjhLVyIsImVuYyI6IkExMjhHQ00iLCJlcGsiOnsia3R5IjoiRUMiLCJ4IjoiNnlzVWZVd09vVWxENUpGZG9qUHFXeFd3ZkJ3b2ttWmpOVmxJRFFrcG1PMCIsInkiOiJKZVpia19QazIybWowVFUwcG5uQjNVaUwySzJJcVl6Tk0xVVRPZS1KY3dZIiwiY3J2IjoiUC0yNTYifX0.e1n3YTorJJ-H7eWby-pfGWzVx0aDScCT.VQLnlbAD3N1O-k-S.mJzcAMoxUMQxXIHFGcVjuEVKw70lC6rNbcGqverZBkycPQ2EDgZCiqMgJenHuecvG_YqShi50uZYVyYS4TTrGh1Bj4jP6iFZ8Ksww3hW_jYzKQbp9CdbmOL1f0f25RKwUq61AraXGoJ1Lrs8IM96tvTjKTGpDkNMJ8xN4kVcRcrM5fjTIx973XKo2_nbuCpn-BlAhB6wzYuw_EFsqis8-8cssPENLuGA-n-xX66akqdhycfh5RiqrTPYUnk5ss1Fo_LWWA.l0-CNccSNLTgVdGW1CZr9w"

	//when
	test, _, err := Decode(token, Ecc256Private())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_ECDH_ES_A192KW_A192GCM(c *C) {
	//given
	token := "eyJhbGciOiJFQ0RILUVTK0ExOTJLVyIsImVuYyI6IkExOTJHQ00iLCJlcGsiOnsia3R5IjoiRUMiLCJ4IjoiX1huZ0s2a1Q0XzR4dWpBZzFFZWpNZHQ3WW9sTkMySENZMDVyNUNTWUNmQVdUQ0phRi1ic1ZJUnpOY3hDeG5GUSIsInkiOiJPekl2T1NIeERjMTh5eXNqeVpQblZvaVhMZWU4OUhFUklvaVhrLW9nclVlTkFfRFFZTUFmbVFEZWtRbFk2SWZ4IiwiY3J2IjoiUC0zODQifX0.edrJoQt5wA7K11UvLxz3ExKyazayJ2O5fZVUETQh1RA.F886lZeI68UEU666.2woVI6lnFEeLNIxmlUbrTUHvV2dz3h47BVUs4ylSoYZ7OcdamP8cvdnxJY6N_kq2cxhyOzgsJAlbk4uG6UpyK0snl8fNExyNiDwyCCKCbUE8jAoGV7zBRDPioTbMIzprWt9qmBOW-pmzh8l_eiI_fPbvUulcIfqGdUK6fDifQAAjCB5RnxMFbkpVVkkcvHXVzaPnNvkZ_GLLRElInLx2H3mpFaby0a-zEK27qLzAe-t6WupS9rIKUhK0skPgeSkdRmHDxw.Qic27YS5eIoc0M4nMP365w"

	//when
	test, _, err := Decode(token, Ecc384Private())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_ECDH_ES_A256KW_A256GCM(c *C) {
	//given
	token := "eyJhbGciOiJFQ0RILUVTK0EyNTZLVyIsImVuYyI6IkEyNTZHQ00iLCJlcGsiOnsia3R5IjoiRUMiLCJ4IjoiQUlMTkMzX2lEN0hhSWFhZEU5bGZPcHI5YzBCYzV6cEtraUtkcUNpeGVubHdydlhFVDF4Y0RqbFMtSWZxSUR6VWNoekhiVFgxN0Z4ZGM2X1dOZ2std3VlUyIsInkiOiJBQlFwLWRsalVrZHlOcHFOTkxlSlEyUEZGVmplUlV4SjRGOFZYNGpOMGw4U1Q3Y0NMY0tmVVpKQmJ5b2FNbzhuX3Q1dDc3a1hSOE1aRllOcFhadFBHQTRWIiwiY3J2IjoiUC01MjEifX0.HbaOJoNm7rKdFPeNNxk5NZGWqyleUBrpme7kBfkPSNgStHua5SGqgA.qvi8Wxs7EFXTvDAI.HnPWLNveG0ypTWGWb-V4ORyshMmoR7IhaWUDKkoV9HoD-V6z9WZqAx3uX45Se2fxO9aqSVWqQbrLkw9C9A4DGGiMv27tr29m_c4jUOF_TEGWuhAUAo3Lv-srLFGxQoA3yNWsCpI5VsUVxqc7Sx1dJcEHe8DQcsYTaroNTh0cUMQuqJX9waQm2H-qFjrkLk3d9bo8rhB3x8JTi_X5uNcB1mNH51pPDtRGO41t7EFII1KmAUgjL8c6bCw__Cc6hoteely2Pg.QRsOLaELO2d2MCjpFZKsCg"

	//when
	test, _, err := Decode(token, Ecc512Private())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestEncrypt_ECDH_ES_A128KW_A128CBC_HS256(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, ECDH_ES_A128KW, A128CBC_HS256, Ecc256Public())

	fmt.Printf("\nECDH-ES+A128KW A128CBC-HS256 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 239)
	c.Assert(len(parts[1]), Equals, 54)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc256Private())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_ECDH_ES_A192KW_A192CBC_HS384(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, ECDH_ES_A192KW, A192CBC_HS384, Ecc384Public())

	fmt.Printf("\nECDH-ES+A192KW A192CBC-HS384 = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 295)
	c.Assert(len(parts[1]), Equals, 75)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 32)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc384Private())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_ECDH_ES_A256KW_A256GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, ECDH_ES_A256KW, A256GCM, Ecc512Public())

	fmt.Printf("\nECDH-ES+A256KW A265GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 351)
	c.Assert(len(parts[1]), Equals, 54)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc512Private())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecrypt_RSA_OAEP_256_A128CBC_HS256(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2In0.bje66yTjMUpyGzbt3QvPNOmCmUPowgEmoBHXw-pByhST2VBSs0_67JKDymKW0VpmQC5Qb7ZLC6nNG8YW5pxTZDOeTQLodhAvzoNAsrx4M2R_N58ZVqBPLKTq7FKi1NNd8oJ80dwWbOJ13dkLH68SlhOK5bhqKFgtbzalnglL2kq8Fki1GkN4YyFnS8-chC-mlrS5bJrPSHUF7oAsG_flL_e9-KzYqYTQgGCB3GYSo_pgalsp2rUO3Oz2Pfe9IEJNlX7R9wOT1nTT0UUg-lSzQ2oOaXNvNyaPgEa76mJ1nk7ZQq7ZNix1m8snjk0Vizd8EOFCSRyOGcp4mHMn7-s00Q.tMFMCdFNQXbhEnwE6mP_XQ.E_O_ZBtJ8P0FvhKOV_W98oxIySDgdd0up0c8FAjo-3OVZ_6XMEQYFDKVG_Zc3zkbaz1Z2hmc7D7M28RbhRdya3yJN6Hcv1KuXeZ9ociI7o739Ni_bPvv8xCmGxlASS5AF7N4JR7XjrWL-SYKGNL1p0XNTlPo3B3qYqgAY6jFNvlcjWupim-pQbWKNqPbO2KmSCtUzyKE5oHjsomH0hnQs0_DXv3cgQ_ZFLFZBc1tC4AjQ8QZex5kWg5BmlJDM5F_jD7QRhb7B1u4Mi563-AKVA.0lraw3IXMM6wPqUZVYA8pg"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_RSA_OAEP_256_A192CBC_HS384(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOiJBMTkyQ0JDLUhTMzg0In0.COuKvozBVi2vkEPpFdx0HTMpU9tmpP1lLngbmGn8RVphY-vjhVaduv8D_Ay_1j8LuMz4tgP98xWtbJkTyhxY1kBwXe0CgqFUOSJ1mTEPRkKSXpdFR7rT1Pv68qug2yKaXT_qcviyBerIcUVFbXBmtiYAosYO4kaPSOE1IvLadFOrMkxdZv6QiiCROzWgJNCCMgNQZGRoPhqLe3wrcxi86DhNO7Bpqq_yeNVyHdU_qObMuMVZIWWEQIDhiU4nE8WGJLG_NtKElc_nQwbmclL_YYgTiHsIAKWZCdj0nwfLe5mwJQN4r7pjakiUVzCbNNgI1-iBH1vJD5VCPxgWldzfYA.7cDs4wzbNDt1Kq40Q5ae4w.u1bR6ChVd90QkFIp3H6IkOCIMwf5aIKsQOvqgFangRLrDjctl5qO5jTHr1o1GwBQvAkRmaGSE7fRIwWB_l-Ayx2c2WDFOkVXFSR_D23GrWaLMLbugPItQd2Mny6H4QOzO3O0EK_Qm7frqwKQI3og72SB8DUqzEaKsrz7HR2z_qMa2CEEApxai_R6NIlAdMUbYvOfZx262MWFGrITBDmma-Mnqiz9WJUv2wexfwjROaaS4wXfkGy5B6ltESifpZZk5NerExR3GA6yX7cFqJc4pQ.FKcbLyB9eP1UXmxyliTu1_GQrnS-JtAB"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecrypt_RSA_OAEP_256_A256CBC_HS512(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIn0.Pt1q6MNdaiVWhMnY7r6DVpkYQmzyIjhb0cj10LowP_FgMu1dOQVuNwhK14MO1ki1y1Pvxouct9wwmb5gE7jNJBy6vU-FrrY62WNr_hKL3Cq2030LlJwauv1XQrEE-GCw1srxOAsw6LNT14v4f0qjeW46mIHNX4CZMEO9ntwojWsHTNsh4Qk6SU1QlS3WbbVl7gjjfqTP54j2ZwZM38s7Cs4pSAChP04UbW6Uhrm65JSi0lyg25OBXIxMEt1z9WY8lnjuh3iL_WttnFn9lf5fUuuR2N70HwANz2mxH3CxjO0ygXJtV-FhFzz3HqI2-ELrve4Igj_2f2_S6OrRTWRucA.er5K9Gk0wp3wF_sq7ib7BQ.L80B9FGSjUbEblpJ6tuiaq6NAsW89YQGD0awxtE-irKN65PT8nndBd0hlel8RRThXRF0kiYYor2GpgvVVaoOzSQcwL-aDgNO7BeRsaOL5ku2NlyT1erbg_8jEVG5BFMM0-jCb4kD0jBKWYCGoB7qs_QQxZ394H5GPwG68vlizKEa8PoaNIM0at5oFT7EHPdmGmwQyQCHR43e6uN4k28PWNxjN9Ndo5lvlYnxnAyDGVDu8lCjozaA_ZTrEPS-UBb6lOEW39CXdwVk1MgvyQfswQ.yuDMf_77Wr9Er3FG1_0FwHXJTOVQPjzBwGoKEg81mQo"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestEncrypt_RSA_OAEP_256_A128GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA_OAEP_256, A128GCM, PubKey())

	fmt.Printf("\nRSA-OAEP-256 A128GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOiJBMTI4R0NNIn0")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA_OAEP_256_A192GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA_OAEP_256, A192GCM, PubKey())

	fmt.Printf("\nRSA-OAEP-256 A192GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 51)
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncrypt_RSA_OAEP_256_A256GCM(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA_OAEP_256, A256GCM, PubKey())

	fmt.Printf("\nRSA-OAEP-256 A256GCM = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 51)
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 24)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecrypt_Deflated(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUCIsInppcCI6IkRFRiIsImVuYyI6IkExMjhDQkMtSFMyNTYifQ.nXSS9jDwE0dXkcGI7UquZBhn2nsB2P8u-YSWEuTAgEeuV54qNU4SlE76bToI1z4LUuABHmZOv9S24xkF45b7Mrap_Fu4JXH8euXrQgKQb9o_HL5FvE8m4zk5Ow13MKGPvHvWKOaNEBFriwYIfPi6QBYrpuqn0BaANc_aMyInV0Fn7e8EAgVmvoagmy7Hxic2sPUeLEIlRCDSGa82mpiGusjo7VMJxymkhnMdKufpGPh4wod7pvgb-jDWasUHpsUkHqSKZxlrDQxcy1-Pu1G37TAnImlWPa9NU7500IXc-W07IJccXhR3qhA5QaIyBbmHY0j1Dn3808oSFOYSF85A9w.uwbZhK-8iNzcjvKRb1a2Ig.jxj1GfH9Ndu1y0b7NRz_yfmjrvX2rXQczyK9ZJGWTWfeNPGR_PZdJmddiam15Qtz7R-pzIeyR4_qQoMzOISkq6fDEvEWVZdHnnTUHQzCoGX1dZoG9jXEwfAk2G1vXYT2vynEQZ72xk0V_OBtKhpIAUEFsXwCUeLAAgjFNY4OGWZl_Kmv9RTGhnePZfVbrbwg.WuV64jlV03OZm99qHMP9wQ"

	//when
	test, _, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "{\"exp\":1392963710,\"sub\":\"alice\",\"nbf\":1392963110,\"aud\":[\"https:\\/\\/app-one.com\",\"https:\\/\\/app-two.com\"],\"iss\":\"https:\\/\\/openid.net\",\"jti\":\"9fa7a38a-28fd-421c-825c-8fab3bbf3fb4\",\"iat\":1392963110}")
}

func (s *TestSuite) TestEncrypt_Deflated(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, RSA_OAEP, A256GCM, PubKey(), Zip(DEF))

	fmt.Printf("\nRSA-OAEP A256GCM DEF = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0EtT0FFUCIsImVuYyI6IkEyNTZHQ00iLCJ6aXAiOiJERUYifQ")
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 32)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, PrivKey())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecodeSignedHeader(c *C) {
	//given
	token := "eyJhbGciOiJSUzUxMiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.KP_mwCVRIxcF6ErdrzNcXZQDFGcL-Hlyocc4tIl3tJfzSfc7rz7qOLPjHpZ6UFH1ncd5TlpRc1B_pgvY-l0BNtx_s7n_QA55X4c1oeD8csrIoXQ6A6mtvdVGoSlGu2JnP6N2aqlDmlcefKqjl_Z-8nwDMGTMkDNhHKfHlIb2_Dliwxeq8LmNMREEdvNH2XVp_ffxBjiaKv2Eqbwc6I17241GCEmjDCvnagSgjX_5uu-da2H7TK2gtPJYUo8r9nzC7uzZJ5SB8suZH0COSofsP-9wvH0FESO40evCyEBylqg3bh9M9dIzeq8_bdTiC5kG93Fal44OEY8_Zm88wB_VjQ"

	//when
	_, test, err := Decode(token, PubKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, DeepEquals, map[string]interface{}{"alg": "RS512", "cty": "text/plain"})
}

func (s *TestSuite) TestDecodeEncryptedHeader(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIn0.Pt1q6MNdaiVWhMnY7r6DVpkYQmzyIjhb0cj10LowP_FgMu1dOQVuNwhK14MO1ki1y1Pvxouct9wwmb5gE7jNJBy6vU-FrrY62WNr_hKL3Cq2030LlJwauv1XQrEE-GCw1srxOAsw6LNT14v4f0qjeW46mIHNX4CZMEO9ntwojWsHTNsh4Qk6SU1QlS3WbbVl7gjjfqTP54j2ZwZM38s7Cs4pSAChP04UbW6Uhrm65JSi0lyg25OBXIxMEt1z9WY8lnjuh3iL_WttnFn9lf5fUuuR2N70HwANz2mxH3CxjO0ygXJtV-FhFzz3HqI2-ELrve4Igj_2f2_S6OrRTWRucA.er5K9Gk0wp3wF_sq7ib7BQ.L80B9FGSjUbEblpJ6tuiaq6NAsW89YQGD0awxtE-irKN65PT8nndBd0hlel8RRThXRF0kiYYor2GpgvVVaoOzSQcwL-aDgNO7BeRsaOL5ku2NlyT1erbg_8jEVG5BFMM0-jCb4kD0jBKWYCGoB7qs_QQxZ394H5GPwG68vlizKEa8PoaNIM0at5oFT7EHPdmGmwQyQCHR43e6uN4k28PWNxjN9Ndo5lvlYnxnAyDGVDu8lCjozaA_ZTrEPS-UBb6lOEW39CXdwVk1MgvyQfswQ.yuDMf_77Wr9Er3FG1_0FwHXJTOVQPjzBwGoKEg81mQo"

	//when
	_, test, err := Decode(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, DeepEquals, map[string]interface{}{"enc": "A256CBC-HS512", "alg": "RSA-OAEP-256"})
}

func (s *TestSuite) TestDecrypt_TwoPhased(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOiJBMjU2Q0JDLUhTNTEyIn0.Pt1q6MNdaiVWhMnY7r6DVpkYQmzyIjhb0cj10LowP_FgMu1dOQVuNwhK14MO1ki1y1Pvxouct9wwmb5gE7jNJBy6vU-FrrY62WNr_hKL3Cq2030LlJwauv1XQrEE-GCw1srxOAsw6LNT14v4f0qjeW46mIHNX4CZMEO9ntwojWsHTNsh4Qk6SU1QlS3WbbVl7gjjfqTP54j2ZwZM38s7Cs4pSAChP04UbW6Uhrm65JSi0lyg25OBXIxMEt1z9WY8lnjuh3iL_WttnFn9lf5fUuuR2N70HwANz2mxH3CxjO0ygXJtV-FhFzz3HqI2-ELrve4Igj_2f2_S6OrRTWRucA.er5K9Gk0wp3wF_sq7ib7BQ.L80B9FGSjUbEblpJ6tuiaq6NAsW89YQGD0awxtE-irKN65PT8nndBd0hlel8RRThXRF0kiYYor2GpgvVVaoOzSQcwL-aDgNO7BeRsaOL5ku2NlyT1erbg_8jEVG5BFMM0-jCb4kD0jBKWYCGoB7qs_QQxZ394H5GPwG68vlizKEa8PoaNIM0at5oFT7EHPdmGmwQyQCHR43e6uN4k28PWNxjN9Ndo5lvlYnxnAyDGVDu8lCjozaA_ZTrEPS-UBb6lOEW39CXdwVk1MgvyQfswQ.yuDMf_77Wr9Er3FG1_0FwHXJTOVQPjzBwGoKEg81mQo"

	//when
	test, _, err := Decode(token, func(headers map[string]interface{}, payload string) interface{} {
		//ensure that callback executed with correct arguments
		c.Assert(headers, DeepEquals, map[string]interface{}{"alg": "RSA-OAEP-256", "enc": "A256CBC-HS512"})

		return PrivKey()
	})

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecode_TwoPhased(c *C) {
	//given
	token := "eyJhbGciOiJFUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.EVnmDMlz-oi05AQzts-R3aqWvaBlwVZddWkmaaHyMx5Phb2NSLgyI0kccpgjjAyo1S5KCB3LIMPfmxCX_obMKA"

	//when
	test, _, err := Decode(token, func(headers map[string]interface{}, payload string) interface{} {
		//ensure that callback executed with correct arguments
		c.Assert(headers, DeepEquals, map[string]interface{}{"alg": "ES256", "cty": "text/plain"})
		c.Assert(payload, Equals, `{"hello": "world"}`)

		return Ecc256Public()
	})

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestDecode_TwoPhased_Error(c *C) {
	//given
	token := "eyJhbGciOiJFUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.EVnmDMlz-oi05AQzts-R3aqWvaBlwVZddWkmaaHyMx5Phb2NSLgyI0kccpgjjAyo1S5KCB3LIMPfmxCX_obMKA"

	//when
	test, _, err := Decode(token, func(headers map[string]interface{}, payload string) interface{} {
		return errors.New("Test error")
	})

	//then
	fmt.Printf("\ntwo phased err= %v\n", err)
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestSignWithExtraHeaders(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Sign(payload, ES256, Ecc256Private(), Header("keyid", "111-222-333"), Header("trans-id", "aaa-bbb"),
		Headers(map[string]interface{}{"alg": "RS256", "cty": "text/plain"}))

	fmt.Printf("\nES256 + extra headers = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 3)
	c.Assert(parts[0], Equals, "eyJhbGciOiJFUzI1NiIsImN0eSI6InRleHQvcGxhaW4iLCJrZXlpZCI6IjExMS0yMjItMzMzIiwidHJhbnMtaWQiOiJhYWEtYmJiIn0")
	c.Assert(parts[1], Equals, "eyJoZWxsbyI6ICJ3b3JsZCJ9")
	c.Assert(len(parts[2]), Equals, 86)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc256Public())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestEncryptWithExtraHeaders(c *C) {
	//given
	payload := `{"hello": "world"}`

	//when
	test, err := Encrypt(payload, ECDH_ES, A128CBC_HS256, Ecc256Public(),
		Header("keyid", "111-222-333"),
		Header("trans-id", "aaa-bbb"),
		Headers(map[string]interface{}{"alg": "RS256", "cty": "text/plain", "zip": "DEFLATE"}))

	fmt.Printf("\nECDH-ES A128CBC_HS256 + extra headers = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(len(parts[0]), Equals, 312)
	c.Assert(len(parts[1]), Equals, 0)
	c.Assert(len(parts[2]), Equals, 22)
	c.Assert(len(parts[3]), Equals, 43)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := Decode(test, Ecc256Private())
	c.Assert(t, Equals, payload)
}

func (s *TestSuite) TestDecodeMissingAlgHeader(c *C) {
	//given
	token := "eyJjdHkiOiJ0ZXh0XC9wbGFpbiJ9.eyJoZWxsbyI6ICJ3b3JsZCJ9.chIoYWrQMA8XL5nFz6oLDJyvgHk2KA4BrFGrKymjC8E"

	//when
	test, _, err := Decode(token, shaKey)

	fmt.Printf("\nmissing sign 'alg' header err= %v\n", err)

	//then
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecodeInvalidAlgHeader(c *C) {
	//given
	token := "eyJhbGciOjEyMywiY3R5IjoidGV4dFwvcGxhaW4ifQ.eyJoZWxsbyI6ICJ3b3JsZCJ9.chIoYWrQMA8XL5nFz6oLDJyvgHk2KA4BrFGrKymjC8E"

	//when
	test, _, err := Decode(token, shaKey)

	fmt.Printf("\ninvalid sign 'alg' header err= %v\n", err)

	//then
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecryptMissingAlgHeader(c *C) {
	//given
	token := "eyJlbmMiOiJBMTI4Q0JDLUhTMjU2In0.bje66yTjMUpyGzbt3QvPNOmCmUPowgEmoBHXw-pByhST2VBSs0_67JKDymKW0VpmQC5Qb7ZLC6nNG8YW5pxTZDOeTQLodhAvzoNAsrx4M2R_N58ZVqBPLKTq7FKi1NNd8oJ80dwWbOJ13dkLH68SlhOK5bhqKFgtbzalnglL2kq8Fki1GkN4YyFnS8-chC-mlrS5bJrPSHUF7oAsG_flL_e9-KzYqYTQgGCB3GYSo_pgalsp2rUO3Oz2Pfe9IEJNlX7R9wOT1nTT0UUg-lSzQ2oOaXNvNyaPgEa76mJ1nk7ZQq7ZNix1m8snjk0Vizd8EOFCSRyOGcp4mHMn7-s00Q.tMFMCdFNQXbhEnwE6mP_XQ.E_O_ZBtJ8P0FvhKOV_W98oxIySDgdd0up0c8FAjo-3OVZ_6XMEQYFDKVG_Zc3zkbaz1Z2hmc7D7M28RbhRdya3yJN6Hcv1KuXeZ9ociI7o739Ni_bPvv8xCmGxlASS5AF7N4JR7XjrWL-SYKGNL1p0XNTlPo3B3qYqgAY6jFNvlcjWupim-pQbWKNqPbO2KmSCtUzyKE5oHjsomH0hnQs0_DXv3cgQ_ZFLFZBc1tC4AjQ8QZex5kWg5BmlJDM5F_jD7QRhb7B1u4Mi563-AKVA.0lraw3IXMM6wPqUZVYA8pg"

	//when
	test, _, err := Decode(token, shaKey)

	fmt.Printf("\nmissing encrypt 'alg' header err= %v\n", err)

	//then
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecryptInvalidAlgHeader(c *C) {
	//given
	token := "eyJhbGciOiAxMTEsICJlbmMiOiJBMTI4Q0JDLUhTMjU2In0.bje66yTjMUpyGzbt3QvPNOmCmUPowgEmoBHXw-pByhST2VBSs0_67JKDymKW0VpmQC5Qb7ZLC6nNG8YW5pxTZDOeTQLodhAvzoNAsrx4M2R_N58ZVqBPLKTq7FKi1NNd8oJ80dwWbOJ13dkLH68SlhOK5bhqKFgtbzalnglL2kq8Fki1GkN4YyFnS8-chC-mlrS5bJrPSHUF7oAsG_flL_e9-KzYqYTQgGCB3GYSo_pgalsp2rUO3Oz2Pfe9IEJNlX7R9wOT1nTT0UUg-lSzQ2oOaXNvNyaPgEa76mJ1nk7ZQq7ZNix1m8snjk0Vizd8EOFCSRyOGcp4mHMn7-s00Q.tMFMCdFNQXbhEnwE6mP_XQ.E_O_ZBtJ8P0FvhKOV_W98oxIySDgdd0up0c8FAjo-3OVZ_6XMEQYFDKVG_Zc3zkbaz1Z2hmc7D7M28RbhRdya3yJN6Hcv1KuXeZ9ociI7o739Ni_bPvv8xCmGxlASS5AF7N4JR7XjrWL-SYKGNL1p0XNTlPo3B3qYqgAY6jFNvlcjWupim-pQbWKNqPbO2KmSCtUzyKE5oHjsomH0hnQs0_DXv3cgQ_ZFLFZBc1tC4AjQ8QZex5kWg5BmlJDM5F_jD7QRhb7B1u4Mi563-AKVA.0lraw3IXMM6wPqUZVYA8pg"

	//when
	test, _, err := Decode(token, shaKey)

	fmt.Printf("\ninvalid encrypt 'alg' header err= %v\n", err)

	//then
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecryptMissingEncHeader(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUC0yNTYifQ.bje66yTjMUpyGzbt3QvPNOmCmUPowgEmoBHXw-pByhST2VBSs0_67JKDymKW0VpmQC5Qb7ZLC6nNG8YW5pxTZDOeTQLodhAvzoNAsrx4M2R_N58ZVqBPLKTq7FKi1NNd8oJ80dwWbOJ13dkLH68SlhOK5bhqKFgtbzalnglL2kq8Fki1GkN4YyFnS8-chC-mlrS5bJrPSHUF7oAsG_flL_e9-KzYqYTQgGCB3GYSo_pgalsp2rUO3Oz2Pfe9IEJNlX7R9wOT1nTT0UUg-lSzQ2oOaXNvNyaPgEa76mJ1nk7ZQq7ZNix1m8snjk0Vizd8EOFCSRyOGcp4mHMn7-s00Q.tMFMCdFNQXbhEnwE6mP_XQ.E_O_ZBtJ8P0FvhKOV_W98oxIySDgdd0up0c8FAjo-3OVZ_6XMEQYFDKVG_Zc3zkbaz1Z2hmc7D7M28RbhRdya3yJN6Hcv1KuXeZ9ociI7o739Ni_bPvv8xCmGxlASS5AF7N4JR7XjrWL-SYKGNL1p0XNTlPo3B3qYqgAY6jFNvlcjWupim-pQbWKNqPbO2KmSCtUzyKE5oHjsomH0hnQs0_DXv3cgQ_ZFLFZBc1tC4AjQ8QZex5kWg5BmlJDM5F_jD7QRhb7B1u4Mi563-AKVA.0lraw3IXMM6wPqUZVYA8pg"

	//when
	test, _, err := Decode(token, shaKey)

	fmt.Printf("\nmissing encrypt 'enc' header err= %v\n", err)

	//then
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecryptInvalidEncHeader(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOjExMX0.bje66yTjMUpyGzbt3QvPNOmCmUPowgEmoBHXw-pByhST2VBSs0_67JKDymKW0VpmQC5Qb7ZLC6nNG8YW5pxTZDOeTQLodhAvzoNAsrx4M2R_N58ZVqBPLKTq7FKi1NNd8oJ80dwWbOJ13dkLH68SlhOK5bhqKFgtbzalnglL2kq8Fki1GkN4YyFnS8-chC-mlrS5bJrPSHUF7oAsG_flL_e9-KzYqYTQgGCB3GYSo_pgalsp2rUO3Oz2Pfe9IEJNlX7R9wOT1nTT0UUg-lSzQ2oOaXNvNyaPgEa76mJ1nk7ZQq7ZNix1m8snjk0Vizd8EOFCSRyOGcp4mHMn7-s00Q.tMFMCdFNQXbhEnwE6mP_XQ.E_O_ZBtJ8P0FvhKOV_W98oxIySDgdd0up0c8FAjo-3OVZ_6XMEQYFDKVG_Zc3zkbaz1Z2hmc7D7M28RbhRdya3yJN6Hcv1KuXeZ9ociI7o739Ni_bPvv8xCmGxlASS5AF7N4JR7XjrWL-SYKGNL1p0XNTlPo3B3qYqgAY6jFNvlcjWupim-pQbWKNqPbO2KmSCtUzyKE5oHjsomH0hnQs0_DXv3cgQ_ZFLFZBc1tC4AjQ8QZex5kWg5BmlJDM5F_jD7QRhb7B1u4Mi563-AKVA.0lraw3IXMM6wPqUZVYA8pg"

	//when
	test, _, err := Decode(token, shaKey)

	fmt.Printf("\ninvalid encrypt 'enc' header err= %v\n", err)

	//then
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecodeBytes_HS512(c *C) {
	//given
	token := "eyJhbGciOiJIUzUxMiIsInR5cCI6IkpXVCJ9.AAECAwQFBgcICQ.iTZ3VKv6n2JdDxMyM4qRZMuiYZOOaZ-58yCvs48vRaCRYQZmbAK6q-DWuBNEutL1LorEVGo8LZC0LyE7b8L1UA"

	//when
	test, _, err := DecodeBytes(token, shaKey)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, DeepEquals, []byte{0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9})
}

func (s *TestSuite) TestDecodeBytes_RSA_OAEP_A256GCM(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOiJBMjU2R0NNIn0.G23wC6QtVVaxoCp9ijgvbK5veMJ6YvoQW_Zdcaxb_2-cNHBbRP8E44kDRVkHXXIj_gPlm1knqK9-y-7lyxhyVbG0w71gZnfSuOKegKwXO9KpCX60dc8NbkrlTSDDey5EbSjmoqLlnllajdCdkssrF1KFPzIcnct8ecfJkhxTeKnmjis8xSfGB2sk6HP8C8eYDAEjeO5qPuYmfGwpm4BaYycbylqv4r0zZpFMOADZx2oJw3u7aFe8DL-JYAo5WbfFukg30MBHAfNNiLMu1tLRrjXvcr9i7MeHaUGgo281d9B8d7KUonbwJSwi4Ov3Lm00zrGYFE5WTgtan3vb33ndpg.tLpzdIKoJeytyZv_.RKIL2pXKTmIa3g.5SjeG9R0jmNtgdNY-sMZKw"

	//when
	test, _, err := DecodeBytes(token, PrivKey())

	//then
	c.Assert(err, IsNil)
	c.Assert(test, DeepEquals, []byte{0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9})
}

func (s *TestSuite) TestEncodeBytes_HS256(c *C) {
	//given
	payload := []byte{0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9}

	//when
	test, err := SignBytes(payload, HS256, shaKey)

	fmt.Printf("\nHS256 (bytes) = %v\n", test)

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, "eyJhbGciOiJIUzI1NiJ9.AAECAwQFBgcICQ.HmKT9jcQJZVtDWPipYjkbyy7cdCMUjyUhiTw25B49Gc")

	//make sure we consistent with ourselfs
	t, _, _ := DecodeBytes(test, shaKey)
	c.Assert(t, DeepEquals, payload)
}

func (s *TestSuite) TestEncryptBytes_RSA_OAEP_256_A128GCM(c *C) {
	//given
	payload := []byte{0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9}

	//when
	test, err := EncryptBytes(payload, RSA_OAEP_256, A128GCM, PubKey())

	fmt.Printf("\nRSA-OAEP-256 A128GCM (bytes) = %v\n", test)

	//then
	c.Assert(err, IsNil)

	parts := strings.Split(test, ".")

	c.Assert(len(parts), Equals, 5)
	c.Assert(parts[0], Equals, "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOiJBMTI4R0NNIn0")
	c.Assert(len(parts[1]), Equals, 342)
	c.Assert(len(parts[2]), Equals, 16)
	c.Assert(len(parts[3]), Equals, 14)
	c.Assert(len(parts[4]), Equals, 22)

	//make sure we consistent with ourselfs
	t, _, _ := DecodeBytes(test, PrivKey())
	c.Assert(t, DeepEquals, payload)
}

func (s *TestSuite) TestDeregisterJwa(c *C) {
	//given
	alg := DeregisterJwa(PBES2_HS256_A128KW)
	token := "eyJhbGciOiJQQkVTMi1IUzI1NitBMTI4S1ciLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwicDJjIjo4MTkyLCJwMnMiOiJiMFlFVmxMemtaNW9UUjBMIn0.dhPAhJ9kmaEbP-02VtEoPOF2QSEYM5085V6zYt1U1qIlVNRcHTGDgQ.4QAAq0dVQT41dQKDG7dhRA.H9MgJmesbU1ow6GCa0lEMwv8A_sHvgaWKkaMcdoj_z6O8LaMSgquxA-G85R_5hEILnHUnFllNJ48oJY7VmAJw0BQW73dMnn58u161S6Ftq7Mjxxq7bcksWvFTVtG5RsqqYSol5BZz5xm8Fcj-y5BMYMvrsCyQhYdeGEHkAvwzRdvZ8pGMsU2XPzl6GqxGjjuRh2vApAeNrj6MwKuD-k6AR0MH46EiNkVCmMkd2w8CNAXjJe9z97zky93xbxlOLozaC3NBRO2Q4bmdGdRg5y4Ew.xNqRi0ouQd7uo5UrPraedg"

	//when
	test, _, err := Decode(token, shaKey)

	fmt.Printf("\nunknown 'alg' header err= %v\n", err)

	//then
	RegisterJwa(alg)
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDeregisterJwe(c *C) {
	//given
	alg := DeregisterJwe(A128CBC_HS256)
	token := "eyJhbGciOiJkaXIiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2In0..3lClLoerWhxIc811QXDLbg.iFd5MNk2eWDlW3hbq7vTFLPJlC0Od_MSyWGakEn5kfYbbPk7BM_SxUMptwcvDnZ5uBKwwPAYOsHIm5IjZ79LKZul9ZnOtJONRvxWLeS9WZiX4CghOLZL7dLypKn-mB22xsmSUbtizMuNSdgJwUCxEmms7vYOpL0Che-0_YrOu3NmBCLBiZzdWVtSSvYw6Ltzbch4OAaX2ye_IIemJoU1VnrdW0y-AjPgnAUA-GY7CAKJ70leS1LyjTW8H_ecB4sDCkLpxNOUsWZs3DN0vxxSQw.bxrZkcOeBgFAo3t0585ZdQ"

	//when
	test, _, err := Decode(token, shaKey)

	fmt.Printf("\nunknown 'enc' header err= %v\n", err)

	//then
	RegisterJwe(alg)
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDeregisterJws(c *C) {
	//given
	alg := DeregisterJws(HS256)
	token := "eyJhbGciOiJIUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.chIoYWrQMA8XL5nFz6oLDJyvgHk2KA4BrFGrKymjC8E"

	//when
	test, _, err := Decode(token, shaKey)

	fmt.Printf("\nunknown 'alg' header err= %v\n", err)

	//then
	RegisterJws(alg)
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDeregisterJwc(c *C) {
	//given
	alg := DeregisterJwc(DEF)
	token := "eyJhbGciOiJSU0EtT0FFUCIsInppcCI6IkRFRiIsImVuYyI6IkExMjhDQkMtSFMyNTYifQ.nXSS9jDwE0dXkcGI7UquZBhn2nsB2P8u-YSWEuTAgEeuV54qNU4SlE76bToI1z4LUuABHmZOv9S24xkF45b7Mrap_Fu4JXH8euXrQgKQb9o_HL5FvE8m4zk5Ow13MKGPvHvWKOaNEBFriwYIfPi6QBYrpuqn0BaANc_aMyInV0Fn7e8EAgVmvoagmy7Hxic2sPUeLEIlRCDSGa82mpiGusjo7VMJxymkhnMdKufpGPh4wod7pvgb-jDWasUHpsUkHqSKZxlrDQxcy1-Pu1G37TAnImlWPa9NU7500IXc-W07IJccXhR3qhA5QaIyBbmHY0j1Dn3808oSFOYSF85A9w.uwbZhK-8iNzcjvKRb1a2Ig.jxj1GfH9Ndu1y0b7NRz_yfmjrvX2rXQczyK9ZJGWTWfeNPGR_PZdJmddiam15Qtz7R-pzIeyR4_qQoMzOISkq6fDEvEWVZdHnnTUHQzCoGX1dZoG9jXEwfAk2G1vXYT2vynEQZ72xk0V_OBtKhpIAUEFsXwCUeLAAgjFNY4OGWZl_Kmv9RTGhnePZfVbrbwg.WuV64jlV03OZm99qHMP9wQ"

	//when
	test, _, err := Decode(token, PrivKey())

	fmt.Printf("\nunknown 'zip' header err= %v\n", err)

	//then
	RegisterJwc(alg)
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecode_TwoPhased_MatchAlg(c *C) {
	//given
	token := "eyJhbGciOiJFUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.EVnmDMlz-oi05AQzts-R3aqWvaBlwVZddWkmaaHyMx5Phb2NSLgyI0kccpgjjAyo1S5KCB3LIMPfmxCX_obMKA"

	//when
	test, _, err := Decode(token, Alg(Ecc256Public(), "ES256"))

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"hello": "world"}`)
}

func (s *TestSuite) TestDecode_TwoPhased_MatchAlg_Invalid(c *C) {
	//given
	token := "eyJhbGciOiJFUzI1NiIsImN0eSI6InRleHRcL3BsYWluIn0.eyJoZWxsbyI6ICJ3b3JsZCJ9.EVnmDMlz-oi05AQzts-R3aqWvaBlwVZddWkmaaHyMx5Phb2NSLgyI0kccpgjjAyo1S5KCB3LIMPfmxCX_obMKA"

	//when
	test, headers, err := Decode(token, Alg(Ecc256Public(), "RS256"))

	fmt.Printf("\nalg doesn't match err=%v\n", err)

	//then
	c.Assert(headers, IsNil)
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecode_TwoPhased_MatchEnc(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOiJBMTkyQ0JDLUhTMzg0In0.COuKvozBVi2vkEPpFdx0HTMpU9tmpP1lLngbmGn8RVphY-vjhVaduv8D_Ay_1j8LuMz4tgP98xWtbJkTyhxY1kBwXe0CgqFUOSJ1mTEPRkKSXpdFR7rT1Pv68qug2yKaXT_qcviyBerIcUVFbXBmtiYAosYO4kaPSOE1IvLadFOrMkxdZv6QiiCROzWgJNCCMgNQZGRoPhqLe3wrcxi86DhNO7Bpqq_yeNVyHdU_qObMuMVZIWWEQIDhiU4nE8WGJLG_NtKElc_nQwbmclL_YYgTiHsIAKWZCdj0nwfLe5mwJQN4r7pjakiUVzCbNNgI1-iBH1vJD5VCPxgWldzfYA.7cDs4wzbNDt1Kq40Q5ae4w.u1bR6ChVd90QkFIp3H6IkOCIMwf5aIKsQOvqgFangRLrDjctl5qO5jTHr1o1GwBQvAkRmaGSE7fRIwWB_l-Ayx2c2WDFOkVXFSR_D23GrWaLMLbugPItQd2Mny6H4QOzO3O0EK_Qm7frqwKQI3og72SB8DUqzEaKsrz7HR2z_qMa2CEEApxai_R6NIlAdMUbYvOfZx262MWFGrITBDmma-Mnqiz9WJUv2wexfwjROaaS4wXfkGy5B6ltESifpZZk5NerExR3GA6yX7cFqJc4pQ.FKcbLyB9eP1UXmxyliTu1_GQrnS-JtAB"

	//when
	test, _, err := Decode(token, Enc(PrivKey(), "RSA-OAEP-256", "A192CBC-HS384"))

	//then
	c.Assert(err, IsNil)
	c.Assert(test, Equals, `{"exp":1392553211,"sub":"alice","nbf":1392552611,"aud":["https:\/\/app-one.com","https:\/\/app-two.com"],"iss":"https:\/\/openid.net","jti":"586dd129-a29f-49c8-9de7-454af1155e27","iat":1392552611}`)
}

func (s *TestSuite) TestDecode_TwoPhased_MatchEnc_InvalidAlg(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOiJBMTkyQ0JDLUhTMzg0In0.COuKvozBVi2vkEPpFdx0HTMpU9tmpP1lLngbmGn8RVphY-vjhVaduv8D_Ay_1j8LuMz4tgP98xWtbJkTyhxY1kBwXe0CgqFUOSJ1mTEPRkKSXpdFR7rT1Pv68qug2yKaXT_qcviyBerIcUVFbXBmtiYAosYO4kaPSOE1IvLadFOrMkxdZv6QiiCROzWgJNCCMgNQZGRoPhqLe3wrcxi86DhNO7Bpqq_yeNVyHdU_qObMuMVZIWWEQIDhiU4nE8WGJLG_NtKElc_nQwbmclL_YYgTiHsIAKWZCdj0nwfLe5mwJQN4r7pjakiUVzCbNNgI1-iBH1vJD5VCPxgWldzfYA.7cDs4wzbNDt1Kq40Q5ae4w.u1bR6ChVd90QkFIp3H6IkOCIMwf5aIKsQOvqgFangRLrDjctl5qO5jTHr1o1GwBQvAkRmaGSE7fRIwWB_l-Ayx2c2WDFOkVXFSR_D23GrWaLMLbugPItQd2Mny6H4QOzO3O0EK_Qm7frqwKQI3og72SB8DUqzEaKsrz7HR2z_qMa2CEEApxai_R6NIlAdMUbYvOfZx262MWFGrITBDmma-Mnqiz9WJUv2wexfwjROaaS4wXfkGy5B6ltESifpZZk5NerExR3GA6yX7cFqJc4pQ.FKcbLyB9eP1UXmxyliTu1_GQrnS-JtAB"

	//when
	test, _, err := Decode(token, Enc(PrivKey(), "RSA-OAEP-256", "A256CBC-HS512"))

	fmt.Printf("\nalg/enc doesn't match err=%v\n", err)

	//then
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

func (s *TestSuite) TestDecode_TwoPhased_MatchEnc_InvalidEnc(c *C) {
	//given
	token := "eyJhbGciOiJSU0EtT0FFUC0yNTYiLCJlbmMiOiJBMTkyQ0JDLUhTMzg0In0.COuKvozBVi2vkEPpFdx0HTMpU9tmpP1lLngbmGn8RVphY-vjhVaduv8D_Ay_1j8LuMz4tgP98xWtbJkTyhxY1kBwXe0CgqFUOSJ1mTEPRkKSXpdFR7rT1Pv68qug2yKaXT_qcviyBerIcUVFbXBmtiYAosYO4kaPSOE1IvLadFOrMkxdZv6QiiCROzWgJNCCMgNQZGRoPhqLe3wrcxi86DhNO7Bpqq_yeNVyHdU_qObMuMVZIWWEQIDhiU4nE8WGJLG_NtKElc_nQwbmclL_YYgTiHsIAKWZCdj0nwfLe5mwJQN4r7pjakiUVzCbNNgI1-iBH1vJD5VCPxgWldzfYA.7cDs4wzbNDt1Kq40Q5ae4w.u1bR6ChVd90QkFIp3H6IkOCIMwf5aIKsQOvqgFangRLrDjctl5qO5jTHr1o1GwBQvAkRmaGSE7fRIwWB_l-Ayx2c2WDFOkVXFSR_D23GrWaLMLbugPItQd2Mny6H4QOzO3O0EK_Qm7frqwKQI3og72SB8DUqzEaKsrz7HR2z_qMa2CEEApxai_R6NIlAdMUbYvOfZx262MWFGrITBDmma-Mnqiz9WJUv2wexfwjROaaS4wXfkGy5B6ltESifpZZk5NerExR3GA6yX7cFqJc4pQ.FKcbLyB9eP1UXmxyliTu1_GQrnS-JtAB"

	//when
	test, _, err := Decode(token, Enc(PrivKey(), "RSA-OAEP", "A192CBC-HS384"))

	fmt.Printf("\nalg/enc doesn't match err=%v\n", err)

	//then
	c.Assert(err, NotNil)
	c.Assert(test, Equals, "")
}

// test utils
func PubKey() *rsa.PublicKey {
	key, _ := Rsa.ReadPublic([]byte(pubKey))
	return key
}

func PrivKey() *rsa.PrivateKey {
	key, _ := Rsa.ReadPrivate([]byte(privKey))
	return key
}

func Ecc256Public() *ecdsa.PublicKey {
	return ecc.NewPublic([]byte{4, 114, 29, 223, 58, 3, 191, 170, 67, 128, 229, 33, 242, 178, 157, 150, 133, 25, 209, 139, 166, 69, 55, 26, 84, 48, 169, 165, 67, 232, 98, 9},
		[]byte{131, 116, 8, 14, 22, 150, 18, 75, 24, 181, 159, 78, 90, 51, 71, 159, 214, 186, 250, 47, 207, 246, 142, 127, 54, 183, 72, 72, 253, 21, 88, 53})
}

func Ecc256Private() *ecdsa.PrivateKey {
	return ecc.NewPrivate([]byte{4, 114, 29, 223, 58, 3, 191, 170, 67, 128, 229, 33, 242, 178, 157, 150, 133, 25, 209, 139, 166, 69, 55, 26, 84, 48, 169, 165, 67, 232, 98, 9},
		[]byte{131, 116, 8, 14, 22, 150, 18, 75, 24, 181, 159, 78, 90, 51, 71, 159, 214, 186, 250, 47, 207, 246, 142, 127, 54, 183, 72, 72, 253, 21, 88, 53},
		[]byte{42, 148, 231, 48, 225, 196, 166, 201, 23, 190, 229, 199, 20, 39, 226, 70, 209, 148, 29, 70, 125, 14, 174, 66, 9, 198, 80, 251, 95, 107, 98, 206})
}

func Ecc384Public() *ecdsa.PublicKey {
	return ecc.NewPublic([]byte{70, 151, 220, 179, 62, 0, 79, 232, 114, 64, 58, 75, 91, 209, 232, 128, 7, 137, 151, 42, 13, 148, 15, 133, 93, 215, 7, 3, 136, 124, 14, 101, 242, 207, 192, 69, 212, 145, 88, 59, 222, 33, 127, 46, 30, 218, 175, 79},
		[]byte{189, 202, 196, 30, 153, 53, 22, 122, 171, 4, 188, 42, 71, 2, 9, 193, 191, 17, 111, 180, 78, 6, 110, 153, 240, 147, 203, 45, 152, 236, 181, 156, 232, 223, 227, 148, 68, 148, 221, 176, 57, 149, 44, 203, 83, 85, 75, 55})
}

func Ecc384Private() *ecdsa.PrivateKey {
	return ecc.NewPrivate([]byte{70, 151, 220, 179, 62, 0, 79, 232, 114, 64, 58, 75, 91, 209, 232, 128, 7, 137, 151, 42, 13, 148, 15, 133, 93, 215, 7, 3, 136, 124, 14, 101, 242, 207, 192, 69, 212, 145, 88, 59, 222, 33, 127, 46, 30, 218, 175, 79},
		[]byte{189, 202, 196, 30, 153, 53, 22, 122, 171, 4, 188, 42, 71, 2, 9, 193, 191, 17, 111, 180, 78, 6, 110, 153, 240, 147, 203, 45, 152, 236, 181, 156, 232, 223, 227, 148, 68, 148, 221, 176, 57, 149, 44, 203, 83, 85, 75, 55},
		[]byte{137, 199, 183, 105, 188, 90, 128, 82, 116, 47, 161, 100, 221, 97, 208, 64, 173, 247, 9, 42, 186, 189, 181, 110, 24, 225, 254, 136, 75, 156, 242, 209, 94, 218, 58, 14, 33, 190, 15, 82, 141, 238, 207, 214, 159, 140, 247, 139})
}

func Ecc512Public() *ecdsa.PublicKey {
	return ecc.NewPublic([]byte{0, 248, 73, 203, 53, 184, 34, 69, 111, 217, 230, 255, 108, 212, 241, 229, 95, 239, 93, 131, 100, 37, 86, 152, 87, 98, 170, 43, 25, 35, 80, 137, 62, 112, 197, 113, 138, 116, 114, 55, 165, 128, 8, 139, 148, 237, 109, 121, 40, 205, 3, 61, 127, 28, 195, 58, 43, 228, 224, 228, 82, 224, 219, 148, 204, 96},
		[]byte{0, 60, 71, 97, 112, 106, 35, 121, 80, 182, 20, 167, 143, 8, 246, 108, 234, 160, 193, 10, 3, 148, 45, 11, 58, 177, 190, 172, 26, 178, 188, 240, 91, 25, 67, 79, 64, 241, 203, 65, 223, 218, 12, 227, 82, 178, 66, 160, 19, 194, 217, 172, 61, 250, 23, 78, 218, 130, 160, 105, 216, 208, 235, 124, 46, 32})
}

func Ecc512Private() *ecdsa.PrivateKey {
	return ecc.NewPrivate([]byte{0, 248, 73, 203, 53, 184, 34, 69, 111, 217, 230, 255, 108, 212, 241, 229, 95, 239, 93, 131, 100, 37, 86, 152, 87, 98, 170, 43, 25, 35, 80, 137, 62, 112, 197, 113, 138, 116, 114, 55, 165, 128, 8, 139, 148, 237, 109, 121, 40, 205, 3, 61, 127, 28, 195, 58, 43, 228, 224, 228, 82, 224, 219, 148, 204, 96},
		[]byte{0, 60, 71, 97, 112, 106, 35, 121, 80, 182, 20, 167, 143, 8, 246, 108, 234, 160, 193, 10, 3, 148, 45, 11, 58, 177, 190, 172, 26, 178, 188, 240, 91, 25, 67, 79, 64, 241, 203, 65, 223, 218, 12, 227, 82, 178, 66, 160, 19, 194, 217, 172, 61, 250, 23, 78, 218, 130, 160, 105, 216, 208, 235, 124, 46, 32},
		[]byte{0, 222, 129, 9, 133, 207, 123, 116, 176, 83, 95, 169, 29, 121, 160, 137, 22, 21, 176, 59, 203, 129, 62, 111, 19, 78, 14, 174, 20, 211, 56, 160, 83, 42, 74, 219, 208, 39, 231, 33, 84, 114, 71, 106, 109, 161, 116, 243, 166, 146, 252, 231, 137, 228, 99, 149, 152, 123, 201, 157, 155, 131, 181, 106, 179, 112})
}
