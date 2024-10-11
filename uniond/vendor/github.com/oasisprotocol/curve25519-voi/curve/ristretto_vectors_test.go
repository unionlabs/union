// Copyright (c) 2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package curve

import (
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/testhelpers"
)

// Test vectors taken from the IETF draft at:
// https://www.ietf.org/archive/id/draft-irtf-cfrg-ristretto255-decaf448-00.txt

func testRistrettoVectors(t *testing.T) {
	t.Run("MultiplesOfGenerator", testRistrettoVectorsMultiplesOfGenerator)
	t.Run("InvalidEncodings", testRistrettoVectorsInvalidEncodings)
	t.Run("UniformBytestrings", testRistrettoVectorsUniformBytestrings)
}

func testRistrettoVectorsMultiplesOfGenerator(t *testing.T) {
	vectors := []*CompressedRistretto{
		mustUnhexRistretto(t, "00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000"),
		mustUnhexRistretto(t, "e2f2ae0a 6abc4e71 a884a961 c500515f 58e30b6a a582dd8d b6a65945 e08d2d76"),
		mustUnhexRistretto(t, "6a493210 f7499cd1 7fecb510 ae0cea23 a110e8d5 b901f8ac add3095c 73a3b919"),
		mustUnhexRistretto(t, "94741f5d 5d52755e ce4f23f0 44ee27d5 d1ea1e2b d196b462 166b1615 2a9d0259"),
		mustUnhexRistretto(t, "da808627 73358b46 6ffadfe0 b3293ab3 d9fd53c5 ea6c9553 58f56832 2daf6a57"),
		mustUnhexRistretto(t, "e882b131 016b52c1 d3337080 187cf768 423efccb b517bb49 5ab812c4 160ff44e"),
		mustUnhexRistretto(t, "f64746d3 c92b1305 0ed8d802 36a7f000 7c3b3f96 2f5ba793 d19a601e bb1df403"),
		mustUnhexRistretto(t, "44f53520 926ec81f bd5a3878 45beb7df 85a96a24 ece18738 bdcfa6a7 822a176d"),
		mustUnhexRistretto(t, "903293d8 f2287ebe 10e2374d c1a53e0b c887e592 699f02d0 77d5263c dd55601c"),
		mustUnhexRistretto(t, "02622ace 8f7303a3 1cafc63f 8fc48fdc 16e1c8c8 d234b2f0 d6685282 a9076031"),
		mustUnhexRistretto(t, "20706fd7 88b2720a 1ed2a5da d4952b01 f413bcf0 e7564de8 cdc81668 9e2db95f"),
		mustUnhexRistretto(t, "bce83f8b a5dd2fa5 72864c24 ba1810f9 522bc600 4afe9587 7ac73241 cafdab42"),
		mustUnhexRistretto(t, "e4549ee1 6b9aa030 99ca208c 67adafca fa4c3f3e 4e5303de 6026e3ca 8ff84460"),
		mustUnhexRistretto(t, "aa52e000 df2e16f5 5fb1032f c33bc427 42dad6bd 5a8fc0be 0167436c 5948501f"),
		mustUnhexRistretto(t, "46376b80 f409b29d c2b5f6f0 c5259199 0896e571 6f41477c d30085ab 7f10301e"),
		mustUnhexRistretto(t, "e0c418f7 c8d9c4cd d7395b93 ea124f3a d99021bb 681dfc33 02a9d99a 2e53e64e"),
	}

	p := NewRistrettoPoint()
	for i, v := range vectors {
		var pCompressed CompressedRistretto
		pCompressed.SetRistrettoPoint(p)

		if pCompressed.Equal(v) != 1 {
			t.Fatalf("B[%d] != vector[%d] (Got: %v)", i, i, pCompressed)
		}

		p.Add(p, RISTRETTO_BASEPOINT_POINT)
	}
}

func testRistrettoVectorsInvalidEncodings(t *testing.T) {
	vectors := []*CompressedRistretto{
		// Non-canonical field encodings.
		mustUnhexRistretto(t, "00ffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffffff"),
		mustUnhexRistretto(t, "ffffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffff7f"),
		mustUnhexRistretto(t, "f3ffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffff7f"),
		mustUnhexRistretto(t, "edffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffff7f"),

		// Negative field elements.
		mustUnhexRistretto(t, "01000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000"),
		mustUnhexRistretto(t, "01ffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffff7f"),
		mustUnhexRistretto(t, "ed57ffd8 c914fb20 1471d1c3 d245ce3c 746fcbe6 3a3679d5 1b6a516e bebe0e20"),
		mustUnhexRistretto(t, "c34c4e18 26e5d403 b78e246e 88aa051c 36ccf0aa febffe13 7d148a2b f9104562"),
		mustUnhexRistretto(t, "c940e5a4 404157cf b1628b10 8db051a8 d439e1a4 21394ec4 ebccb9ec 92a8ac78"),
		mustUnhexRistretto(t, "47cfc549 7c53dc8e 61c91d17 fd626ffb 1c49e2bc a94eed05 2281b510 b1117a24"),
		mustUnhexRistretto(t, "f1c6165d 33367351 b0da8f6e 4511010c 68174a03 b6581212 c71c0e1d 026c3c72"),
		mustUnhexRistretto(t, "87260f7a 2f124951 18360f02 c26a470f 450dadf3 4a413d21 042b43b9 d93e1309"),

		// Non-square x^2.
		mustUnhexRistretto(t, "26948d35 ca62e643 e26a8317 7332e6b6 afeb9d08 e4268b65 0f1f5bbd 8d81d371"),
		mustUnhexRistretto(t, "4eac077a 713c57b4 f4397629 a4145982 c661f480 44dd3f96 427d40b1 47d9742f"),
		mustUnhexRistretto(t, "de6a7b00 deadc788 eb6b6c8d 20c0ae96 c2f20190 78fa604f ee5b87d6 e989ad7b"),
		mustUnhexRistretto(t, "bcab477b e20861e0 1e4a0e29 5284146a 510150d9 817763ca f1a6f4b4 22d67042"),
		mustUnhexRistretto(t, "2a292df7 e32cabab bd9de088 d1d1abec 9fc0440f 637ed2fb a145094d c14bea08"),
		mustUnhexRistretto(t, "f4a9e534 fc0d216c 44b218fa 0c42d996 35a0127e e2e53c71 2f706096 49fdff22"),
		mustUnhexRistretto(t, "8268436f 8c412619 6cf64b3c 7ddbda90 746a3786 25f9813d d9b84570 77256731"),
		mustUnhexRistretto(t, "2810e5cb c2cc4d4e ece54f61 c6f69758 e289aa7a b440b3cb eaa21995 c2f4232b"),

		// Negative xy value.
		mustUnhexRistretto(t, "3eb858e7 8f5a7254 d8c97311 74a94f76 755fd394 1c0ac937 35c07ba1 4579630e"),
		mustUnhexRistretto(t, "a45fdc55 c76448c0 49a1ab33 f17023ed fb2be358 1e9c7aad e8a61252 15e04220"),
		mustUnhexRistretto(t, "d483fe81 3c6ba647 ebbfd3ec 41adca1c 6130c2be eee9d9bf 065c8d15 1c5f396e"),
		mustUnhexRistretto(t, "8a2e1d30 050198c6 5a544831 23960ccc 38aef684 8e1ec8f5 f780e852 3769ba32"),
		mustUnhexRistretto(t, "32888462 f8b486c6 8ad7dd96 10be5192 bbeaf3b4 43951ac1 a8118419 d9fa097b"),
		mustUnhexRistretto(t, "22714250 1b9d4355 ccba2904 04bde415 75b03769 3cef1f43 8c47f8fb f35d1165"),
		mustUnhexRistretto(t, "5c37cc49 1da847cf eb9281d4 07efc41e 15144c87 6e0170b4 99a96a22 ed31e01e"),
		mustUnhexRistretto(t, "44542511 7cb8c90e dcbc7c1c c0e74f74 7f2c1efa 5630a967 c64f2877 92a48a4b"),

		// s = -1, which causes y = 0.
		mustUnhexRistretto(t, "ecffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffffff ffffff7f"),
	}

	for i, v := range vectors {
		p := NewRistrettoPoint()
		if _, err := p.SetCompressed(v); err == nil {
			t.Fatalf("failed to reject test vector[%d]", i)
		}
	}
}

func testRistrettoVectorsUniformBytestrings(t *testing.T) {
	type testVector struct {
		a, b []byte
		p    *CompressedRistretto
	}

	vectors := []testVector{
		{
			a: testhelpers.MustUnhex(t, "5d1be09e3d0c82fc538112490e35701979d99e06ca3e2b5b54bffe8b4dc772c1"),
			b: testhelpers.MustUnhex(t, "4d98b696a1bbfb5ca32c436cc61c16563790306c79eaca7705668b47dffe5bb6"),
			p: mustUnhexRistretto(t, "3066f82a 1a747d45 120d1740 f1435853 1a8f04bb ffe6a819 f86dfe50 f44a0a46"),
		},
		{
			a: testhelpers.MustUnhex(t, "f116b34b8f17ceb56e8732a60d913dd10cce47a6d53bee9204be8b44f6678b27"),
			b: testhelpers.MustUnhex(t, "0102a56902e2488c46120e9276cfe54638286b9e4b3cdb470b542d46c2068d38"),
			p: mustUnhexRistretto(t, "f26e5b6f 7d362d2d 2a94c5d0 e7602cb4 773c95a2 e5c31a64 f133189f a76ed61b"),
		},

		{
			a: testhelpers.MustUnhex(t, "8422e1bbdaab52938b81fd602effb6f89110e1e57208ad12d9ad767e2e25510c"),
			b: testhelpers.MustUnhex(t, "27140775f9337088b982d83d7fcf0b2fa1edffe51952cbe7365e95c86eaf325c"),
			p: mustUnhexRistretto(t, "006ccd2a 9e6867e6 a2c5cea8 3d3302cc 9de128dd 2a9a57dd 8ee7b9d7 ffe02826"),
		},
		{
			a: testhelpers.MustUnhex(t, "ac22415129b61427bf464e17baee8db65940c233b98afce8d17c57beeb7876c2"),
			b: testhelpers.MustUnhex(t, "150d15af1cb1fb824bbd14955f2b57d08d388aab431a391cfc33d5bafb5dbbaf"),
			p: mustUnhexRistretto(t, "f8f0c87c f237953c 5890aec3 99816900 5dae3eca 1fbb0454 8c635953 c817f92a"),
		},
		{
			a: testhelpers.MustUnhex(t, "165d697a1ef3d5cf3c38565beefcf88c0f282b8e7dbd28544c483432f1cec767"),
			b: testhelpers.MustUnhex(t, "5debea8ebb4e5fe7d6f6e5db15f15587ac4d4d4a1de7191e0c1ca6664abcc413"),
			p: mustUnhexRistretto(t, "ae81e7de df20a497 e10c304a 765c1767 a42d6e06 029758d2 d7e8ef7c c4c41179"),
		},
		{
			a: testhelpers.MustUnhex(t, "a836e6c9a9ca9f1e8d486273ad56a78c70cf18f0ce10abb1c7172ddd605d7fd2"),
			b: testhelpers.MustUnhex(t, "979854f47ae1ccf204a33102095b4200e5befc0465accc263175485f0e17ea5c"),
			p: mustUnhexRistretto(t, "e2705652 ff9f5e44 d3e841bf 1c251cf7 dddb77d1 40870d1a b2ed64f1 a9ce8628"),
		},
		{
			a: testhelpers.MustUnhex(t, "2cdc11eaeb95daf01189417cdddbf95952993aa9cb9c640eb5058d09702c7462"),
			b: testhelpers.MustUnhex(t, "2c9965a697a3b345ec24ee56335b556e677b30e6f90ac77d781064f866a3c982"),
			p: mustUnhexRistretto(t, "80bd0726 2511cdde 4863f8a7 434cef69 6750681c b9510eea 557088f7 6d9e5065"),
		},
	}

	pSame := mustUnhexRistretto(t, "30428279 1023b731 28d277bd cb5c7746 ef2eac08 dde9f298 3379cb8e 5ef0517f")
	for _, v := range []struct {
		a, b []byte
	}{
		{
			a: testhelpers.MustUnhex(t, "edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
			b: testhelpers.MustUnhex(t, "1200000000000000000000000000000000000000000000000000000000000000"),
		},
		{
			a: testhelpers.MustUnhex(t, "edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f"),
			b: testhelpers.MustUnhex(t, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
		},
		{
			a: testhelpers.MustUnhex(t, "0000000000000000000000000000000000000000000000000000000000000080"),
			b: testhelpers.MustUnhex(t, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f"),
		},
		{
			a: testhelpers.MustUnhex(t, "0000000000000000000000000000000000000000000000000000000000000000"),
			b: testhelpers.MustUnhex(t, "1200000000000000000000000000000000000000000000000000000000000080"),
		},
	} {
		vectors = append(vectors, testVector{
			a: v.a,
			b: v.b,
			p: pSame,
		})
	}

	for i, v := range vectors {
		input := append([]byte{}, v.a...)
		input = append(input, v.b...)

		p := NewRistrettoPoint()
		if _, err := p.SetUniformBytes(input); err != nil {
			t.Fatalf("failed to set vector[%d] input: %v", i, err)
		}

		var pCompressed CompressedRistretto
		pCompressed.SetRistrettoPoint(p)

		if pCompressed.Equal(v.p) != 1 {
			t.Fatalf("p.SetUniformBytes(input[%d]) != vec.p (Got: %v)", i, pCompressed)
		}
	}
}

func mustUnhexRistretto(t *testing.T, x string) *CompressedRistretto {
	b := testhelpers.MustUnhex(t, x)

	var p CompressedRistretto
	if _, err := p.SetBytes(b); err != nil {
		t.Fatalf("ristretto: failed to set bytes: %v", err)
	}

	return &p
}
