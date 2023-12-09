package keeper

import (
	"bytes"
	"context"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"math"
	"reflect"
	"strconv"
	"strings"
	"time"

	wasmvm "github.com/CosmWasm/wasmvm"
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"

	"cosmossdk.io/collections"
	corestoretypes "cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	vestingexported "github.com/cosmos/cosmos-sdk/x/auth/vesting/exported"

	"github.com/CosmWasm/wasmd/x/wasm/ioutils"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// contractMemoryLimit is the memory limit of each contract execution (in MiB)
// constant value so all nodes run with the same limit.
const contractMemoryLimit = 32

// Option is an extension point to instantiate keeper with non default values
type Option interface {
	apply(*Keeper)
}

// WasmVMQueryHandler is an extension point for custom query handler implementations
type WasmVMQueryHandler interface {
	// HandleQuery executes the requested query
	HandleQuery(ctx sdk.Context, caller sdk.AccAddress, request wasmvmtypes.QueryRequest) ([]byte, error)
}

type CoinTransferrer interface {
	// TransferCoins sends the coin amounts from the source to the destination with rules applied.
	TransferCoins(ctx sdk.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) error
}

// AccountPruner handles the balances and data cleanup for accounts that are pruned on contract instantiate.
// This is an extension point to attach custom logic
type AccountPruner interface {
	// CleanupExistingAccount handles the cleanup process for balances and data of the given account. The persisted account
	// type is already reset to base account at this stage.
	// The method returns true when the account address can be reused. Unsupported account types are rejected by returning false
	CleanupExistingAccount(ctx sdk.Context, existingAccount sdk.AccountI) (handled bool, err error)
}

// WasmVMResponseHandler is an extension point to handles the response data returned by a contract call.
type WasmVMResponseHandler interface {
	// Handle processes the data returned by a contract invocation.
	Handle(
		ctx sdk.Context,
		contractAddr sdk.AccAddress,
		ibcPort string,
		messages []wasmvmtypes.SubMsg,
		origRspData []byte,
	) ([]byte, error)
}

// list of account types that are accepted for wasm contracts. Chains importing wasmd
// can overwrite this list with the WithAcceptedAccountTypesOnContractInstantiation option.
var defaultAcceptedAccountTypes = map[reflect.Type]struct{}{
	reflect.TypeOf(&authtypes.BaseAccount{}): {},
}

// Keeper will have a reference to Wasm Engine with it's own data directory.
type Keeper struct {
	// The (unexposed) keys used to access the stores from the Context.
	storeService          corestoretypes.KVStoreService
	cdc                   codec.Codec
	accountKeeper         types.AccountKeeper
	bank                  CoinTransferrer
	portKeeper            types.PortKeeper
	capabilityKeeper      types.CapabilityKeeper
	wasmVM                types.WasmEngine
	wasmVMQueryHandler    WasmVMQueryHandler
	wasmVMResponseHandler WasmVMResponseHandler
	messenger             Messenger
	// queryGasLimit is the max wasmvm gas that can be spent on executing a query with a contract
	queryGasLimit        uint64
	gasRegister          types.GasRegister
	maxQueryStackSize    uint32
	acceptedAccountTypes map[reflect.Type]struct{}
	accountPruner        AccountPruner
	params               collections.Item[types.Params]
	// propagate gov authZ to sub-messages
	propagateGovAuthorization map[types.AuthorizationPolicyAction]struct{}

	// the address capable of executing a MsgUpdateParams message. Typically, this
	// should be the x/gov module account.
	authority string
}

func (k Keeper) getUploadAccessConfig(ctx context.Context) types.AccessConfig {
	return k.GetParams(ctx).CodeUploadAccess
}

func (k Keeper) getInstantiateAccessConfig(ctx context.Context) types.AccessType {
	return k.GetParams(ctx).InstantiateDefaultPermission
}

// GetParams returns the total set of wasm parameters.
func (k Keeper) GetParams(ctx context.Context) types.Params {
	p, err := k.params.Get(ctx)
	if err != nil {
		panic(err)
	}
	return p
}

// SetParams sets all wasm parameters.
func (k Keeper) SetParams(ctx context.Context, ps types.Params) error {
	return k.params.Set(ctx, ps)
}

// GetAuthority returns the x/wasm module's authority.
func (k Keeper) GetAuthority() string {
	return k.authority
}

// GetGasRegister returns the x/wasm module's gas register.
func (k Keeper) GetGasRegister() types.GasRegister {
	return k.gasRegister
}

func (k Keeper) create(ctx context.Context, creator sdk.AccAddress, wasmCode []byte, instantiateAccess *types.AccessConfig, authZ types.AuthorizationPolicy) (codeID uint64, checksum []byte, err error) {
	if creator == nil {
		return 0, checksum, errorsmod.Wrap(sdkerrors.ErrInvalidAddress, "cannot be nil")
	}
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	// figure out proper instantiate access
	defaultAccessConfig := k.getInstantiateAccessConfig(sdkCtx).With(creator)
	if instantiateAccess == nil {
		instantiateAccess = &defaultAccessConfig
	}
	chainConfigs := types.ChainAccessConfigs{
		Instantiate: defaultAccessConfig,
		Upload:      k.getUploadAccessConfig(sdkCtx),
	}

	if !authZ.CanCreateCode(chainConfigs, creator, *instantiateAccess) {
		return 0, checksum, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "can not create code")
	}

	if ioutils.IsGzip(wasmCode) {
		sdkCtx.GasMeter().ConsumeGas(k.gasRegister.UncompressCosts(len(wasmCode)), "Uncompress gzip bytecode")
		wasmCode, err = ioutils.Uncompress(wasmCode, int64(types.MaxWasmSize))
		if err != nil {
			return 0, checksum, types.ErrCreateFailed.Wrap(errorsmod.Wrap(err, "uncompress wasm archive").Error())
		}
	}

	sdkCtx.GasMeter().ConsumeGas(k.gasRegister.CompileCosts(len(wasmCode)), "Compiling wasm bytecode")
	checksum, err = k.wasmVM.StoreCode(wasmCode)
	if err != nil {
		return 0, checksum, errorsmod.Wrap(types.ErrCreateFailed, err.Error())
	}
	report, err := k.wasmVM.AnalyzeCode(checksum)
	if err != nil {
		return 0, checksum, errorsmod.Wrap(types.ErrCreateFailed, err.Error())
	}
	codeID = k.mustAutoIncrementID(sdkCtx, types.KeySequenceCodeID)
	k.Logger(sdkCtx).Debug("storing new contract", "capabilities", report.RequiredCapabilities, "code_id", codeID)
	codeInfo := types.NewCodeInfo(checksum, creator, *instantiateAccess)
	k.mustStoreCodeInfo(sdkCtx, codeID, codeInfo)

	evt := sdk.NewEvent(
		types.EventTypeStoreCode,
		sdk.NewAttribute(types.AttributeKeyChecksum, hex.EncodeToString(checksum)),
		sdk.NewAttribute(types.AttributeKeyCodeID, strconv.FormatUint(codeID, 10)), // last element to be compatible with scripts
	)
	for _, f := range strings.Split(report.RequiredCapabilities, ",") {
		evt.AppendAttributes(sdk.NewAttribute(types.AttributeKeyRequiredCapability, strings.TrimSpace(f)))
	}
	sdkCtx.EventManager().EmitEvent(evt)

	return codeID, checksum, nil
}

func (k Keeper) mustStoreCodeInfo(ctx context.Context, codeID uint64, codeInfo types.CodeInfo) {
	store := k.storeService.OpenKVStore(ctx)
	// 0x01 | codeID (uint64) -> ContractInfo
	err := store.Set(types.GetCodeKey(codeID), k.cdc.MustMarshal(&codeInfo))
	if err != nil {
		panic(err)
	}
}

func (k Keeper) importCode(ctx context.Context, codeID uint64, codeInfo types.CodeInfo, wasmCode []byte) error {
	if ioutils.IsGzip(wasmCode) {
		var err error
		wasmCode, err = ioutils.Uncompress(wasmCode, math.MaxInt64)
		if err != nil {
			return types.ErrCreateFailed.Wrap(errorsmod.Wrap(err, "uncompress wasm archive").Error())
		}
	}
	newCodeHash, err := k.wasmVM.StoreCodeUnchecked(wasmCode)
	if err != nil {
		return errorsmod.Wrap(types.ErrCreateFailed, err.Error())
	}
	if !bytes.Equal(codeInfo.CodeHash, newCodeHash) {
		return errorsmod.Wrap(types.ErrInvalid, "code hashes not same")
	}

	store := k.storeService.OpenKVStore(ctx)
	key := types.GetCodeKey(codeID)
	ok, err := store.Has(key)
	if err != nil {
		return errorsmod.Wrap(err, "has code-id key")
	}
	if ok {
		return errorsmod.Wrapf(types.ErrDuplicate, "duplicate code: %d", codeID)
	}
	// 0x01 | codeID (uint64) -> ContractInfo
	return store.Set(key, k.cdc.MustMarshal(&codeInfo))
}

func (k Keeper) instantiate(
	ctx context.Context,
	codeID uint64,
	creator, admin sdk.AccAddress,
	initMsg []byte,
	label string,
	deposit sdk.Coins,
	addressGenerator AddressGenerator,
	authPolicy types.AuthorizationPolicy,
) (sdk.AccAddress, []byte, error) {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "instantiate")

	if creator == nil {
		return nil, nil, types.ErrEmpty.Wrap("creator")
	}
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	instanceCosts := k.gasRegister.NewContractInstanceCosts(k.IsPinnedCode(sdkCtx, codeID), len(initMsg))
	sdkCtx.GasMeter().ConsumeGas(instanceCosts, "Loading CosmWasm module: instantiate")

	codeInfo := k.GetCodeInfo(ctx, codeID)
	if codeInfo == nil {
		return nil, nil, types.ErrNoSuchCodeFn(codeID).Wrapf("code id %d", codeID)
	}
	if !authPolicy.CanInstantiateContract(codeInfo.InstantiateConfig, creator) {
		return nil, nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "can not instantiate")
	}
	contractAddress := addressGenerator(ctx, codeID, codeInfo.CodeHash)
	if k.HasContractInfo(ctx, contractAddress) {
		return nil, nil, types.ErrDuplicate.Wrap("instance with this code id, sender and label exists: try a different label")
	}

	// check account
	// every cosmos module can define custom account types when needed. The cosmos-sdk comes with extension points
	// to support this and a set of base and vesting account types that we integrated in our default lists.
	// But not all account types of other modules are known or may make sense for contracts, therefore we kept this
	// decision logic also very flexible and extendable. We provide new options to overwrite the default settings via WithAcceptedAccountTypesOnContractInstantiation and
	// WithPruneAccountTypesOnContractInstantiation as constructor arguments
	existingAcct := k.accountKeeper.GetAccount(sdkCtx, contractAddress)
	if existingAcct != nil {
		if existingAcct.GetSequence() != 0 || existingAcct.GetPubKey() != nil {
			return nil, nil, types.ErrAccountExists.Wrap("address is claimed by external account")
		}
		if _, accept := k.acceptedAccountTypes[reflect.TypeOf(existingAcct)]; accept {
			// keep account and balance as it is
			k.Logger(sdkCtx).Info("instantiate contract with existing account", "address", contractAddress.String())
		} else {
			// consider an account in the wasmd namespace spam and overwrite it.
			k.Logger(sdkCtx).Info("pruning existing account for contract instantiation", "address", contractAddress.String())
			contractAccount := k.accountKeeper.NewAccountWithAddress(sdkCtx, contractAddress)
			k.accountKeeper.SetAccount(sdkCtx, contractAccount)
			// also handle balance to not open cases where these accounts are abused and become liquid
			switch handled, err := k.accountPruner.CleanupExistingAccount(sdkCtx, existingAcct); {
			case err != nil:
				return nil, nil, errorsmod.Wrap(err, "prune balance")
			case !handled:
				return nil, nil, types.ErrAccountExists.Wrap("address is claimed by external account")
			}
		}
	} else {
		// create an empty account (so we don't have issues later)
		contractAccount := k.accountKeeper.NewAccountWithAddress(sdkCtx, contractAddress)
		k.accountKeeper.SetAccount(sdkCtx, contractAccount)
	}
	// deposit initial contract funds
	if !deposit.IsZero() {
		if err := k.bank.TransferCoins(sdkCtx, creator, contractAddress, deposit); err != nil {
			return nil, nil, err
		}
	}

	// prepare params for contract instantiate call
	env := types.NewEnv(sdkCtx, contractAddress)
	info := types.NewInfo(creator, deposit)

	// create prefixed data store
	// 0x03 | BuildContractAddressClassic (sdk.AccAddress)
	prefixStoreKey := types.GetContractStorePrefix(contractAddress)
	vmStore := types.NewStoreAdapter(prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(sdkCtx)), prefixStoreKey))

	// prepare querier
	querier := k.newQueryHandler(sdkCtx, contractAddress)

	// instantiate wasm contract
	gas := k.runtimeGasForContract(sdkCtx)
	res, gasUsed, err := k.wasmVM.Instantiate(codeInfo.CodeHash, env, info, initMsg, vmStore, cosmwasmAPI, querier, k.gasMeter(sdkCtx), gas, costJSONDeserialization)
	k.consumeRuntimeGas(sdkCtx, gasUsed)
	if err != nil {
		return nil, nil, errorsmod.Wrap(types.ErrInstantiateFailed, err.Error())
	}

	// persist instance first
	createdAt := types.NewAbsoluteTxPosition(sdkCtx)
	contractInfo := types.NewContractInfo(codeID, creator, admin, label, createdAt)

	// check for IBC flag
	report, err := k.wasmVM.AnalyzeCode(codeInfo.CodeHash)
	if err != nil {
		return nil, nil, errorsmod.Wrap(types.ErrInstantiateFailed, err.Error())
	}
	if report.HasIBCEntryPoints {
		// register IBC port
		ibcPort, err := k.ensureIbcPort(sdkCtx, contractAddress)
		if err != nil {
			return nil, nil, err
		}
		contractInfo.IBCPortID = ibcPort
	}

	// store contract before dispatch so that contract could be called back
	historyEntry := contractInfo.InitialHistory(initMsg)
	err = k.addToContractCodeSecondaryIndex(sdkCtx, contractAddress, historyEntry)
	if err != nil {
		return nil, nil, err
	}
	err = k.addToContractCreatorSecondaryIndex(sdkCtx, creator, historyEntry.Updated, contractAddress)
	if err != nil {
		return nil, nil, err
	}
	err = k.appendToContractHistory(sdkCtx, contractAddress, historyEntry)
	if err != nil {
		return nil, nil, err
	}

	k.mustStoreContractInfo(sdkCtx, contractAddress, &contractInfo)

	sdkCtx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeInstantiate,
		sdk.NewAttribute(types.AttributeKeyContractAddr, contractAddress.String()),
		sdk.NewAttribute(types.AttributeKeyCodeID, strconv.FormatUint(codeID, 10)),
	))

	sdkCtx = types.WithSubMsgAuthzPolicy(sdkCtx, authPolicy.SubMessageAuthorizationPolicy(types.AuthZActionInstantiate))
	data, err := k.handleContractResponse(sdkCtx, contractAddress, contractInfo.IBCPortID, res.Messages, res.Attributes, res.Data, res.Events)
	if err != nil {
		return nil, nil, errorsmod.Wrap(err, "dispatch")
	}

	return contractAddress, data, nil
}

// Execute executes the contract instance
func (k Keeper) execute(ctx context.Context, contractAddress, caller sdk.AccAddress, msg []byte, coins sdk.Coins) ([]byte, error) {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "execute")
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	contractInfo, codeInfo, prefixStore, err := k.contractInstance(ctx, contractAddress)
	if err != nil {
		return nil, err
	}

	executeCosts := k.gasRegister.InstantiateContractCosts(k.IsPinnedCode(ctx, contractInfo.CodeID), len(msg))
	sdkCtx.GasMeter().ConsumeGas(executeCosts, "Loading CosmWasm module: execute")

	// add more funds
	if !coins.IsZero() {
		if err := k.bank.TransferCoins(sdkCtx, caller, contractAddress, coins); err != nil {
			return nil, err
		}
	}

	env := types.NewEnv(sdkCtx, contractAddress)
	info := types.NewInfo(caller, coins)

	// prepare querier
	querier := k.newQueryHandler(sdkCtx, contractAddress)
	gas := k.runtimeGasForContract(sdkCtx)
	res, gasUsed, execErr := k.wasmVM.Execute(codeInfo.CodeHash, env, info, msg, prefixStore, cosmwasmAPI, querier, k.gasMeter(sdkCtx), gas, costJSONDeserialization)
	k.consumeRuntimeGas(sdkCtx, gasUsed)
	if execErr != nil {
		return nil, errorsmod.Wrap(types.ErrExecuteFailed, execErr.Error())
	}

	sdkCtx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeExecute,
		sdk.NewAttribute(types.AttributeKeyContractAddr, contractAddress.String()),
	))

	data, err := k.handleContractResponse(sdkCtx, contractAddress, contractInfo.IBCPortID, res.Messages, res.Attributes, res.Data, res.Events)
	if err != nil {
		return nil, errorsmod.Wrap(err, "dispatch")
	}

	return data, nil
}

func (k Keeper) migrate(
	ctx context.Context,
	contractAddress sdk.AccAddress,
	caller sdk.AccAddress,
	newCodeID uint64,
	msg []byte,
	authZ types.AuthorizationPolicy,
) ([]byte, error) {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "migrate")

	sdkCtx := sdk.UnwrapSDKContext(ctx)
	migrateSetupCosts := k.gasRegister.InstantiateContractCosts(k.IsPinnedCode(ctx, newCodeID), len(msg))
	sdkCtx.GasMeter().ConsumeGas(migrateSetupCosts, "Loading CosmWasm module: migrate")

	contractInfo := k.GetContractInfo(ctx, contractAddress)
	if contractInfo == nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "unknown contract")
	}
	if !authZ.CanModifyContract(contractInfo.AdminAddr(), caller) {
		return nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "can not migrate")
	}

	newCodeInfo := k.GetCodeInfo(ctx, newCodeID)
	if newCodeInfo == nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "unknown code")
	}

	if !authZ.CanInstantiateContract(newCodeInfo.InstantiateConfig, caller) {
		return nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "to use new code")
	}

	// check for IBC flag
	switch report, err := k.wasmVM.AnalyzeCode(newCodeInfo.CodeHash); {
	case err != nil:
		return nil, errorsmod.Wrap(types.ErrMigrationFailed, err.Error())
	case !report.HasIBCEntryPoints && contractInfo.IBCPortID != "":
		// prevent update to non ibc contract
		return nil, errorsmod.Wrap(types.ErrMigrationFailed, "requires ibc callbacks")
	case report.HasIBCEntryPoints && contractInfo.IBCPortID == "":
		// add ibc port
		ibcPort, err := k.ensureIbcPort(sdkCtx, contractAddress)
		if err != nil {
			return nil, err
		}
		contractInfo.IBCPortID = ibcPort
	}

	env := types.NewEnv(sdkCtx, contractAddress)

	// prepare querier
	querier := k.newQueryHandler(sdkCtx, contractAddress)

	prefixStoreKey := types.GetContractStorePrefix(contractAddress)
	vmStore := types.NewStoreAdapter(prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(sdkCtx)), prefixStoreKey))
	gas := k.runtimeGasForContract(sdkCtx)
	res, gasUsed, err := k.wasmVM.Migrate(newCodeInfo.CodeHash, env, msg, vmStore, cosmwasmAPI, &querier, k.gasMeter(sdkCtx), gas, costJSONDeserialization)
	k.consumeRuntimeGas(sdkCtx, gasUsed)
	if err != nil {
		return nil, errorsmod.Wrap(types.ErrMigrationFailed, err.Error())
	}
	// delete old secondary index entry
	err = k.removeFromContractCodeSecondaryIndex(ctx, contractAddress, k.mustGetLastContractHistoryEntry(sdkCtx, contractAddress))
	if err != nil {
		return nil, err
	}
	// persist migration updates
	historyEntry := contractInfo.AddMigration(sdkCtx, newCodeID, msg)
	err = k.appendToContractHistory(ctx, contractAddress, historyEntry)
	if err != nil {
		return nil, err
	}
	err = k.addToContractCodeSecondaryIndex(ctx, contractAddress, historyEntry)
	if err != nil {
		return nil, err
	}
	k.mustStoreContractInfo(ctx, contractAddress, contractInfo)

	sdkCtx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeMigrate,
		sdk.NewAttribute(types.AttributeKeyCodeID, strconv.FormatUint(newCodeID, 10)),
		sdk.NewAttribute(types.AttributeKeyContractAddr, contractAddress.String()),
	))

	sdkCtx = types.WithSubMsgAuthzPolicy(sdkCtx, authZ.SubMessageAuthorizationPolicy(types.AuthZActionMigrateContract))
	data, err := k.handleContractResponse(sdkCtx, contractAddress, contractInfo.IBCPortID, res.Messages, res.Attributes, res.Data, res.Events)
	if err != nil {
		return nil, errorsmod.Wrap(err, "dispatch")
	}

	return data, nil
}

// Sudo allows privileged access to a contract. This can never be called by an external tx, but only by
// another native Go module directly, or on-chain governance (if sudo proposals are enabled). Thus, the keeper doesn't
// place any access controls on it, that is the responsibility or the app developer (who passes the wasm.Keeper in app.go)
//
// Sub-messages returned from the sudo call to the contract are executed with the default authorization policy. This can be
// customized though by passing a new policy with the context. See types.WithSubMsgAuthzPolicy.
// The policy will be read in msgServer.selectAuthorizationPolicy and used for sub-message executions.
// This is an extension point for some very advanced scenarios only. Use with care!
func (k Keeper) Sudo(ctx context.Context, contractAddress sdk.AccAddress, msg []byte) ([]byte, error) {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "sudo")

	contractInfo, codeInfo, prefixStore, err := k.contractInstance(ctx, contractAddress)
	if err != nil {
		return nil, err
	}

	sudoSetupCosts := k.gasRegister.InstantiateContractCosts(k.IsPinnedCode(ctx, contractInfo.CodeID), len(msg))
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	sdkCtx.GasMeter().ConsumeGas(sudoSetupCosts, "Loading CosmWasm module: sudo")

	env := types.NewEnv(sdkCtx, contractAddress)

	// prepare querier
	querier := k.newQueryHandler(sdkCtx, contractAddress)
	gas := k.runtimeGasForContract(sdkCtx)
	res, gasUsed, execErr := k.wasmVM.Sudo(codeInfo.CodeHash, env, msg, prefixStore, cosmwasmAPI, querier, k.gasMeter(sdkCtx), gas, costJSONDeserialization)
	k.consumeRuntimeGas(sdkCtx, gasUsed)
	if execErr != nil {
		return nil, errorsmod.Wrap(types.ErrExecuteFailed, execErr.Error())
	}

	sdkCtx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeSudo,
		sdk.NewAttribute(types.AttributeKeyContractAddr, contractAddress.String()),
	))

	// sudo submessages are executed with the default authorization policy
	data, err := k.handleContractResponse(sdkCtx, contractAddress, contractInfo.IBCPortID, res.Messages, res.Attributes, res.Data, res.Events)
	if err != nil {
		return nil, errorsmod.Wrap(err, "dispatch")
	}

	return data, nil
}

// reply is only called from keeper internal functions (dispatchSubmessages) after processing the submessage
func (k Keeper) reply(ctx sdk.Context, contractAddress sdk.AccAddress, reply wasmvmtypes.Reply) ([]byte, error) {
	contractInfo, codeInfo, prefixStore, err := k.contractInstance(ctx, contractAddress)
	if err != nil {
		return nil, err
	}

	// always consider this pinned
	replyCosts := k.gasRegister.ReplyCosts(true, reply)
	ctx.GasMeter().ConsumeGas(replyCosts, "Loading CosmWasm module: reply")

	env := types.NewEnv(ctx, contractAddress)

	// prepare querier
	querier := k.newQueryHandler(ctx, contractAddress)
	gas := k.runtimeGasForContract(ctx)

	res, gasUsed, execErr := k.wasmVM.Reply(codeInfo.CodeHash, env, reply, prefixStore, cosmwasmAPI, querier, k.gasMeter(ctx), gas, costJSONDeserialization)
	k.consumeRuntimeGas(ctx, gasUsed)
	if execErr != nil {
		return nil, errorsmod.Wrap(types.ErrExecuteFailed, execErr.Error())
	}

	ctx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeReply,
		sdk.NewAttribute(types.AttributeKeyContractAddr, contractAddress.String()),
	))

	data, err := k.handleContractResponse(ctx, contractAddress, contractInfo.IBCPortID, res.Messages, res.Attributes, res.Data, res.Events)
	if err != nil {
		return nil, errorsmod.Wrap(err, "dispatch")
	}

	return data, nil
}

// addToContractCodeSecondaryIndex adds element to the index for contracts-by-codeid queries
func (k Keeper) addToContractCodeSecondaryIndex(ctx context.Context, contractAddress sdk.AccAddress, entry types.ContractCodeHistoryEntry) error {
	store := k.storeService.OpenKVStore(ctx)
	return store.Set(types.GetContractByCreatedSecondaryIndexKey(contractAddress, entry), []byte{})
}

// removeFromContractCodeSecondaryIndex removes element to the index for contracts-by-codeid queries
func (k Keeper) removeFromContractCodeSecondaryIndex(ctx context.Context, contractAddress sdk.AccAddress, entry types.ContractCodeHistoryEntry) error {
	return k.storeService.OpenKVStore(ctx).Delete(types.GetContractByCreatedSecondaryIndexKey(contractAddress, entry))
}

// addToContractCreatorSecondaryIndex adds element to the index for contracts-by-creator queries
func (k Keeper) addToContractCreatorSecondaryIndex(ctx context.Context, creatorAddress sdk.AccAddress, position *types.AbsoluteTxPosition, contractAddress sdk.AccAddress) error {
	store := k.storeService.OpenKVStore(ctx)
	return store.Set(types.GetContractByCreatorSecondaryIndexKey(creatorAddress, position.Bytes(), contractAddress), []byte{})
}

// IterateContractsByCreator iterates over all contracts with given creator address in order of creation time asc.
func (k Keeper) IterateContractsByCreator(ctx context.Context, creator sdk.AccAddress, cb func(address sdk.AccAddress) bool) {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), types.GetContractsByCreatorPrefix(creator))
	for iter := prefixStore.Iterator(nil, nil); iter.Valid(); iter.Next() {
		key := iter.Key()
		if cb(key[types.AbsoluteTxPositionLen:]) {
			return
		}
	}
}

// IterateContractsByCode iterates over all contracts with given codeID ASC on code update time.
func (k Keeper) IterateContractsByCode(ctx context.Context, codeID uint64, cb func(address sdk.AccAddress) bool) {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), types.GetContractByCodeIDSecondaryIndexPrefix(codeID))
	iter := prefixStore.Iterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		key := iter.Key()
		if cb(key[types.AbsoluteTxPositionLen:]) {
			return
		}
	}
}

func (k Keeper) setContractAdmin(ctx context.Context, contractAddress, caller, newAdmin sdk.AccAddress, authZ types.AuthorizationPolicy) error {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	contractInfo := k.GetContractInfo(sdkCtx, contractAddress)
	if contractInfo == nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "unknown contract")
	}
	if !authZ.CanModifyContract(contractInfo.AdminAddr(), caller) {
		return errorsmod.Wrap(sdkerrors.ErrUnauthorized, "can not modify contract")
	}
	newAdminStr := newAdmin.String()
	contractInfo.Admin = newAdminStr
	k.mustStoreContractInfo(sdkCtx, contractAddress, contractInfo)
	sdkCtx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeUpdateContractAdmin,
		sdk.NewAttribute(types.AttributeKeyContractAddr, contractAddress.String()),
		sdk.NewAttribute(types.AttributeKeyNewAdmin, newAdminStr),
	))

	return nil
}

func (k Keeper) setContractLabel(ctx context.Context, contractAddress, caller sdk.AccAddress, newLabel string, authZ types.AuthorizationPolicy) error {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	contractInfo := k.GetContractInfo(sdkCtx, contractAddress)
	if contractInfo == nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "unknown contract")
	}
	if !authZ.CanModifyContract(contractInfo.AdminAddr(), caller) {
		return errorsmod.Wrap(sdkerrors.ErrUnauthorized, "can not modify contract")
	}
	contractInfo.Label = newLabel
	k.mustStoreContractInfo(sdkCtx, contractAddress, contractInfo)
	sdkCtx.EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeUpdateContractLabel,
		sdk.NewAttribute(types.AttributeKeyContractAddr, contractAddress.String()),
		sdk.NewAttribute(types.AttributeKeyNewLabel, newLabel),
	))

	return nil
}

func (k Keeper) appendToContractHistory(ctx context.Context, contractAddr sdk.AccAddress, newEntries ...types.ContractCodeHistoryEntry) error {
	store := k.storeService.OpenKVStore(ctx)
	// find last element position
	var pos uint64
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(store), types.GetContractCodeHistoryElementPrefix(contractAddr))
	iter := prefixStore.ReverseIterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		if len(iter.Key()) == 8 { // add extra safety in a mixed contract length environment
			pos = sdk.BigEndianToUint64(iter.Key())
			break
		}
	}
	// then store with incrementing position
	for _, e := range newEntries {
		pos++
		key := types.GetContractCodeHistoryElementKey(contractAddr, pos)
		if err := store.Set(key, k.cdc.MustMarshal(&e)); err != nil { //nolint:gosec
			return err
		}
	}
	return nil
}

func (k Keeper) GetContractHistory(ctx context.Context, contractAddr sdk.AccAddress) []types.ContractCodeHistoryEntry {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), types.GetContractCodeHistoryElementPrefix(contractAddr))
	r := make([]types.ContractCodeHistoryEntry, 0)
	iter := prefixStore.Iterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		if len(iter.Key()) != 8 { // add extra safety in a mixed contract length environment
			continue
		}

		var e types.ContractCodeHistoryEntry
		k.cdc.MustUnmarshal(iter.Value(), &e)
		r = append(r, e)
	}
	return r
}

// mustGetLastContractHistoryEntry returns the last element from history. To be used internally only as it panics when none exists
func (k Keeper) mustGetLastContractHistoryEntry(ctx context.Context, contractAddr sdk.AccAddress) types.ContractCodeHistoryEntry {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), types.GetContractCodeHistoryElementPrefix(contractAddr))
	iter := prefixStore.ReverseIterator(nil, nil)
	defer iter.Close()

	var r types.ContractCodeHistoryEntry
	for ; iter.Valid(); iter.Next() {
		if len(iter.Key()) == 8 { // add extra safety in a mixed contract length environment
			k.cdc.MustUnmarshal(iter.Value(), &r)
			return r
		}
	}
	// all contracts have a history
	panic(fmt.Sprintf("no history for %s", contractAddr.String()))
}

// QuerySmart queries the smart contract itself.
func (k Keeper) QuerySmart(ctx context.Context, contractAddr sdk.AccAddress, req []byte) ([]byte, error) {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "query-smart")
	// checks and increase query stack size
	sdkCtx, err := checkAndIncreaseQueryStackSize(sdk.UnwrapSDKContext(ctx), k.maxQueryStackSize)
	if err != nil {
		return nil, err
	}

	contractInfo, codeInfo, prefixStore, err := k.contractInstance(sdkCtx, contractAddr)
	if err != nil {
		return nil, err
	}

	smartQuerySetupCosts := k.gasRegister.InstantiateContractCosts(k.IsPinnedCode(sdkCtx, contractInfo.CodeID), len(req))
	sdkCtx.GasMeter().ConsumeGas(smartQuerySetupCosts, "Loading CosmWasm module: query")

	// prepare querier
	querier := k.newQueryHandler(sdkCtx, contractAddr)

	env := types.NewEnv(sdkCtx, contractAddr)
	queryResult, gasUsed, qErr := k.wasmVM.Query(codeInfo.CodeHash, env, req, prefixStore, cosmwasmAPI, querier, k.gasMeter(sdkCtx), k.runtimeGasForContract(sdkCtx), costJSONDeserialization)
	k.consumeRuntimeGas(sdkCtx, gasUsed)
	if qErr != nil {
		return nil, errorsmod.Wrap(types.ErrQueryFailed, qErr.Error())
	}
	return queryResult, nil
}

func checkAndIncreaseQueryStackSize(ctx context.Context, maxQueryStackSize uint32) (sdk.Context, error) {
	var queryStackSize uint32 = 0
	if size, ok := types.QueryStackSize(ctx); ok {
		queryStackSize = size
	}

	// increase
	queryStackSize++

	// did we go too far?
	if queryStackSize > maxQueryStackSize {
		return sdk.Context{}, types.ErrExceedMaxQueryStackSize
	}

	// set updated stack size
	return types.WithQueryStackSize(sdk.UnwrapSDKContext(ctx), queryStackSize), nil
}

// QueryRaw returns the contract's state for give key. Returns `nil` when key is `nil`.
func (k Keeper) QueryRaw(ctx context.Context, contractAddress sdk.AccAddress, key []byte) []byte {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "query-raw")
	if key == nil {
		return nil
	}
	prefixStoreKey := types.GetContractStorePrefix(contractAddress)
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), prefixStoreKey)
	return prefixStore.Get(key)
}

// internal helper function
func (k Keeper) contractInstance(ctx context.Context, contractAddress sdk.AccAddress) (types.ContractInfo, types.CodeInfo, wasmvm.KVStore, error) {
	store := k.storeService.OpenKVStore(ctx)

	contractBz, err := store.Get(types.GetContractAddressKey(contractAddress))
	if err != nil {
		return types.ContractInfo{}, types.CodeInfo{}, nil, err
	}
	if contractBz == nil {
		return types.ContractInfo{}, types.CodeInfo{}, nil, types.ErrNoSuchContractFn(contractAddress.String()).
			Wrapf("address %s", contractAddress.String())
	}
	var contractInfo types.ContractInfo
	k.cdc.MustUnmarshal(contractBz, &contractInfo)

	codeInfoBz, err := store.Get(types.GetCodeKey(contractInfo.CodeID))
	if err != nil {
		return types.ContractInfo{}, types.CodeInfo{}, nil, err
	}

	if codeInfoBz == nil {
		return contractInfo, types.CodeInfo{}, nil, types.ErrNoSuchCodeFn(contractInfo.CodeID).
			Wrapf("code id %d", contractInfo.CodeID)
	}
	var codeInfo types.CodeInfo
	k.cdc.MustUnmarshal(codeInfoBz, &codeInfo)
	prefixStoreKey := types.GetContractStorePrefix(contractAddress)
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), prefixStoreKey)
	return contractInfo, codeInfo, types.NewStoreAdapter(prefixStore), nil
}

func (k Keeper) GetContractInfo(ctx context.Context, contractAddress sdk.AccAddress) *types.ContractInfo {
	store := k.storeService.OpenKVStore(ctx)
	var contract types.ContractInfo
	contractBz, err := store.Get(types.GetContractAddressKey(contractAddress))
	if err != nil {
		panic(err)
	}
	if contractBz == nil {
		return nil
	}
	k.cdc.MustUnmarshal(contractBz, &contract)
	return &contract
}

func (k Keeper) HasContractInfo(ctx context.Context, contractAddress sdk.AccAddress) bool {
	store := k.storeService.OpenKVStore(ctx)
	ok, err := store.Has(types.GetContractAddressKey(contractAddress))
	if err != nil {
		panic(err)
	}
	return ok
}

// mustStoreContractInfo persists the ContractInfo. No secondary index updated here.
func (k Keeper) mustStoreContractInfo(ctx context.Context, contractAddress sdk.AccAddress, contract *types.ContractInfo) {
	store := k.storeService.OpenKVStore(ctx)
	err := store.Set(types.GetContractAddressKey(contractAddress), k.cdc.MustMarshal(contract))
	if err != nil {
		panic(err)
	}
}

func (k Keeper) IterateContractInfo(ctx context.Context, cb func(sdk.AccAddress, types.ContractInfo) bool) {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), types.ContractKeyPrefix)
	iter := prefixStore.Iterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		var contract types.ContractInfo
		k.cdc.MustUnmarshal(iter.Value(), &contract)
		// cb returns true to stop early
		if cb(iter.Key(), contract) {
			break
		}
	}
}

// IterateContractState iterates through all elements of the key value store for the given contract address and passes
// them to the provided callback function. The callback method can return true to abort early.
func (k Keeper) IterateContractState(ctx context.Context, contractAddress sdk.AccAddress, cb func(key, value []byte) bool) {
	prefixStoreKey := types.GetContractStorePrefix(contractAddress)
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), prefixStoreKey)
	iter := prefixStore.Iterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		if cb(iter.Key(), iter.Value()) {
			break
		}
	}
}

func (k Keeper) importContractState(ctx context.Context, contractAddress sdk.AccAddress, models []types.Model) error {
	prefixStoreKey := types.GetContractStorePrefix(contractAddress)
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), prefixStoreKey)
	for _, model := range models {
		if model.Value == nil {
			model.Value = []byte{}
		}
		if prefixStore.Has(model.Key) {
			return errorsmod.Wrapf(types.ErrDuplicate, "duplicate key: %x", model.Key)
		}
		prefixStore.Set(model.Key, model.Value)
	}
	return nil
}

func (k Keeper) GetCodeInfo(ctx context.Context, codeID uint64) *types.CodeInfo {
	store := k.storeService.OpenKVStore(ctx)
	var codeInfo types.CodeInfo
	codeInfoBz, err := store.Get(types.GetCodeKey(codeID))
	if err != nil {
		panic(err)
	}
	if codeInfoBz == nil {
		return nil
	}
	k.cdc.MustUnmarshal(codeInfoBz, &codeInfo)
	return &codeInfo
}

func (k Keeper) containsCodeInfo(ctx context.Context, codeID uint64) bool {
	store := k.storeService.OpenKVStore(ctx)
	ok, err := store.Has(types.GetCodeKey(codeID))
	if err != nil {
		panic(err)
	}
	return ok
}

func (k Keeper) IterateCodeInfos(ctx context.Context, cb func(uint64, types.CodeInfo) bool) {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), types.CodeKeyPrefix)
	iter := prefixStore.Iterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		var c types.CodeInfo
		k.cdc.MustUnmarshal(iter.Value(), &c)
		// cb returns true to stop early
		if cb(binary.BigEndian.Uint64(iter.Key()), c) {
			return
		}
	}
}

func (k Keeper) GetByteCode(ctx context.Context, codeID uint64) ([]byte, error) {
	store := k.storeService.OpenKVStore(ctx)
	var codeInfo types.CodeInfo
	codeInfoBz, err := store.Get(types.GetCodeKey(codeID))
	if err != nil {
		return nil, err
	}
	if codeInfoBz == nil {
		return nil, nil
	}
	k.cdc.MustUnmarshal(codeInfoBz, &codeInfo)
	return k.wasmVM.GetCode(codeInfo.CodeHash)
}

// PinCode pins the wasm contract in wasmvm cache
func (k Keeper) pinCode(ctx context.Context, codeID uint64) error {
	codeInfo := k.GetCodeInfo(ctx, codeID)
	if codeInfo == nil {
		return types.ErrNoSuchCodeFn(codeID).Wrapf("code id %d", codeID)
	}

	if err := k.wasmVM.Pin(codeInfo.CodeHash); err != nil {
		return errorsmod.Wrap(types.ErrPinContractFailed, err.Error())
	}
	store := k.storeService.OpenKVStore(ctx)
	// store 1 byte to not run into `nil` debugging issues
	err := store.Set(types.GetPinnedCodeIndexPrefix(codeID), []byte{1})
	if err != nil {
		return err
	}

	sdk.UnwrapSDKContext(ctx).EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypePinCode,
		sdk.NewAttribute(types.AttributeKeyCodeID, strconv.FormatUint(codeID, 10)),
	))
	return nil
}

// UnpinCode removes the wasm contract from wasmvm cache
func (k Keeper) unpinCode(ctx context.Context, codeID uint64) error {
	codeInfo := k.GetCodeInfo(ctx, codeID)
	if codeInfo == nil {
		return types.ErrNoSuchCodeFn(codeID).Wrapf("code id %d", codeID)
	}
	if err := k.wasmVM.Unpin(codeInfo.CodeHash); err != nil {
		return errorsmod.Wrap(types.ErrUnpinContractFailed, err.Error())
	}

	store := k.storeService.OpenKVStore(ctx)
	err := store.Delete(types.GetPinnedCodeIndexPrefix(codeID))
	if err != nil {
		return err
	}

	sdk.UnwrapSDKContext(ctx).EventManager().EmitEvent(sdk.NewEvent(
		types.EventTypeUnpinCode,
		sdk.NewAttribute(types.AttributeKeyCodeID, strconv.FormatUint(codeID, 10)),
	))
	return nil
}

// IsPinnedCode returns true when codeID is pinned in wasmvm cache
func (k Keeper) IsPinnedCode(ctx context.Context, codeID uint64) bool {
	store := k.storeService.OpenKVStore(ctx)
	ok, err := store.Has(types.GetPinnedCodeIndexPrefix(codeID))
	if err != nil {
		panic(err)
	}
	return ok
}

// InitializePinnedCodes updates wasmvm to pin to cache all contracts marked as pinned
func (k Keeper) InitializePinnedCodes(ctx context.Context) error {
	store := prefix.NewStore(runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx)), types.PinnedCodeIndexPrefix)
	iter := store.Iterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		codeID := types.ParsePinnedCodeIndex(iter.Key())
		codeInfo := k.GetCodeInfo(ctx, codeID)
		if codeInfo == nil {
			return types.ErrNoSuchCodeFn(codeID).Wrapf("code id %d", codeID)
		}
		if err := k.wasmVM.Pin(codeInfo.CodeHash); err != nil {
			return errorsmod.Wrap(types.ErrPinContractFailed, err.Error())
		}
	}
	return nil
}

// setContractInfoExtension updates the extension point data that is stored with the contract info
func (k Keeper) setContractInfoExtension(ctx context.Context, contractAddr sdk.AccAddress, ext types.ContractInfoExtension) error {
	info := k.GetContractInfo(ctx, contractAddr)
	if info == nil {
		return types.ErrNoSuchContractFn(contractAddr.String()).
			Wrapf("address %s", contractAddr.String())
	}
	if err := info.SetExtension(ext); err != nil {
		return err
	}
	k.mustStoreContractInfo(ctx, contractAddr, info)
	return nil
}

// setAccessConfig updates the access config of a code id.
func (k Keeper) setAccessConfig(ctx context.Context, codeID uint64, caller sdk.AccAddress, newConfig types.AccessConfig, authz types.AuthorizationPolicy) error {
	info := k.GetCodeInfo(ctx, codeID)
	if info == nil {
		return types.ErrNoSuchCodeFn(codeID).Wrapf("code id %d", codeID)
	}
	isSubset := newConfig.Permission.IsSubset(k.getInstantiateAccessConfig(ctx))
	if !authz.CanModifyCodeAccessConfig(sdk.MustAccAddressFromBech32(info.Creator), caller, isSubset) {
		return errorsmod.Wrap(sdkerrors.ErrUnauthorized, "can not modify code access config")
	}

	info.InstantiateConfig = newConfig
	k.mustStoreCodeInfo(ctx, codeID, *info)
	evt := sdk.NewEvent(
		types.EventTypeUpdateCodeAccessConfig,
		sdk.NewAttribute(types.AttributeKeyCodePermission, newConfig.Permission.String()),
		sdk.NewAttribute(types.AttributeKeyCodeID, strconv.FormatUint(codeID, 10)),
	)
	if addrs := newConfig.AllAuthorizedAddresses(); len(addrs) != 0 {
		attr := sdk.NewAttribute(types.AttributeKeyAuthorizedAddresses, strings.Join(addrs, ","))
		evt.Attributes = append(evt.Attributes, attr.ToKVPair())
	}
	sdk.UnwrapSDKContext(ctx).EventManager().EmitEvent(evt)
	return nil
}

// handleContractResponse processes the contract response data by emitting events and sending sub-/messages.
func (k *Keeper) handleContractResponse(
	ctx sdk.Context,
	contractAddr sdk.AccAddress,
	ibcPort string,
	msgs []wasmvmtypes.SubMsg,
	attrs []wasmvmtypes.EventAttribute,
	data []byte,
	evts wasmvmtypes.Events,
) ([]byte, error) {
	attributeGasCost := k.gasRegister.EventCosts(attrs, evts)
	ctx.GasMeter().ConsumeGas(attributeGasCost, "Custom contract event attributes")
	// emit all events from this contract itself
	if len(attrs) != 0 {
		wasmEvents, err := newWasmModuleEvent(attrs, contractAddr)
		if err != nil {
			return nil, err
		}
		ctx.EventManager().EmitEvents(wasmEvents)
	}
	if len(evts) > 0 {
		customEvents, err := newCustomEvents(evts, contractAddr)
		if err != nil {
			return nil, err
		}
		ctx.EventManager().EmitEvents(customEvents)
	}
	return k.wasmVMResponseHandler.Handle(ctx, contractAddr, ibcPort, msgs, data)
}

func (k Keeper) runtimeGasForContract(ctx sdk.Context) uint64 {
	meter := ctx.GasMeter()
	if meter.IsOutOfGas() {
		return 0
	}
	if meter.Limit() == math.MaxUint64 { // infinite gas meter and not out of gas
		return math.MaxUint64
	}
	return k.gasRegister.ToWasmVMGas(meter.Limit() - meter.GasConsumedToLimit())
}

func (k Keeper) consumeRuntimeGas(ctx sdk.Context, gas uint64) {
	consumed := k.gasRegister.FromWasmVMGas(gas)
	ctx.GasMeter().ConsumeGas(consumed, "wasm contract")
	// throw OutOfGas error if we ran out (got exactly to zero due to better limit enforcing)
	if ctx.GasMeter().IsOutOfGas() {
		panic(storetypes.ErrorOutOfGas{Descriptor: "Wasm engine function execution"})
	}
}

func (k Keeper) mustAutoIncrementID(ctx context.Context, sequenceKey []byte) uint64 {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(sequenceKey)
	if err != nil {
		panic(err)
	}
	id := uint64(1)
	if bz != nil {
		id = binary.BigEndian.Uint64(bz)
	}
	bz = sdk.Uint64ToBigEndian(id + 1)
	err = store.Set(sequenceKey, bz)
	if err != nil {
		panic(err)
	}
	return id
}

// PeekAutoIncrementID reads the current value without incrementing it.
func (k Keeper) PeekAutoIncrementID(ctx context.Context, sequenceKey []byte) (uint64, error) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(sequenceKey)
	if err != nil {
		return 0, errorsmod.Wrap(err, "sequence key")
	}
	id := uint64(1)
	if bz != nil {
		id = binary.BigEndian.Uint64(bz)
	}
	return id, nil
}

func (k Keeper) importAutoIncrementID(ctx context.Context, sequenceKey []byte, val uint64) error {
	store := k.storeService.OpenKVStore(ctx)
	ok, err := store.Has(sequenceKey)
	if err != nil {
		return errorsmod.Wrap(err, "sequence key")
	}
	if ok {
		return errorsmod.Wrapf(types.ErrDuplicate, "autoincrement id: %s", string(sequenceKey))
	}
	bz := sdk.Uint64ToBigEndian(val)
	return store.Set(sequenceKey, bz)
}

func (k Keeper) importContract(ctx context.Context, contractAddr sdk.AccAddress, c *types.ContractInfo, state []types.Model, historyEntries []types.ContractCodeHistoryEntry) error {
	if !k.containsCodeInfo(ctx, c.CodeID) {
		return types.ErrNoSuchCodeFn(c.CodeID).Wrapf("code id %d", c.CodeID)
	}
	if k.HasContractInfo(ctx, contractAddr) {
		return errorsmod.Wrapf(types.ErrDuplicate, "contract: %s", contractAddr)
	}
	if len(historyEntries) == 0 {
		return types.ErrEmpty.Wrap("contract history")
	}

	creatorAddress, err := sdk.AccAddressFromBech32(c.Creator)
	if err != nil {
		return err
	}

	err = k.appendToContractHistory(ctx, contractAddr, historyEntries...)
	if err != nil {
		return err
	}
	k.mustStoreContractInfo(ctx, contractAddr, c)
	err = k.addToContractCodeSecondaryIndex(ctx, contractAddr, historyEntries[len(historyEntries)-1])
	if err != nil {
		return err
	}
	err = k.addToContractCreatorSecondaryIndex(ctx, creatorAddress, historyEntries[0].Updated, contractAddr)
	if err != nil {
		return err
	}
	return k.importContractState(ctx, contractAddr, state)
}

func (k Keeper) newQueryHandler(ctx sdk.Context, contractAddress sdk.AccAddress) QueryHandler {
	return NewQueryHandler(ctx, k.wasmVMQueryHandler, contractAddress, k.gasRegister)
}

// MultipliedGasMeter wraps the GasMeter from context and multiplies all reads by out defined multiplier
type MultipliedGasMeter struct {
	originalMeter storetypes.GasMeter
	GasRegister   types.GasRegister
}

func NewMultipliedGasMeter(originalMeter storetypes.GasMeter, gr types.GasRegister) MultipliedGasMeter {
	return MultipliedGasMeter{originalMeter: originalMeter, GasRegister: gr}
}

var _ wasmvm.GasMeter = MultipliedGasMeter{}

func (m MultipliedGasMeter) GasConsumed() storetypes.Gas {
	return m.GasRegister.ToWasmVMGas(m.originalMeter.GasConsumed())
}

func (k Keeper) gasMeter(ctx sdk.Context) MultipliedGasMeter {
	return NewMultipliedGasMeter(ctx.GasMeter(), k.gasRegister)
}

// Logger returns a module-specific logger.
func (k Keeper) Logger(ctx sdk.Context) log.Logger {
	return moduleLogger(ctx)
}

func moduleLogger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", fmt.Sprintf("x/%s", types.ModuleName))
}

// Querier creates a new grpc querier instance
func Querier(k *Keeper) *GrpcQuerier {
	return NewGrpcQuerier(k.cdc, k.storeService, k, k.queryGasLimit)
}

// QueryGasLimit returns the gas limit for smart queries.
func (k Keeper) QueryGasLimit() storetypes.Gas {
	return k.queryGasLimit
}

// BankCoinTransferrer replicates the cosmos-sdk behavior as in
// https://github.com/cosmos/cosmos-sdk/blob/v0.41.4/x/bank/keeper/msg_server.go#L26
type BankCoinTransferrer struct {
	keeper types.BankKeeper
}

func NewBankCoinTransferrer(keeper types.BankKeeper) BankCoinTransferrer {
	return BankCoinTransferrer{
		keeper: keeper,
	}
}

// TransferCoins transfers coins from source to destination account when coin send was enabled for them and the recipient
// is not in the blocked address list.
func (c BankCoinTransferrer) TransferCoins(parentCtx sdk.Context, fromAddr, toAddr sdk.AccAddress, amount sdk.Coins) error {
	em := sdk.NewEventManager()
	ctx := parentCtx.WithEventManager(em)
	if err := c.keeper.IsSendEnabledCoins(ctx, amount...); err != nil {
		return err
	}
	if c.keeper.BlockedAddr(toAddr) {
		return errorsmod.Wrapf(sdkerrors.ErrUnauthorized, "%s is not allowed to receive funds", toAddr.String())
	}

	sdkerr := c.keeper.SendCoins(ctx, fromAddr, toAddr, amount)
	if sdkerr != nil {
		return sdkerr
	}
	for _, e := range em.Events() {
		if e.Type == sdk.EventTypeMessage { // skip messages as we talk to the keeper directly
			continue
		}
		parentCtx.EventManager().EmitEvent(e)
	}
	return nil
}

var _ AccountPruner = VestingCoinBurner{}

// VestingCoinBurner default implementation for AccountPruner to burn the coins
type VestingCoinBurner struct {
	bank types.BankKeeper
}

// NewVestingCoinBurner constructor
func NewVestingCoinBurner(bank types.BankKeeper) VestingCoinBurner {
	if bank == nil {
		panic("bank keeper must not be nil")
	}
	return VestingCoinBurner{bank: bank}
}

// CleanupExistingAccount accepts only vesting account types to burns all their original vesting coin balances.
// Other account types will be rejected and returned as unhandled.
func (b VestingCoinBurner) CleanupExistingAccount(ctx sdk.Context, existingAcc sdk.AccountI) (handled bool, err error) {
	v, ok := existingAcc.(vestingexported.VestingAccount)
	if !ok {
		return false, nil
	}

	ctx = ctx.WithGasMeter(storetypes.NewInfiniteGasMeter())
	coinsToBurn := sdk.NewCoins()
	for _, orig := range v.GetOriginalVesting() { // focus on the coin denoms that were setup originally; getAllBalances has some issues
		coinsToBurn = append(coinsToBurn, b.bank.GetBalance(ctx, existingAcc.GetAddress(), orig.Denom))
	}
	if err := b.bank.SendCoinsFromAccountToModule(ctx, existingAcc.GetAddress(), types.ModuleName, coinsToBurn); err != nil {
		return false, errorsmod.Wrap(err, "prune account balance")
	}
	if err := b.bank.BurnCoins(ctx, types.ModuleName, coinsToBurn); err != nil {
		return false, errorsmod.Wrap(err, "burn account balance")
	}
	return true, nil
}

type msgDispatcher interface {
	DispatchSubmessages(ctx sdk.Context, contractAddr sdk.AccAddress, ibcPort string, msgs []wasmvmtypes.SubMsg) ([]byte, error)
}

// DefaultWasmVMContractResponseHandler default implementation that first dispatches submessage then normal messages.
// The Submessage execution may include an success/failure response handling by the contract that can overwrite the
// original
type DefaultWasmVMContractResponseHandler struct {
	md msgDispatcher
}

func NewDefaultWasmVMContractResponseHandler(md msgDispatcher) *DefaultWasmVMContractResponseHandler {
	return &DefaultWasmVMContractResponseHandler{md: md}
}

// Handle processes the data returned by a contract invocation.
func (h DefaultWasmVMContractResponseHandler) Handle(ctx sdk.Context, contractAddr sdk.AccAddress, ibcPort string, messages []wasmvmtypes.SubMsg, origRspData []byte) ([]byte, error) {
	result := origRspData
	switch rsp, err := h.md.DispatchSubmessages(ctx, contractAddr, ibcPort, messages); {
	case err != nil:
		return nil, errorsmod.Wrap(err, "submessages")
	case rsp != nil:
		result = rsp
	}
	return result, nil
}
