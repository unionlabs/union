// Copyright (c) 2017 Minio Inc. All rights reserved.
// Use of this source code is governed by a license that can be
// found in the LICENSE file.

package highwayhash

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"hash"
	"math/rand"
	"runtime"
	"strings"
	"sync/atomic"
	"testing"
)

func TestUpdateFunc(t *testing.T) {

	const want = `0xea006216335db212 - 0x97280efedaccd722
0x2a0ba13d83848de6 - 0x476fe817e850c8cd
0x5b8176dd6d555cf6 - 0x5740be93095fd2b5
0xda9d392c1fd04836 - 0x28398f8f4d922993
0x41a1794ac4aec11e - 0x41123afc4b6d0e35
0x761f6fbe30a051ce - 0x6a4afc2b722e4155
0x3f0887c550d61850 - 0x3368c4779ce6c2c8
0xd1f892c21e69b968 - 0x974e980803544d90`

	var state [16]uint64
	state[0] = 0x0102030405060708
	state[1] = 0x090a0b0c0d0e0f10
	state[2] = 0x1112131415161718
	state[3] = 0x191a1b1c1d1e1f20
	state[4] = 0x2122232425262728
	state[5] = 0x292a2b2c2d2e2f30
	state[6] = 0x3132333435363738
	state[7] = 0x393a3b3c3d3e3f40
	state[8] = 0x4142434445464748
	state[9] = 0x494a4b4c4d4e4f50
	state[10] = 0x5152535455565758
	state[11] = 0x595a5b5c5d5e5f60
	state[12] = 0x6162636465666768
	state[13] = 0x696a6b6c6d6e6f70
	state[14] = 0x7172737475767778
	state[15] = 0x797a7b7c7d7e7f80

	msg := make([]byte, 64)
	for i := range msg {
		msg[i] = 0x33 + byte(i)
	}

	updateGeneric(&state, msg)

	buf := bytes.Buffer{}
	buf.WriteString(fmt.Sprintf("0x%016x - 0x%016x\n", state[0], state[1]))
	buf.WriteString(fmt.Sprintf("0x%016x - 0x%016x\n", state[2], state[3]))
	buf.WriteString(fmt.Sprintf("0x%016x - 0x%016x\n", state[4], state[5]))
	buf.WriteString(fmt.Sprintf("0x%016x - 0x%016x\n", state[6], state[7]))
	buf.WriteString(fmt.Sprintf("0x%016x - 0x%016x\n", state[8], state[9]))
	buf.WriteString(fmt.Sprintf("0x%016x - 0x%016x\n", state[10], state[11]))
	buf.WriteString(fmt.Sprintf("0x%016x - 0x%016x\n", state[12], state[13]))
	buf.WriteString(fmt.Sprintf("0x%016x - 0x%016x\n", state[14], state[15]))

	got := strings.TrimSpace(buf.String())
	if got != want {
		t.Errorf("TestUpdateFunc:\ngot: \n%v\nwant: \n%v", got, want)
	}
}

func TestVectors(t *testing.T) {
	defer func(sse4, avx2, neon, vmx bool) {
		useSSE4, useAVX2, useNEON, useVMX = sse4, avx2, neon, vmx
	}(useSSE4, useAVX2, useNEON, useVMX)

	if useAVX2 {
		t.Run("AVX2 version", func(t *testing.T) {
			testVectors(func(key []byte) (hash.Hash, error) { return New64(key) }, testVectors64, t)
			testVectors(New128, testVectors128, t)
			testVectors(New, testVectors256, t)
			useAVX2 = false
		})
	}
	if useSSE4 {
		t.Run("SSE4 version", func(t *testing.T) {
			testVectors(func(key []byte) (hash.Hash, error) { return New64(key) }, testVectors64, t)
			testVectors(New128, testVectors128, t)
			testVectors(New, testVectors256, t)
			useSSE4 = false
		})
	}
	if useSVE2 {
		t.Run("SVE2 version", func(t *testing.T) {
			testVectors(func(key []byte) (hash.Hash, error) { return New64(key) }, testVectors64, t)
			testVectors(New128, testVectors128, t)
			testVectors(New, testVectors256, t)
			useSVE2 = false
		})
	}
	if useSVE {
		t.Run("SVE version", func(t *testing.T) {
			testVectors(func(key []byte) (hash.Hash, error) { return New64(key) }, testVectors64, t)
			testVectors(New128, testVectors128, t)
			testVectors(New, testVectors256, t)
			useSVE = false
		})
	}
	if useNEON {
		t.Run("NEON version", func(t *testing.T) {
			testVectors(func(key []byte) (hash.Hash, error) { return New64(key) }, testVectors64, t)
			testVectors(New128, testVectors128, t)
			testVectors(New, testVectors256, t)
			useNEON = false
		})
	}
	if useVMX {
		t.Run("VMX version", func(t *testing.T) {
			testVectors(func(key []byte) (hash.Hash, error) { return New64(key) }, testVectors64, t)
			testVectors(New128, testVectors128, t)
			testVectors(New, testVectors256, t)
			useVMX = false
		})
	}
	t.Run("Generic version", func(t *testing.T) {
		testVectors(func(key []byte) (hash.Hash, error) { return New64(key) }, testVectors64, t)
		testVectors(New128, testVectors128, t)
		testVectors(New, testVectors256, t)
	})
}

func testVectors(NewFunc func([]byte) (hash.Hash, error), vectors []string, t *testing.T) {
	key, err := hex.DecodeString("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
	if err != nil {
		t.Fatalf("Failed to decode key: %v", err)
	}
	input := make([]byte, len(vectors))

	h, err := NewFunc(key[:])
	if err != nil {
		t.Fatalf("Failed to create highwayhash instance:  %v", err)
	}
	for i, v := range vectors {
		input[i] = byte(i)

		expected, err := hex.DecodeString(v)
		if err != nil {
			t.Fatalf("Failed to decode test vector: %v error:  %v", v, err)
		}

		h.Write(input[:i])
		if sum := h.Sum(nil); !bytes.Equal(sum, expected[:]) {
			t.Errorf("Test %d: hash mismatch: got: %v want: %v", i, hex.EncodeToString(sum), hex.EncodeToString(expected))
		}
		h.Reset()

		switch h.Size() {
		case Size:
			if sum := Sum(input[:i], key); !bytes.Equal(sum[:], expected) {
				t.Errorf("Test %d: Sum mismatch: got: %v want: %v", i, hex.EncodeToString(sum[:]), hex.EncodeToString(expected))
			}
		case Size128:
			if sum := Sum128(input[:i], key); !bytes.Equal(sum[:], expected) {
				t.Errorf("Test %d: Sum mismatch: got: %v want: %v", i, hex.EncodeToString(sum[:]), hex.EncodeToString(expected))
			}
		case Size64:
			var sum [Size64]byte
			binary.LittleEndian.PutUint64(sum[:], Sum64(input[:i], key))
			if !bytes.Equal(sum[:], expected) {
				t.Errorf("Test %d: Sum mismatch: got: %v want: %v", i, hex.EncodeToString(sum[:]), hex.EncodeToString(expected))
			}
		}
	}
}

// HighwayHash is frozen, so the golden values will not change.
var testVectors64 = []string{
	"536ec222de567a90", "78ddcdc7aa43ab7e", "623db5b09a56d0b8", "803d468aabef6b5c", "da7e009368a405f2", "4145a9e468168a2b", "6fcaef5b32cc4cbd", "8294f53817ae024d",
	"71315fe5085120e1", "84157ac74e64d232", "0ba903b1cd0ae1f6", "155c415b61f4bbc3", "9cfa630004c23c24", "ff41e665ce589aa8", "235a4548a331b024", "3bf349a4863f7940",
	"32b87ef98934abcf", "e2c0c5c8d267fe19", "c25c569ca690dd04", "04c571238e51d975", "16ddd341119bad38", "e0708acd2c436402", "90336888625adba9", "8c023f009254b0d7",
	"1ee559ea5a615f20", "8428052196c8e0ee", "4f4f28a7931afc1b", "1da90db7b5752151", "39c6a2a076891ff7", "e7e3841fef3f09ae", "0f866111b092ca22", "685a03cf7c00c79f",
	"fc80d5ecd964c9a0", "fc8131a03cf7902c", "9eeb91564ef85c18", "9baa5227eff5c14f", "eb330a5e1a39b7f5", "9c6ce9b4834bb8b9", "b4d95c2a71fe425e", "dc973f0cf9f250a1",
	"7d632d5ed722a57f", "2bd3ff0dccd01a18", "2840851e98ed8938", "2dee86c5e89742fb", "9c0528bb454a066d", "0c86ecb309365690", "66c69740e9fca47a", "081e916bc0ba2613",
	"344f152b8d1626b9", "8d94b14589841999", "be5e8234c58fa9a2", "b6f03e21959080e9", "e9c07b7083542e58", "f56a8aa814946e08", "3d74f6208db986ee", "a7c0b109f67f9bf8",
	"e8c3229ec19c7d4c", "6f2a56245000979a", "efebe623f41cd45d", "27e268049c6013df", "5a158841f6a40d6e", "a1d4d7504bba55b7", "bd79746484347a88", "a03921bfe9eb8eab",
	"ffa6d24c5d2c5475",
}

var testVectors128 = []string{
	"c7fe8f9d8f26ed0f6f3e097f765e5633", "a8e7813689a8b0d6b4dc9cebf91d29dc", "04da165a26ad153d68e832dc38560878", "eb0b5f291b62070679ddced90f9ae6bf",
	"9ee4ac6db49e392608923139d02a922e", "d82ed186c3bd50323ac2636c90103819", "476589cbb36a476f1910ed376f57de7c", "b4717169ca1f402a6c79029fff031fbe",
	"e8520528846de9a1c20aec3bc6f15c69", "b2631ef302212a14cc00505b8cb9851a", "5bbcb6260eb7a1515955a42d3b1f9e92", "5b419a0562039988137d7bc4221fd2be",
	"6695af1c5f1f1fcdd4c8f9e08cba18a8", "5761fe12415625a248b8ddb8784ce9b2", "1909ccd1eb2f49bda2415602bc1dcdce", "54afc42ba5372214d7bc266e0b6c79e0",
	"ad01a4d5ff604441c8189f01d5a39e02", "62991cc5964b2ac5a05e9b16b178b8ec", "ceeafb118fca40d931d5f816d6463af9", "f5cbc0e50a9dc48a937c1df58dbffd3f",
	"a8002d859b276dac46aaeba56b3acd7d", "568af093bd2116f1d5d93d1698c37331", "9ff88cf650e24c0ced981841da3c12b3", "ce519a3ded97ab150e0869914774e27c",
	"b845488d191e00cd772daad88bd9d9d0", "793d49a017d6f334167e7f39f604d37d", "b6c6f4a99068b55c4f30676516290813", "c0d15b248b6fda308c74d93f7e8b826f",
	"c0124c20490358e01c445fac0cdaf693", "453007a51b7348f67659b64f1197b85f", "06528a7354834f0291097eeb18499a50", "297ca5e865b4e70646d4f5073a5e4152",
	"aa4a43c166df8419b9e4b3f95819fc16", "6cc3c6e0af7816119d84a2e59db558f9", "9004fb4084bc3f7736856543d2d56ec9", "41c9b60b71dce391e9aceec10b6a33ea",
	"d4d97a5d81e3cf259ec58f828c4fe9f2", "f288c23cb838fbb904ec50f8c8c47974", "8c2b9825c5d5851df4db486fc1b1266e", "e7bd6060bd554e8ad03f8b0599d53421",
	"368f7794f98f952a23641de61a2d05e8", "333245bee63a2389b9c0e8d7879ccf3a", "d5c8a97ee2f5584440512aca9bb48f41", "682ad17e83010309e661c83396f61710",
	"9095d40447d80d33e4a64b3aadf19d33", "76c5f263a6639356f65ec9e3953d3b36", "3707b98685d0c8ace9284e7d08e8a02b", "20956dc8277ac2392e936051a420b68d",
	"2d071a67eb4a6a8ee67ee4101a56d36e", "4ac7beb165d711002e84de6e656e0ed8", "4cc66a932bd615257d8a08d7948708ce", "af236ec152156291efcc23eb94004f26",
	"803426970d88211e8610a3d3074865d8", "2d437f09af6ad7393947079de0e117a5", "145ac637f3a4170fd476f9695f21512f", "445e8912da5cfba0d13cf1d1c43d8c56",
	"ce469cd800fcc893690e337e94dad5ba", "94561a1d50077c812bacbf2ce76e4d58", "bf53f073af68d691ede0c18376648ef9", "8bcf3c6befe18152d8836016dfc34cbc",
	"b9eeaabe6d1bd6aa7b78160c009d96ff", "795847c04fd825432d1c5f90bd19b914", "d1a66baad176a179862b3aa5c520f7f1", "f03e2f021870bd74cb4b5fada894ea3a",
	"f2c4d498711fbb98c88f91de7105bce0",
}

var testVectors256 = []string{
	"f574c8c22a4844dd1f35c713730146d9ff1487b9ccbeaeb3f41d75453123da41", "54825fe4bc41b9ed0fc6ca3def440de2474a32cb9b1b657284e475b24c627320",
	"54e4af24dff9df3f73e80a1b1abfc4117a592269cc6951112cb4330d59f60812", "5cd9d10dd7a00a48d0d111697c5e22895a86bb8b6b42a88e22c7e190c3fb3de2",
	"dce42b2197c4cfc99b92d2aff69d5fa89e10f41d219fda1f9b4f4d377a27e407", "b385dca466f5b4b44201465eba634bbfe31ddccd688ef415c68580387d58740f",
	"b4b9ad860ac74564b6ceb48427fb9ca913dbb2a0409de2da70119d9af26d52b6", "81ad8709a0b166d6376d8ceb38f8f1a430e063d4076e22e96c522c067dd65457",
	"c08b76edb005b9f1453afffcf36f97e67897d0d98d51be4f330d1e37ebafa0d9", "81293c0dd7e4d880a1f12464d1bb0ff1d10c3f9dbe2d5ccff273b601f7e8bfc0",
	"be62a2e5508ce4ade038fefdb192948e38b8e92f4bb78407cd6d65db74d5410e", "cf071853b977bea138971a6adea797ba1f268e9cef4c27afe8e84cc735b9393e",
	"575840e30238ad15a053e839dccb119d25b2313c993eea232e21f4cae3e9d96c", "367cd7b15e6fc901a6951f53c1f967a3b8dcda7c42a3941fd3d53bbf0a00f197",
	"418effee1ee915085ddf216efa280c0e745309ed628ead4ee6739d1cda01fd3f", "2e604278700519c146b1018501dbc362c10634fa17adf58547c3fed47bf884c8",
	"1fcdb6a189d91af5d97b622ad675f0f7068af279f5d5017e9f4d176ac115d41a", "8e06a42ca8cff419b975923abd4a9d3bc610c0e9ddb000801356214909d58488",
	"5d9fab817f6c6d12ee167709c5a3da4e493edda7731512af2dc380aa85ac0190", "fa559114f9beaa063d1ce744414f86dfda64bc60e8bcbafdb61c499247a52bde",
	"db9f0735406bfcad656e488e32b787a0ea23465a93a9d14644ee3c0d445c89e3", "dfb3a3ee1dd3f9b533e1060ae224308f20e18f28c8384cf24997d69bcf1d3f70",
	"e3ef9447850b3c2ba0ceda9b963f5d1c2eac63a5af6af1817530d0795a1c4423", "6237fd93c7f88a4124f9d761948e6bbc789e1a2a6af26f776eca17d4bfb7a03a",
	"c1a355d22aea03cd2a1b9cb5e5fe8501e473974fd438f4d1e4763bf867dd69be", "fba0873887a851f9aee048a5d2317b2cfa6e18b638388044729f21bec78ec7a3",
	"088c0dea51f18f958834f6b497897e4b6d38c55143078ec7faee206f557755d9", "0654b07f8017a9298c571f3584f81833faa7f6f66eea24ddffae975e469343e7",
	"cb6c5e9380082498da979fb071d2d01f83b100274786e7561778749ff9491629", "56c554704f95d41beb6c597cff2edbff5b6bab1b9ac66a7c53c17f537076030f",
	"9874599788e32588c13263afebf67c6417c928dc03d92b55abc5bf002c63d772", "4d641a6076e28068dab70fb1208b72b36ed110060612bdd0f22e4533ef14ef8a",
	"fec3a139908ce3bc8912c1a32663d542a9aefc64f79555e3995a47c96b3cb0c9", "e5a634f0cb1501f6d046cebf75ea366c90597282d3c8173b357a0011eda2da7e",
	"a2def9ed59e926130c729f73016877c42ff662d70f506951ab29250ad9d00d8a", "d442d403d549519344d1da0213b46bffec369dcd12b09c333022cc9e61531de6",
	"96b650aa88c88b52fce18460a3ecaeb8763424c01e1558a144ec7c09ad4ac102", "27c31722a788d6be3f8760f71451e61ea602307db3265c3fb997156395e8f2dd",
	"ad510b2bcf21dbe76cabb0f42463fcfa5b9c2dc2447285b09c84051e8d88adf0", "00cb4dcd93975105eb7d0663314a593c349e11cf1a0875ac94b05c809762c85a",
	"9e77b5228c8d2209847e6b51b24d6419a04131f8abc8922b9193e125d75a787f", "4ba7d0465d2ec459646003ca653ca55eb4ae35b66b91a948d4e9543f14dfe6ba",
	"e3d0036d6923b65e92a01db4bc783dd50db1f652dc4823fe118c2c6357248064", "8154b8c4b21bb643a1807e71258c31c67d689c6f4d7f4a8c7c1d4035e01702bd",
	"374c824357ca517f3a701db15e4d4cb069f3f6cb1e1e514de2565421ea7567d6", "cc457ef8ee09b439b379fc59c4e8b852248c85d1180992444901ee5e647bf080",
	"14d59abed19486cee73668522690a1bf7d2a90e4f6fda41efee196d658440c38", "a4a023f88be189d1d7a701e53b353b1f84282ee0b4774fa20c18f9746f64947e",
	"48ec25d335c6f8af0b8d0314a40a2e2c6774441a617fd34e8914503be338ec39", "97f1835fadfd2b2acc74f2be6e3e3d0155617277043c56e17e0332e95d8a5af1",
	"326312c81ef9d1d511ffb1f99b0b111032601c5426ab75a15215702857dcba87", "842808d82ca9b5c7fbee2e1bb62aa6dd2f73aefeec82988ffb4f1fc05cbd386b",
	"f0323d7375f26ecf8b7dbfa22d82f0a36a4012f535744e302d17b3ebefe3280b", "dbe9b20107f898e628888a9a812aae66c9f2b8c92490ea14a4b53e52706141a7",
	"b7ed07e3877e913ac15244e3dadeb41770cc11e762f189f60edd9c78fe6bce29", "8e5d15cbd83aff0ea244084cad9ecd47eb21fee60ee4c846510a34f05dc2f3de",
	"4dd0822be686fd036d131707600dab32897a852b830e2b68b1393744f1e38c13", "02f9d7c454c7772feabfadd9a9e053100ae74a546863e658ca83dd729c828ac4",
	"9fa066e419eb00f914d3c7a8019ebe3171f408cab8c6fe3afbe7ff870febc0b8", "fb8e3cbe8f7d27db7ba51ae17768ce537d7e9a0dd2949c71c93c459263b545b3",
	"c9f2a4db3b9c6337c86d4636b3e795608ab8651e7949803ad57c92e5cd88c982", "e44a2314a7b11f6b7e46a65b252e562075d6f3402d892b3e68d71ee4fbe30cf4",
	"2ac987b2b11ce18e6d263df6efaac28f039febe6873464667368d5e81da98a57", "67eb3a6a26f8b1f5dd1aec4dbe40b083aefb265b63c8e17f9fd7fede47a4a3f4",
	"7524c16affe6d890f2c1da6e192a421a02b08e1ffe65379ebecf51c3c4d7bdc1",
}

func benchmarkWrite(size int64, b *testing.B) {
	var key [32]byte
	data := make([]byte, size)

	h, err := New128(key[:])
	if err != nil {
		panic(err)
	}

	b.SetBytes(size)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		h.Write(data)
	}
}

func BenchmarkWrite_8(b *testing.B)  { benchmarkWrite(8, b) }
func BenchmarkWrite_16(b *testing.B) { benchmarkWrite(16, b) }
func BenchmarkWrite_64(b *testing.B) { benchmarkWrite(64, b) }
func BenchmarkWrite_1K(b *testing.B) { benchmarkWrite(1024, b) }
func BenchmarkWrite_8K(b *testing.B) { benchmarkWrite(8*1024, b) }

func benchmarkSum64(size int64, b *testing.B) {
	var key [32]byte
	data := make([]byte, size)

	b.SetBytes(size)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		Sum64(data, key[:])
	}
}

func BenchmarkSum64_8(b *testing.B)  { benchmarkSum64(8, b) }
func BenchmarkSum64_16(b *testing.B) { benchmarkSum64(16, b) }
func BenchmarkSum64_64(b *testing.B) { benchmarkSum64(64, b) }
func BenchmarkSum64_1K(b *testing.B) { benchmarkSum64(1024, b) }
func BenchmarkSum64_8K(b *testing.B) { benchmarkSum64(8*1024, b) }

func benchmarkSum256(size int64, b *testing.B) {
	var key [32]byte
	data := make([]byte, size)

	b.SetBytes(size)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		Sum(data, key[:])
	}
}

func BenchmarkSum256_8(b *testing.B)   { benchmarkSum256(8, b) }
func BenchmarkSum256_16(b *testing.B)  { benchmarkSum256(16, b) }
func BenchmarkSum256_64(b *testing.B)  { benchmarkSum256(64, b) }
func BenchmarkSum256_1K(b *testing.B)  { benchmarkSum256(1024, b) }
func BenchmarkSum256_8K(b *testing.B)  { benchmarkSum256(8*1024, b) }
func BenchmarkSum256_1M(b *testing.B)  { benchmarkSum256(1024*1024, b) }
func BenchmarkSum256_5M(b *testing.B)  { benchmarkSum256(5*1024*1024, b) }
func BenchmarkSum256_10M(b *testing.B) { benchmarkSum256(10*1024*1024, b) }
func BenchmarkSum256_25M(b *testing.B) { benchmarkSum256(25*1024*1024, b) }

func benchmarkParallel(b *testing.B, size int) {

	c := runtime.GOMAXPROCS(0)

	var key [32]byte

	rng := rand.New(rand.NewSource(0xabadc0cac01a))
	data := make([][]byte, c)
	for i := range data {
		data[i] = make([]byte, size)
		rng.Read(data[i])
	}

	b.SetBytes(int64(size))
	b.ResetTimer()

	counter := uint64(0)
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			index := atomic.AddUint64(&counter, 1)
			Sum(data[int(index)%len(data)], key[:])
		}
	})
}

func BenchmarkParallel_1M(b *testing.B)  { benchmarkParallel(b, 1024*1024) }
func BenchmarkParallel_5M(b *testing.B)  { benchmarkParallel(b, 5*1024*1024) }
func BenchmarkParallel_10M(b *testing.B) { benchmarkParallel(b, 10*1024*1024) }
func BenchmarkParallel_25M(b *testing.B) { benchmarkParallel(b, 25*1024*1024) }
