package tx_test

import (
	"testing"

	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	_ "cosmossdk.io/api/cosmos/crypto/secp256k1"
	coretransaction "cosmossdk.io/core/transaction"
	"cosmossdk.io/x/tx/signing"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/std"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/x/auth/tx"
	txtestutil "github.com/cosmos/cosmos-sdk/x/auth/tx/testutil"
)

func TestGenerator(t *testing.T) {
	interfaceRegistry := testutil.CodecOptions{}.NewInterfaceRegistry()
	std.RegisterInterfaces(interfaceRegistry)
	interfaceRegistry.RegisterImplementations((*coretransaction.Msg)(nil), &testdata.TestMsg{})
	protoCodec := codec.NewProtoCodec(interfaceRegistry)
	signingCtx := protoCodec.InterfaceRegistry().SigningContext()
	suite.Run(t, txtestutil.NewTxConfigTestSuite(tx.NewTxConfig(protoCodec, signingCtx.AddressCodec(), signingCtx.ValidatorAddressCodec(), tx.DefaultSignModes)))
}

func TestConfigOptions(t *testing.T) {
	interfaceRegistry := testutil.CodecOptions{}.NewInterfaceRegistry()
	protoCodec := codec.NewProtoCodec(interfaceRegistry)
	configOptions := tx.ConfigOptions{SigningOptions: &signing.Options{
		AddressCodec:          interfaceRegistry.SigningContext().AddressCodec(),
		ValidatorAddressCodec: interfaceRegistry.SigningContext().ValidatorAddressCodec(),
	}}
	txConfig, err := tx.NewTxConfigWithOptions(protoCodec, configOptions)
	require.NoError(t, err)
	require.NotNil(t, txConfig)
	handler := txConfig.SignModeHandler()
	require.NotNil(t, handler)
}
