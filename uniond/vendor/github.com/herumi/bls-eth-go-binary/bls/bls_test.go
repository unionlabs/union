package bls

import (
	"crypto/rand"
	"encoding/csv"
	"encoding/hex"
	"io"
	"io/ioutil"
	"os"
	"testing"
)

func testUncompressedG1(t *testing.T, gen1 *G1) {
	buf, err := ioutil.ReadFile("tests/g1_uncompressed_valid_test_vectors.dat")
	if err != nil {
		t.Fatalf("ReadFile")
	}
	one := CastToPublicKey(gen1)
	var p1, p2 PublicKey
	for i := 0; i < 1000; i++ {
		if p1.DeserializeUncompressed(buf[i*96:(i+1)*96]) != nil {
			t.Fatalf("i=%d X.Deserialize", i)
		}
		if !p1.IsEqual(&p2) {
			t.Fatalf("i=%d p1=%x\np2=%x\n", i, p1.Serialize(), p2.Serialize())
		}
		p2.Add(one)
	}
}

func testCompressedG1(t *testing.T, gen1 *G1) {
	buf, err := ioutil.ReadFile("tests/g1_compressed_valid_test_vectors.dat")
	if err != nil {
		t.Fatalf("ReadFile")
	}
	one := CastToPublicKey(gen1)
	var p1, p2 PublicKey
	for i := 0; i < 1000; i++ {
		if p1.Deserialize(buf[i*48:(i+1)*48]) != nil {
			t.Fatalf("err i=%d\n", i)
		}
		if !p1.IsEqual(&p2) {
			t.Fatalf("p1=%x\np2=%x\n", p1.Serialize(), p2.Serialize())
		}
		p2.Add(one)
	}
}

func testUncompressedG2(t *testing.T, gen2 *G2) {
	buf, err := ioutil.ReadFile("tests/g2_uncompressed_valid_test_vectors.dat")
	if err != nil {
		t.Fatalf("ReadFile")
	}
	one := CastToSign(gen2)
	var p1, p2 Sign
	for i := 0; i < 1000; i++ {
		if p1.DeserializeUncompressed(buf[i*192:(i+1)*192]) != nil {
			t.Fatalf("i=%d X.Deserialize", i)
		}
		if !p1.IsEqual(&p2) {
			t.Fatalf("i=%d p1=%x\np2=%x\n", i, p1.Serialize(), p2.Serialize())
		}
		p2.Add(one)
	}
}

func testCompressedG2(t *testing.T, gen2 *G2) {
	buf, err := ioutil.ReadFile("tests/g2_compressed_valid_test_vectors.dat")
	if err != nil {
		t.Fatalf("ReadFile")
	}
	one := CastToSign(gen2)
	var p1, p2 Sign
	for i := 0; i < 1000; i++ {
		if p1.Deserialize(buf[i*96:(i+1)*96]) != nil {
			t.Fatalf("err i=%d\n", i)
		}
		if !p1.IsEqual(&p2) {
			t.Fatalf("p1=%x\np2=%x\n", p1.Serialize(), p2.Serialize())
		}
		p2.Add(one)
	}
}

func getSecPubHash() (*SecretKey, *PublicKey, []byte) {
	var sec SecretKey
	sec.SetByCSPRNG()
	pub := sec.GetPublicKey()
	var x Fp2
	x.D[0].SetByCSPRNG()
	x.D[1].SetByCSPRNG()
	hash := x.Serialize()
	return &sec, pub, hash
}

func testSignAndVerifyHash(t *testing.T) {
	sec, pub, hash := getSecPubHash()
	sig := sec.SignHash(hash)
	if sig == nil {
		t.Fatal("SignHash")
	}
	if !sig.VerifyHash(pub, hash) {
		t.Fatal("VerifyHash 1")
	}
	hash[0] = hash[0] + 1
	if sig.VerifyHash(pub, hash) {
		t.Fatal("VerifyHash 2")
	}
}

func getSecPubHashVec(n int) ([]PublicKey, [][]byte, []Sign) {
	pubVec := make([]PublicKey, n)
	hashVec := make([][]byte, n)
	sigVec := make([]Sign, n)
	var x Fp2
	var sec SecretKey
	for i := 0; i < n; i++ {
		sec.SetByCSPRNG()
		pubVec[i] = *sec.GetPublicKey()
		x.D[0].SetByCSPRNG()
		x.D[1].SetByCSPRNG()
		hashVec[i] = x.Serialize()
		sigVec[i] = *sec.SignHash(hashVec[i])
	}
	return pubVec, hashVec, sigVec
}

func testVerifyAggreageteHash(t *testing.T) {
	const N = 100
	pubVec, hashVec, sigVec := getSecPubHashVec(N)
	agg := sigVec[0]
	for i := 1; i < N; i++ {
		agg.Add(&sigVec[i])
	}
	if !agg.VerifyAggregateHashes(pubVec, hashVec) {
		t.Fatal("VerifyAggregateHashes 1")
	}
	hashVec[0][0] = hashVec[0][0] + 1
	if agg.VerifyAggregateHashes(pubVec, hashVec) {
		t.Fatal("VerifyAggregateHashes 2")
	}
}

func TestAreAllMsgDifferent(t *testing.T) {
	padTo32Bytes := func(b []byte) []byte {
		dst := [32]byte{}
		copy(dst[:], b)
		return dst[:]
	}
	makeMsg := func(b ...[]byte) []byte {
		var dst []byte
		for _, bb := range b {
			dst = append(dst, padTo32Bytes(bb)...)
		}
		return dst
	}
	type args struct {
		msgVec []byte
	}
	tests := []struct {
		name string
		args args
		want bool
	}{
		{
			name: "all unique",
			args: args{
				msgVec: makeMsg(
					[]byte("a"),
					[]byte("b"),
					[]byte("c"),
					[]byte("d"),
					[]byte("e"),
				),
			},
			want: true,
		}, {
			name: "some duplicates",
			args: args{
				msgVec: makeMsg(
					[]byte("a"),
					[]byte("b"),
					[]byte("c"),
					[]byte("a"),
					[]byte("a"),
				),
			},
			want: false,
		}, {
			name: "long string1",
			args: args{
				msgVec: makeMsg(
					[]byte("abcdefg"),
					[]byte("csadfasdfasereaaesfaefa"),
					[]byte("01234567890123456789012345678901"),
					[]byte("xxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxxxxxxx"),
				),
			},
			want: true,
		}, {
			name: "long string2",
			args: args{
				msgVec: makeMsg(
					[]byte("abcdefg"),
					[]byte("csadfasdfasereaaesfaefa"),
					[]byte("01234567890123456789012345678901"),
					[]byte("xxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxxxxxx"),
					[]byte("xxxxxxxxxxxxxxxxxxxxxx"),
					[]byte("01234567890123456789012345678901"),
				),
			},
			want: false,
		}, {
			name: "empty input",
			args: args{
				msgVec: []byte{},
			},
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := AreAllMsgDifferent(tt.args.msgVec); got != tt.want {
				t.Errorf("AreAllMsgDifferent() = %v, want %v", got, tt.want)
			}
		})
	}
}

func ethSignOneTest(t *testing.T, secHex string, msgHex string, sigHex string) {
	var sec SecretKey
	if sec.DeserializeHexStr(secHex) != nil {
		t.Fatalf("bad sec")
	}
	var sig Sign
	if sig.DeserializeHexStr(sigHex) != nil {
		t.Logf("bad sig %v\n", sigHex)
		return
	}
	pub := sec.GetPublicKey()
	msg, _ := hex.DecodeString(msgHex)
	sig = *sec.SignByte(msg)
	if !sig.VerifyByte(pub, msg) {
		t.Fatalf("bad verify %v %v", secHex, msgHex)
	}
	s := sig.SerializeToHexStr()
	if s != sigHex {
		t.Fatalf("bad sign\nL=%v\nR=%v\nsec=%v\nmsg=%v", s, sigHex, secHex, msgHex)
	}
}

func ethVerifyOneTest(t *testing.T, pubHex string, msgHex string, sigHex string, outStr string) {
	var pub PublicKey
	if pub.DeserializeHexStr(pubHex) != nil {
		t.Fatalf("bad pub %v", pubHex)
	}
	var sig Sign
	if sig.DeserializeHexStr(sigHex) != nil {
		t.Logf("bad sig %v\n", sigHex)
		return
	}
	msg, _ := hex.DecodeString(msgHex)
	b := sig.VerifyByte(&pub, msg)
	expect := outStr == "true"
	if b != expect {
		t.Fatalf("bad verify\nL=%v\nR=%v\npub=%v\nmsg=%v\nsig=%v\n", b, expect, pubHex, msgHex, sigHex)
	}
}

func ethSignTest(t *testing.T) {
	fileName := "tests/sign.txt"
	fp, err := os.Open(fileName)
	if err != nil {
		t.Fatalf("can't open %v %v", fileName, err)
	}
	defer fp.Close()
	reader := csv.NewReader(fp)
	reader.Comma = ' '
	for {
		secHex, err := reader.Read()
		if err == io.EOF {
			break
		}
		msgHex, _ := reader.Read()
		sigHex, _ := reader.Read()
		ethSignOneTest(t, secHex[1], msgHex[1], sigHex[1])
	}
}

func ethVerifyTest(t *testing.T) {
	fileName := "tests/verify.txt"
	fp, err := os.Open(fileName)
	if err != nil {
		t.Fatalf("can't open %v %v", fileName, err)
	}
	defer fp.Close()
	reader := csv.NewReader(fp)
	reader.Comma = ' '
	for {
		pubHex, err := reader.Read()
		if err == io.EOF {
			break
		}
		msgHex, _ := reader.Read()
		sigHex, _ := reader.Read()
		outStr, _ := reader.Read()
		ethVerifyOneTest(t, pubHex[1], msgHex[1], sigHex[1], outStr[1])
	}
}

func ethFastAggregateVerifyTest(t *testing.T) {
	fileName := "tests/fast_aggregate_verify.txt"
	fp, err := os.Open(fileName)
	if err != nil {
		t.Fatalf("can't open %v %v", fileName, err)
	}
	defer fp.Close()

	reader := csv.NewReader(fp)
	reader.Comma = ' '
	i := 0
	for {
		var pubVec []PublicKey
		var s []string
		var err error
		for {
			s, err = reader.Read()
			if err == io.EOF {
				return
			}
			if s[0] == "msg" {
				break
			}
			var pub PublicKey
			if pub.DeserializeHexStr(s[1]) != nil {
				t.Fatalf("bad signature")
			}
			pubVec = append(pubVec, pub)
		}
		t.Logf("i=%v\n", i)
		i++
		msg, _ := hex.DecodeString(s[1])
		sigHex, _ := reader.Read()
		outHex, _ := reader.Read()
		var sig Sign
		if sig.DeserializeHexStr(sigHex[1]) != nil {
			t.Logf("bad signature %v", sigHex[1])
			continue
		}
		if !sig.IsValidOrder() {
			t.Logf("bad order %v", sigHex[1])
			continue
		}
		out := outHex[1] == "true"
		if sig.FastAggregateVerify(pubVec, msg) != out {
			t.Fatalf("bad FastAggregateVerify")
		}
	}
}

func makeMultiSig(n int) (pubs []PublicKey, sigs []Sign, msgs []byte) {
	msgSize := 32
	pubs = make([]PublicKey, n)
	sigs = make([]Sign, n)
	msgs = make([]byte, n*msgSize)
	for i := 0; i < n; i++ {
		var sec SecretKey
		sec.SetByCSPRNG()
		pubs[i] = *sec.GetPublicKey()
		msgs[msgSize*i] = byte(i)
		sigs[i] = *sec.SignByte(msgs[msgSize*i : msgSize*(i+1)])
	}
	return pubs, sigs, msgs
}

func blsAggregateVerifyNoCheckTestOne(t *testing.T, n int) {
	t.Logf("blsAggregateVerifyNoCheckTestOne %v\n", n)
	pubs, sigs, msgs := makeMultiSig(n)
	if !AreAllMsgDifferent(msgs) {
		t.Fatalf("bad msgs")
	}
	var aggSig Sign
	aggSig.Aggregate(sigs)
	if !aggSig.AggregateVerifyNoCheck(pubs, msgs) {
		t.Fatalf("bad AggregateVerifyNoCheck 1")
	}
	msgs[1] = 1
	if aggSig.AggregateVerifyNoCheck(pubs, msgs) {
		t.Fatalf("bad AggregateVerifyNoCheck 2")
	}
}

func blsAggregateVerifyNoCheckTest(t *testing.T) {
	nTbl := []int{1, 2, 15, 16, 17, 50}
	for i := 0; i < len(nTbl); i++ {
		blsAggregateVerifyNoCheckTestOne(t, nTbl[i])
	}
}

func ethAggregateTest(t *testing.T) {
	fileName := "tests/aggregate.txt"
	fp, err := os.Open(fileName)
	if err != nil {
		t.Fatalf("can't open %v %v", fileName, err)
	}
	defer fp.Close()

	reader := csv.NewReader(fp)
	reader.Comma = ' '
	i := 0
	for {
		t.Logf("ethAggregateTest i=%v\n", i)
		i++
		var sigVec []Sign
		var s []string
		var err error
		for {
			s, err = reader.Read()
			if err == io.EOF {
				return
			}
			if s[0] == "out" {
				break
			}
			var sig Sign
			if sig.DeserializeHexStr(s[1]) != nil {
				t.Fatalf("bad signature")
			}
			sigVec = append(sigVec, sig)
		}
		var sig Sign
		if sig.DeserializeHexStr(s[1]) != nil {
			t.Logf("bad signature %v", s[1])
			continue
		}
		var agg Sign
		agg.Aggregate(sigVec)
		if !agg.IsEqual(&sig) {
			t.Fatalf("bad aggregate")
		}
	}
}

func ethAggregateVerifyTest(t *testing.T) {
	fileName := "tests/aggregate_verify.txt"
	fp, err := os.Open(fileName)
	if err != nil {
		t.Fatalf("can't open %v %v", fileName, err)
	}
	defer fp.Close()

	reader := csv.NewReader(fp)
	reader.Comma = ' '
	i := 0
	for {
		t.Logf("i=%v\n", i)
		i++
		var pubVec []PublicKey
		var s []string
		var err error
		for {
			s, err = reader.Read()
			if err == io.EOF {
				return
			}
			if s[0] == "msg" {
				break
			}
			var pub PublicKey
			if pub.DeserializeHexStr(s[1]) != nil {
				t.Fatalf("bad signature")
			}
			pubVec = append(pubVec, pub)
		}
		msg, _ := hex.DecodeString(s[1])
		for j := 1; j < len(pubVec); j++ {
			s, _ := reader.Read()
			b, _ := hex.DecodeString(s[1])
			msg = append(msg, b...)
		}
		sigHex, _ := reader.Read()
		outHex, _ := reader.Read()
		var sig Sign
		if sig.DeserializeHexStr(sigHex[1]) != nil {
			t.Logf("bad signature %v", sigHex[1])
			continue
		}
		out := outHex[1] == "true"
		b := sig.AggregateVerifyNoCheck(pubVec, msg)
		if b != out {
			t.Fatalf("bad AggregateVerify")
		}
	}
}

func testEthDraft07(t *testing.T) {
	secHex := "0000000000000000000000000000000000000000000000000000000000000001"
	msgHex := "61736466"
	sigHex := "b45a264e0d6f8614c4640ea97bae13effd3c74c4e200e3b1596d6830debc952602a7d210eca122dc4f596fa01d7f6299106933abd29477606f64588595e18349afe22ecf2aeeeb63753e88a42ef85b24140847e05620a28422f8c30f1d33b9aa"
	ethSignOneTest(t, secHex, msgHex, sigHex)
	ethSignTest(t)
	ethFastAggregateVerifyTest(t)
	ethVerifyTest(t)
	ethAggregateTest(t)
	ethAggregateVerifyTest(t)
	blsAggregateVerifyNoCheckTest(t)
}

func testMultiVerifyOne(n int) bool {
	pubs, sigs, msgs := makeMultiSig(n)
	return MultiVerify(sigs, pubs, msgs)
}

func testMultiVerify(t *testing.T) {
	tbl := []int{1, 2, 15, 16, 17, 50, 50, 100, 200, 400}
	for i := 0; i < len(tbl); i++ {
		n := tbl[i]
		ret := testMultiVerifyOne(n)
		if !ret {
			t.Fatalf("testMultiVerifyOne n=%v ret=%v\n", n, ret)
		}
	}
}

func testGetSafePublicKey(t *testing.T) {
	var sec SecretKey
	pub, err := sec.GetSafePublicKey()
	if pub != nil || err == nil {
		t.Fatalf("sec must be zero")
	}
	sec.SetByCSPRNG()
	pub, err = sec.GetSafePublicKey()
	if pub == nil || err != nil {
		t.Fatalf("sec must be non-zero")
	}
}

func Test(t *testing.T) {
	if Init(BLS12_381) != nil {
		t.Fatalf("Init")
	}
	var gen1 G1
	if gen1.SetString("1 0x17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb 0x08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1", 16) != nil {
		t.Fatalf("gen1.SetString")
	}
	var gen2 G2
	if gen2.SetString("1 0x024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8 0x13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e 0x0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801 0x0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be", 16) != nil {
		t.Fatalf("gen2.SetString")
	}

	testUncompressedG1(t, &gen1)
	testCompressedG1(t, &gen1)
	testUncompressedG2(t, &gen2)
	testCompressedG2(t, &gen2)
	testSignAndVerifyHash(t)
	testVerifyAggreageteHash(t)
	testEthDraft07(t)
	testMultiVerify(t)
	testGetSafePublicKey(t)
}

func BenchmarkPairing(b *testing.B) {
	b.StopTimer()
	err := Init(BLS12_381)
	if err != nil {
		b.Fatal(err)
	}
	var P G1
	var Q G2
	var e GT
	P.HashAndMapTo([]byte("abc"))
	Q.HashAndMapTo([]byte("abc"))
	b.StartTimer()
	for n := 0; n < b.N; n++ {
		Pairing(&e, &P, &Q)
	}
	b.StopTimer()
}

func BenchmarkSignHash(b *testing.B) {
	b.StopTimer()
	err := Init(BLS12_381)
	if err != nil {
		b.Fatal(err)
	}
	sec, _, hash := getSecPubHash()
	b.StartTimer()
	for n := 0; n < b.N; n++ {
		sec.SignHash(hash)
	}
	b.StopTimer()
}

func BenchmarkVerifyHash(b *testing.B) {
	b.StopTimer()
	err := Init(BLS12_381)
	if err != nil {
		b.Fatal(err)
	}
	sec, pub, hash := getSecPubHash()
	sig := sec.SignHash(hash)
	b.StartTimer()
	for n := 0; n < b.N; n++ {
		sig.VerifyHash(pub, hash)
	}
	b.StopTimer()
}

func BenchmarkVerifyAggreageteHash(b *testing.B) {
	b.StopTimer()
	err := Init(BLS12_381)
	if err != nil {
		b.Fatal(err)
	}
	const N = 50
	pubVec, hashVec, sigVec := getSecPubHashVec(N)
	agg := sigVec[0]
	for i := 1; i < N; i++ {
		agg.Add(&sigVec[i])
	}
	b.StartTimer()
	for n := 0; n < b.N; n++ {
		agg.VerifyAggregateHashes(pubVec, hashVec)
	}
}

func BenchmarkDeserialization1(b *testing.B) {
	b.StopTimer()
	err := Init(BLS12_381)
	if err != nil {
		b.Fatal(err)
	}
	VerifyOrderG1(false)
	VerifyOrderG2(false)
	const N = 50
	sec, _, hash := getSecPubHash()
	sig := sec.SignHash(hash)
	buf := sig.Serialize()
	b.StartTimer()
	for n := 0; n < b.N; n++ {
		sig.Deserialize(buf)
	}
}

func BenchmarkDeserialization2(b *testing.B) {
	b.StopTimer()
	err := Init(BLS12_381)
	if err != nil {
		b.Fatal(err)
	}
	VerifyOrderG1(true)
	VerifyOrderG2(true)
	const N = 50
	sec, _, hash := getSecPubHash()
	sig := sec.SignHash(hash)
	buf := sig.Serialize()
	b.StartTimer()
	for n := 0; n < b.N; n++ {
		sig.Deserialize(buf)
	}
}

func BenchmarkAreAllMsgDifferent(b *testing.B) {
	b.StopTimer()
	const N = 32 * 3000
	msgVec := make([]byte, N)
	rand.Read(msgVec)
	b.StartTimer()
	for i := 0; i < b.N; i++ {
		AreAllMsgDifferent(msgVec)
	}
}

func NaieveMultiVerify(sigs []Sign, pubs []PublicKey, concatenatedMsg []byte) bool {
	msgSize := 32
	n := len(sigs)
	if n == 0 || len(pubs) != n || len(concatenatedMsg) != n*msgSize {
		return false
	}
	for i := 0; i < n; i++ {
		if !sigs[i].VerifyByte(&pubs[i], concatenatedMsg[msgSize*i:msgSize*(i+1)]) {
			return false
		}
	}
	return true
}

func BenchmarkNaieveMultiVerify(b *testing.B) {
	b.StopTimer()
	pubs, sigs, msgs := makeMultiSig(400)
	b.StartTimer()
	for i := 0; i < b.N; i++ {
		NaieveMultiVerify(sigs, pubs, msgs)
	}
}

func BenchmarkMultiVerify(b *testing.B) {
	b.StopTimer()
	pubs, sigs, msgs := makeMultiSig(400)
	b.StartTimer()
	for i := 0; i < b.N; i++ {
		MultiVerify(sigs, pubs, msgs)
	}
}
