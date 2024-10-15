package tendermint

import (
	"encoding/json"

	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/module"
)

var (
	_ module.AppModuleBasic = (*AppModule)(nil)
	_ appmodule.AppModule   = (*AppModule)(nil)
)

// Name returns the tendermint module name.
func (AppModule) Name() string {
	return ModuleName
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (AppModule) IsAppModule() {}

// RegisterLegacyAminoCodec performs a no-op. The Tendermint client does not support amino.
func (AppModule) RegisterLegacyAminoCodec(registry.AminoRegistrar) {}

// RegisterInterfaces registers module concrete types into protobuf Any. This allows core IBC
// to unmarshal tendermint light client types.
func (AppModule) RegisterInterfaces(registry registry.InterfaceRegistrar) {
	RegisterInterfaces(registry)
}

// DefaultGenesis performs a no-op. Genesis is not supported for the tendermint light client.
func (AppModule) DefaultGenesis(cdc codec.JSONCodec) json.RawMessage {
	return nil
}

// ValidateGenesis performs a no-op. Genesis is not supported for the tendermint light cilent.
func (AppModule) ValidateGenesis(cdc codec.JSONCodec, config client.TxEncodingConfig, bz json.RawMessage) error {
	return nil
}

// RegisterGRPCGatewayRoutes performs a no-op.
func (AppModule) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *runtime.ServeMux) {}

// GetTxCmd performs a no-op. Please see the 02-client cli commands.
func (AppModule) GetTxCmd() *cobra.Command {
	return nil
}

// GetQueryCmd performs a no-op. Please see the 02-client cli commands.
func (AppModule) GetQueryCmd() *cobra.Command {
	return nil
}

// AppModule is the application module for the Tendermint client module
type AppModule struct {
}

// NewAppModule creates a new Tendermint client module
func NewAppModule() AppModule {
	return AppModule{}
}
