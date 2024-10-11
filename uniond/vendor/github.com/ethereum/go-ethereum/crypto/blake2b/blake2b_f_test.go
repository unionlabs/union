package blake2b

import (
	"fmt"
	"reflect"
	"testing"
)

func TestF(t *testing.T) {
	for i, test := range testVectorsF {
		t.Run(fmt.Sprintf("test vector %v", i), func(t *testing.T) {
			//toEthereumTestCase(test)

			h := test.hIn
			F(&h, test.m, test.c, test.f, test.rounds)

			if !reflect.DeepEqual(test.hOut, h) {
				t.Errorf("Unexpected result\nExpected: [%#x]\nActual:   [%#x]\n", test.hOut, h)
			}
		})
	}
}

type testVector struct {
	hIn    [8]uint64
	m      [16]uint64
	c      [2]uint64
	f      bool
	rounds uint32
	hOut   [8]uint64
}

// https://tools.ietf.org/html/rfc7693#appendix-A
var testVectorsF = []testVector{
	{
		hIn: [8]uint64{
			0x6a09e667f2bdc948, 0xbb67ae8584caa73b,
			0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
			0x510e527fade682d1, 0x9b05688c2b3e6c1f,
			0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
		},
		m: [16]uint64{
			0x0000000000636261, 0x0000000000000000, 0x0000000000000000,
			0x0000000000000000, 0x0000000000000000, 0x0000000000000000,
			0x0000000000000000, 0x0000000000000000, 0x0000000000000000,
			0x0000000000000000, 0x0000000000000000, 0x0000000000000000,
			0x0000000000000000, 0x0000000000000000, 0x0000000000000000,
			0x0000000000000000,
		},
		c:      [2]uint64{3, 0},
		f:      true,
		rounds: 12,
		hOut: [8]uint64{
			0x0D4D1C983FA580BA, 0xE9F6129FB697276A, 0xB7C45A68142F214C,
			0xD1A2FFDB6FBB124B, 0x2D79AB2A39C5877D, 0x95CC3345DED552C2,
			0x5A92F1DBA88AD318, 0x239900D4ED8623B9,
		},
	},
}
