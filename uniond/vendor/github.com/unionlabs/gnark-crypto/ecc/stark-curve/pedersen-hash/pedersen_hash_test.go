package pedersenhash

import (
	"fmt"
	"testing"

	"github.com/consensys/gnark-crypto/ecc/stark-curve/fp"
)

func TestPedersen(t *testing.T) {
	tests := []struct {
		a, b string
		want string
	}{
		{
			"0x03d937c035c878245caf64531a5756109c53068da139362728feb561405371cb",
			"0x0208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a",
			"0x030e480bed5fe53fa909cc0f8c4d99b8f9f2c016be4c41e13a4848797979c662",
		},
		{
			"0x58f580910a6ca59b28927c08fe6c43e2e303ca384badc365795fc645d479d45",
			"0x78734f65a067be9bdb39de18434d71e79f7b6466a4b66bbd979ab9e7515fe0b",
			"0x68cc0b76cddd1dd4ed2301ada9b7c872b23875d5ff837b3a87993e0d9996b87",
		},
	}
	for i, tt := range tests {
		t.Run(fmt.Sprintf("TestHash %d", i), func(t *testing.T) {
			a, err := new(fp.Element).SetString(tt.a)
			if err != nil {
				t.Errorf("expected no error but got %s", err)
			}
			b, err := new(fp.Element).SetString(tt.b)
			if err != nil {
				t.Errorf("expected no error but got %s", err)
			}

			want, err := new(fp.Element).SetString(tt.want)
			if err != nil {
				t.Errorf("expected no error but got %s", err)
			}

			ans := Pedersen(a, b)
			if !ans.Equal(want) {
				t.Errorf("TestHash got %s, want %s", ans.Text(16), want.Text(16))
			}
		})
	}
}

func TestPedersenArray(t *testing.T) {
	tests := [...]struct {
		input []string
		want  string
	}{
		// Contract address calculation. See the following links for how the
		// calculation is carried out and the result referenced.
		//
		// https://docs.starknet.io/documentation/develop/Contracts/contract-address/
		// https://alpha4.starknet.io/feeder_gateway/get_transaction?transactionHash=0x1b50380d45ebd70876518203f131a12428b2ac1a3a75f1a74241a4abdd614e8
		{
			input: []string{
				// Hex representation of []byte("STARKNET_CONTRACT_ADDRESS").
				"0x535441524b4e45545f434f4e54524143545f41444452455353",
				// caller_address.
				"0x0",
				// salt.
				"0x5bebda1b28ba6daa824126577b9fbc984033e8b18360f5e1ef694cb172c7aa5",
				// contract_hash. See the following for reference https://alpha4.starknet.io/feeder_gateway/get_block?blockHash=0x53e61cb9a53136ecb782e7396f7330e6bb3d069763d866612da3cf93cdf55b5.
				"0x0439218681f9108b470d2379cf589ef47e60dc5888ee49ec70071671d74ca9c6",
				// calldata_hash. (here h(0, 0) where h is the Pedersen hash
				// function).
				"0x49ee3eba8c1600700ee1b87eb599f16716b0b1022947733551fde4050ca6804",
			},
			// contract_address.
			want: "0x43c6817e70b3fd99a4f120790b2e82c6843df62b573fdadf9e2d677b60ac5eb",
		},
		// Transaction hash calculation. See the following for reference.
		//
		// https://alpha-mainnet.starknet.io/feeder_gateway/get_transaction?transactionHash=e0a2e45a80bb827967e096bcf58874f6c01c191e0a0530624cba66a508ae75.
		{
			input: []string{
				// Hex representation of []byte("deploy").
				"0x6465706c6f79",
				// contract_address.
				"0x20cfa74ee3564b4cd5435cdace0f9c4d43b939620e4a0bb5076105df0a626c6",
				// Hex representation of keccak.Digest250([]byte("constructor")).
				"0x28ffe4ff0f226a9107253e17a904099aa4f63a02a5621de0576e5aa71bc5194",
				// calldata_hash.
				"0x7885ba4f628b6cdcd0b5e6282d2a1b17fe7cd4dd536230c5db3eac890528b4d",
				// chain_id. Hex representation of []byte("SN_MAIN").
				"0x534e5f4d41494e",
			},
			want: "0xe0a2e45a80bb827967e096bcf58874f6c01c191e0a0530624cba66a508ae75",
		},
		// Hash of an empty array is defined to be h(0, 0).
		{
			input: make([]string, 0),
			// The value below was found using the reference implementation. See:
			// https://github.com/starkware-libs/cairo-lang/blob/de741b92657f245a50caab99cfaef093152fd8be/src/starkware/crypto/signature/fast_pedersen_hash.py#L34
			want: "0x49ee3eba8c1600700ee1b87eb599f16716b0b1022947733551fde4050ca6804",
		},
	}
	for _, test := range tests {
		var data []*fp.Element
		for _, item := range test.input {
			elem, _ := new(fp.Element).SetString(item)
			data = append(data, elem)
		}
		want, _ := new(fp.Element).SetString(test.want)
		got := PedersenArray(data...)
		if !got.Equal(want) {
			t.Errorf("PedersenArray(%x) = %x, want %x", data, got, want)
		}
	}
}

var feltBench fp.Element

// go test -bench=. -run=^# -cpu=1,2,4,8,16
func BenchmarkPedersenArray(b *testing.B) {
	numOfElems := []int{3, 5, 10, 15, 20, 25, 30, 35, 40}
	createRandomFelts := func(n int) []*fp.Element {
		var felts []*fp.Element
		for i := 0; i < n; i++ {
			f, err := new(fp.Element).SetRandom()
			if err != nil {
				b.Fatalf("error while generating random felt: %x", err)
			}
			felts = append(felts, f)
		}
		return felts
	}

	for _, i := range numOfElems {
		b.Run(fmt.Sprintf("Number of felts: %d", i), func(b *testing.B) {
			var f fp.Element
			randomFelts := createRandomFelts(i)
			for n := 0; n < b.N; n++ {
				f = PedersenArray(randomFelts...)
			}
			feltBench = f
		})
	}
}

func BenchmarkPedersen(b *testing.B) {
	e0, err := new(fp.Element).SetString("0x3d937c035c878245caf64531a5756109c53068da139362728feb561405371cb")
	if err != nil {
		b.Errorf("Error occurred %s", err)
	}

	e1, err := new(fp.Element).SetString("0x208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a")
	if err != nil {
		b.Errorf("Error occurred %s", err)
	}

	var f fp.Element
	for n := 0; n < b.N; n++ {
		f = Pedersen(e0, e1)
	}
	feltBench = f
}
