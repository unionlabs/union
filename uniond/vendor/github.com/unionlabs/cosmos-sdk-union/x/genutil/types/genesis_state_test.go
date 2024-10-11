package types_test

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	banktypes "cosmossdk.io/x/bank/types"
	"cosmossdk.io/x/staking"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

var (
	pk1 = ed25519.GenPrivKey().PubKey()
	pk2 = ed25519.GenPrivKey().PubKey()
)

func TestNetGenesisState(t *testing.T) {
	gen := types.NewGenesisState(nil)
	assert.NotNil(t, gen.GenTxs) // https://github.com/cosmos/cosmos-sdk/issues/5086

	gen = types.NewGenesisState(
		[]json.RawMessage{
			[]byte(`{"foo":"bar"}`),
		},
	)
	assert.Equal(t, string(gen.GenTxs[0]), `{"foo":"bar"}`)
}

func TestValidateGenesisMultipleMessages(t *testing.T) {
	desc := stakingtypes.NewDescription("testname", "", "", "", "")
	comm := stakingtypes.CommissionRates{}
	valAc := codectestutil.CodecOptions{}.GetValidatorCodec()

	pk1Addr, err := valAc.BytesToString(pk1.Address())
	require.NoError(t, err)
	msg1, err := stakingtypes.NewMsgCreateValidator(pk1Addr, pk1, sdk.NewInt64Coin(sdk.DefaultBondDenom, 50), desc, comm, math.OneInt())
	require.NoError(t, err)

	pk2Addr, err := valAc.BytesToString(pk2.Address())
	require.NoError(t, err)
	msg2, err := stakingtypes.NewMsgCreateValidator(pk2Addr, pk2,
		sdk.NewInt64Coin(sdk.DefaultBondDenom, 50), desc, comm, math.OneInt())
	require.NoError(t, err)

	txConfig := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, staking.AppModule{}, genutil.AppModule{}).TxConfig
	txBuilder := txConfig.NewTxBuilder()
	require.NoError(t, txBuilder.SetMsgs(msg1, msg2))

	tx := txBuilder.GetTx()
	genesisState := types.NewGenesisStateFromTx(txConfig.TxJSONEncoder(), []sdk.Tx{tx})

	err = types.ValidateGenesis(genesisState, txConfig.TxJSONDecoder(), types.DefaultMessageValidator)
	require.Error(t, err)
}

func TestValidateGenesisBadMessage(t *testing.T) {
	desc := stakingtypes.NewDescription("testname", "", "", "", "")
	pk1Addr, err := codectestutil.CodecOptions{}.GetValidatorCodec().BytesToString(pk1.Address())
	require.NoError(t, err)
	msg1 := stakingtypes.NewMsgEditValidator(pk1Addr, desc, nil, nil)

	txConfig := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, staking.AppModule{}, genutil.AppModule{}).TxConfig
	txBuilder := txConfig.NewTxBuilder()
	err = txBuilder.SetMsgs(msg1)
	require.NoError(t, err)

	tx := txBuilder.GetTx()
	genesisState := types.NewGenesisStateFromTx(txConfig.TxJSONEncoder(), []sdk.Tx{tx})

	err = types.ValidateGenesis(genesisState, txConfig.TxJSONDecoder(), types.DefaultMessageValidator)
	require.Error(t, err)
}

func TestGenesisStateFromGenFile(t *testing.T) {
	cdc := codec.NewLegacyAmino()

	genFile := "../../../tests/fixtures/adr-024-coin-metadata_genesis.json"
	genesisState, _, err := types.GenesisStateFromGenFile(genFile)
	require.NoError(t, err)

	var bankGenesis banktypes.GenesisState
	cdc.MustUnmarshalJSON(genesisState[banktypes.ModuleName], &bankGenesis)

	require.True(t, bankGenesis.Params.DefaultSendEnabled)
	require.Equal(t, "1000nametoken,100000000stake", bankGenesis.Balances[0].GetCoins().String())
	require.Equal(t, "cosmos106vrzv5xkheqhjm023pxcxlqmcjvuhtfyachz4", bankGenesis.Balances[0].Address)
	require.Equal(t, "The native staking token of the Cosmos Hub.", bankGenesis.DenomMetadata[0].GetDescription())
	require.Equal(t, "uatom", bankGenesis.DenomMetadata[0].GetBase())
	require.Equal(t, "matom", bankGenesis.DenomMetadata[0].GetDenomUnits()[1].GetDenom())
	require.Equal(t, []string{"milliatom"}, bankGenesis.DenomMetadata[0].GetDenomUnits()[1].GetAliases())
	require.Equal(t, uint32(3), bankGenesis.DenomMetadata[0].GetDenomUnits()[1].GetExponent())
}
