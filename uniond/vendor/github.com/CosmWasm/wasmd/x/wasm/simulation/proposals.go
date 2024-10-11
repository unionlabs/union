package simulation

import (
	"context"
	"fmt"
	"math/rand"

	"cosmossdk.io/core/address"

	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"

	"github.com/CosmWasm/wasmd/x/wasm/keeper/testdata"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

const (
	WeightStoreCodeProposal                   = "weight_store_code_proposal"
	WeightInstantiateContractProposal         = "weight_instantiate_contract_proposal"
	WeightUpdateAdminProposal                 = "weight_update_admin_proposal"
	WeightExeContractProposal                 = "weight_execute_contract_proposal"
	WeightClearAdminProposal                  = "weight_clear_admin_proposal"
	WeightMigrateContractProposal             = "weight_migrate_contract_proposal"
	WeightSudoContractProposal                = "weight_sudo_contract_proposal"
	WeightPinCodesProposal                    = "weight_pin_codes_proposal"
	WeightUnpinCodesProposal                  = "weight_unpin_codes_proposal"
	WeightUpdateInstantiateConfigProposal     = "weight_update_instantiate_config_proposal"
	WeightStoreAndInstantiateContractProposal = "weight_store_and_instantiate_contract_proposal"

	DefaultWeightStoreCodeProposal                   int = 5
	DefaultWeightInstantiateContractProposal         int = 5
	DefaultWeightUpdateAdminProposal                 int = 5
	DefaultWeightExecuteContractProposal             int = 5
	DefaultWeightClearAdminProposal                  int = 5
	DefaultWeightMigrateContractProposal             int = 5
	DefaultWeightSudoContractProposal                int = 5
	DefaultWeightPinCodesProposal                    int = 5
	DefaultWeightUnpinCodesProposal                  int = 5
	DefaultWeightUpdateInstantiateConfigProposal     int = 5
	DefaultWeightStoreAndInstantiateContractProposal int = 5
)

func ProposalMsgs(bk BankKeeper, wasmKeeper WasmKeeper) []simtypes.WeightedProposalMsg {
	return []simtypes.WeightedProposalMsg{
		simulation.NewWeightedProposalMsgX(
			WeightInstantiateContractProposal,
			DefaultWeightInstantiateContractProposal,
			SimulateInstantiateContractProposal(
				bk,
				wasmKeeper,
				DefaultSimulationCodeIDSelector,
			),
		),
		simulation.NewWeightedProposalMsgX(
			WeightUpdateAdminProposal,
			DefaultWeightUpdateAdminProposal,
			SimulateUpdateAdminProposal(
				wasmKeeper,
				DefaultSimulateUpdateAdminProposalContractSelector,
			),
		),
		simulation.NewWeightedProposalMsgX(
			WeightExeContractProposal,
			DefaultWeightExecuteContractProposal,
			SimulateExecuteContractProposal(
				bk,
				wasmKeeper,
				DefaultSimulationExecuteContractSelector,
				DefaultSimulationExecuteSenderSelector,
				DefaultSimulationExecutePayloader,
			),
		),
		simulation.NewWeightedProposalMsgX(
			WeightClearAdminProposal,
			DefaultWeightClearAdminProposal,
			SimulateClearAdminProposal(
				wasmKeeper,
				DefaultSimulateContractSelector,
			),
		),
		simulation.NewWeightedProposalMsgX(
			WeightMigrateContractProposal,
			DefaultWeightMigrateContractProposal,
			SimulateMigrateContractProposal(
				wasmKeeper,
				DefaultSimulateContractSelector,
				DefaultSimulationCodeIDSelector,
			),
		),
		simulation.NewWeightedProposalMsgX(
			WeightPinCodesProposal,
			DefaultWeightPinCodesProposal,
			SimulatePinContractProposal(
				wasmKeeper,
				DefaultSimulationCodeIDSelector,
			),
		),
		simulation.NewWeightedProposalMsgX(
			WeightUnpinCodesProposal,
			DefaultWeightUnpinCodesProposal,
			SimulateUnpinContractProposal(
				wasmKeeper,
				DefaultSimulationCodeIDSelector,
			),
		),
		simulation.NewWeightedProposalMsgX(
			WeightUpdateInstantiateConfigProposal,
			DefaultWeightUpdateInstantiateConfigProposal,
			SimulateUpdateInstantiateConfigProposal(
				wasmKeeper,
				DefaultSimulationCodeIDSelector,
			),
		),
	}
}

// simulate store code proposal (unused now)
// Current problem: out of gas (defaul gaswanted config of gov SimulateMsgSubmitProposal is 10_000_000)
// but this proposal may need more than it
func SimulateStoreCodeProposal(wasmKeeper WasmKeeper) simtypes.MsgSimulatorFnX {
	return func(ctx context.Context, r *rand.Rand, accs []simtypes.Account, cdc address.Codec) (sdk.Msg, error) {
		authority := wasmKeeper.GetAuthority()

		simAccount, _ := simtypes.RandomAcc(r, accs)

		wasmBz := testdata.ReflectContractWasm()

		permission := wasmKeeper.GetParams(ctx).InstantiateDefaultPermission.With(simAccount.Address)

		return &types.MsgStoreCode{
			Sender:                authority,
			WASMByteCode:          wasmBz,
			InstantiatePermission: &permission,
		}, nil
	}
}

// Simulate instantiate contract proposal
func SimulateInstantiateContractProposal(bk BankKeeper, wasmKeeper WasmKeeper, codeSelector CodeIDSelector) simtypes.MsgSimulatorFnX {
	return func(ctx context.Context, r *rand.Rand, accs []simtypes.Account, cdc address.Codec) (sdk.Msg, error) {
		authority := wasmKeeper.GetAuthority()

		// admin
		adminAccount, _ := simtypes.RandomAcc(r, accs)
		// get codeID
		codeID := codeSelector(ctx, wasmKeeper)
		if codeID == 0 {
			return nil, fmt.Errorf("code not found: %v", codeID)
		}

		return &types.MsgInstantiateContract{
			Sender: authority,
			Admin:  adminAccount.Address.String(),
			CodeID: codeID,
			Label:  simtypes.RandStringOfLength(r, 10),
			Msg:    []byte(`{}`),
			Funds:  sdk.Coins{},
		}, nil
	}
}

// Simulate execute contract proposal
func SimulateExecuteContractProposal(
	_ BankKeeper,
	wasmKeeper WasmKeeper,
	contractSelector MsgExecuteContractSelector,
	senderSelector MsgExecuteSenderSelector,
	payloader MsgExecutePayloader,
) simtypes.MsgSimulatorFnX {
	return func(ctx context.Context, r *rand.Rand, accs []simtypes.Account, cdc address.Codec) (sdk.Msg, error) {
		authority := wasmKeeper.GetAuthority()

		ctAddress := contractSelector(ctx, wasmKeeper)
		if ctAddress == nil {
			return nil, fmt.Errorf("bla")
		}

		msg := &types.MsgExecuteContract{
			Sender:   authority,
			Contract: ctAddress.String(),
			Funds:    sdk.Coins{},
		}

		if err := payloader(msg); err != nil {
			return nil, err
		}

		return msg, nil
	}
}

type UpdateAdminContractSelector func(context.Context, WasmKeeper, string) (sdk.AccAddress, types.ContractInfo)

func DefaultSimulateUpdateAdminProposalContractSelector(
	ctx context.Context,
	wasmKeeper WasmKeeper,
	adminAddress string,
) (sdk.AccAddress, types.ContractInfo) {
	var contractAddr sdk.AccAddress
	var contractInfo types.ContractInfo
	wasmKeeper.IterateContractInfo(ctx, func(address sdk.AccAddress, info types.ContractInfo) bool {
		if info.Admin != adminAddress {
			return false
		}
		contractAddr = address
		contractInfo = info
		return true
	})
	return contractAddr, contractInfo
}

// Simulate update admin contract proposal
func SimulateUpdateAdminProposal(wasmKeeper WasmKeeper, contractSelector UpdateAdminContractSelector) simtypes.MsgSimulatorFnX {
	return func(ctx context.Context, r *rand.Rand, accs []simtypes.Account, cdc address.Codec) (sdk.Msg, error) {
		authority := wasmKeeper.GetAuthority()
		simAccount, _ := simtypes.RandomAcc(r, accs)
		ctAddress, _ := contractSelector(ctx, wasmKeeper, simAccount.Address.String())
		if ctAddress == nil {
			return nil, fmt.Errorf("contract not found: %v", ctAddress)
		}

		return &types.MsgUpdateAdmin{
			Sender:   authority,
			NewAdmin: simtypes.RandomAccounts(r, 1)[0].Address.String(),
			Contract: ctAddress.String(),
		}, nil
	}
}

type ClearAdminContractSelector func(context.Context, WasmKeeper) sdk.AccAddress

func DefaultSimulateContractSelector(
	ctx context.Context,
	wasmKeeper WasmKeeper,
) sdk.AccAddress {
	var contractAddr sdk.AccAddress
	wasmKeeper.IterateContractInfo(ctx, func(address sdk.AccAddress, info types.ContractInfo) bool {
		contractAddr = address
		return true
	})
	return contractAddr
}

// Simulate clear admin proposal
func SimulateClearAdminProposal(wasmKeeper WasmKeeper, contractSelector ClearAdminContractSelector) simtypes.MsgSimulatorFnX {
	return func(ctx context.Context, r *rand.Rand, accs []simtypes.Account, cdc address.Codec) (sdk.Msg, error) {
		authority := wasmKeeper.GetAuthority()

		ctAddress := contractSelector(ctx, wasmKeeper)
		if ctAddress == nil {
			return nil, fmt.Errorf("contract not found: %v", ctAddress)
		}
		return &types.MsgClearAdmin{
			Sender:   authority,
			Contract: ctAddress.String(),
		}, nil
	}
}

type MigrateContractProposalContractSelector func(context.Context, WasmKeeper) sdk.AccAddress

// Simulate migrate contract proposal
func SimulateMigrateContractProposal(wasmKeeper WasmKeeper, contractSelector MigrateContractProposalContractSelector, codeSelector CodeIDSelector) simtypes.MsgSimulatorFnX {
	return func(ctx context.Context, r *rand.Rand, accs []simtypes.Account, cdc address.Codec) (sdk.Msg, error) {
		authority := wasmKeeper.GetAuthority()

		ctAddress := contractSelector(ctx, wasmKeeper)
		if ctAddress == nil {
			return nil, fmt.Errorf("contract not found: %v", ctAddress)
		}

		codeID := codeSelector(ctx, wasmKeeper)
		if codeID == 0 {
			return nil, fmt.Errorf("code nout found: %v", codeID)
		}

		return &types.MsgMigrateContract{
			Sender:   authority,
			Contract: ctAddress.String(),
			CodeID:   codeID,
			Msg:      []byte(`{}`),
		}, nil
	}
}

type SudoContractProposalContractSelector func(context.Context, WasmKeeper) sdk.AccAddress

// Simulate sudo contract proposal
func SimulateSudoContractProposal(wasmKeeper WasmKeeper, contractSelector SudoContractProposalContractSelector) simtypes.MsgSimulatorFnX {
	return func(ctx context.Context, r *rand.Rand, accs []simtypes.Account, cdc address.Codec) (sdk.Msg, error) {
		authority := wasmKeeper.GetAuthority()

		ctAddress := contractSelector(ctx, wasmKeeper)
		if ctAddress == nil {
			return nil, fmt.Errorf("contract not found: %v", ctAddress)
		}

		return &types.MsgSudoContract{
			Authority: authority,
			Contract:  ctAddress.String(),
			Msg:       []byte(`{}`),
		}, nil
	}
}

// Simulate pin contract proposal
func SimulatePinContractProposal(wasmKeeper WasmKeeper, codeSelector CodeIDSelector) simtypes.MsgSimulatorFnX {
	return func(ctx context.Context, r *rand.Rand, accs []simtypes.Account, cdc address.Codec) (sdk.Msg, error) {
		authority := wasmKeeper.GetAuthority()

		codeID := codeSelector(ctx, wasmKeeper)
		if codeID == 0 {
			return nil, fmt.Errorf("code not found: %v", codeID)
		}

		return &types.MsgPinCodes{
			Authority: authority,
			CodeIDs:   []uint64{codeID},
		}, nil
	}
}

// Simulate unpin contract proposal
func SimulateUnpinContractProposal(wasmKeeper WasmKeeper, codeSelector CodeIDSelector) simtypes.MsgSimulatorFnX {
	return func(ctx context.Context, r *rand.Rand, accs []simtypes.Account, cdc address.Codec) (sdk.Msg, error) {
		authority := wasmKeeper.GetAuthority()

		codeID := codeSelector(ctx, wasmKeeper)
		if codeID == 0 {
			return nil, fmt.Errorf("code not found: %v", codeID)
		}

		return &types.MsgUnpinCodes{
			Authority: authority,
			CodeIDs:   []uint64{codeID},
		}, nil
	}
}

// Simulate update instantiate config proposal
func SimulateUpdateInstantiateConfigProposal(wasmKeeper WasmKeeper, codeSelector CodeIDSelector) simtypes.MsgSimulatorFnX {
	return func(ctx context.Context, r *rand.Rand, accs []simtypes.Account, cdc address.Codec) (sdk.Msg, error) {
		authority := wasmKeeper.GetAuthority()

		codeID := codeSelector(ctx, wasmKeeper)
		if codeID == 0 {
			return nil, fmt.Errorf("code not found: %v", codeID)
		}

		simAccount, _ := simtypes.RandomAcc(r, accs)
		permission := wasmKeeper.GetParams(ctx).InstantiateDefaultPermission
		config := permission.With(simAccount.Address)

		return &types.MsgUpdateInstantiateConfig{
			Sender:                   authority,
			CodeID:                   codeID,
			NewInstantiatePermission: &config,
		}, nil
	}
}

func SimulateStoreAndInstantiateContractProposal(wasmKeeper WasmKeeper) simtypes.MsgSimulatorFnX {
	return func(ctx context.Context, r *rand.Rand, accs []simtypes.Account, cdc address.Codec) (sdk.Msg, error) {
		authority := wasmKeeper.GetAuthority()

		simAccount, _ := simtypes.RandomAcc(r, accs)
		adminAccount, _ := simtypes.RandomAcc(r, accs)

		wasmBz := testdata.ReflectContractWasm()
		permission := wasmKeeper.GetParams(ctx).InstantiateDefaultPermission.With(simAccount.Address)

		return &types.MsgStoreAndInstantiateContract{
			Authority:             authority,
			WASMByteCode:          wasmBz,
			InstantiatePermission: &permission,
			UnpinCode:             false,
			Admin:                 adminAccount.Address.String(),
			Label:                 simtypes.RandStringOfLength(r, 10),
			Msg:                   []byte(`{}`),
			Funds:                 sdk.Coins{},
		}, nil
	}
}
