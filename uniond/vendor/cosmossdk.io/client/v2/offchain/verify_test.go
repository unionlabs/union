package offchain

import (
	"testing"

	"github.com/stretchr/testify/require"

	_ "cosmossdk.io/api/cosmos/crypto/secp256k1"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
)

func Test_Verify(t *testing.T) {
	ctx := client.Context{
		TxConfig:     newTestConfig(t),
		Codec:        getCodec(),
		AddressCodec: address.NewBech32Codec("cosmos"),
	}

	tests := []struct {
		name       string
		digest     []byte
		fileFormat string
		ctx        client.Context
		wantErr    bool
	}{
		{
			name:       "verify json",
			digest:     []byte("{\"body\":{\"messages\":[{\"@type\":\"/offchain.MsgSignArbitraryData\", \"appDomain\":\"simd\", \"signer\":\"cosmos1x33fy6rusfprkntvjsfregss7rvsvyy4lkwrqu\", \"data\":\"{\\n\\t\\\"name\\\": \\\"John\\\",\\n\\t\\\"surname\\\": \\\"Connor\\\",\\n\\t\\\"age\\\": 15\\n}\\n\"}]}, \"authInfo\":{\"signerInfos\":[{\"publicKey\":{\"@type\":\"/cosmos.crypto.secp256k1.PubKey\", \"key\":\"A/Bfsb7grZtysreo48oB1XAXbcgHnEJyhAqzDMgbLlXw\"}, \"modeInfo\":{\"single\":{\"mode\":\"SIGN_MODE_TEXTUAL\"}}}], \"fee\":{}}, \"signatures\":[\"gRufjcmATaJ3hZSiXII3lcsLDJlHM4OhQs3O/QgAK4weQ73kmj30/gw3HwTKxGb4pnVe0iyLXrKRNeSl1O3zSQ==\"]}"),
			fileFormat: "json",
			ctx:        ctx,
		},
		{
			name:       "wrong signer json",
			digest:     []byte("{\"body\":{\"messages\":[{\"@type\":\"/offchain.MsgSignArbitraryData\", \"appDomain\":\"simd\", \"signer\":\"cosmos1450l4uau674z55c36df0v7904rnvdk9aq8w96j\", \"data\":\"{\\n\\t\\\"name\\\": \\\"John\\\",\\n\\t\\\"surname\\\": \\\"Connor\\\",\\n\\t\\\"age\\\": 15\\n}\\n\"}]}, \"authInfo\":{\"signerInfos\":[{\"publicKey\":{\"@type\":\"/cosmos.crypto.secp256k1.PubKey\", \"key\":\"A/Bfsb7grZtysreo48oB1XAXbcgHnEJyhAqzDMgbLlXw\"}, \"modeInfo\":{\"single\":{\"mode\":\"SIGN_MODE_TEXTUAL\"}}}], \"fee\":{}}, \"signatures\":[\"gRufjcmATaJ3hZSiXII3lcsLDJlHM4OhQs3O/QgAK4weQ73kmj30/gw3HwTKxGb4pnVe0iyLXrKRNeSl1O3zSQ==\"]}"),
			fileFormat: "json",
			ctx:        ctx,
			wantErr:    true,
		},
		{
			name:       "verify text",
			digest:     []byte("body:{messages:{[/offchain.MsgSignArbitraryData]:{app_domain:\"simd\"  signer:\"cosmos1x33fy6rusfprkntvjsfregss7rvsvyy4lkwrqu\"  data:\"{\\n\\t\\\"name\\\": \\\"John\\\",\\n\\t\\\"surname\\\": \\\"Connor\\\",\\n\\t\\\"age\\\": 15\\n}\\n\"}}}  auth_info:{signer_infos:{public_key:{[/cosmos.crypto.secp256k1.PubKey]:{key:\"\\x03\\xf0_\\xb1\\xbe\u0B5Br\\xb2\\xb7\\xa8\\xe3\\xca\\x01\\xd5p\\x17m\\xc8\\x07\\x9cBr\\x84\\n\\xb3\\x0c\\xc8\\x1b.U\\xf0\"}}  mode_info:{single:{mode:SIGN_MODE_TEXTUAL}}}  fee:{}}  signatures:\"\\x81\\x1b\\x9f\\x8dɀM\\xa2w\\x85\\x94\\xa2\\\\\\x827\\x95\\xcb\\x0b\\x0c\\x99G3\\x83\\xa1B\\xcd\\xce\\xfd\\x08\\x00+\\x8c\\x1eC\\xbd\\xe4\\x9a=\\xf4\\xfe\\x0c7\\x1f\\x04\\xca\\xc4f\\xf8\\xa6u^\\xd2,\\x8b^\\xb2\\x915\\xe4\\xa5\\xd4\\xed\\xf3I\"\n"),
			fileFormat: "text",
			ctx:        ctx,
		},
		{
			name:       "wrong signer text",
			digest:     []byte("\"body:{messages:{[/offchain.MsgSignArbitraryData]:{app_domain:\\\"simd\\\"  signer:\\\"cosmos1450l4uau674z55c36df0v7904rnvdk9aq8w96j\\\"  data:\\\"{\\\\n\\\\t\\\\\\\"name\\\\\\\": \\\\\\\"John\\\\\\\",\\\\n\\\\t\\\\\\\"surname\\\\\\\": \\\\\\\"Connor\\\\\\\",\\\\n\\\\t\\\\\\\"age\\\\\\\": 15\\\\n}\\\\n\\\"}}}  auth_info:{signer_infos:{public_key:{[/cosmos.crypto.secp256k1.PubKey]:{key:\\\"\\\\x03\\\\xf0_\\\\xb1\\\\xbe\\u0B5Br\\\\xb2\\\\xb7\\\\xa8\\\\xe3\\\\xca\\\\x01\\\\xd5p\\\\x17m\\\\xc8\\\\x07\\\\x9cBr\\\\x84\\\\n\\\\xb3\\\\x0c\\\\xc8\\\\x1b.U\\\\xf0\\\"}}  mode_info:{single:{mode:SIGN_MODE_TEXTUAL}}}  fee:{}}  signatures:\\\"\\\\x81\\\\x1b\\\\x9f\\\\x8dɀM\\\\xa2w\\\\x85\\\\x94\\\\xa2\\\\\\\\\\\\x827\\\\x95\\\\xcb\\\\x0b\\\\x0c\\\\x99G3\\\\x83\\\\xa1B\\\\xcd\\\\xce\\\\xfd\\\\x08\\\\x00+\\\\x8c\\\\x1eC\\\\xbd\\\\xe4\\\\x9a=\\\\xf4\\\\xfe\\\\x0c7\\\\x1f\\\\x04\\\\xca\\\\xc4f\\\\xf8\\\\xa6u^\\\\xd2,\\\\x8b^\\\\xb2\\\\x915\\\\xe4\\\\xa5\\\\xd4\\\\xed\\\\xf3I\\\"\\n"),
			fileFormat: "text",
			ctx:        ctx,
			wantErr:    true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := Verify(tt.ctx, tt.digest, tt.fileFormat)
			if tt.wantErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func Test_SignVerify(t *testing.T) {
	k := keyring.NewInMemory(getCodec())
	_, err := k.NewAccount("signVerify", mnemonic, "", "m/44'/118'/0'/0/0", hd.Secp256k1)
	require.NoError(t, err)

	ctx := client.Context{
		TxConfig:     newTestConfig(t),
		Codec:        getCodec(),
		AddressCodec: address.NewBech32Codec("cosmos"),
		Keyring:      k,
	}

	tx, err := sign(ctx, "signVerify", "digest")
	require.NoError(t, err)

	err = verify(ctx, tx)
	require.NoError(t, err)
}

func Test_unmarshal(t *testing.T) {
	tests := []struct {
		name       string
		digest     []byte
		fileFormat string
	}{
		{
			name:       "json test",
			digest:     []byte(`{"body":{"messages":[{"@type":"/offchain.MsgSignArbitraryData", "appDomain":"simd", "signer":"cosmos1x33fy6rusfprkntvjsfregss7rvsvyy4lkwrqu", "data":"{\n\t\"name\": \"John\",\n\t\"surname\": \"Connor\",\n\t\"age\": 15\n}\n"}]}, "authInfo":{"signerInfos":[{"publicKey":{"@type":"/cosmos.crypto.secp256k1.PubKey", "key":"A/Bfsb7grZtysreo48oB1XAXbcgHnEJyhAqzDMgbLlXw"}, "modeInfo":{"single":{"mode":"SIGN_MODE_TEXTUAL"}}}], "fee":{}}, "signatures":["gRufjcmATaJ3hZSiXII3lcsLDJlHM4OhQs3O/QgAK4weQ73kmj30/gw3HwTKxGb4pnVe0iyLXrKRNeSl1O3zSQ=="]}`),
			fileFormat: "json",
		},
		{
			name:       "text test",
			digest:     []byte("body:{messages:{[/offchain.MsgSignArbitraryData]:{app_domain:\"simd\"  signer:\"cosmos1x33fy6rusfprkntvjsfregss7rvsvyy4lkwrqu\"  data:\"{\\n\\t\\\"name\\\": \\\"John\\\",\\n\\t\\\"surname\\\": \\\"Connor\\\",\\n\\t\\\"age\\\": 15\\n}\\n\"}}}  auth_info:{signer_infos:{public_key:{[/cosmos.crypto.secp256k1.PubKey]:{key:\"\\x03\\xf0_\\xb1\\xbe\u0B5Br\\xb2\\xb7\\xa8\\xe3\\xca\\x01\\xd5p\\x17m\\xc8\\x07\\x9cBr\\x84\\n\\xb3\\x0c\\xc8\\x1b.U\\xf0\"}}  mode_info:{single:{mode:SIGN_MODE_TEXTUAL}}}  fee:{}}  signatures:\"\\x81\\x1b\\x9f\\x8dɀM\\xa2w\\x85\\x94\\xa2\\\\\\x827\\x95\\xcb\\x0b\\x0c\\x99G3\\x83\\xa1B\\xcd\\xce\\xfd\\x08\\x00+\\x8c\\x1eC\\xbd\\xe4\\x9a=\\xf4\\xfe\\x0c7\\x1f\\x04\\xca\\xc4f\\xf8\\xa6u^\\xd2,\\x8b^\\xb2\\x915\\xe4\\xa5\\xd4\\xed\\xf3I\"\n"),
			fileFormat: "text",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := unmarshal(tt.digest, tt.fileFormat)
			require.NoError(t, err)
			require.NotNil(t, got)
		})
	}
}
