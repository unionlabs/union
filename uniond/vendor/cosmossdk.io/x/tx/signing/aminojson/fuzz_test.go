package aminojson

import (
	"context"
	"encoding/json"
	"testing"

	fuzz "github.com/google/gofuzz"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/x/tx/signing/testutil"
)

func FuzzSignModeGetSignBytes(f *testing.F) {
	if testing.Short() {
		f.Skip("not running in -short mode")
	}

	// 1. Create seeds.
	fee := &txv1beta1.Fee{
		Amount: []*basev1beta1.Coin{{Denom: "uatom", Amount: "1000"}},
	}
	seed := &testutil.HandlerArgumentOptions{
		ChainID: "test-chain",
		Memo:    "sometestmemo",

		Msg: &bankv1beta1.MsgSend{
			FromAddress: "foo",
			ToAddress:   "bar",
			Amount:      []*basev1beta1.Coin{{Denom: "demon", Amount: "100"}},
		},
		AccNum:        1,
		AccSeq:        2,
		SignerAddress: "signerAddress",
		Fee:           fee,
	}

	gf := fuzz.New()
	for i := 0; i < 1e4; i++ {
		blob, err := json.Marshal(seed)
		if err != nil {
			f.Fatal(err)
		}
		f.Add(blob)

		// 1.5. Mutate the seed for the next iteration.
		// gofuzz cannot handle mutating "&bankv1beta1.MsgSend",
		// hence why we are mutating fields individually.
		gf.Fuzz(&seed.ChainID)
		gf.Fuzz(&seed.Memo)
		gf.Fuzz(&seed.AccNum)
		gf.Fuzz(&seed.AccSeq)
		gf.Fuzz(seed.Fee)
		gf.Fuzz(&seed.SignerAddress)
	}

	ctx := context.Background()
	handler := NewSignModeHandler(SignModeHandlerOptions{})

	// 2. Now run the fuzzers.
	f.Fuzz(func(t *testing.T, in []byte) {
		opts := new(testutil.HandlerArgumentOptions)
		if err := json.Unmarshal(in, opts); err != nil {
			return
		}

		signerData, txData, err := testutil.MakeHandlerArguments(*opts)
		if err != nil {
			return
		}
		_, _ = handler.GetSignBytes(ctx, signerData, txData)
	})
}
