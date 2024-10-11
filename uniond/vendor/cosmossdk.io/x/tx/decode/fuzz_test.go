package decode

import (
	"encoding/hex"
	"testing"

	"github.com/cosmos/cosmos-proto/anyutil"
	fuzz "github.com/google/gofuzz"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	"cosmossdk.io/api/cosmos/crypto/secp256k1"
	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/x/tx/signing"
)

var (
	accSeq = uint64(2)

	signerInfo = []*txv1beta1.SignerInfo{
		{
			PublicKey: pkAny,
			ModeInfo: &txv1beta1.ModeInfo{
				Sum: &txv1beta1.ModeInfo_Single_{
					Single: &txv1beta1.ModeInfo_Single{
						Mode: signingv1beta1.SignMode_SIGN_MODE_DIRECT,
					},
				},
			},
			Sequence: accSeq,
		},
	}

	anyMsg, _ = anyutil.New(&bankv1beta1.MsgSend{})

	pkAny, _ = anyutil.New(&secp256k1.PubKey{Key: []byte("foo")})
)

func generateAndAddSeedsFromTx(f *testing.F) {
	f.Helper()
	// 1. Add some seeds.
	tx := &txv1beta1.Tx{
		Body: &txv1beta1.TxBody{
			Messages:      []*anypb.Any{anyMsg},
			Memo:          "memo",
			TimeoutHeight: 0,
		},
		AuthInfo: &txv1beta1.AuthInfo{
			SignerInfos: signerInfo,
			Fee: &txv1beta1.Fee{
				Amount:   []*basev1beta1.Coin{{Amount: "100", Denom: "denom"}},
				GasLimit: 100,
				Payer:    "payer",
				Granter:  "",
			},
		},
		Signatures: nil,
	}
	f.Add(mustMarshal(f, tx))
	fz := fuzz.New()
	// 1.1. Mutate tx as much and add those as seeds.
	for i := 0; i < 1e4; i++ {
		func() {
			defer func() {
				_ = recover() // Catch any panics and continue
			}()
			fz.Fuzz(tx)
			f.Add(mustMarshal(f, tx))
		}()
	}
}

func FuzzInternal_rejectNonADR027TxRaw(f *testing.F) {
	if testing.Short() {
		f.Skip("Skipping in -short mode")
	}

	// 1. Add some seeds.
	generateAndAddSeedsFromTx(f)

	// 2. Now run the fuzzer.
	f.Fuzz(func(t *testing.T, in []byte) {
		// Just ensure it doesn't crash.
		_ = rejectNonADR027TxRaw(in)
	})
}

func FuzzDecode(f *testing.F) {
	if testing.Short() {
		f.Skip("Skipping in -short mode")
	}

	// 1. Add some seeds.
	generateAndAddSeedsFromTx(f)

	// 2. Now fuzz it.
	cdc := new(asHexCodec)
	signingCtx, err := signing.NewContext(signing.Options{
		AddressCodec:          cdc,
		ValidatorAddressCodec: cdc,
	})
	if err != nil {
		return
	}
	dec, err := NewDecoder(Options{
		SigningContext: signingCtx,
	})
	if err != nil {
		return
	}

	f.Fuzz(func(t *testing.T, in []byte) {
		txr, err := dec.Decode(in)
		if err == nil && txr == nil {
			t.Fatal("inconsistency: err==nil yet tx==nil")
		}
	})
}

func mustMarshal(f *testing.F, m proto.Message) []byte {
	f.Helper()
	blob, err := proto.Marshal(m)
	if err != nil {
		f.Fatal(err)
	}
	return blob
}

type asHexCodec int

func (d asHexCodec) StringToBytes(text string) ([]byte, error) {
	return hex.DecodeString(text)
}

func (d asHexCodec) BytesToString(bz []byte) (string, error) {
	return hex.EncodeToString(bz), nil
}
