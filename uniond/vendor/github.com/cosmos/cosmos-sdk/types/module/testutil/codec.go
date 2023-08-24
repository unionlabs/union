package testutil

import (
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/std"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/x/auth/tx"
)

// TestEncodingConfig defines an encoding configuration that is used for testing
// purposes. Note, MakeTestEncodingConfig takes a series of AppModuleBasic types
// which should only contain the relevant module being tested and any potential
// dependencies.
type TestEncodingConfig struct {
	InterfaceRegistry types.InterfaceRegistry
	Codec             codec.Codec
	TxConfig          client.TxConfig
	Amino             *codec.LegacyAmino
}

func MakeTestEncodingConfig(modules ...module.AppModuleBasic) TestEncodingConfig {
	cdc := codec.NewLegacyAmino()
	interfaceRegistry := types.NewInterfaceRegistry()
	codec := codec.NewProtoCodec(interfaceRegistry)

	encCfg := TestEncodingConfig{
		InterfaceRegistry: interfaceRegistry,
		Codec:             codec,
		TxConfig:          tx.NewTxConfig(codec, tx.DefaultSignModes),
		Amino:             cdc,
	}

	mb := module.NewBasicManager(modules...)

	std.RegisterLegacyAminoCodec(encCfg.Amino)
	std.RegisterInterfaces(encCfg.InterfaceRegistry)
	mb.RegisterLegacyAminoCodec(encCfg.Amino)
	mb.RegisterInterfaces(encCfg.InterfaceRegistry)

	return encCfg
}

func MakeTestTxConfig() client.TxConfig {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)
	return tx.NewTxConfig(cdc, tx.DefaultSignModes)
}

type TestBuilderTxConfig struct {
	client.TxConfig
	TxBuilder *TestTxBuilder
}

func MakeBuilderTestTxConfig() TestBuilderTxConfig {
	return TestBuilderTxConfig{
		TxConfig: MakeTestTxConfig(),
	}
}

func (cfg TestBuilderTxConfig) NewTxBuilder() client.TxBuilder {
	if cfg.TxBuilder == nil {
		cfg.TxBuilder = &TestTxBuilder{
			TxBuilder: cfg.TxConfig.NewTxBuilder(),
		}
	}
	return cfg.TxBuilder
}

type TestTxBuilder struct {
	client.TxBuilder
	ExtOptions []*types.Any
}

func (b *TestTxBuilder) SetExtensionOptions(extOpts ...*types.Any) {
	b.ExtOptions = extOpts
}
