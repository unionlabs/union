package simulation

import (
	"math/rand"

	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"

	"github.com/CosmWasm/wasmd/app/params"
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
)

func ProposalMsgs(bk BankKeeper, wasmKeeper WasmKeeper) []simtypes.WeightedProposalMsg {
	return []simtypes.WeightedProposalMsg{
		simulation.NewWeightedProposalMsg(
			WeightInstantiateContractProposal,
			params.DefaultWeightInstantiateContractProposal,
			SimulateInstantiateContractProposal(
				bk,
				wasmKeeper,
				DefaultSimulationCodeIDSelector,
			),
		),
		simulation.NewWeightedProposalMsg(
			WeightUpdateAdminProposal,
			params.DefaultWeightUpdateAdminProposal,
			SimulateUpdateAdminProposal(
				wasmKeeper,
				DefaultSimulateUpdateAdminProposalContractSelector,
			),
		),
		simulation.NewWeightedProposalMsg(
			WeightExeContractProposal,
			params.DefaultWeightExecuteContractProposal,
			SimulateExecuteContractProposal(
				bk,
				wasmKeeper,
				DefaultSimulationExecuteContractSelector,
				DefaultSimulationExecuteSenderSelector,
				DefaultSimulationExecutePayloader,
			),
		),
		simulation.NewWeightedProposalMsg(
			WeightClearAdminProposal,
			params.DefaultWeightClearAdminProposal,
			SimulateClearAdminProposal(
				wasmKeeper,
				DefaultSimulateContractSelector,
			),
		),
		simulation.NewWeightedProposalMsg(
			WeightMigrateContractProposal,
			params.DefaultWeightMigrateContractProposal,
			SimulateMigrateContractProposal(
				wasmKeeper,
				DefaultSimulateContractSelector,
				DefaultSimulationCodeIDSelector,
			),
		),
		simulation.NewWeightedProposalMsg(
			WeightPinCodesProposal,
			params.DefaultWeightPinCodesProposal,
			SimulatePinContractProposal(
				wasmKeeper,
				DefaultSimulationCodeIDSelector,
			),
		),
		simulation.NewWeightedProposalMsg(
			WeightUnpinCodesProposal,
			params.DefaultWeightUnpinCodesProposal,
			SimulateUnpinContractProposal(
				wasmKeeper,
				DefaultSimulationCodeIDSelector,
			),
		),
		simulation.NewWeightedProposalMsg(
			WeightUpdateInstantiateConfigProposal,
			params.DefaultWeightUpdateInstantiateConfigProposal,
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
func SimulateStoreCodeProposal(wasmKeeper WasmKeeper) simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
		authority := wasmKeeper.GetAuthority()

		simAccount, _ := simtypes.RandomAcc(r, accs)

		wasmBz := testdata.ReflectContractWasm()

		permission := wasmKeeper.GetParams(ctx).InstantiateDefaultPermission.With(simAccount.Address)

		return &types.MsgStoreCode{
			Sender:                authority,
			WASMByteCode:          wasmBz,
			InstantiatePermission: &permission,
		}
	}
}

// Simulate instantiate contract proposal
func SimulateInstantiateContractProposal(bk BankKeeper, wasmKeeper WasmKeeper, codeSelector CodeIDSelector) simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
		authority := wasmKeeper.GetAuthority()

		// admin
		adminAccount, _ := simtypes.RandomAcc(r, accs)
		// get codeID
		codeID := codeSelector(ctx, wasmKeeper)
		if codeID == 0 {
			return nil
		}

		return &types.MsgInstantiateContract{
			Sender: authority,
			Admin:  adminAccount.Address.String(),
			CodeID: codeID,
			Label:  simtypes.RandStringOfLength(r, 10),
			Msg:    []byte(`{}`),
			Funds:  sdk.Coins{},
		}
	}
}

// Simulate execute contract proposal
func SimulateExecuteContractProposal(
	_ BankKeeper,
	wasmKeeper WasmKeeper,
	contractSelector MsgExecuteContractSelector,
	senderSelector MsgExecuteSenderSelector,
	payloader MsgExecutePayloader,
) simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
		authority := wasmKeeper.GetAuthority()

		ctAddress := contractSelector(ctx, wasmKeeper)
		if ctAddress == nil {
			return nil
		}

		msg := &types.MsgExecuteContract{
			Sender:   authority,
			Contract: ctAddress.String(),
			Funds:    sdk.Coins{},
		}

		if err := payloader(msg); err != nil {
			return nil
		}

		return msg
	}
}

type UpdateAdminContractSelector func(sdk.Context, WasmKeeper, string) (sdk.AccAddress, types.ContractInfo)

func DefaultSimulateUpdateAdminProposalContractSelector(
	ctx sdk.Context,
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
func SimulateUpdateAdminProposal(wasmKeeper WasmKeeper, contractSelector UpdateAdminContractSelector) simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
		authority := wasmKeeper.GetAuthority()
		simAccount, _ := simtypes.RandomAcc(r, accs)
		ctAddress, _ := contractSelector(ctx, wasmKeeper, simAccount.Address.String())
		if ctAddress == nil {
			return nil
		}

		return &types.MsgUpdateAdmin{
			Sender:   authority,
			NewAdmin: simtypes.RandomAccounts(r, 1)[0].Address.String(),
			Contract: ctAddress.String(),
		}
	}
}

type ClearAdminContractSelector func(sdk.Context, WasmKeeper) sdk.AccAddress

func DefaultSimulateContractSelector(
	ctx sdk.Context,
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
func SimulateClearAdminProposal(wasmKeeper WasmKeeper, contractSelector ClearAdminContractSelector) simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
		authority := wasmKeeper.GetAuthority()

		ctAddress := contractSelector(ctx, wasmKeeper)
		if ctAddress == nil {
			return nil
		}
		return &types.MsgClearAdmin{
			Sender:   authority,
			Contract: ctAddress.String(),
		}
	}
}

type MigrateContractProposalContractSelector func(sdk.Context, WasmKeeper) sdk.AccAddress

// Simulate migrate contract proposal
func SimulateMigrateContractProposal(wasmKeeper WasmKeeper, contractSelector MigrateContractProposalContractSelector, codeSelector CodeIDSelector) simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
		authority := wasmKeeper.GetAuthority()

		ctAddress := contractSelector(ctx, wasmKeeper)
		if ctAddress == nil {
			return nil
		}

		codeID := codeSelector(ctx, wasmKeeper)
		if codeID == 0 {
			return nil
		}

		return &types.MsgMigrateContract{
			Sender:   authority,
			Contract: ctAddress.String(),
			CodeID:   codeID,
			Msg:      []byte(`{}`),
		}
	}
}

type SudoContractProposalContractSelector func(sdk.Context, WasmKeeper) sdk.AccAddress

// Simulate sudo contract proposal
func SimulateSudoContractProposal(wasmKeeper WasmKeeper, contractSelector SudoContractProposalContractSelector) simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
		authority := wasmKeeper.GetAuthority()

		ctAddress := contractSelector(ctx, wasmKeeper)
		if ctAddress == nil {
			return nil
		}

		return &types.MsgSudoContract{
			Authority: authority,
			Contract:  ctAddress.String(),
			Msg:       []byte(`{}`),
		}
	}
}

// Simulate pin contract proposal
func SimulatePinContractProposal(wasmKeeper WasmKeeper, codeSelector CodeIDSelector) simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
		authority := wasmKeeper.GetAuthority()

		codeID := codeSelector(ctx, wasmKeeper)
		if codeID == 0 {
			return nil
		}

		return &types.MsgPinCodes{
			Authority: authority,
			CodeIDs:   []uint64{codeID},
		}
	}
}

// Simulate unpin contract proposal
func SimulateUnpinContractProposal(wasmKeeper WasmKeeper, codeSelector CodeIDSelector) simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
		authority := wasmKeeper.GetAuthority()

		codeID := codeSelector(ctx, wasmKeeper)
		if codeID == 0 {
			return nil
		}

		return &types.MsgUnpinCodes{
			Authority: authority,
			CodeIDs:   []uint64{codeID},
		}
	}
}

// Simulate update instantiate config proposal
func SimulateUpdateInstantiateConfigProposal(wasmKeeper WasmKeeper, codeSelector CodeIDSelector) simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
		authority := wasmKeeper.GetAuthority()

		codeID := codeSelector(ctx, wasmKeeper)
		if codeID == 0 {
			return nil
		}

		simAccount, _ := simtypes.RandomAcc(r, accs)
		permission := wasmKeeper.GetParams(ctx).InstantiateDefaultPermission
		config := permission.With(simAccount.Address)

		return &types.MsgUpdateInstantiateConfig{
			Sender:                   authority,
			CodeID:                   codeID,
			NewInstantiatePermission: &config,
		}
	}
}

func SimulateStoreAndInstantiateContractProposal(wasmKeeper WasmKeeper) simtypes.MsgSimulatorFn {
	return func(r *rand.Rand, ctx sdk.Context, accs []simtypes.Account) sdk.Msg {
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
		}
	}
}
