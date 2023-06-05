package simulation

import (
	"encoding/json"
	"math/rand"
	"os"

	errorsmod "cosmossdk.io/errors"

	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/auth/tx"
	"github.com/cosmos/cosmos-sdk/x/simulation"

	"github.com/CosmWasm/wasmd/app/params"
	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/keeper/testdata"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// Simulation operation weights constants
//
//nolint:gosec
const (
	OpWeightMsgStoreCode           = "op_weight_msg_store_code"
	OpWeightMsgInstantiateContract = "op_weight_msg_instantiate_contract"
	OpWeightMsgExecuteContract     = "op_weight_msg_execute_contract"
	OpWeightMsgUpdateAdmin         = "op_weight_msg_update_admin"
	OpWeightMsgClearAdmin          = "op_weight_msg_clear_admin"
	OpWeightMsgMigrateContract     = "op_weight_msg_migrate_contract"
	OpReflectContractPath          = "op_reflect_contract_path"
)

// WasmKeeper is a subset of the wasm keeper used by simulations
type WasmKeeper interface {
	GetAuthority() string
	GetParams(ctx sdk.Context) types.Params
	IterateCodeInfos(ctx sdk.Context, cb func(uint64, types.CodeInfo) bool)
	IterateContractInfo(ctx sdk.Context, cb func(sdk.AccAddress, types.ContractInfo) bool)
	QuerySmart(ctx sdk.Context, contractAddr sdk.AccAddress, req []byte) ([]byte, error)
	PeekAutoIncrementID(ctx sdk.Context, lastIDKey []byte) uint64
}
type BankKeeper interface {
	simulation.BankKeeper
	IsSendEnabledCoin(ctx sdk.Context, coin sdk.Coin) bool
}

// WeightedOperations returns all the operations from the module with their respective weights
func WeightedOperations(
	simstate *module.SimulationState,
	ak types.AccountKeeper,
	bk BankKeeper,
	wasmKeeper WasmKeeper,
) simulation.WeightedOperations {
	var (
		weightMsgStoreCode           int
		weightMsgInstantiateContract int
		weightMsgExecuteContract     int
		weightMsgUpdateAdmin         int
		weightMsgClearAdmin          int
		weightMsgMigrateContract     int
		wasmContractPath             string
	)

	simstate.AppParams.GetOrGenerate(simstate.Cdc, OpWeightMsgStoreCode, &weightMsgStoreCode, nil,
		func(_ *rand.Rand) {
			weightMsgStoreCode = params.DefaultWeightMsgStoreCode
		},
	)
	simstate.AppParams.GetOrGenerate(simstate.Cdc, OpWeightMsgInstantiateContract, &weightMsgInstantiateContract, nil,
		func(_ *rand.Rand) {
			weightMsgInstantiateContract = params.DefaultWeightMsgInstantiateContract
		},
	)
	simstate.AppParams.GetOrGenerate(simstate.Cdc, OpWeightMsgExecuteContract, &weightMsgInstantiateContract, nil,
		func(_ *rand.Rand) {
			weightMsgExecuteContract = params.DefaultWeightMsgExecuteContract
		},
	)
	simstate.AppParams.GetOrGenerate(simstate.Cdc, OpWeightMsgUpdateAdmin, &weightMsgUpdateAdmin, nil,
		func(_ *rand.Rand) {
			weightMsgUpdateAdmin = params.DefaultWeightMsgUpdateAdmin
		},
	)
	simstate.AppParams.GetOrGenerate(simstate.Cdc, OpWeightMsgClearAdmin, &weightMsgClearAdmin, nil,
		func(_ *rand.Rand) {
			weightMsgClearAdmin = params.DefaultWeightMsgClearAdmin
		},
	)
	simstate.AppParams.GetOrGenerate(simstate.Cdc, OpWeightMsgMigrateContract, &weightMsgMigrateContract, nil,
		func(_ *rand.Rand) {
			weightMsgMigrateContract = params.DefaultWeightMsgMigrateContract
		},
	)
	simstate.AppParams.GetOrGenerate(simstate.Cdc, OpReflectContractPath, &wasmContractPath, nil,
		func(_ *rand.Rand) {
			wasmContractPath = ""
		},
	)

	var wasmBz []byte
	if wasmContractPath == "" {
		wasmBz = testdata.MigrateReflectContractWasm()
	} else {
		var err error
		wasmBz, err = os.ReadFile(wasmContractPath)
		if err != nil {
			panic(err)
		}
	}
	return simulation.WeightedOperations{
		simulation.NewWeightedOperation(
			weightMsgStoreCode,
			SimulateMsgStoreCode(ak, bk, wasmKeeper, wasmBz),
		),
		simulation.NewWeightedOperation(
			weightMsgInstantiateContract,
			SimulateMsgInstantiateContract(ak, bk, wasmKeeper, DefaultSimulationCodeIDSelector),
		),
		simulation.NewWeightedOperation(
			weightMsgExecuteContract,
			SimulateMsgExecuteContract(
				ak,
				bk,
				wasmKeeper,
				DefaultSimulationExecuteContractSelector,
				DefaultSimulationExecuteSenderSelector,
				DefaultSimulationExecutePayloader,
			),
		),
		simulation.NewWeightedOperation(
			weightMsgUpdateAdmin,
			SimulateMsgUpdateAmin(
				ak,
				bk,
				wasmKeeper,
				DefaultSimulationUpdateAdminContractSelector,
			),
		),
		simulation.NewWeightedOperation(
			weightMsgClearAdmin,
			SimulateMsgClearAdmin(
				ak,
				bk,
				wasmKeeper,
				DefaultSimulationClearAdminContractSelector,
			),
		),
		simulation.NewWeightedOperation(
			weightMsgMigrateContract,
			SimulateMsgMigrateContract(
				ak,
				bk,
				wasmKeeper,
				DefaultSimulationMigrateContractSelector,
				DefaultSimulationMigrateCodeIDSelector,
			),
		),
	}
}

type (
	MsgMigrateContractSelector func(sdk.Context, WasmKeeper, string) (sdk.AccAddress, types.ContractInfo)
	MsgMigrateCodeIDSelector   func(sdk.Context, WasmKeeper, uint64) uint64
)

func DefaultSimulationMigrateContractSelector(ctx sdk.Context, wasmKeeper WasmKeeper, adminAddress string) (sdk.AccAddress, types.ContractInfo) {
	var contractAddress sdk.AccAddress
	var contractInfo types.ContractInfo
	wasmKeeper.IterateContractInfo(ctx, func(address sdk.AccAddress, info types.ContractInfo) bool {
		if info.Admin != adminAddress {
			return false
		}
		contractAddress = address
		contractInfo = info
		return true
	})
	return contractAddress, contractInfo
}

func DefaultSimulationMigrateCodeIDSelector(ctx sdk.Context, wasmKeeper WasmKeeper, currentCodeID uint64) uint64 {
	var codeID uint64
	wasmKeeper.IterateCodeInfos(ctx, func(u uint64, info types.CodeInfo) bool {
		if (info.InstantiateConfig.Permission != types.AccessTypeEverybody) || (u == currentCodeID) {
			return false
		}
		codeID = u
		return true
	})
	return codeID
}

func SimulateMsgMigrateContract(
	ak types.AccountKeeper,
	bk BankKeeper,
	wasmKeeper WasmKeeper,
	contractSelector MsgMigrateContractSelector,
	codeIDSelector MsgMigrateCodeIDSelector,
) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		simAccount, _ := simtypes.RandomAcc(r, accs)
		ctAddress, info := contractSelector(ctx, wasmKeeper, simAccount.Address.String())
		if ctAddress == nil {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgMigrateContract{}.Type(), "no contract instance available"), nil, nil
		}

		codeID := codeIDSelector(ctx, wasmKeeper, info.CodeID)
		if codeID == 0 {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgMigrateContract{}.Type(), "no target contract available"), nil, nil
		}
		migrateMsg := types.MsgMigrateContract{
			Sender:   simAccount.Address.String(),
			Contract: ctAddress.String(),
			CodeID:   codeID,
			Msg:      []byte(`{}`),
		}

		txCtx := BuildOperationInput(r, app, ctx, &migrateMsg, simAccount, ak, bk, nil)
		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

type MsgClearAdminContractSelector func(sdk.Context, WasmKeeper, string) sdk.AccAddress

func DefaultSimulationClearAdminContractSelector(ctx sdk.Context, wasmKeeper WasmKeeper, adminAddress string) sdk.AccAddress {
	var ctAddress sdk.AccAddress
	wasmKeeper.IterateContractInfo(ctx, func(addr sdk.AccAddress, info types.ContractInfo) bool {
		if info.Admin != adminAddress {
			return false
		}
		ctAddress = addr
		return true
	})
	return ctAddress
}

func SimulateMsgClearAdmin(
	ak types.AccountKeeper,
	bk BankKeeper,
	wasmKeeper WasmKeeper,
	contractSelector MsgClearAdminContractSelector,
) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accounts []simtypes.Account,
		chainID string,
	) (OperationMsg simtypes.OperationMsg, futureOps []simtypes.FutureOperation, err error) {
		simAccount, _ := simtypes.RandomAcc(r, accounts)
		ctAddress := contractSelector(ctx, wasmKeeper, simAccount.Address.String())
		if ctAddress == nil {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgClearAdmin{}.Type(), "no contract instance available"), nil, nil
		}

		msg := types.MsgClearAdmin{
			Sender:   simAccount.Address.String(),
			Contract: ctAddress.String(),
		}
		txCtx := BuildOperationInput(r, app, ctx, &msg, simAccount, ak, bk, nil)
		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

type MsgUpdateAdminContractSelector func(sdk.Context, WasmKeeper, string) (sdk.AccAddress, types.ContractInfo)

// DefaultSimulationUpdateAdminContractSelector picks the first contract which Admin != ""
func DefaultSimulationUpdateAdminContractSelector(ctx sdk.Context, wasmKeeper WasmKeeper, adminAddress string) (sdk.AccAddress, types.ContractInfo) {
	var contractAddress sdk.AccAddress
	var contractInfo types.ContractInfo
	wasmKeeper.IterateContractInfo(ctx, func(address sdk.AccAddress, info types.ContractInfo) bool {
		if info.Admin != adminAddress {
			return false
		}
		contractAddress = address
		contractInfo = info
		return true
	})
	return contractAddress, contractInfo
}

func SimulateMsgUpdateAmin(
	ak types.AccountKeeper,
	bk BankKeeper,
	wasmKeeper WasmKeeper,
	contractSelector MsgUpdateAdminContractSelector,
) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		simAccount, _ := simtypes.RandomAcc(r, accs)
		ctAddress, _ := contractSelector(ctx, wasmKeeper, simAccount.Address.String())
		if ctAddress == nil {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgUpdateAdmin{}.Type(), "no contract instance available"), nil, nil
		}

		newAdmin, _ := simtypes.RandomAcc(r, accs)
		if newAdmin.Address.String() == simAccount.Address.String() {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgUpdateAdmin{}.Type(), "new admin cannot be the same as current admin"), nil, nil
		}

		msg := types.MsgUpdateAdmin{
			Sender:   simAccount.Address.String(),
			NewAdmin: newAdmin.Address.String(),
			Contract: ctAddress.String(),
		}
		txCtx := BuildOperationInput(r, app, ctx, &msg, simAccount, ak, bk, nil)
		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// SimulateMsgStoreCode generates a MsgStoreCode with random values
func SimulateMsgStoreCode(
	ak types.AccountKeeper,
	bk BankKeeper,
	wasmKeeper WasmKeeper,
	wasmBz []byte,
) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		if wasmKeeper.GetParams(ctx).CodeUploadAccess.Permission != types.AccessTypeEverybody {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgStoreCode{}.Type(), "no chain permission"), nil, nil
		}

		simAccount, _ := simtypes.RandomAcc(r, accs)

		permission := wasmKeeper.GetParams(ctx).InstantiateDefaultPermission
		config := permission.With(simAccount.Address)

		msg := types.MsgStoreCode{
			Sender:                simAccount.Address.String(),
			WASMByteCode:          wasmBz,
			InstantiatePermission: &config,
		}
		txCtx := BuildOperationInput(r, app, ctx, &msg, simAccount, ak, bk, nil)
		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// CodeIDSelector returns code id to be used in simulations
type CodeIDSelector = func(ctx sdk.Context, wasmKeeper WasmKeeper) uint64

// DefaultSimulationCodeIDSelector picks the first code id
func DefaultSimulationCodeIDSelector(ctx sdk.Context, wasmKeeper WasmKeeper) uint64 {
	var codeID uint64
	wasmKeeper.IterateCodeInfos(ctx, func(u uint64, info types.CodeInfo) bool {
		if info.InstantiateConfig.Permission != types.AccessTypeEverybody {
			return false
		}
		codeID = u
		return true
	})
	return codeID
}

// SimulateMsgInstantiateContract generates a MsgInstantiateContract with random values
func SimulateMsgInstantiateContract(
	ak types.AccountKeeper,
	bk BankKeeper,
	wasmKeeper WasmKeeper,
	codeSelector CodeIDSelector,
) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		simAccount, _ := simtypes.RandomAcc(r, accs)

		codeID := codeSelector(ctx, wasmKeeper)
		if codeID == 0 {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgInstantiateContract{}.Type(), "no codes with permission available"), nil, nil
		}
		deposit := sdk.Coins{}
		spendableCoins := bk.SpendableCoins(ctx, simAccount.Address)
		for _, v := range spendableCoins {
			if bk.IsSendEnabledCoin(ctx, v) {
				deposit = deposit.Add(simtypes.RandSubsetCoins(r, sdk.NewCoins(v))...)
			}
		}

		adminAccount, _ := simtypes.RandomAcc(r, accs)

		msg := types.MsgInstantiateContract{
			Sender: simAccount.Address.String(),
			Admin:  adminAccount.Address.String(),
			CodeID: codeID,
			Label:  simtypes.RandStringOfLength(r, 10),
			Msg:    []byte(`{}`),
			Funds:  deposit,
		}
		txCtx := BuildOperationInput(r, app, ctx, &msg, simAccount, ak, bk, deposit)
		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// MsgExecuteContractSelector returns contract address to be used in simulations
type MsgExecuteContractSelector = func(ctx sdk.Context, wasmKeeper WasmKeeper) sdk.AccAddress

// MsgExecutePayloader extension point to modify msg with custom payload
type MsgExecutePayloader func(msg *types.MsgExecuteContract) error

// MsgExecuteSenderSelector extension point that returns the sender address
type MsgExecuteSenderSelector func(wasmKeeper WasmKeeper, ctx sdk.Context, contractAddr sdk.AccAddress, accs []simtypes.Account) (simtypes.Account, error)

// SimulateMsgExecuteContract create a execute message a reflect contract instance
func SimulateMsgExecuteContract(
	ak types.AccountKeeper,
	bk BankKeeper,
	wasmKeeper WasmKeeper,
	contractSelector MsgExecuteContractSelector,
	senderSelector MsgExecuteSenderSelector,
	payloader MsgExecutePayloader,
) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		contractAddr := contractSelector(ctx, wasmKeeper)
		if contractAddr == nil {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgExecuteContract{}.Type(), "no contract instance available"), nil, nil
		}
		simAccount, err := senderSelector(wasmKeeper, ctx, contractAddr, accs)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgExecuteContract{}.Type(), "query contract owner"), nil, err
		}

		deposit := sdk.Coins{}
		spendableCoins := bk.SpendableCoins(ctx, simAccount.Address)
		for _, v := range spendableCoins {
			if bk.IsSendEnabledCoin(ctx, v) {
				deposit = deposit.Add(simtypes.RandSubsetCoins(r, sdk.NewCoins(v))...)
			}
		}
		if deposit.IsZero() {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgExecuteContract{}.Type(), "broke account"), nil, nil
		}
		msg := types.MsgExecuteContract{
			Sender:   simAccount.Address.String(),
			Contract: contractAddr.String(),
			Funds:    deposit,
		}
		if err := payloader(&msg); err != nil {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgExecuteContract{}.Type(), "contract execute payload"), nil, err
		}

		txCtx := BuildOperationInput(r, app, ctx, &msg, simAccount, ak, bk, deposit)
		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// BuildOperationInput helper to build object
func BuildOperationInput(
	r *rand.Rand,
	app *baseapp.BaseApp,
	ctx sdk.Context,
	msg interface {
		sdk.Msg
		Type() string
	},
	simAccount simtypes.Account,
	ak types.AccountKeeper,
	bk BankKeeper,
	deposit sdk.Coins,
) simulation.OperationInput {
	interfaceRegistry := codectypes.NewInterfaceRegistry()
	txConfig := tx.NewTxConfig(codec.NewProtoCodec(interfaceRegistry), tx.DefaultSignModes)
	return simulation.OperationInput{
		R:               r,
		App:             app,
		TxGen:           txConfig,
		Cdc:             nil,
		Msg:             msg,
		MsgType:         msg.Type(),
		Context:         ctx,
		SimAccount:      simAccount,
		AccountKeeper:   ak,
		Bankkeeper:      bk,
		ModuleName:      types.ModuleName,
		CoinsSpentInMsg: deposit,
	}
}

// DefaultSimulationExecuteContractSelector picks the first contract address
func DefaultSimulationExecuteContractSelector(ctx sdk.Context, wasmKeeper WasmKeeper) sdk.AccAddress {
	var r sdk.AccAddress
	wasmKeeper.IterateContractInfo(ctx, func(address sdk.AccAddress, info types.ContractInfo) bool {
		r = address
		return true
	})
	return r
}

// DefaultSimulationExecuteSenderSelector queries reflect contract for owner address and selects accounts
func DefaultSimulationExecuteSenderSelector(wasmKeeper WasmKeeper, ctx sdk.Context, contractAddr sdk.AccAddress, accs []simtypes.Account) (simtypes.Account, error) {
	var none simtypes.Account
	bz, err := json.Marshal(testdata.ReflectQueryMsg{Owner: &struct{}{}})
	if err != nil {
		return none, errorsmod.Wrap(err, "build smart query")
	}
	got, err := wasmKeeper.QuerySmart(ctx, contractAddr, bz)
	if err != nil {
		return none, errorsmod.Wrap(err, "exec smart query")
	}
	var ownerRes testdata.OwnerResponse
	if err := json.Unmarshal(got, &ownerRes); err != nil || ownerRes.Owner == "" {
		return none, errorsmod.Wrap(err, "parse smart query response")
	}
	ownerAddr, err := sdk.AccAddressFromBech32(ownerRes.Owner)
	if err != nil {
		return none, errorsmod.Wrap(err, "parse contract owner address")
	}
	simAccount, ok := simtypes.FindAccount(accs, ownerAddr)
	if !ok {
		return none, errorsmod.Wrap(err, "unknown contract owner address")
	}
	return simAccount, nil
}

// DefaultSimulationExecutePayloader implements a bank msg to send the
// tokens from contract account back to original sender
func DefaultSimulationExecutePayloader(msg *types.MsgExecuteContract) error {
	reflectSend := testdata.ReflectHandleMsg{
		Reflect: &testdata.ReflectPayload{
			Msgs: []wasmvmtypes.CosmosMsg{{
				Bank: &wasmvmtypes.BankMsg{
					Send: &wasmvmtypes.SendMsg{
						ToAddress: msg.Sender, //
						Amount:    wasmkeeper.ConvertSdkCoinsToWasmCoins(msg.Funds),
					},
				},
			}},
		},
	}
	reflectSendBz, err := json.Marshal(reflectSend)
	if err != nil {
		return err
	}
	msg.Msg = reflectSendBz
	return nil
}
