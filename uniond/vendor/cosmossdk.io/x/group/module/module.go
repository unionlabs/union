package module

import (
	"context"
	"encoding/json"
	"fmt"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"
	"google.golang.org/grpc"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"
	"cosmossdk.io/x/group"
	"cosmossdk.io/x/group/client/cli"
	"cosmossdk.io/x/group/keeper"
	"cosmossdk.io/x/group/simulation"

	sdkclient "github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/simsx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

// ConsensusVersion defines the current x/group module consensus version.
const ConsensusVersion = 2

var (
	_ module.HasAminoCodec       = AppModule{}
	_ module.HasGRPCGateway      = AppModule{}
	_ module.AppModuleSimulation = AppModule{}
	_ module.HasInvariants       = AppModule{}

	_ appmodule.AppModule             = AppModule{}
	_ appmodule.HasEndBlocker         = AppModule{}
	_ appmodule.HasMigrations         = AppModule{}
	_ appmodule.HasRegisterInterfaces = AppModule{}
	_ appmodule.HasGenesis            = AppModule{}
)

type AppModule struct {
	cdc      codec.Codec
	registry cdctypes.InterfaceRegistry

	keeper     keeper.Keeper
	bankKeeper group.BankKeeper
	accKeeper  group.AccountKeeper
}

// NewAppModule creates a new AppModule object
func NewAppModule(cdc codec.Codec, keeper keeper.Keeper, ak group.AccountKeeper, bk group.BankKeeper, registry cdctypes.InterfaceRegistry) AppModule {
	return AppModule{
		cdc:        cdc,
		keeper:     keeper,
		bankKeeper: bk,
		accKeeper:  ak,
		registry:   registry,
	}
}

// IsAppModule implements the appmodule.AppModule interface.
func (AppModule) IsAppModule() {}

// Name returns the group module's name.
// Deprecated: kept for legacy reasons.
func (am AppModule) Name() string {
	return group.ModuleName
}

// GetTxCmd returns the transaction commands for the group module
func (am AppModule) GetTxCmd() *cobra.Command {
	return cli.TxCmd(am.Name())
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the group module.
func (am AppModule) RegisterGRPCGatewayRoutes(clientCtx sdkclient.Context, mux *gwruntime.ServeMux) {
	if err := group.RegisterQueryHandlerClient(context.Background(), mux, group.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// RegisterInterfaces registers the group module's interface types
func (AppModule) RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	group.RegisterInterfaces(registrar)
}

// RegisterLegacyAminoCodec registers the group module's types for the given codec.
func (AppModule) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	group.RegisterLegacyAminoCodec(registrar)
}

// RegisterInvariants does nothing, there are no invariants to enforce
func (am AppModule) RegisterInvariants(ir sdk.InvariantRegistry) {
	keeper.RegisterInvariants(ir, am.keeper)
}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	group.RegisterMsgServer(registrar, am.keeper)
	group.RegisterQueryServer(registrar, am.keeper)

	return nil
}

// RegisterMigrations registers module migrations
func (am AppModule) RegisterMigrations(mr appmodule.MigrationRegistrar) error {
	m := keeper.NewMigrator(am.keeper)
	if err := mr.Register(group.ModuleName, 1, m.Migrate1to2); err != nil {
		return fmt.Errorf("failed to migrate x/%s from version 1 to 2: %w", group.ModuleName, err)
	}

	return nil
}

// ConsensusVersion implements HasConsensusVersion
func (AppModule) ConsensusVersion() uint64 { return ConsensusVersion }

// EndBlock implements the group module's EndBlock.
func (am AppModule) EndBlock(ctx context.Context) error {
	return am.keeper.EndBlocker(ctx)
}

// DefaultGenesis returns default genesis state as raw bytes for the group module.
func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(group.NewGenesisState())
}

// ValidateGenesis performs genesis state validation for the group module.
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var data group.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &data); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", group.ModuleName, err)
	}
	return data.Validate()
}

// InitGenesis performs genesis initialization for the group module.
func (am AppModule) InitGenesis(ctx context.Context, data json.RawMessage) error {
	return am.keeper.InitGenesis(ctx, am.cdc, data)
}

// ExportGenesis returns the exported genesis state as raw bytes for the group module.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs, err := am.keeper.ExportGenesis(ctx, am.cdc)
	if err != nil {
		return nil, err
	}
	return am.cdc.MarshalJSON(gs)
}

// GenerateGenesisState creates a randomized GenState of the group module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// RegisterStoreDecoder registers a decoder for group module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[group.StoreKey] = simulation.NewDecodeStore(am.cdc)
}

func (am AppModule) WeightedOperationsX(weights simsx.WeightSource, reg simsx.Registry) {
	s := simulation.NewSharedState()
	// note: using old keys for backwards compatibility
	reg.Add(weights.Get("msg_create_group", 100), simulation.MsgCreateGroupFactory())
	reg.Add(weights.Get("msg_update_group_admin", 5), simulation.MsgUpdateGroupAdminFactory(am.keeper, s))
	reg.Add(weights.Get("msg_update_group_metadata", 5), simulation.MsgUpdateGroupMetadataFactory(am.keeper, s))
	reg.Add(weights.Get("msg_update_group_members", 5), simulation.MsgUpdateGroupMembersFactory(am.keeper, s))
	reg.Add(weights.Get("msg_create_group_account", 50), simulation.MsgCreateGroupPolicyFactory(am.keeper, s))
	reg.Add(weights.Get("msg_create_group_with_policy", 50), simulation.MsgCreateGroupWithPolicyFactory())
	reg.Add(weights.Get("msg_update_group_account_admin", 5), simulation.MsgUpdateGroupPolicyAdminFactory(am.keeper, s))
	reg.Add(weights.Get("msg_update_group_account_decision_policy", 5), simulation.MsgUpdateGroupPolicyDecisionPolicyFactory(am.keeper, s))
	reg.Add(weights.Get("msg_update_group_account_metadata", 5), simulation.MsgUpdateGroupPolicyMetadataFactory(am.keeper, s))
	reg.Add(weights.Get("msg_submit_proposal", 2*90), simulation.MsgSubmitProposalFactory(am.keeper, s))
	reg.Add(weights.Get("msg_withdraw_proposal", 20), simulation.MsgWithdrawProposalFactory(am.keeper, s))
	reg.Add(weights.Get("msg_vote", 90), simulation.MsgVoteFactory(am.keeper, s))
	reg.Add(weights.Get("msg_exec", 90), simulation.MsgExecFactory(am.keeper, s))
	reg.Add(weights.Get("msg_leave_group", 5), simulation.MsgLeaveGroupFactory(am.keeper, s))
}
