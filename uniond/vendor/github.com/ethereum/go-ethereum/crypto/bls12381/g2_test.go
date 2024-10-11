package bls12381

import (
	"bytes"
	"crypto/rand"
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/common"
)

func (g *G2) one() *PointG2 {
	one, _ := g.fromBytesUnchecked(
		common.FromHex("" +
			"13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e" +
			"024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8" +
			"0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be" +
			"0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801",
		),
	)
	return one
}

func (g *G2) rand() *PointG2 {
	k, err := rand.Int(rand.Reader, q)
	if err != nil {
		panic(err)
	}
	return g.MulScalar(&PointG2{}, g.one(), k)
}

func TestG2Serialization(t *testing.T) {
	g2 := NewG2()
	for i := 0; i < fuz; i++ {
		a := g2.rand()
		buf := g2.ToBytes(a)
		b, err := g2.FromBytes(buf)
		if err != nil {
			t.Fatal(err)
		}
		if !g2.Equal(a, b) {
			t.Fatal("bad serialization from/to")
		}
	}
	for i := 0; i < fuz; i++ {
		a := g2.rand()
		encoded := g2.EncodePoint(a)
		b, err := g2.DecodePoint(encoded)
		if err != nil {
			t.Fatal(err)
		}
		if !g2.Equal(a, b) {
			t.Fatal("bad serialization encode/decode")
		}
	}
}

func TestG2IsOnCurve(t *testing.T) {
	g := NewG2()
	zero := g.Zero()
	if !g.IsOnCurve(zero) {
		t.Fatal("zero must be on curve")
	}
	one := new(fe2).one()
	p := &PointG2{*one, *one, *one}
	if g.IsOnCurve(p) {
		t.Fatal("(1, 1) is not on curve")
	}
}

func TestG2AdditiveProperties(t *testing.T) {
	g := NewG2()
	t0, t1 := g.New(), g.New()
	zero := g.Zero()
	for i := 0; i < fuz; i++ {
		a, b := g.rand(), g.rand()
		_, _, _ = b, t1, zero
		g.Add(t0, a, zero)
		if !g.Equal(t0, a) {
			t.Fatal("a + 0 == a")
		}
		g.Add(t0, zero, zero)
		if !g.Equal(t0, zero) {
			t.Fatal("0 + 0 == 0")
		}
		g.Sub(t0, a, zero)
		if !g.Equal(t0, a) {
			t.Fatal("a - 0 == a")
		}
		g.Sub(t0, zero, zero)
		if !g.Equal(t0, zero) {
			t.Fatal("0 - 0 == 0")
		}
		g.Neg(t0, zero)
		if !g.Equal(t0, zero) {
			t.Fatal("- 0 == 0")
		}
		g.Sub(t0, zero, a)
		g.Neg(t0, t0)
		if !g.Equal(t0, a) {
			t.Fatal(" - (0 - a) == a")
		}
		g.Double(t0, zero)
		if !g.Equal(t0, zero) {
			t.Fatal("2 * 0 == 0")
		}
		g.Double(t0, a)
		g.Sub(t0, t0, a)
		if !g.Equal(t0, a) || !g.IsOnCurve(t0) {
			t.Fatal(" (2 * a) - a == a")
		}
		g.Add(t0, a, b)
		g.Add(t1, b, a)
		if !g.Equal(t0, t1) {
			t.Fatal("a + b == b + a")
		}
		g.Sub(t0, a, b)
		g.Sub(t1, b, a)
		g.Neg(t1, t1)
		if !g.Equal(t0, t1) {
			t.Fatal("a - b == - ( b - a )")
		}
		c := g.rand()
		g.Add(t0, a, b)
		g.Add(t0, t0, c)
		g.Add(t1, a, c)
		g.Add(t1, t1, b)
		if !g.Equal(t0, t1) {
			t.Fatal("(a + b) + c == (a + c ) + b")
		}
		g.Sub(t0, a, b)
		g.Sub(t0, t0, c)
		g.Sub(t1, a, c)
		g.Sub(t1, t1, b)
		if !g.Equal(t0, t1) {
			t.Fatal("(a - b) - c == (a - c) -b")
		}
	}
}

func TestG2MultiplicativeProperties(t *testing.T) {
	g := NewG2()
	t0, t1 := g.New(), g.New()
	zero := g.Zero()
	for i := 0; i < fuz; i++ {
		a := g.rand()
		s1, s2, s3 := randScalar(q), randScalar(q), randScalar(q)
		sone := big.NewInt(1)
		g.MulScalar(t0, zero, s1)
		if !g.Equal(t0, zero) {
			t.Fatal(" 0 ^ s == 0")
		}
		g.MulScalar(t0, a, sone)
		if !g.Equal(t0, a) {
			t.Fatal(" a ^ 1 == a")
		}
		g.MulScalar(t0, zero, s1)
		if !g.Equal(t0, zero) {
			t.Fatal(" 0 ^ s == a")
		}
		g.MulScalar(t0, a, s1)
		g.MulScalar(t0, t0, s2)
		s3.Mul(s1, s2)
		g.MulScalar(t1, a, s3)
		if !g.Equal(t0, t1) {
			t.Errorf(" (a ^ s1) ^ s2 == a ^ (s1 * s2)")
		}
		g.MulScalar(t0, a, s1)
		g.MulScalar(t1, a, s2)
		g.Add(t0, t0, t1)
		s3.Add(s1, s2)
		g.MulScalar(t1, a, s3)
		if !g.Equal(t0, t1) {
			t.Errorf(" (a ^ s1) + (a ^ s2) == a ^ (s1 + s2)")
		}
	}
}

func TestG2MultiExpExpected(t *testing.T) {
	g := NewG2()
	one := g.one()
	var scalars [2]*big.Int
	var bases [2]*PointG2
	scalars[0] = big.NewInt(2)
	scalars[1] = big.NewInt(3)
	bases[0], bases[1] = new(PointG2).Set(one), new(PointG2).Set(one)
	expected, result := g.New(), g.New()
	g.MulScalar(expected, one, big.NewInt(5))
	_, _ = g.MultiExp(result, bases[:], scalars[:])
	if !g.Equal(expected, result) {
		t.Fatal("bad multi-exponentiation")
	}
}

func TestG2MultiExpBatch(t *testing.T) {
	g := NewG2()
	one := g.one()
	n := 1000
	bases := make([]*PointG2, n)
	scalars := make([]*big.Int, n)
	// scalars: [s0,s1 ... s(n-1)]
	// bases: [P0,P1,..P(n-1)] = [s(n-1)*G, s(n-2)*G ... s0*G]
	for i, j := 0, n-1; i < n; i, j = i+1, j-1 {
		scalars[j], _ = rand.Int(rand.Reader, big.NewInt(100000))
		bases[i] = g.New()
		g.MulScalar(bases[i], one, scalars[j])
	}
	// expected: s(n-1)*P0 + s(n-2)*P1 + s0*P(n-1)
	expected, tmp := g.New(), g.New()
	for i := 0; i < n; i++ {
		g.MulScalar(tmp, bases[i], scalars[i])
		g.Add(expected, expected, tmp)
	}
	result := g.New()
	_, _ = g.MultiExp(result, bases, scalars)
	if !g.Equal(expected, result) {
		t.Fatal("bad multi-exponentiation")
	}
}

func TestG2MapToCurve(t *testing.T) {
	for i, v := range []struct {
		u        []byte
		expected []byte
	}{
		{
			u:        make([]byte, 96),
			expected: common.FromHex("0a67d12118b5a35bb02d2e86b3ebfa7e23410db93de39fb06d7025fa95e96ffa428a7a27c3ae4dd4b40bd251ac658892" + "018320896ec9eef9d5e619848dc29ce266f413d02dd31d9b9d44ec0c79cd61f18b075ddba6d7bd20b7ff27a4b324bfce" + "04c69777a43f0bda07679d5805e63f18cf4e0e7c6112ac7f70266d199b4f76ae27c6269a3ceebdae30806e9a76aadf5c" + "0260e03644d1a2c321256b3246bad2b895cad13890cbe6f85df55106a0d334604fb143c7a042d878006271865bc35941"),
		},
		{
			u:        common.FromHex("025fbc07711ba267b7e70c82caa70a16fbb1d470ae24ceef307f5e2000751677820b7013ad4e25492dcf30052d3e5eca" + "0e775d7827adf385b83e20e4445bd3fab21d7b4498426daf3c1d608b9d41e9edb5eda0df022e753b8bb4bc3bb7db4914"),
			expected: common.FromHex("0d4333b77becbf9f9dfa3ca928002233d1ecc854b1447e5a71f751c9042d000f42db91c1d6649a5e0ad22bd7bf7398b8" + "027e4bfada0b47f9f07e04aec463c7371e68f2fd0c738cd517932ea3801a35acf09db018deda57387b0f270f7a219e4d" + "0cc76dc777ea0d447e02a41004f37a0a7b1fafb6746884e8d9fc276716ccf47e4e0899548a2ec71c2bdf1a2a50e876db" + "053674cba9ef516ddc218fedb37324e6c47de27f88ab7ef123b006127d738293c0277187f7e2f80a299a24d84ed03da7"),
		},
		{
			u:        common.FromHex("1870a7dbfd2a1deb74015a3546b20f598041bf5d5202997956a94a368d30d3f70f18cdaa1d33ce970a4e16af961cbdcb" + "045ab31ce4b5a8ba7c4b2851b64f063a66cd1223d3c85005b78e1beee65e33c90ceef0244e45fc45a5e1d6eab6644fdb"),
			expected: common.FromHex("18f0f87b40af67c056915dbaf48534c592524e82c1c2b50c3734d02c0172c80df780a60b5683759298a3303c5d942778" + "09349f1cb5b2e55489dcd45a38545343451cc30a1681c57acd4fb0a6db125f8352c09f4a67eb7d1d8242cb7d3405f97b" + "10a2ba341bc689ab947b7941ce6ef39be17acaab067bd32bd652b471ab0792c53a2bd03bdac47f96aaafe96e441f63c0" + "02f2d9deb2c7742512f5b8230bf0fd83ea42279d7d39779543c1a43b61c885982b611f6a7a24b514995e8a098496b811"),
		},
		{
			u:        common.FromHex("088fe329b054db8a6474f21a7fbfdf17b4c18044db299d9007af582c3d5f17d00e56d99921d4b5640fce44b05219b5de" + "0b6e6135a4cd31ba980ddbd115ac48abef7ec60e226f264d7befe002c165f3a496f36f76dd524efd75d17422558d10b4"),
			expected: common.FromHex("19808ec5930a53c7cf5912ccce1cc33f1b3dcff24a53ce1cc4cba41fd6996dbed4843ccdd2eaf6a0cd801e562718d163" + "149fe43777d34f0d25430dea463889bd9393bdfb4932946db23671727081c629ebb98a89604f3433fba1c67d356a4af7" + "04783e391c30c83f805ca271e353582fdf19d159f6a4c39b73acbb637a9b8ac820cfbe2738d683368a7c07ad020e3e33" + "04c0d6793a766233b2982087b5f4a254f261003ccb3262ea7c50903eecef3e871d1502c293f9e063d7d293f6384f4551"),
		},
		{
			u:        common.FromHex("03df16a66a05e4c1188c234788f43896e0565bfb64ac49b9639e6b284cc47dad73c47bb4ea7e677db8d496beb907fbb6" + "0f45b50647d67485295aa9eb2d91a877b44813677c67c8d35b2173ff3ba95f7bd0806f9ca8a1436b8b9d14ee81da4d7e"),
			expected: common.FromHex("0b8e0094c886487870372eb6264613a6a087c7eb9804fab789be4e47a57b29eb19b1983a51165a1b5eb025865e9fc63a" + "0804152cbf8474669ad7d1796ab92d7ca21f32d8bed70898a748ed4e4e0ec557069003732fc86866d938538a2ae95552" + "14c80f068ece15a3936bb00c3c883966f75b4e8d9ddde809c11f781ab92d23a2d1d103ad48f6f3bb158bf3e3a4063449" + "09e5c8242dd7281ad32c03fe4af3f19167770016255fb25ad9b67ec51d62fade31a1af101e8f6172ec2ee8857662be3a"),
		},
	} {
		g := NewG2()
		p0, err := g.MapToCurve(v.u)
		if err != nil {
			t.Fatal("map to curve fails", i, err)
		}
		if !bytes.Equal(g.ToBytes(p0), v.expected) {
			t.Fatal("map to curve fails", i)
		}
	}
}

func BenchmarkG2Add(t *testing.B) {
	g2 := NewG2()
	a, b, c := g2.rand(), g2.rand(), PointG2{}
	t.ResetTimer()
	for i := 0; i < t.N; i++ {
		g2.Add(&c, a, b)
	}
}

func BenchmarkG2Mul(t *testing.B) {
	worstCaseScalar, _ := new(big.Int).SetString("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", 16)
	g2 := NewG2()
	a, e, c := g2.rand(), worstCaseScalar, PointG2{}
	t.ResetTimer()
	for i := 0; i < t.N; i++ {
		g2.MulScalar(&c, a, e)
	}
}

func BenchmarkG2SWUMap(t *testing.B) {
	a := make([]byte, 96)
	g2 := NewG2()
	t.ResetTimer()
	for i := 0; i < t.N; i++ {
		_, err := g2.MapToCurve(a)
		if err != nil {
			t.Fatal(err)
		}
	}
}
