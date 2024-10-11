package decode_test

import (
	"encoding/hex"
	"fmt"
	"strings"
	"testing"

	"github.com/cosmos/cosmos-proto/anyutil"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	"cosmossdk.io/api/cosmos/crypto/secp256k1"
	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/x/tx/decode"
	"cosmossdk.io/x/tx/internal/testpb"
	"cosmossdk.io/x/tx/signing"
)

func TestDecode(t *testing.T) {
	accSeq := uint64(2)

	pkAny, err := anyutil.New(&secp256k1.PubKey{Key: []byte("foo")})
	require.NoError(t, err)
	var signerInfo []*txv1beta1.SignerInfo
	signerInfo = append(signerInfo, &txv1beta1.SignerInfo{
		PublicKey: pkAny,
		ModeInfo: &txv1beta1.ModeInfo{
			Sum: &txv1beta1.ModeInfo_Single_{
				Single: &txv1beta1.ModeInfo_Single{
					Mode: signingv1beta1.SignMode_SIGN_MODE_DIRECT,
				},
			},
		},
		Sequence: accSeq,
	})

	signingCtx, err := signing.NewContext(signing.Options{
		AddressCodec:          dummyAddressCodec{},
		ValidatorAddressCodec: dummyAddressCodec{},
	})
	require.NoError(t, err)
	decoder, err := decode.NewDecoder(decode.Options{
		SigningContext: signingCtx,
	})
	require.NoError(t, err)

	testCases := []struct {
		name  string
		msg   proto.Message
		error string
	}{
		{
			name: "happy path",
			msg:  &bankv1beta1.MsgSend{},
		},
		{
			name:  "empty signer option",
			msg:   &testpb.A{},
			error: "no cosmos.msg.v1.signer option found for message A; use DefineCustomGetSigners to specify a custom getter: tx parse error",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			_, err := proto.Marshal(tc.msg)
			require.NoError(t, err)

			anyMsg, err := anyutil.New(tc.msg)
			require.NoError(t, err)
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
			txBytes, err := proto.Marshal(tx)
			require.NoError(t, err)

			decodeTx, err := decoder.Decode(txBytes)
			if tc.error != "" {
				require.EqualError(t, err, tc.error)
				return
			}
			require.NoError(t, err)

			require.Equal(t,
				fmt.Sprintf("/%s", tc.msg.ProtoReflect().Descriptor().FullName()),
				decodeTx.Tx.Body.Messages[0].TypeUrl)
		})
	}
}

type dummyAddressCodec struct{}

func (d dummyAddressCodec) StringToBytes(text string) ([]byte, error) {
	return hex.DecodeString(text)
}

func (d dummyAddressCodec) BytesToString(bz []byte) (string, error) {
	return hex.EncodeToString(bz), nil
}

func TestDecodeTxBodyPanic(t *testing.T) {
	crashVector := []byte{
		0x0a, 0x0a, 0x09, 0xe7, 0xbf, 0xba, 0xe6, 0x82, 0x9a, 0xe6, 0xaa, 0x30,
	}

	cdc := new(dummyAddressCodec)
	signingCtx, err := signing.NewContext(signing.Options{
		AddressCodec:          cdc,
		ValidatorAddressCodec: cdc,
	})
	if err != nil {
		t.Fatal(err)
	}
	dec, err := decode.NewDecoder(decode.Options{
		SigningContext: signingCtx,
	})
	if err != nil {
		t.Fatal(err)
	}
	_, err = dec.Decode(crashVector)
	if err == nil {
		t.Fatal("expected a non-nil error")
	}
	if g, w := err.Error(), "could not consume length prefix"; !strings.Contains(g, w) {
		t.Fatalf("error mismatch\n%s\nodes not contain\n\t%q", g, w)
	}
}
