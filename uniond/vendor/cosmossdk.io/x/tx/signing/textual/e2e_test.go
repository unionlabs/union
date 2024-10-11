package textual_test

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"

	_ "cosmossdk.io/api/cosmos/auth/v1beta1"
	_ "cosmossdk.io/api/cosmos/authz/v1beta1"
	_ "cosmossdk.io/api/cosmos/crypto/ed25519"
	_ "cosmossdk.io/api/cosmos/crypto/multisig"
	_ "cosmossdk.io/api/cosmos/crypto/secp256k1"
	_ "cosmossdk.io/api/cosmos/gov/v1"
	"cosmossdk.io/x/tx/signing"
	"cosmossdk.io/x/tx/signing/textual"
	"cosmossdk.io/x/tx/signing/textual/internal/textualpb"
)

type e2eJSONTest struct {
	txJSONTest
	Cbor string
}

func TestE2EJSONTestcases(t *testing.T) {
	raw, err := os.ReadFile("./internal/testdata/e2e.json")
	require.NoError(t, err)

	var testcases []e2eJSONTest
	err = json.Unmarshal(raw, &testcases)
	require.NoError(t, err)

	for _, tc := range testcases {
		t.Run(tc.Name, func(t *testing.T) {
			_, bodyBz, _, authInfoBz, signerData := createTextualData(t, tc.Proto, tc.SignerData)

			tr, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: mockCoinMetadataQuerier})
			require.NoError(t, err)
			rend := textual.NewTxValueRenderer(tr)
			ctx := addMetadataToContext(context.Background(), tc.Metadata)

			data := &textualpb.TextualData{
				BodyBytes:     bodyBz,
				AuthInfoBytes: authInfoBz,
				SignerData: &textualpb.SignerData{
					Address:       signerData.Address,
					ChainId:       signerData.ChainID,
					AccountNumber: signerData.AccountNumber,
					Sequence:      signerData.Sequence,
					PubKey:        signerData.PubKey,
				},
			}

			// Make sure the screens match.
			val := protoreflect.ValueOf(data.ProtoReflect())
			screens, err := rend.Format(ctx, val)
			if tc.Error {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
			require.Equal(t, tc.Screens, screens)

			// Make sure CBOR match.
			signDoc, err := tr.GetSignBytes(ctx, signerData, signing.TxData{
				BodyBytes:     bodyBz,
				AuthInfoBytes: authInfoBz,
			})
			require.NoError(t, err)
			decodeWant, err := hex.DecodeString(tc.Cbor)
			require.NoError(t, err)
			t.Log("got: " + string(signDoc))
			t.Log("want " + string(decodeWant))
			require.Equal(t, tc.Cbor, hex.EncodeToString(signDoc))
		})
	}
}
